use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::{Result, anyhow};
use vapoursynth::frame::Frame;

use crate::{
    mv::MotionVector,
    mv_gof::MVGroupOfFrames,
    params::{DctMode, DivideMode, MotionFlags, PenaltyScaling, SearchType, Subpel},
    plane_of_blocks::{MvsOutput, PlaneOfBlocks},
    util::Pixel,
};

#[derive(Debug, Clone)]
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
        bad_range: usize,
        meander: bool,
        try_many: bool,
        search_type_coarse: SearchType,
    ) -> MvsOutput {
        let mut vectors = MvsOutput {
            validity: true,
            blocks: self.init_output_blocks(),
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
        self.planes[self.level_count - 1].search_mvs(
            0,
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
        );

        // Refining the search until we reach the highest detail interpolation.
        for i in (0..(self.level_count - 1)).rev() {
            todo!()
        }

        vectors
    }

    pub fn extra_divide(&self, vectors: &mut MvsOutput) {
        todo!()
    }

    pub(crate) fn init_output_blocks(&self) -> Vec<Vec<u8>> {
        let mut output = Vec::with_capacity(self.level_count);
        for i in (0..self.level_count).rev() {
            output.push(vec![
                0;
                self.planes[i].get_array_size(self.divide_extra).get()
            ]);
        }
        output
    }
}
