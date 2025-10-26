use std::{
    mem::transmute,
    num::{NonZeroU8, NonZeroUsize},
};

use anyhow::{Result, anyhow};
use vapoursynth::frame::Frame;

use crate::{
    mv::{MV_SIZE, MotionVector},
    mv_gof::MVGroupOfFrames,
    params::{DctMode, DivideMode, MotionFlags, PenaltyScaling, SearchType, Subpel},
    plane_of_blocks::{MvsOutput, PlaneOfBlocks},
    util::{Pixel, median},
};

#[derive(Clone)]
pub struct GroupOfPlanes<T: Pixel> {
    pub blk_size_x: NonZeroUsize,
    pub blk_size_y: NonZeroUsize,
    pub level_count: usize,
    pub overlap_x: usize,
    pub overlap_y: usize,
    pub x_ratio_uv: NonZeroU8,
    pub y_ratio_uv: NonZeroU8,
    pub divide_extra: DivideMode,
    pub planes: Vec<PlaneOfBlocks<T>>,
}

impl<T: Pixel> GroupOfPlanes<T> {
    pub fn new(
        blk_size_x: NonZeroUsize,
        blk_size_y: NonZeroUsize,
        level_count: usize,
        pel: Subpel,
        motion_flags: MotionFlags,
        overlap_x: usize,
        overlap_y: usize,
        blk_x: NonZeroUsize,
        blk_y: NonZeroUsize,
        x_ratio_uv: NonZeroU8,
        y_ratio_uv: NonZeroU8,
        divide_extra: DivideMode,
        bits_per_sample: NonZeroU8,
    ) -> Result<Self> {
        let mut planes = Vec::with_capacity(level_count);

        let mut pel_current = pel;
        let mut motion_flags_current = motion_flags;
        let width_b = NonZeroUsize::new((blk_size_x.get() - overlap_x) * blk_x.get() + overlap_x)
            .ok_or(anyhow!("invalid width calculation"))?;
        let height_b = NonZeroUsize::new((blk_size_y.get() - overlap_y) * blk_y.get() + overlap_y)
            .ok_or(anyhow!("invalid height calculation"))?;

        for i in 0..level_count {
            if i == level_count - 1 {
                motion_flags_current |= MotionFlags::SMALLEST_PLANE;
            }

            let blk_x_current = NonZeroUsize::new(
                ((width_b.get() >> i) - overlap_x) / (blk_size_x.get() - overlap_x),
            )
            .ok_or(anyhow!("invalid block x calculation"))?;
            let blk_y_current = NonZeroUsize::new(
                ((height_b.get() >> i) - overlap_y) / (blk_size_y.get() - overlap_y),
            )
            .ok_or(anyhow!("invalid block y calculation"))?;

            planes.push(PlaneOfBlocks::new(
                blk_x_current,
                blk_y_current,
                blk_size_x,
                blk_size_y,
                pel_current,
                i,
                motion_flags_current,
                overlap_x,
                overlap_y,
                x_ratio_uv,
                y_ratio_uv,
                bits_per_sample,
            ));
            pel_current = Subpel::Full;
        }

        Ok(Self {
            blk_size_x,
            blk_size_y,
            level_count,
            overlap_x,
            overlap_y,
            x_ratio_uv,
            y_ratio_uv,
            divide_extra,
            planes,
        })
    }

    pub fn search_mvs(
        &mut self,
        src_gof: &MVGroupOfFrames,
        src_frame_data: &Frame,
        ref_gof: &MVGroupOfFrames,
        ref_frame_data: &Frame,
        search_type: SearchType,
        search_param: usize,
        pel_search: usize,
        lambda: u32,
        lambda_sad: u32,
        penalty_new: u16,
        penalty_level: PenaltyScaling,
        global: bool,
        field_shift: isize,
        dct_mode: DctMode,
        penalty_zero: u16,
        mut penalty_global: u16,
        bad_sad: u64,
        bad_range: isize,
        meander: bool,
        try_many: bool,
        search_type_coarse: SearchType,
    ) -> Result<MvsOutput> {
        let mut vectors = MvsOutput {
            validity: true,
            block_data: vec![0; self.get_array_size()].into_boxed_slice(),
        };

        let field_shift_cur = if self.level_count - 1 == 0 {
            field_shift
        } else {
            0
        };

        let mut global_mv = MotionVector::zero();
        if !global {
            penalty_global = penalty_zero
        }

        // Search the motion vectors, for the low details interpolations first
        let mut mean_luma_change = 0;
        let search_type_smallest = if self.level_count == 1
            || [SearchType::Horizontal, SearchType::Vertical].contains(&search_type)
        {
            search_type
        } else {
            search_type_coarse
        };
        let search_param_smallest = if self.level_count == 1 {
            pel_search
        } else {
            search_param
        };
        let try_many_level = try_many && self.level_count > 1;
        let mut out_idx = 0;
        self.planes[self.level_count - 1].search_mvs(
            out_idx,
            &src_gof.frames[self.level_count - 1],
            src_frame_data,
            &ref_gof.frames[self.level_count - 1],
            ref_frame_data,
            search_type_smallest,
            search_param_smallest,
            lambda,
            lambda_sad,
            penalty_new,
            penalty_level,
            &mut vectors,
            &mut global_mv,
            field_shift_cur,
            dct_mode,
            &mut mean_luma_change,
            penalty_zero,
            penalty_global,
            bad_sad,
            bad_range,
            meander,
            try_many_level,
        )?;

        // Refining the search until we reach the highest detail interpolation.
        for i in (0..=(self.level_count - 2)).rev() {
            out_idx += 1;

            // full search for coarse planes
            let search_type_level = if i == 0
                || search_type == SearchType::Horizontal
                || search_type == SearchType::Vertical
            {
                search_type
            } else {
                search_type_coarse
            };
            // special case for finest level
            let search_param_level = if i == 0 { pel_search } else { search_param };
            if global {
                // get updated global MV (doubled)
                self.planes[i + 1].estimate_global_mv_doubled(global_mv);
            }
            self.planes[i].interpolate_prediction(&self.planes[i + 1]);
            // may be non zero for finest level only
            let field_shift_cur = if i == 0 { field_shift } else { 0 };
            // not for finest level to not decrease speed
            let try_many_level = try_many && i > 0;

            self.planes[i].search_mvs(
                out_idx,
                &src_gof.frames[i],
                src_frame_data,
                &ref_gof.frames[i],
                ref_frame_data,
                search_type_level,
                search_param_level,
                lambda,
                lambda_sad,
                penalty_new,
                penalty_level,
                &mut vectors,
                &mut global_mv,
                field_shift_cur,
                dct_mode,
                &mut mean_luma_change,
                penalty_zero,
                penalty_global,
                bad_sad,
                bad_range,
                meander,
                try_many_level,
            )?;
        }

        Ok(vectors)
    }

    pub fn extra_divide(&self, out: &mut MvsOutput) {
        let mut start_idx = 0;
        // skip all levels up to finest estimated
        for i in (1..self.level_count).rev() {
            start_idx += self.planes[i].get_array_size(DivideMode::None).get();
        }

        let size = usize::from_le_bytes(
            out.block_data[start_idx..][..size_of::<usize>()]
                .try_into()
                .expect("slice with incorrect length"),
        );
        let blk_y = self.planes[0].blk_y.get();
        let blk_x = self.planes[0].blk_x.get();
        // finest estimated plane
        let mut in_idx = start_idx + size_of::<usize>();
        // position for divided subblocks data
        let mut out_idx = start_idx + size + size_of::<usize>();

        // top blocks
        for bx in 0..blk_x {
            extra_divide_block_data(out, in_idx, out_idx, bx, blk_x);
        }

        out_idx += blk_x * 4 * MV_SIZE;
        in_idx += blk_x * MV_SIZE;

        // middle blocks
        for _by in 1..(blk_y - 1) {
            let bx = 0;
            extra_divide_block_data(out, in_idx, out_idx, bx, blk_x);

            for bx in 1..(blk_x - 1) {
                extra_divide_block_data(out, in_idx, out_idx, bx, blk_x);

                if self.divide_extra == DivideMode::Median {
                    assign_median(out, in_idx, out_idx, bx, bx - 1, bx - blk_x, bx * 2);
                    assign_median(out, in_idx, out_idx, bx, bx + 1, bx - blk_x, bx * 2 + 1);
                    assign_median(
                        out,
                        in_idx,
                        out_idx,
                        bx,
                        bx - 1,
                        bx + blk_x,
                        bx * 2 + blk_x * 2,
                    );
                    assign_median(
                        out,
                        in_idx,
                        out_idx,
                        bx,
                        bx + 1,
                        bx + blk_x,
                        bx * 2 + blk_x * 2 + 1,
                    );
                }
            }

            let bx = blk_x - 1;
            extra_divide_block_data(out, in_idx, out_idx, bx, blk_x);

            out_idx += blk_x * 4 * MV_SIZE;
            in_idx += blk_x * MV_SIZE;
        }

        // bottom blocks
        for bx in 0..blk_x {
            extra_divide_block_data(out, in_idx, out_idx, bx, blk_x);
        }
    }

    fn get_array_size(&self) -> usize {
        let mut size = 0;
        for i in (0..self.level_count).rev() {
            size += self.planes[i].get_array_size(self.divide_extra).get();
        }
        size
    }
}

fn extra_divide_block_data(
    out: &mut MvsOutput,
    in_idx: usize,
    out_idx: usize,
    bx: usize,
    blk_x: usize,
) {
    // SAFETY: Size is checked
    let mut block: MotionVector = unsafe {
        transmute::<[u8; MV_SIZE], _>(
            out.block_data[in_idx + bx * MV_SIZE..][..MV_SIZE]
                .try_into()
                .expect("slice with incorrect length"),
        )
    };
    block.sad >>= 2;

    // SAFETY: I hate every part of this, but this is what the C code does.
    let blocks_out: &mut [MotionVector] = unsafe { transmute(&mut out.block_data[out_idx..]) };
    // top left subblock
    blocks_out[bx * 2] = block;
    // top right subblock
    blocks_out[bx * 2 + 1] = block;
    // bottom left subblock
    blocks_out[bx * 2 + blk_x * 2] = block;
    // bottom right subblock
    blocks_out[bx * 2 + blk_x * 2 + 1] = block;
}

fn get_median(v: &mut MotionVector, v1: MotionVector, v2: MotionVector, v3: MotionVector) {
    v.x = median(v1.x, v2.x, v3.x);
    v.y = median(v1.y, v2.y, v3.y);

    if (v.x == v1.x && v.y == v1.y) || (v.x == v2.x && v.y == v2.y) || (v.x == v3.x && v.y == v3.y)
    {
        return;
    }

    v.x = v1.x;
    v.y = v1.y;
}

fn assign_median(
    out: &mut MvsOutput,
    in_idx: usize,
    out_idx: usize,
    in_1_offset: usize,
    in_2_offset: usize,
    in_3_offset: usize,
    out_offset: usize,
) {
    let blkin_1: MotionVector = unsafe {
        transmute::<[u8; MV_SIZE], _>(
            out.block_data[in_idx + in_1_offset * MV_SIZE..][..MV_SIZE]
                .try_into()
                .expect("slice with incorrect length"),
        )
    };
    let blkin_2: MotionVector = unsafe {
        transmute::<[u8; MV_SIZE], _>(
            out.block_data[in_idx + in_2_offset * MV_SIZE..][..MV_SIZE]
                .try_into()
                .expect("slice with incorrect length"),
        )
    };
    let blkin_3: MotionVector = unsafe {
        transmute::<[u8; MV_SIZE], _>(
            out.block_data[in_idx + in_3_offset * MV_SIZE..][..MV_SIZE]
                .try_into()
                .expect("slice with incorrect length"),
        )
    };
    let blkout: &mut MotionVector = unsafe {
        transmute::<&mut [u8; MV_SIZE], _>(
            &mut out.block_data[out_idx + out_offset * MV_SIZE..][..MV_SIZE]
                .try_into()
                .expect("slice with incorrect length"),
        )
    };
    get_median(blkout, blkin_1, blkin_2, blkin_3);
}
