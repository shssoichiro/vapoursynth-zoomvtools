use crate::{
    dct::DctHelper,
    mv::{CheckMVFlags, MV_SIZE, MotionVector},
    mv_frame::MVFrame,
    params::{DctMode, DivideMode, MVPlaneSet, MotionFlags, PenaltyScaling, SearchType, Subpel},
    util::{Pixel, get_sad, get_satd, luma_sum, median, plane_with_padding},
};
use anyhow::Result;
use bitflags::bitflags;
use smallvec::SmallVec;
use std::{
    cmp::{max, min},
    mem::transmute,
    num::{NonZeroU8, NonZeroUsize},
};
use vapoursynth::frame::Frame;

// max block width * max block height
const MAX_BLOCK_SIZE: usize = 128 * 128;
// right now 5 should be enough (TSchniede)
const MAX_PREDICTOR: usize = 5;

#[derive(Clone)]
pub(crate) struct PlaneOfBlocks<T: Pixel> {
    pub pel: Subpel,
    pub log_pel: u8,
    pub log_scale: usize,
    pub scale: usize,
    /// width of a block
    pub blk_size_x: NonZeroUsize,
    /// height of a block
    pub blk_size_y: NonZeroUsize,
    /// horizontal overlap of blocks
    pub overlap_x: usize,
    /// vertical overlap of blocks
    pub overlap_y: usize,
    /// width in number of blocks
    pub blk_x: NonZeroUsize,
    /// height in number of blocks
    pub blk_y: NonZeroUsize,
    /// number of blocks in the plane (isn't this just `blk_x` * `blk_y`?)
    pub blk_count: NonZeroUsize,
    pub x_ratio_uv: NonZeroU8,
    pub y_ratio_uv: NonZeroU8,
    pub log_x_ratio_uv: u8,
    pub log_y_ratio_uv: u8,
    pub bits_per_sample: NonZeroU8,
    pub smallest_plane: bool,
    pub chroma: bool,
    pub can_use_satd: bool,
    pub global_mv_predictor: MotionVector,
    pub vectors: Vec<MotionVector>,
    pub dct_pitch: NonZeroUsize,
    pub freq_size: NonZeroUsize,
    pub freq_array: Vec<i32>,
    pub very_big_sad: NonZeroUsize,

    // TODO: We might want to move these away from this struct
    dct: Option<DctHelper>,
    dct_src: SmallVec<[T; MAX_BLOCK_SIZE]>,
    dct_ref: SmallVec<[T; MAX_BLOCK_SIZE]>,
    src_pitch_temp: [NonZeroUsize; 3],
    src_temp: [SmallVec<[T; MAX_BLOCK_SIZE]>; 3],

    // Stuff that's not initialized until MV search
    dct_mode: Option<DctMode>,
    dct_weight_16: u32,
    bad_sad: u64,
    bad_range: isize,
    zero_mv_field_shifted: MotionVector,
    /// absolute x coordinate of the origin of the block in the reference frame
    x: [isize; 3],
    /// absolute y coordinate of the origin of the block in the reference frame
    y: [isize; 3],
    src_pitch: [NonZeroUsize; 3],
    ref_pitch: [NonZeroUsize; 3],
    search_type: SearchType,
    search_param: usize,
    penalty_zero: u16,
    penalty_global: u16,
    penalty_new: u16,
    bad_count: usize,
    try_many: bool,
    /// direction of scan (1 is left to rught, -1 is right to left)
    blk_scan_dir: i8,
    lambda: u32,
    lambda_sad: u32,
    dx_max: isize,
    dy_max: isize,
    dx_min: isize,
    dy_min: isize,
    predictor: MotionVector,
    predictors: [MotionVector; MAX_PREDICTOR],
    best_mv: MotionVector,
    src_offset: [usize; 3],
    src_luma: u64,
    ref_luma: u64,
    sum_luma_change: i64,
    min_cost: i64,
    blk_x_i: usize,
    blk_y_i: usize,
    blk_idx: usize,
}

impl<T: Pixel> PlaneOfBlocks<T> {
    #[must_use]
    pub fn new(
        blk_x: NonZeroUsize,
        blk_y: NonZeroUsize,
        blk_size_x: NonZeroUsize,
        blk_size_y: NonZeroUsize,
        pel: Subpel,
        level: usize,
        motion_flags: MotionFlags,
        overlap_x: usize,
        overlap_y: usize,
        x_ratio_uv: NonZeroU8,
        y_ratio_uv: NonZeroU8,
        bits_per_sample: NonZeroU8,
    ) -> Self {
        debug_assert!(
            bits_per_sample.get() as usize > (size_of::<T>() - 1) * 8
                && (bits_per_sample.get() as usize <= size_of::<T>() * 8)
        );

        let blk_count = blk_x.saturating_mul(blk_y);
        // SAFETY: pel can never be 0, so this can never be 0
        let freq_size = unsafe { NonZeroUsize::new_unchecked(8192 * u8::from(pel) as usize * 2) };
        let dct_pitch = blk_size_x;
        // SAFETY: valid values will never result in this being 0
        let chroma_src_pitch =
            unsafe { NonZeroUsize::new_unchecked(blk_size_x.get() / x_ratio_uv.get() as usize) };
        let src_pitch_temp = [blk_size_x, chroma_src_pitch, chroma_src_pitch];
        Self {
            pel,
            log_pel: u8::from(pel).ilog2() as u8,
            log_scale: level,
            scale: 2usize.pow(level as u32),
            blk_size_x,
            blk_size_y,
            overlap_x,
            overlap_y,
            blk_x,
            blk_y,
            blk_count,
            x_ratio_uv,
            y_ratio_uv,
            log_x_ratio_uv: x_ratio_uv.ilog2() as u8,
            log_y_ratio_uv: y_ratio_uv.ilog2() as u8,
            bits_per_sample,
            smallest_plane: motion_flags.contains(MotionFlags::SMALLEST_PLANE),
            chroma: motion_flags.contains(MotionFlags::USE_CHROMA_MOTION),
            can_use_satd: !(blk_size_x.get() == 16 && blk_size_y.get() == 2),
            global_mv_predictor: MotionVector::zero(),
            vectors: vec![MotionVector::zero(); blk_count.get()],
            dct_pitch,
            freq_size,
            freq_array: vec![0; freq_size.get()],
            // SAFETY: constant can never be 0
            very_big_sad: blk_size_x
                .saturating_mul(blk_size_y)
                .saturating_mul(unsafe { NonZeroUsize::new_unchecked(1 << bits_per_sample.get()) }),
            dct: None,
            dct_src: SmallVec::from_elem(T::from(0), blk_size_y.get() * dct_pitch.get()),
            dct_ref: SmallVec::from_elem(T::from(0), blk_size_y.get() * dct_pitch.get()),
            src_pitch_temp,
            src_temp: [
                SmallVec::from_elem(T::from(0), blk_size_y.get() * src_pitch_temp[0].get()),
                SmallVec::from_elem(
                    T::from(0),
                    blk_size_y.get() / y_ratio_uv.get() as usize * src_pitch_temp[1].get(),
                ),
                SmallVec::from_elem(
                    T::from(0),
                    blk_size_y.get() / y_ratio_uv.get() as usize * src_pitch_temp[2].get(),
                ),
            ],
            // fields that get filled in during search
            dct_mode: Default::default(),
            dct_weight_16: Default::default(),
            bad_sad: Default::default(),
            bad_range: Default::default(),
            zero_mv_field_shifted: Default::default(),
            x: Default::default(),
            y: Default::default(),
            src_pitch: src_pitch_temp,
            ref_pitch: src_pitch_temp,
            search_type: SearchType::Hex2,
            search_param: Default::default(),
            penalty_zero: Default::default(),
            penalty_global: Default::default(),
            bad_count: Default::default(),
            try_many: Default::default(),
            sum_luma_change: Default::default(),
            blk_scan_dir: Default::default(),
            penalty_new: Default::default(),
            lambda: Default::default(),
            lambda_sad: Default::default(),
            dx_max: Default::default(),
            dy_max: Default::default(),
            dx_min: Default::default(),
            dy_min: Default::default(),
            predictor: Default::default(),
            predictors: Default::default(),
            best_mv: Default::default(),
            src_offset: Default::default(),
            src_luma: Default::default(),
            ref_luma: Default::default(),
            min_cost: Default::default(),
            blk_idx: Default::default(),
            blk_x_i: Default::default(),
            blk_y_i: Default::default(),
        }
    }

    pub fn search_mvs<'a>(
        &mut self,
        out_idx: usize,
        src_frame: &'a MVFrame,
        src_frame_data: &'a Frame<'a>,
        ref_frame: &'a MVFrame,
        ref_frame_data: &'a Frame<'a>,
        search_type: SearchType,
        search_param: usize,
        lambda: u32,
        lambda_sad: u32,
        penalty_new: u16,
        penalty_level: PenaltyScaling,
        out: &'a mut MvsOutput,
        global_mv: &'a mut MotionVector,
        field_shift: isize,
        dct_mode: DctMode,
        mean_luma_change: &'a mut i32,
        penalty_zero: u16,
        penalty_global: u16,
        bad_sad: u64,
        bad_range: isize,
        meander: bool,
        try_many: bool,
    ) -> Result<()> {
        let args = SearchMvsArgs {
            out_idx,
            src_frame,
            src_frame_data,
            ref_frame,
            ref_frame_data,
            search_type,
            search_param,
            lambda,
            lambda_sad,
            penalty_new,
            penalty_level,
            out,
            global_mv,
            field_shift,
            mean_luma_change,
            penalty_zero,
            penalty_global,
            bad_sad,
            bad_range,
            meander,
            try_many,
        };
        match (u8::from(dct_mode), self.log_pel) {
            (0, 0) => self.search_mvs_internal::<0, 0>(args),
            (1, 0) => self.search_mvs_internal::<1, 0>(args),
            (2, 0) => self.search_mvs_internal::<2, 0>(args),
            (3, 0) => self.search_mvs_internal::<3, 0>(args),
            (4, 0) => self.search_mvs_internal::<4, 0>(args),
            (5, 0) => self.search_mvs_internal::<5, 0>(args),
            (6, 0) => self.search_mvs_internal::<6, 0>(args),
            (7, 0) => self.search_mvs_internal::<7, 0>(args),
            (8, 0) => self.search_mvs_internal::<8, 0>(args),
            (9, 0) => self.search_mvs_internal::<9, 0>(args),
            (10, 0) => self.search_mvs_internal::<10, 0>(args),
            (0, 1) => self.search_mvs_internal::<0, 1>(args),
            (1, 1) => self.search_mvs_internal::<1, 1>(args),
            (2, 1) => self.search_mvs_internal::<2, 1>(args),
            (3, 1) => self.search_mvs_internal::<3, 1>(args),
            (4, 1) => self.search_mvs_internal::<4, 1>(args),
            (5, 1) => self.search_mvs_internal::<5, 1>(args),
            (6, 1) => self.search_mvs_internal::<6, 1>(args),
            (7, 1) => self.search_mvs_internal::<7, 1>(args),
            (8, 1) => self.search_mvs_internal::<8, 1>(args),
            (9, 1) => self.search_mvs_internal::<9, 1>(args),
            (10, 1) => self.search_mvs_internal::<10, 1>(args),
            (0, 2) => self.search_mvs_internal::<0, 2>(args),
            (1, 2) => self.search_mvs_internal::<1, 2>(args),
            (2, 2) => self.search_mvs_internal::<2, 2>(args),
            (3, 2) => self.search_mvs_internal::<3, 2>(args),
            (4, 2) => self.search_mvs_internal::<4, 2>(args),
            (5, 2) => self.search_mvs_internal::<5, 2>(args),
            (6, 2) => self.search_mvs_internal::<6, 2>(args),
            (7, 2) => self.search_mvs_internal::<7, 2>(args),
            (8, 2) => self.search_mvs_internal::<8, 2>(args),
            (9, 2) => self.search_mvs_internal::<9, 2>(args),
            (10, 2) => self.search_mvs_internal::<10, 2>(args),
            _ => unreachable!(),
        }
    }

    fn search_mvs_internal<const DCT_MODE: u8, const LOG_PEL: usize>(
        &mut self,
        args: SearchMvsArgs,
    ) -> Result<()> {
        let SearchMvsArgs {
            out_idx,
            src_frame,
            src_frame_data,
            ref_frame,
            ref_frame_data,
            search_type,
            search_param,
            lambda,
            lambda_sad,
            penalty_new,
            penalty_level,
            out,
            global_mv,
            field_shift,
            mean_luma_change,
            penalty_zero,
            penalty_global,
            bad_sad,
            bad_range,
            meander,
            try_many,
        } = args;

        // TODO: Do we really need to be setting all of these as fields on the struct?
        if (1..=4).contains(&DCT_MODE) {
            self.dct = Some(DctHelper::new(
                self.blk_size_x,
                self.blk_size_y,
                self.bits_per_sample,
            )?);
        }
        self.dct_mode = Some(DctMode::try_from(DCT_MODE as i64).unwrap());
        self.dct_weight_16 = min(
            16,
            mean_luma_change.unsigned_abs()
                / (self.blk_size_x.get() * self.blk_size_y.get()) as u32,
        );
        self.bad_sad = bad_sad;
        self.bad_range = bad_range;
        self.zero_mv_field_shifted = MotionVector {
            x: 0,
            y: field_shift,
            sad: 0,
        };
        self.global_mv_predictor = MotionVector {
            x: (1 << LOG_PEL) * global_mv.x,
            y: (1 << LOG_PEL) * global_mv.y + field_shift,
            sad: global_mv.sad,
        };

        // SAFETY: We only modify the contents of this data within this function,
        // so we control the layout.
        let blk_data: &mut [MotionVector] =
            unsafe { transmute(&mut out.block_data[out_idx * MV_SIZE..]) };
        self.y[0] = src_frame.planes[0].vpad as isize;
        if (src_frame.yuv_mode & MVPlaneSet::UPLANE).bits() > 0 {
            self.y[1] = src_frame.planes[1].vpad as isize;
        }
        if (src_frame.yuv_mode & MVPlaneSet::VPLANE).bits() > 0 {
            self.y[2] = src_frame.planes[2].vpad as isize;
        }
        self.src_pitch[0] = src_frame.planes[0].pitch;
        if self.chroma {
            self.src_pitch[1] = src_frame.planes[1].pitch;
            self.src_pitch[2] = src_frame.planes[2].pitch;
        }
        self.ref_pitch[0] = ref_frame.planes[0].pitch;
        if self.chroma {
            self.ref_pitch[1] = ref_frame.planes[1].pitch;
            self.ref_pitch[2] = ref_frame.planes[2].pitch;
        }
        self.search_type = search_type;
        self.search_param = search_param;
        let mut lambda_level = lambda / (1u32 << LOG_PEL).pow(2);
        if penalty_level == PenaltyScaling::Linear {
            lambda_level *= self.scale as u32;
        } else if penalty_level == PenaltyScaling::Quadratic {
            lambda_level *= self.scale.pow(2) as u32;
        }
        self.penalty_zero = penalty_zero;
        self.penalty_global = penalty_global;
        self.bad_count = 0;
        self.try_many = try_many;
        self.sum_luma_change = 0;

        // Functions using float must not be used here
        // TODO: why?
        let mut blk_data_offset = 0;
        for blk_y in 0..self.blk_y.get() {
            self.blk_y_i = blk_y;
            self.blk_scan_dir = if blk_y % 2 == 0 || !meander { 1 } else { -1 };
            // meander (alternate) scan blocks (even row left to right, odd row right to left)
            let blk_x_start = if blk_y % 2 == 0 || !meander {
                0
            } else {
                self.blk_x.get() - 1
            };
            if self.blk_scan_dir == 1 {
                self.x[0] = src_frame.planes[0].hpad as isize;
                if self.chroma {
                    self.x[1] = src_frame.planes[1].hpad as isize;
                    self.x[2] = src_frame.planes[2].hpad as isize;
                }
            } else {
                // start with rightmost block, but it is already set at prev row
                self.x[0] = (src_frame.planes[0].hpad
                    + (self.blk_size_x.get() - self.overlap_x) * (self.blk_x.get() - 1))
                    as isize;
                if self.chroma {
                    self.x[1] = (src_frame.planes[1].hpad
                        + (self.blk_size_x.get() - self.overlap_x) / self.x_ratio_uv.get() as usize
                            * (self.blk_x.get() - 1)) as isize;
                    self.x[2] = (src_frame.planes[2].hpad
                        + (self.blk_size_x.get() - self.overlap_x) / self.x_ratio_uv.get() as usize
                            * (self.blk_x.get() - 1)) as isize;
                }
            }

            for iblk_x in 0..self.blk_x.get() {
                let blk_x =
                    (blk_x_start as isize + iblk_x as isize * self.blk_scan_dir as isize) as usize;
                self.blk_x_i = blk_x;
                self.blk_idx = (blk_y * self.blk_x.get()) + blk_x;

                self.src_offset[0] = src_frame.planes[0].get_pel_offset(self.x[0], self.y[0]);
                if self.chroma {
                    self.src_offset[1] = src_frame.planes[1].get_pel_offset(self.x[1], self.y[1]);
                    self.src_offset[2] = src_frame.planes[2].get_pel_offset(self.x[2], self.y[2]);
                }
                // In the C version they copy to a temp aligned array here.
                // I don't think we need that since we are not using x264's ASM,
                // and it's probably better for performance to not need to copy the data.
                self.src_pitch[0] = src_frame.planes[0].pitch;
                if self.chroma {
                    self.src_pitch[1] = src_frame.planes[1].pitch;
                    self.src_pitch[2] = src_frame.planes[2].pitch;
                }

                // TODO: (from C) should these be scaled by pel?
                self.lambda = if blk_y == 0 { 0 } else { lambda_level };
                self.penalty_new = penalty_new;
                self.lambda_sad = lambda_sad;

                // decreased padding of coarse levels
                let hpad_scaled = src_frame.planes[0].hpad >> self.log_scale;
                let vpad_scaled = src_frame.planes[0].vpad >> self.log_scale;

                // compute search boundaries
                self.dx_max = (src_frame.planes[0].padded_width.get() as isize
                    - self.x[0]
                    - self.blk_size_x.get() as isize
                    - src_frame.planes[0].hpad as isize
                    + hpad_scaled as isize)
                    << LOG_PEL;
                self.dy_max = (src_frame.planes[0].padded_height.get() as isize
                    - self.y[0]
                    - self.blk_size_y.get() as isize
                    - src_frame.planes[0].vpad as isize
                    + vpad_scaled as isize)
                    << LOG_PEL;
                self.dx_min = -((self.x[0] - src_frame.planes[0].hpad as isize
                    + hpad_scaled as isize)
                    << LOG_PEL);
                self.dy_min = -((self.y[0] - src_frame.planes[0].vpad as isize
                    + vpad_scaled as isize)
                    << LOG_PEL);

                // search the MV
                self.predictor = self.clip_mv(self.vectors[self.blk_idx]);
                self.predictors[4] = self.clip_mv(MotionVector::zero());

                self.pseudo_epz_search::<DCT_MODE, LOG_PEL>(
                    src_frame_data,
                    ref_frame,
                    ref_frame_data,
                )?;

                // write the results
                blk_data[blk_data_offset] = self.best_mv;
                blk_data_offset += 1;

                if self.smallest_plane {
                    self.sum_luma_change += luma_sum(
                        self.blk_size_x,
                        self.blk_size_y,
                        self.get_ref_block::<LOG_PEL>(ref_frame, ref_frame_data, 0, 0)?,
                        self.ref_pitch[0],
                    ) as i64
                        - luma_sum(
                            self.blk_size_x,
                            self.blk_size_y,
                            plane_with_padding::<T>(src_frame_data, 0)?,
                            self.src_pitch[0],
                        ) as i64;
                }

                // increment indexes
                if iblk_x < self.blk_x.get() - 1 {
                    self.x[0] += (self.blk_size_x.get() - self.overlap_x) as isize
                        * self.blk_scan_dir as isize;
                    if (src_frame.yuv_mode & MVPlaneSet::UPLANE).bits() > 0 {
                        self.x[1] += ((self.blk_size_x.get() - self.overlap_x)
                            >> self.log_x_ratio_uv) as isize
                            * self.blk_scan_dir as isize;
                    }
                    if (src_frame.yuv_mode & MVPlaneSet::VPLANE).bits() > 0 {
                        self.x[2] += ((self.blk_size_x.get() - self.overlap_x)
                            >> self.log_x_ratio_uv) as isize
                            * self.blk_scan_dir as isize;
                    }
                }
            }
            self.y[0] += (self.blk_size_y.get() - self.overlap_y) as isize;
            if (src_frame.yuv_mode & MVPlaneSet::UPLANE).bits() > 0 {
                self.y[0] +=
                    ((self.blk_size_y.get() - self.overlap_y) >> self.log_y_ratio_uv) as isize;
            }
            if (src_frame.yuv_mode & MVPlaneSet::VPLANE).bits() > 0 {
                self.y[0] +=
                    ((self.blk_size_y.get() - self.overlap_y) >> self.log_y_ratio_uv) as isize;
            }
        }

        if self.smallest_plane {
            *mean_luma_change = (self.sum_luma_change / self.blk_count.get() as i64) as i32;
        }

        Ok(())
    }

    #[must_use]
    pub(crate) fn get_array_size(&self, divide: DivideMode) -> NonZeroUsize {
        let mut len = self.blk_count;
        if self.log_scale == 0 && divide != DivideMode::None {
            // reserve space for divided subblocks extra level
            len = len.saturating_add(self.blk_count.get() * 4);
        }
        len
    }

    pub(crate) fn estimate_global_mv_doubled(&mut self, mv: MotionVector) {
        todo!()
    }

    pub(crate) fn interpolate_prediction(&mut self, other: &Self) {
        todo!()
    }

    #[must_use]
    fn clip_mv(&self, v: MotionVector) -> MotionVector {
        MotionVector {
            x: self.clip_mv_x(v.x),
            y: self.clip_mv_y(v.y),
            sad: v.sad,
        }
    }

    #[must_use]
    fn clip_mv_x(&self, x: isize) -> isize {
        min(max(x, self.dx_min), self.dx_max - 1)
    }

    #[must_use]
    fn clip_mv_y(&self, y: isize) -> isize {
        min(max(y, self.dy_min), self.dy_max - 1)
    }

    fn pseudo_epz_search<const DCT_MODE: u8, const LOG_PEL: usize>(
        &mut self,
        src_frame_data: &Frame,
        ref_frame: &MVFrame,
        ref_frame_data: &Frame,
    ) -> Result<()> {
        let src_plane_y = plane_with_padding(src_frame_data, 0)?;
        let src_plane_u = if self.chroma {
            plane_with_padding(src_frame_data, 1)?
        } else {
            &[]
        };
        let src_plane_v = if self.chroma {
            plane_with_padding(src_frame_data, 2)?
        } else {
            &[]
        };
        let src_planes = [src_plane_y, src_plane_u, src_plane_v];

        self.fetch_predictors();

        if (1..=4).contains(&DCT_MODE) {
            // make dct of source block
            // don't do the slow dct conversion if SATD used
            self.dct.as_mut().unwrap().bytes_2d(
                src_plane_y,
                self.src_pitch[0],
                &mut self.dct_src,
                self.dct_pitch,
            )?;
        }

        if DCT_MODE >= 3 {
            // most use it and it should be fast anyway
            // TODO: Do we only need this for modes 3 and 4?
            self.src_luma = luma_sum(
                self.blk_size_x,
                self.blk_size_y,
                &src_plane_y[self.src_offset[0]..],
                self.src_pitch[0],
            )
        }

        // We treat zero alone
        // Do we bias zero with not taking into account distortion ?
        self.best_mv.x = self.zero_mv_field_shifted.x;
        self.best_mv.y = self.zero_mv_field_shifted.y;
        // Compute zero MV blocks
        let mut sad = self.luma_sad::<DCT_MODE>(
            src_plane_y,
            self.src_pitch[0],
            self.get_ref_block::<LOG_PEL>(
                ref_frame,
                ref_frame_data,
                0,
                self.zero_mv_field_shifted.y,
            )?,
            self.ref_pitch[0],
        );
        if self.chroma {
            sad += self.chroma_sad(
                src_plane_u,
                self.src_pitch[1],
                self.get_ref_block_u::<LOG_PEL>(ref_frame, ref_frame_data, 0, 0)?,
                self.ref_pitch[1],
            );
            sad += self.chroma_sad(
                src_plane_v,
                self.src_pitch[2],
                self.get_ref_block_v::<LOG_PEL>(ref_frame, ref_frame_data, 0, 0)?,
                self.ref_pitch[2],
            );
        }
        self.best_mv.sad = sad as i64;
        self.min_cost = (sad + ((self.penalty_zero as u64 * sad) >> 8)) as i64;

        let mut best_mv_many = [MotionVector::zero(); 8];
        let mut min_cost_many = [0; 8];
        if self.try_many {
            // refine around zero
            self.refine::<DCT_MODE, LOG_PEL>(src_planes, ref_frame, ref_frame_data)?;
            best_mv_many[0] = self.best_mv;
            min_cost_many[0] = self.min_cost;
        }

        // Global MV predictor
        self.global_mv_predictor = self.clip_mv(self.global_mv_predictor);
        let mut sad = self.luma_sad::<DCT_MODE>(
            src_plane_y,
            self.src_pitch[0],
            self.get_ref_block::<LOG_PEL>(
                ref_frame,
                ref_frame_data,
                self.global_mv_predictor.x,
                self.global_mv_predictor.y,
            )?,
            self.ref_pitch[0],
        );
        if self.chroma {
            sad += self.chroma_sad(
                src_plane_u,
                self.src_pitch[1],
                self.get_ref_block_u::<LOG_PEL>(
                    ref_frame,
                    ref_frame_data,
                    self.global_mv_predictor.x,
                    self.global_mv_predictor.y,
                )?,
                self.ref_pitch[1],
            );
            sad += self.chroma_sad(
                src_plane_v,
                self.src_pitch[2],
                self.get_ref_block_v::<LOG_PEL>(
                    ref_frame,
                    ref_frame_data,
                    self.global_mv_predictor.x,
                    self.global_mv_predictor.y,
                )?,
                self.ref_pitch[2],
            );
        }
        let cost = (sad + ((self.penalty_global as u64 * sad) >> 8)) as i64;

        if cost < self.min_cost || self.try_many {
            self.best_mv.x = self.global_mv_predictor.x;
            self.best_mv.y = self.global_mv_predictor.y;
            self.best_mv.sad = sad as i64;
            self.min_cost = cost;
        }
        if self.try_many {
            // refine around global
            self.refine::<DCT_MODE, LOG_PEL>(src_planes, ref_frame, ref_frame_data)?;
            best_mv_many[1] = self.best_mv;
            min_cost_many[1] = self.min_cost;
        }

        // Predictor blocks
        let mut sad = self.luma_sad::<DCT_MODE>(
            src_plane_y,
            self.src_pitch[0],
            self.get_ref_block::<LOG_PEL>(
                ref_frame,
                ref_frame_data,
                self.predictor.x,
                self.predictor.y,
            )?,
            self.ref_pitch[0],
        );
        if self.chroma {
            sad += self.chroma_sad(
                src_plane_u,
                self.src_pitch[1],
                self.get_ref_block_u::<LOG_PEL>(
                    ref_frame,
                    ref_frame_data,
                    self.predictor.x,
                    self.predictor.y,
                )?,
                self.ref_pitch[1],
            );
            sad += self.chroma_sad(
                src_plane_v,
                self.src_pitch[2],
                self.get_ref_block_v::<LOG_PEL>(
                    ref_frame,
                    ref_frame_data,
                    self.predictor.x,
                    self.predictor.y,
                )?,
                self.ref_pitch[2],
            );
        }
        let cost = sad;

        if (cost as i64) < self.min_cost || self.try_many {
            self.best_mv.x = self.predictor.x;
            self.best_mv.y = self.predictor.y;
            self.best_mv.sad = sad as i64;
            self.min_cost = cost as i64;
        }
        if self.try_many {
            // refine around predictor
            self.refine::<DCT_MODE, LOG_PEL>(src_planes, ref_frame, ref_frame_data)?;
            best_mv_many[2] = self.best_mv;
            min_cost_many[2] = self.min_cost;
        }

        // then all the other predictors
        let npred = 4;
        for i in 0..npred {
            if self.try_many {
                self.min_cost = self.very_big_sad.get() as i64 + 1;
            }
            self.check_mv0::<DCT_MODE, LOG_PEL>(
                src_planes,
                ref_frame,
                ref_frame_data,
                self.predictors[i].x,
                self.predictors[i].y,
            )?;

            if self.try_many {
                // refine around predictor
                self.refine::<DCT_MODE, LOG_PEL>(src_planes, ref_frame, ref_frame_data)?;
                best_mv_many[i + 3] = self.best_mv;
                min_cost_many[i + 3] = self.min_cost;
            }
        }

        if self.try_many {
            self.min_cost = self.very_big_sad.get() as i64 + 1;
            for i in 0..(npred + 3) {
                if min_cost_many[i] < self.min_cost {
                    self.best_mv = best_mv_many[i];
                    self.min_cost = min_cost_many[i];
                }
            }
        } else {
            self.refine::<DCT_MODE, LOG_PEL>(src_planes, ref_frame, ref_frame_data)?;
        }

        let found_sad = self.best_mv.sad;
        const BADCOUNT_LIMIT: u64 = 16;
        if self.blk_idx > 1
            && found_sad
                > ((self.bad_sad + self.bad_sad * self.bad_count as u64 / BADCOUNT_LIMIT) as i64)
        {
            // bad vector, try wide search with some soft limit of bad cured vectors (time consumed)
            self.bad_count += 1;

            if self.bad_range > 0 {
                // UMH, good mv not found so try around zero
                self.umh_search::<DCT_MODE, LOG_PEL>(
                    src_planes,
                    ref_frame,
                    ref_frame_data,
                    self.bad_range * (1 << LOG_PEL),
                    0,
                    0,
                )?;
            } else if self.bad_range < 0 {
                // ESA
                for i in (1..(-self.bad_range * (1 << LOG_PEL))).step_by(1 << LOG_PEL) {
                    // at radius
                    self.expanding_search::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        i,
                        1 << LOG_PEL,
                        0,
                        0,
                    )?;
                    if self.best_mv.sad < found_sad / 4 {
                        // stop search if good MV is found
                        break;
                    }
                }
            }

            for i in 1..(1 << LOG_PEL) {
                // small radius
                self.expanding_search::<DCT_MODE, LOG_PEL>(
                    src_planes,
                    ref_frame,
                    ref_frame_data,
                    i,
                    1,
                    self.best_mv.x,
                    self.best_mv.y,
                )?;
            }
        }

        // store the result
        self.vectors[self.blk_idx] = self.best_mv;
        Ok(())
    }

    fn get_ref_block<'a, const LOG_PEL: usize>(
        &self,
        ref_frame: &MVFrame,
        ref_frame_data: &'a Frame,
        vx: isize,
        vy: isize,
    ) -> Result<&'a [T]> {
        let plane = plane_with_padding(ref_frame_data, 0)?;
        let mvplane = &ref_frame.planes[0];
        let offset = match LOG_PEL {
            0 => mvplane.get_absolute_offset_pel1(self.x[0] + vx, self.y[0] + vy),
            1 => mvplane.get_absolute_offset_pel2(self.x[0] * 2 + vx, self.y[0] * 2 + vy),
            2 => mvplane.get_absolute_offset_pel4(self.x[0] * 4 + vx, self.y[0] * 4 + vy),
            _ => unreachable!(),
        };
        Ok(&plane[offset..])
    }
    fn get_ref_block_u<'a, const LOG_PEL: usize>(
        &self,
        ref_frame: &MVFrame,
        ref_frame_data: &'a Frame,
        vx: isize,
        vy: isize,
    ) -> Result<&'a [T]> {
        self.get_ref_block_chroma::<LOG_PEL>(ref_frame, ref_frame_data, vx, vy, 1)
    }

    fn get_ref_block_v<'a, const LOG_PEL: usize>(
        &self,
        ref_frame: &MVFrame,
        ref_frame_data: &'a Frame,
        vx: isize,
        vy: isize,
    ) -> Result<&'a [T]> {
        self.get_ref_block_chroma::<LOG_PEL>(ref_frame, ref_frame_data, vx, vy, 2)
    }

    fn get_ref_block_chroma<'a, const LOG_PEL: usize>(
        &self,
        ref_frame: &MVFrame,
        ref_frame_data: &'a Frame,
        vx: isize,
        vy: isize,
        plane_idx: usize,
    ) -> Result<&'a [T]> {
        let xbias = if vx < 0 { -1 } else { 1 } * ((1 << self.log_x_ratio_uv) - 1);
        let ybias = if vy < 0 { -1 } else { 1 } * ((1 << self.log_y_ratio_uv) - 1);

        let plane = plane_with_padding(ref_frame_data, plane_idx)?;
        let mvplane = &ref_frame.planes[plane_idx];
        let offset = match LOG_PEL {
            0 => mvplane.get_absolute_offset_pel1(
                self.x[plane_idx] + ((vx + xbias) >> self.log_x_ratio_uv),
                self.y[plane_idx] + ((vy + ybias) >> self.log_y_ratio_uv),
            ),
            1 => mvplane.get_absolute_offset_pel2(
                self.x[plane_idx] * 2 + ((vx + xbias) >> self.log_x_ratio_uv),
                self.y[plane_idx] * 2 + ((vy + ybias) >> self.log_y_ratio_uv),
            ),
            2 => mvplane.get_absolute_offset_pel4(
                self.x[plane_idx] * 4 + ((vx + xbias) >> self.log_x_ratio_uv),
                self.y[plane_idx] * 4 + ((vy + ybias) >> self.log_y_ratio_uv),
            ),
            _ => unreachable!(),
        };
        Ok(&plane[offset..])
    }

    fn fetch_predictors(&mut self) {
        // Left (or right) predictor
        if (self.blk_scan_dir == 1 && self.blk_x_i > 0)
            || (self.blk_scan_dir == -1 && self.blk_x_i < self.blk_x.get() - 1)
        {
            self.predictors[1] = self.clip_mv(
                self.vectors[(self.blk_idx as isize - self.blk_scan_dir as isize) as usize],
            );
        } else {
            self.predictors[1] = self.clip_mv(self.zero_mv_field_shifted);
        }

        // Up predictor
        if self.blk_y_i > 0 {
            self.predictors[2] = self.clip_mv(self.vectors[self.blk_idx - self.blk_x.get()]);
        } else {
            self.predictors[2] = self.clip_mv(self.zero_mv_field_shifted);
        }

        // bottom-right pridictor (from coarse level)
        if (self.blk_y_i < self.blk_y.get() - 1)
            && ((self.blk_scan_dir == 1 && self.blk_x_i < self.blk_x.get() - 1)
                || (self.blk_scan_dir == -1 && self.blk_x_i > 0))
        {
            self.predictors[3] = self.clip_mv(
                self.vectors[((self.blk_idx + self.blk_x.get()) as isize
                    + self.blk_scan_dir as isize) as usize],
            );
        } else if (self.blk_y_i > 0)
            && ((self.blk_scan_dir == 1 && self.blk_x_i < self.blk_x.get() - 1)
                || (self.blk_scan_dir == -1 && self.blk_x_i > 0))
        {
            // Up-right predictor
            self.predictors[3] = self.clip_mv(
                self.vectors[(self.blk_idx as isize - self.blk_x.get() as isize
                    + self.blk_scan_dir as isize) as usize],
            );
        } else {
            self.predictors[3] = self.clip_mv(self.zero_mv_field_shifted);
        }

        // Median predictor
        if self.blk_y_i > 0 {
            // replaced 1 by 0 - Fizick
            self.predictors[0].x = median(
                self.predictors[1].x,
                self.predictors[2].x,
                self.predictors[3].x,
            );
            self.predictors[0].y = median(
                self.predictors[1].y,
                self.predictors[2].y,
                self.predictors[3].y,
            );
            // but it is not true median vector (x and y may be mixed) and not its sad.
            // we really do not know SAD, here is more safe estimation especially for
            // phaseshift method
            self.predictors[0].sad = max(
                self.predictors[1].sad,
                max(self.predictors[2].sad, self.predictors[3].sad),
            );
        } else {
            // but for top line we have only predictor[1] left
            self.predictors[0] = self.predictors[1];
        }

        // if there are no other planes, predictor is the median
        if self.smallest_plane {
            self.predictor = self.predictors[0];
        }
        let scale = self.lambda_sad as i64 / (self.lambda_sad as i64 + (self.predictor.sad >> 1));
        self.lambda = (self.lambda as i64 * scale * scale) as u32;
    }

    #[must_use]
    fn luma_sad<const DCT_MODE: u8>(
        &mut self,
        src_plane: &[T],
        src_pitch: NonZeroUsize,
        ref_plane: &[T],
        ref_pitch: NonZeroUsize,
    ) -> u64 {
        let dct_mode = DctMode::try_from(DCT_MODE as i64).expect("invalid dct mode");
        match dct_mode {
            DctMode::Spatial => get_sad(
                self.blk_size_x,
                self.blk_size_y,
                src_plane,
                src_pitch,
                ref_plane,
                ref_pitch,
            ),
            DctMode::Dct => self.reduction_corrected_dct(ref_plane, ref_pitch),
            DctMode::MixedSpatialDct => {
                let sad = get_sad(
                    self.blk_size_x,
                    self.blk_size_y,
                    src_plane,
                    src_pitch,
                    ref_plane,
                    ref_pitch,
                );
                let dct_sad = if self.dct_weight_16 > 0 {
                    self.reduction_corrected_dct(ref_plane, ref_pitch)
                } else {
                    0
                };
                (sad * (16 - self.dct_weight_16 as u64) + dct_sad * self.dct_weight_16 as u64) / 16
            }
            DctMode::AdaptiveSpatialMixed => {
                self.ref_luma = luma_sum(self.blk_size_x, self.blk_size_y, ref_plane, ref_pitch);
                let sad = get_sad(
                    self.blk_size_x,
                    self.blk_size_y,
                    src_plane,
                    src_pitch,
                    ref_plane,
                    ref_pitch,
                );
                if (self.src_luma as i64 - self.ref_luma as i64).unsigned_abs()
                    > ((self.src_luma + self.ref_luma) >> 5)
                {
                    let dct_sad = self.bsize_corrected_dct(ref_plane, ref_pitch);
                    sad / 2 + dct_sad / 2
                } else {
                    sad
                }
            }
            DctMode::AdaptiveSpatialDct => {
                self.ref_luma = luma_sum(self.blk_size_x, self.blk_size_y, ref_plane, ref_pitch);
                let sad = get_sad(
                    self.blk_size_x,
                    self.blk_size_y,
                    src_plane,
                    src_pitch,
                    ref_plane,
                    ref_pitch,
                );
                if (self.src_luma as i64 - self.ref_luma as i64).unsigned_abs()
                    > ((self.src_luma + self.ref_luma) >> 5)
                {
                    let dct_sad = self.bsize_corrected_dct(ref_plane, ref_pitch);
                    sad / 4 + dct_sad / 2 + dct_sad / 4
                } else {
                    sad
                }
            }
            DctMode::Satd => get_satd(
                self.blk_size_x,
                self.blk_size_y,
                src_plane,
                src_pitch,
                ref_plane,
                ref_pitch,
            ),
            DctMode::MixedSatdDct => {
                let sad = get_sad(
                    self.blk_size_x,
                    self.blk_size_y,
                    src_plane,
                    src_pitch,
                    ref_plane,
                    ref_pitch,
                );
                if self.dct_weight_16 > 0 {
                    let dct_sad = get_satd(
                        self.blk_size_x,
                        self.blk_size_y,
                        src_plane,
                        src_pitch,
                        ref_plane,
                        ref_pitch,
                    );
                    (sad * (16 - self.dct_weight_16 as u64) + dct_sad * self.dct_weight_16 as u64)
                        / 16
                } else {
                    sad
                }
            }
            DctMode::AdaptiveSatdMixed => {
                self.ref_luma = luma_sum(self.blk_size_x, self.blk_size_y, ref_plane, ref_pitch);
                let sad = get_sad(
                    self.blk_size_x,
                    self.blk_size_y,
                    src_plane,
                    src_pitch,
                    ref_plane,
                    ref_pitch,
                );
                if (self.src_luma as i64 - self.ref_luma as i64).unsigned_abs()
                    > ((self.src_luma + self.ref_luma) >> 5)
                {
                    let dct_sad = get_satd(
                        self.blk_size_x,
                        self.blk_size_y,
                        src_plane,
                        src_pitch,
                        ref_plane,
                        ref_pitch,
                    );
                    sad / 2 + dct_sad / 2
                } else {
                    sad
                }
            }
            DctMode::AdaptiveSatdDct => {
                self.ref_luma = luma_sum(self.blk_size_x, self.blk_size_y, ref_plane, ref_pitch);
                let sad = get_sad(
                    self.blk_size_x,
                    self.blk_size_y,
                    src_plane,
                    src_pitch,
                    ref_plane,
                    ref_pitch,
                );
                if (self.src_luma as i64 - self.ref_luma as i64).unsigned_abs()
                    > ((self.src_luma + self.ref_luma) >> 5)
                {
                    let dct_sad = get_satd(
                        self.blk_size_x,
                        self.blk_size_y,
                        src_plane,
                        src_pitch,
                        ref_plane,
                        ref_pitch,
                    );
                    sad / 4 + dct_sad / 2 + dct_sad / 4
                } else {
                    sad
                }
            }
            DctMode::MixedSadEqSatdDct => {
                let sad = get_sad(
                    self.blk_size_x,
                    self.blk_size_y,
                    src_plane,
                    src_pitch,
                    ref_plane,
                    ref_pitch,
                );
                if self.dct_weight_16 > 1 {
                    let dct_weight_half = self.dct_weight_16 as u64 / 2;
                    let dct_sad = get_satd(
                        self.blk_size_x,
                        self.blk_size_y,
                        src_plane,
                        src_pitch,
                        ref_plane,
                        ref_pitch,
                    );
                    (sad * (16 - dct_weight_half) + dct_sad * dct_weight_half) / 16
                } else {
                    sad
                }
            }
            DctMode::AdaptiveSatdLuma => {
                self.ref_luma = luma_sum(self.blk_size_x, self.blk_size_y, ref_plane, ref_pitch);
                let sad = get_sad(
                    self.blk_size_x,
                    self.blk_size_y,
                    src_plane,
                    src_pitch,
                    ref_plane,
                    ref_pitch,
                );
                if (self.src_luma as i64 - self.ref_luma as i64).unsigned_abs()
                    > ((self.src_luma + self.ref_luma) >> 4)
                {
                    let dct_sad = get_satd(
                        self.blk_size_x,
                        self.blk_size_y,
                        src_plane,
                        src_pitch,
                        ref_plane,
                        ref_pitch,
                    );
                    sad / 2 + dct_sad / 4 + sad / 4
                } else {
                    sad
                }
            }
        }
    }

    #[must_use]
    #[inline(always)]
    fn chroma_sad(
        &self,
        src_plane: &[T],
        src_pitch: NonZeroUsize,
        ref_plane: &[T],
        ref_pitch: NonZeroUsize,
    ) -> u64 {
        // Just use basic SAD algorithm for chroma because it's faster
        get_sad(
            // sAFETY: all values are NonZero typed
            unsafe {
                NonZeroUsize::new_unchecked(self.blk_size_x.get() / self.x_ratio_uv.get() as usize)
            },
            // sAFETY: all values are NonZero typed
            unsafe {
                NonZeroUsize::new_unchecked(self.blk_size_y.get() / self.y_ratio_uv.get() as usize)
            },
            src_plane,
            src_pitch,
            ref_plane,
            ref_pitch,
        )
    }

    #[must_use]
    fn bsize_corrected_dct(&mut self, ref_plane: &[T], ref_pitch: NonZeroUsize) -> u64 {
        self.dct
            .as_mut()
            .expect("dct helper should be defined")
            .bytes_2d(ref_plane, ref_pitch, &mut self.dct_ref, self.dct_pitch)
            .expect("dct should not fail with valid params");

        get_sad(
            self.blk_size_x,
            self.blk_size_y,
            &self.dct_src,
            self.dct_pitch,
            &self.dct_ref,
            self.dct_pitch,
        ) * self.blk_size_x.get() as u64
            / 2
    }

    #[must_use]
    fn reduction_corrected_dct(&mut self, ref_plane: &[T], ref_pitch: NonZeroUsize) -> u64 {
        self.dct
            .as_mut()
            .expect("dct helper should be defined")
            .bytes_2d(ref_plane, ref_pitch, &mut self.dct_ref, self.dct_pitch)
            .expect("dct should not fail with valid params");

        // correct reduced DC component
        let src0: i64 = self.dct_src[0].into();
        let ref0: i64 = self.dct_ref[0].into();
        get_sad(
            self.blk_size_x,
            self.blk_size_y,
            &self.dct_src,
            self.dct_pitch,
            &self.dct_ref,
            self.dct_pitch,
        ) + (src0 - ref0).unsigned_abs() * 3 * self.blk_size_x.get() as u64 / 2
    }

    fn refine<const DCT_MODE: u8, const LOG_PEL: usize>(
        &mut self,
        src_planes: [&[T]; 3],
        ref_frame: &MVFrame,
        ref_frame_data: &Frame,
    ) -> Result<()> {
        match self.search_type {
            SearchType::Onetime => {
                let mut i = self.search_param;
                while i > 0 {
                    self.one_time_search::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        i as isize,
                    )?;
                    i /= 2;
                }
            }
            SearchType::Nstep => {
                self.n_step_search::<DCT_MODE, LOG_PEL>(
                    src_planes,
                    ref_frame,
                    ref_frame_data,
                    self.search_param as isize,
                )?;
            }
            SearchType::Logarithmic => {
                let mut i = self.search_param;
                while i > 0 {
                    self.diamond_search::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        i as isize,
                    )?;
                    i /= 2;
                }
            }
            SearchType::Exhaustive => {
                let mvx = self.best_mv.x;
                let mvy = self.best_mv.y;
                for i in 1..=self.search_param {
                    // region is same as enhausted, but ordered by radius (from near to far)
                    self.expanding_search::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        i as isize,
                        1,
                        mvx,
                        mvy,
                    )?;
                }
            }
            SearchType::Hex2 => {
                self.hex2_search::<DCT_MODE, LOG_PEL>(
                    src_planes,
                    ref_frame,
                    ref_frame_data,
                    self.search_param as isize,
                )?;
            }
            SearchType::UnevenMultiHexagon => {
                self.umh_search::<DCT_MODE, LOG_PEL>(
                    src_planes,
                    ref_frame,
                    ref_frame_data,
                    self.search_param as isize,
                    self.best_mv.x,
                    self.best_mv.y,
                )?;
            }
            SearchType::Horizontal => {
                let mvx = self.best_mv.x;
                let mvy = self.best_mv.y;
                for i in 1..=self.search_param {
                    self.check_mv::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        mvx - i as isize,
                        mvy,
                    )?;
                    self.check_mv::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        mvx + i as isize,
                        mvy,
                    )?;
                }
            }
            SearchType::Vertical => {
                let mvx = self.best_mv.x;
                let mvy = self.best_mv.y;
                for i in 1..=self.search_param {
                    self.check_mv::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        mvx,
                        mvy - i as isize,
                    )?;
                    self.check_mv::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        mvx,
                        mvy + i as isize,
                    )?;
                }
            }
        }

        Ok(())
    }

    /// check if the vector (vx, vy) is better than the best vector found so far without penalty new
    #[inline(always)]
    fn check_mv0<const DCT_MODE: u8, const LOG_PEL: usize>(
        &mut self,
        src_planes: [&[T]; 3],
        ref_frame: &MVFrame,
        ref_frame_data: &Frame,
        vx: isize,
        vy: isize,
    ) -> Result<()> {
        // here the chance for default values are high especially
        // for zeroMVfieldShifted (on left/top border)
        self.check_mv_impl::<DCT_MODE, LOG_PEL, { CheckMVFlags::UPDATE_BEST_MV.bits() }>(
            src_planes,
            ref_frame,
            ref_frame_data,
            vx,
            vy,
            &mut 0,
            0,
        )
    }

    /// check if the vector (vx, vy) is better than the best vector found so far
    #[inline(always)]
    fn check_mv<const DCT_MODE: u8, const LOG_PEL: usize>(
        &mut self,
        src_planes: [&[T]; 3],
        ref_frame: &MVFrame,
        ref_frame_data: &Frame,
        vx: isize,
        vy: isize,
    ) -> Result<()> {
        // here the chance for default values are high especially
        // for zeroMVfieldShifted (on left/top border)
        self.check_mv_impl::<DCT_MODE, LOG_PEL, { CheckMVFlags::PENALTY_NEW.bits() | CheckMVFlags::UPDATE_BEST_MV.bits() }>(
            src_planes,ref_frame,ref_frame_data,vx, vy, &mut 0, 0,
        )
    }

    /// check if the vector (vx, vy) is better, and update dir accordingly
    #[inline(always)]
    fn check_mv2<const DCT_MODE: u8, const LOG_PEL: usize>(
        &mut self,
        src_planes: [&[T]; 3],
        ref_frame: &MVFrame,
        ref_frame_data: &Frame,
        vx: isize,
        vy: isize,
        direction: &mut isize,
        val: isize,
    ) -> Result<()> {
        self.check_mv_impl::<DCT_MODE, LOG_PEL, {
            CheckMVFlags::PENALTY_NEW.bits()
                | CheckMVFlags::UPDATE_DIR.bits()
                | CheckMVFlags::UPDATE_BEST_MV.bits()
        }>(
            src_planes,
            ref_frame,
            ref_frame_data,
            vx,
            vy,
            direction,
            val,
        )
    }

    /// check if the vector (vx, vy) is better, and update dir accordingly, but not bestMV.x, y
    #[inline(always)]
    fn check_mv_dir<const DCT_MODE: u8, const LOG_PEL: usize>(
        &mut self,
        src_planes: [&[T]; 3],
        ref_frame: &MVFrame,
        ref_frame_data: &Frame,
        vx: isize,
        vy: isize,
        direction: &mut isize,
        val: isize,
    ) -> Result<()> {
        self.check_mv_impl::<DCT_MODE, LOG_PEL, {
            CheckMVFlags::PENALTY_NEW.bits()
                | CheckMVFlags::UPDATE_DIR.bits()
        }>(src_planes, ref_frame, ref_frame_data, vx, vy, direction, val)
    }

    fn check_mv_impl<const DCT_MODE: u8, const LOG_PEL: usize, const CHECK_MV_FLAGS: u32>(
        &mut self,
        src_planes: [&[T]; 3],
        ref_frame: &MVFrame,
        ref_frame_data: &Frame,
        vx: isize,
        vy: isize,
        direction: &mut isize,
        val: isize,
    ) -> Result<()> {
        if !self.is_vector_ok(vx, vy) {
            return Ok(());
        }

        let mut cost = self.motion_distortion(vx, vy);
        if cost >= self.min_cost {
            return Ok(());
        }

        let flags = CheckMVFlags::from_bits(CHECK_MV_FLAGS).expect("invalid check mv flags");
        let sad = self.luma_sad::<DCT_MODE>(
            src_planes[0],
            self.src_pitch[0],
            self.get_ref_block::<LOG_PEL>(ref_frame, ref_frame_data, vx, vy)?,
            self.ref_pitch[0],
        ) as i64;
        cost += sad
            + if flags.contains(CheckMVFlags::PENALTY_NEW) {
                (self.penalty_new as i64 * sad) >> 8
            } else {
                0
            };
        if cost >= self.min_cost {
            return Ok(());
        }

        let mut sad_uv = 0;
        if self.chroma {
            sad_uv += self.chroma_sad(
                src_planes[1],
                self.src_pitch[1],
                self.get_ref_block_u::<LOG_PEL>(ref_frame, ref_frame_data, vx, vy)?,
                self.ref_pitch[1],
            ) as i64;
            sad_uv += self.chroma_sad(
                src_planes[2],
                self.src_pitch[2],
                self.get_ref_block_v::<LOG_PEL>(ref_frame, ref_frame_data, vx, vy)?,
                self.ref_pitch[2],
            ) as i64;
            cost += sad_uv
                + if flags.contains(CheckMVFlags::PENALTY_NEW) {
                    (self.penalty_new as i64 * sad_uv) >> 8
                } else {
                    0
                };
            if cost >= self.min_cost {
                return Ok(());
            }
        }

        if flags.contains(CheckMVFlags::UPDATE_BEST_MV) {
            self.best_mv.x = vx;
            self.best_mv.y = vy;
        }
        self.min_cost = cost;
        self.best_mv.sad = sad + sad_uv;
        if flags.contains(CheckMVFlags::UPDATE_DIR) {
            *direction = val;
        }

        Ok(())
    }

    fn one_time_search<const DCT_MODE: u8, const LOG_PEL: usize>(
        &mut self,
        src_planes: [&[T]; 3],
        ref_frame: &MVFrame,
        ref_frame_data: &Frame,
        length: isize,
    ) -> Result<()> {
        let mut direction = 0;
        let mut dx = self.best_mv.x;
        let mut dy = self.best_mv.y;

        self.check_mv2::<DCT_MODE, LOG_PEL>(
            src_planes,
            ref_frame,
            ref_frame_data,
            dx - length,
            dy,
            &mut direction,
            2,
        )?;
        self.check_mv2::<DCT_MODE, LOG_PEL>(
            src_planes,
            ref_frame,
            ref_frame_data,
            dx + length,
            dy,
            &mut direction,
            1,
        )?;

        if direction == 1 {
            while direction > 0 {
                direction = 0;
                dx += length;
                self.check_mv2::<DCT_MODE, LOG_PEL>(
                    src_planes,
                    ref_frame,
                    ref_frame_data,
                    dx + length,
                    dy,
                    &mut direction,
                    1,
                )?;
            }
        } else if direction == 2 {
            while direction > 0 {
                direction = 0;
                dx -= length;
                self.check_mv2::<DCT_MODE, LOG_PEL>(
                    src_planes,
                    ref_frame,
                    ref_frame_data,
                    dx - length,
                    dy,
                    &mut direction,
                    1,
                )?;
            }
        }

        self.check_mv2::<DCT_MODE, LOG_PEL>(
            src_planes,
            ref_frame,
            ref_frame_data,
            dx,
            dy - length,
            &mut direction,
            2,
        )?;
        self.check_mv2::<DCT_MODE, LOG_PEL>(
            src_planes,
            ref_frame,
            ref_frame_data,
            dx,
            dy + length,
            &mut direction,
            1,
        )?;

        if direction == 1 {
            while direction > 0 {
                direction = 0;
                dx += length;
                self.check_mv2::<DCT_MODE, LOG_PEL>(
                    src_planes,
                    ref_frame,
                    ref_frame_data,
                    dx,
                    dy + length,
                    &mut direction,
                    1,
                )?;
            }
        } else if direction == 2 {
            while direction > 0 {
                direction = 0;
                dx -= length;
                self.check_mv2::<DCT_MODE, LOG_PEL>(
                    src_planes,
                    ref_frame,
                    ref_frame_data,
                    dx,
                    dy - length,
                    &mut direction,
                    1,
                )?;
            }
        }

        Ok(())
    }

    fn n_step_search<const DCT_MODE: u8, const LOG_PEL: usize>(
        &mut self,
        src_planes: [&[T]; 3],
        ref_frame: &MVFrame,
        ref_frame_data: &Frame,
        step: isize,
    ) -> Result<()> {
        todo!()
    }

    fn diamond_search<const DCT_MODE: u8, const LOG_PEL: usize>(
        &mut self,
        src_planes: [&[T]; 3],
        ref_frame: &MVFrame,
        ref_frame_data: &Frame,
        length: isize,
    ) -> Result<()> {
        bitflags! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            struct Direction: isize {
                const RIGHT = 1;
                const LEFT = 2;
                const DOWN = 4;
                const UP = 8;
            }
        }

        let mut dx;
        let mut dy;

        let mut direction = Direction::all().bits();
        let mut last_direction;

        while direction > 0 {
            dx = self.best_mv.x;
            dy = self.best_mv.y;
            last_direction = Direction::from_bits(direction).expect("valid direction");
            direction = Direction::empty().bits();

            // First, we look the directions that were hinted by the previous step
            // of the algorithm. If we find one, we add it to the set of directions
            // we'll test next
            if last_direction.contains(Direction::RIGHT) {
                self.check_mv2::<DCT_MODE, LOG_PEL>(
                    src_planes,
                    ref_frame,
                    ref_frame_data,
                    dx + length,
                    dy,
                    &mut direction,
                    Direction::RIGHT.bits(),
                )?;
            }
            if last_direction.contains(Direction::LEFT) {
                self.check_mv2::<DCT_MODE, LOG_PEL>(
                    src_planes,
                    ref_frame,
                    ref_frame_data,
                    dx - length,
                    dy,
                    &mut direction,
                    Direction::LEFT.bits(),
                )?;
            }
            if last_direction.contains(Direction::DOWN) {
                self.check_mv2::<DCT_MODE, LOG_PEL>(
                    src_planes,
                    ref_frame,
                    ref_frame_data,
                    dx,
                    dy + length,
                    &mut direction,
                    Direction::DOWN.bits(),
                )?;
            }
            if last_direction.contains(Direction::UP) {
                self.check_mv2::<DCT_MODE, LOG_PEL>(
                    src_planes,
                    ref_frame,
                    ref_frame_data,
                    dx,
                    dy - length,
                    &mut direction,
                    Direction::UP.bits(),
                )?;
            }

            // If one of the directions improves the SAD,
            // we make further tests on the diagonals
            if direction > 0 {
                last_direction = Direction::from_bits(direction).expect("valid direction");
                dx = self.best_mv.x;
                dy = self.best_mv.y;

                if last_direction.bits() & (Direction::RIGHT.bits() + Direction::LEFT.bits()) > 0 {
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx,
                        dy + length,
                        &mut direction,
                        Direction::DOWN.bits(),
                    )?;
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx,
                        dy - length,
                        &mut direction,
                        Direction::UP.bits(),
                    )?;
                } else {
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx + length,
                        dy,
                        &mut direction,
                        Direction::RIGHT.bits(),
                    )?;
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx - length,
                        dy,
                        &mut direction,
                        Direction::LEFT.bits(),
                    )?;
                }
            } else {
                // If not, we do not stop here. We infer from the last direction the
                // diagonals to be checked, because we might be lucky.
                if last_direction.bits() == Direction::RIGHT.bits() {
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx + length,
                        dy + length,
                        &mut direction,
                        Direction::RIGHT.bits() + Direction::DOWN.bits(),
                    )?;
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx + length,
                        dy - length,
                        &mut direction,
                        Direction::RIGHT.bits() + Direction::UP.bits(),
                    )?;
                } else if last_direction.bits() == Direction::LEFT.bits() {
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx - length,
                        dy + length,
                        &mut direction,
                        Direction::LEFT.bits() + Direction::DOWN.bits(),
                    )?;
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx - length,
                        dy - length,
                        &mut direction,
                        Direction::LEFT.bits() + Direction::UP.bits(),
                    )?;
                } else if last_direction.bits() == Direction::DOWN.bits() {
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx + length,
                        dy + length,
                        &mut direction,
                        Direction::RIGHT.bits() + Direction::DOWN.bits(),
                    )?;
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx - length,
                        dy + length,
                        &mut direction,
                        Direction::LEFT.bits() + Direction::DOWN.bits(),
                    )?;
                } else if last_direction.bits() == Direction::UP.bits() {
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx + length,
                        dy - length,
                        &mut direction,
                        Direction::RIGHT.bits() + Direction::UP.bits(),
                    )?;
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx - length,
                        dy - length,
                        &mut direction,
                        Direction::LEFT.bits() + Direction::UP.bits(),
                    )?;
                } else if last_direction.bits() == Direction::RIGHT.bits() + Direction::DOWN.bits()
                {
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx + length,
                        dy + length,
                        &mut direction,
                        Direction::RIGHT.bits() + Direction::DOWN.bits(),
                    )?;
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx - length,
                        dy + length,
                        &mut direction,
                        Direction::LEFT.bits() + Direction::DOWN.bits(),
                    )?;
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx + length,
                        dy - length,
                        &mut direction,
                        Direction::RIGHT.bits() + Direction::UP.bits(),
                    )?;
                } else if last_direction.bits() == Direction::LEFT.bits() + Direction::DOWN.bits() {
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx + length,
                        dy + length,
                        &mut direction,
                        Direction::RIGHT.bits() + Direction::DOWN.bits(),
                    )?;
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx - length,
                        dy + length,
                        &mut direction,
                        Direction::LEFT.bits() + Direction::DOWN.bits(),
                    )?;
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx - length,
                        dy - length,
                        &mut direction,
                        Direction::LEFT.bits() + Direction::UP.bits(),
                    )?;
                } else if last_direction.bits() == Direction::RIGHT.bits() + Direction::UP.bits() {
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx + length,
                        dy + length,
                        &mut direction,
                        Direction::RIGHT.bits() + Direction::DOWN.bits(),
                    )?;
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx - length,
                        dy - length,
                        &mut direction,
                        Direction::LEFT.bits() + Direction::UP.bits(),
                    )?;
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx + length,
                        dy - length,
                        &mut direction,
                        Direction::RIGHT.bits() + Direction::UP.bits(),
                    )?;
                } else if last_direction.bits() == Direction::LEFT.bits() + Direction::UP.bits() {
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx - length,
                        dy - length,
                        &mut direction,
                        Direction::LEFT.bits() + Direction::UP.bits(),
                    )?;
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx - length,
                        dy + length,
                        &mut direction,
                        Direction::LEFT.bits() + Direction::DOWN.bits(),
                    )?;
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx + length,
                        dy - length,
                        &mut direction,
                        Direction::RIGHT.bits() + Direction::UP.bits(),
                    )?;
                } else {
                    // Even the default case may happen, in the first step of the
                    // algorithm for example.
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx + length,
                        dy + length,
                        &mut direction,
                        Direction::RIGHT.bits() + Direction::DOWN.bits(),
                    )?;
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx - length,
                        dy + length,
                        &mut direction,
                        Direction::LEFT.bits() + Direction::DOWN.bits(),
                    )?;
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx + length,
                        dy - length,
                        &mut direction,
                        Direction::RIGHT.bits() + Direction::UP.bits(),
                    )?;
                    self.check_mv2::<DCT_MODE, LOG_PEL>(
                        src_planes,
                        ref_frame,
                        ref_frame_data,
                        dx - length,
                        dy - length,
                        &mut direction,
                        Direction::LEFT.bits() + Direction::UP.bits(),
                    )?;
                }
            }
        }

        Ok(())
    }

    fn hex2_search<const DCT_MODE: u8, const LOG_PEL: usize>(
        &mut self,
        src_planes: [&[T]; 3],
        ref_frame: &MVFrame,
        ref_frame_data: &Frame,
        i_me_range: isize,
    ) -> Result<()> {
        todo!()
    }

    fn umh_search<const DCT_MODE: u8, const LOG_PEL: usize>(
        &mut self,
        src_planes: [&[T]; 3],
        ref_frame: &MVFrame,
        ref_frame_data: &Frame,
        me_range: isize,
        omx: isize,
        omy: isize,
    ) -> Result<()> {
        todo!()
    }

    fn expanding_search<const DCT_MODE: u8, const LOG_PEL: usize>(
        &mut self,
        src_planes: [&[T]; 3],
        ref_frame: &MVFrame,
        ref_frame_data: &Frame,
        r: isize,
        s: isize,
        mvx: isize,
        mvy: isize,
    ) -> Result<()> {
        todo!()
    }

    #[must_use]
    fn is_vector_ok(&self, vx: isize, vy: isize) -> bool {
        (vx >= self.dx_min) && (vy >= self.dy_min) && (vx < self.dx_max) && (vy < self.dy_max)
    }

    /// computes the cost of a vector (vx, vy)
    #[must_use]
    fn motion_distortion(&self, vx: isize, vy: isize) -> i64 {
        let dist = self.predictor.square_difference_norm(vx, vy);
        (self.lambda as i64 * dist as i64) >> 8
    }
}

#[derive(Debug, Clone)]
pub struct MvsOutput {
    pub validity: bool,
    pub block_data: Box<[u8]>,
}

// This only exists so we don't have 500 lines of code building a jump table.
struct SearchMvsArgs<'a> {
    pub out_idx: usize,
    pub src_frame: &'a MVFrame,
    pub src_frame_data: &'a Frame<'a>,
    pub ref_frame: &'a MVFrame,
    pub ref_frame_data: &'a Frame<'a>,
    pub search_type: SearchType,
    pub search_param: usize,
    pub lambda: u32,
    pub lambda_sad: u32,
    pub penalty_new: u16,
    pub penalty_level: PenaltyScaling,
    pub out: &'a mut MvsOutput,
    pub global_mv: &'a mut MotionVector,
    pub field_shift: isize,
    pub mean_luma_change: &'a mut i32,
    pub penalty_zero: u16,
    pub penalty_global: u16,
    pub bad_sad: u64,
    pub bad_range: isize,
    pub meander: bool,
    pub try_many: bool,
}
