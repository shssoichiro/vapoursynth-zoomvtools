use crate::{
    dct::DctHelper,
    mv::MotionVector,
    mv_frame::MVFrame,
    params::{DctMode, DivideMode, MVPlaneSet, MotionFlags, PenaltyScaling, SearchType, Subpel},
    util::{Pixel, luma_sum, median, plane_with_padding},
};
use anyhow::Result;
use smallvec::SmallVec;
use std::{
    cmp::{max, min},
    num::{NonZeroU8, NonZeroUsize},
};
use vapoursynth::frame::Frame;

// max block width * max block height
const MAX_BLOCK_SIZE: usize = 128 * 128;
// right now 5 should be enough (TSchniede)
const MAX_PREDICTOR: usize = 5;

#[derive(Clone)]
pub struct PlaneOfBlocks<T: Pixel> {
    pel: Subpel,
    log_pel: u8,
    log_scale: usize,
    scale: usize,
    blk_size_x: NonZeroUsize,
    blk_size_y: NonZeroUsize,
    overlap_x: usize,
    overlap_y: usize,
    blk_x: NonZeroUsize,
    blk_y: NonZeroUsize,
    blk_count: NonZeroUsize,
    x_ratio_uv: NonZeroU8,
    y_ratio_uv: NonZeroU8,
    log_x_ratio_uv: u8,
    log_y_ratio_uv: u8,
    bits_per_sample: NonZeroU8,
    smallest_plane: bool,
    chroma: bool,
    can_use_satd: bool,
    global_mv_predictor: MotionVector,
    vectors: Vec<MotionVector>,
    dct_pitch: NonZeroUsize,
    freq_size: NonZeroUsize,
    freq_array: Vec<i32>,
    very_big_sad: NonZeroUsize,

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
    sum_luma_change: i64,
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
            min_cost: Default::default(),
            blk_idx: Default::default(),
            blk_x_i: Default::default(),
            blk_y_i: Default::default(),
        }
    }

    pub fn search_mvs(
        &mut self,
        out_idx: usize,
        src_frame: &MVFrame,
        src_frame_data: &Frame,
        ref_frame: &MVFrame,
        ref_frame_data: &Frame,
        search_type: SearchType,
        search_param: usize,
        lambda: u32,
        lambda_sad: u32,
        penalty_new: u16,
        penalty_level: PenaltyScaling,
        out: &mut MvsOutput,
        global_mv: &mut MotionVector,
        field_shift: isize,
        dct_mode: DctMode,
        mean_luma_change: &mut i32,
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

        let blk_data = &mut out.blocks[out_idx];
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
                    src_frame,
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
        src_frame: &MVFrame,
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

        self.fetch_predictors();

        if (1..=4).contains(&DCT_MODE) {
            // make dct of source block
            // don't do the slow dct conversion if SATD used
            self.dct.as_ref().unwrap().bytes_2d();
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
        // Do we bias zero with not taking into account distorsion ?
        self.best_mv.x = self.zero_mv_field_shifted.x;
        self.best_mv.y = self.zero_mv_field_shifted.y;
        let zero_mv_blocks = [
            self.get_ref_block::<LOG_PEL>(
                ref_frame,
                ref_frame_data,
                0,
                self.zero_mv_field_shifted.y,
            )?,
            if self.chroma {
                self.get_ref_block_u::<LOG_PEL>(ref_frame, ref_frame_data, 0, 0)?
            } else {
                &[]
            },
            if self.chroma {
                self.get_ref_block_v::<LOG_PEL>(ref_frame, ref_frame_data, 0, 0)?
            } else {
                &[]
            },
        ];
        let mut sad = self.sad_luma::<DCT_MODE>(
            src_plane_y,
            self.src_pitch[0],
            zero_mv_blocks[0],
            self.ref_pitch[0],
        );
        if self.chroma {
            sad += self.sad_chroma(
                src_plane_u,
                self.src_pitch[1],
                zero_mv_blocks[1],
                self.ref_pitch[1],
            );
            sad += self.sad_chroma(
                src_plane_v,
                self.src_pitch[2],
                zero_mv_blocks[2],
                self.ref_pitch[2],
            );
        }
        self.best_mv.sad = sad as i64;
        self.min_cost = (sad + ((self.penalty_zero as u64 * sad) >> 8)) as i64;

        let mut best_mv_many = [MotionVector::zero(); 8];
        let mut min_cost_many = [0; 8];
        if self.try_many {
            // refine around zero
            self.refine::<DCT_MODE, LOG_PEL>();
            best_mv_many[0] = self.best_mv;
            min_cost_many[0] = self.min_cost;
        }

        // Global MV predictor
        self.global_mv_predictor = self.clip_mv(self.global_mv_predictor);
        let global_pred_blocks = [
            self.get_ref_block::<LOG_PEL>(
                ref_frame,
                ref_frame_data,
                self.global_mv_predictor.x,
                self.global_mv_predictor.y,
            )?,
            if self.chroma {
                self.get_ref_block_u::<LOG_PEL>(
                    ref_frame,
                    ref_frame_data,
                    self.global_mv_predictor.x,
                    self.global_mv_predictor.y,
                )?
            } else {
                &[]
            },
            if self.chroma {
                self.get_ref_block_v::<LOG_PEL>(
                    ref_frame,
                    ref_frame_data,
                    self.global_mv_predictor.x,
                    self.global_mv_predictor.y,
                )?
            } else {
                &[]
            },
        ];
        let mut sad = self.sad_luma::<DCT_MODE>(
            src_plane_y,
            self.src_pitch[0],
            global_pred_blocks[0],
            self.ref_pitch[0],
        );
        if self.chroma {
            sad += self.sad_chroma(
                src_plane_u,
                self.src_pitch[1],
                global_pred_blocks[1],
                self.ref_pitch[1],
            );
            sad += self.sad_chroma(
                src_plane_v,
                self.src_pitch[2],
                global_pred_blocks[2],
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
            self.refine::<DCT_MODE, LOG_PEL>();
            best_mv_many[1] = self.best_mv;
            min_cost_many[1] = self.min_cost;
        }
        let pred_blocks = [
            self.get_ref_block::<LOG_PEL>(
                ref_frame,
                ref_frame_data,
                self.predictor.x,
                self.predictor.y,
            )?,
            if self.chroma {
                self.get_ref_block_u::<LOG_PEL>(
                    ref_frame,
                    ref_frame_data,
                    self.predictor.x,
                    self.predictor.y,
                )?
            } else {
                &[]
            },
            if self.chroma {
                self.get_ref_block_v::<LOG_PEL>(
                    ref_frame,
                    ref_frame_data,
                    self.predictor.x,
                    self.predictor.y,
                )?
            } else {
                &[]
            },
        ];
        let mut sad = self.sad_luma::<DCT_MODE>(
            src_plane_y,
            self.src_pitch[0],
            pred_blocks[0],
            self.ref_pitch[0],
        );
        if self.chroma {
            sad += self.sad_chroma(
                src_plane_u,
                self.src_pitch[1],
                pred_blocks[1],
                self.ref_pitch[1],
            );
            sad += self.sad_chroma(
                src_plane_v,
                self.src_pitch[2],
                pred_blocks[2],
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
            self.refine::<DCT_MODE, LOG_PEL>();
            best_mv_many[2] = self.best_mv;
            min_cost_many[2] = self.min_cost;
        }

        // then all the other predictors
        let npred = 4;
        for i in 0..npred {
            if self.try_many {
                self.min_cost = self.very_big_sad.get() as i64 + 1;
            }
            self.check_mv0::<DCT_MODE, LOG_PEL>(self.predictors[i].x, self.predictors[i].y);

            if self.try_many {
                // refine around predictor
                self.refine::<DCT_MODE, LOG_PEL>();
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
            self.refine::<DCT_MODE, LOG_PEL>();
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
                self.umh_search::<DCT_MODE, LOG_PEL>(self.bad_range * (1 << LOG_PEL), 0, 0);
            } else if self.bad_range < 0 {
                // ESA
                for i in (1..(-self.bad_range * (1 << LOG_PEL))).step_by(1 << LOG_PEL) {
                    // at radius
                    self.expanding_search::<DCT_MODE, LOG_PEL>(i, 1 << LOG_PEL, 0, 0);
                    if self.best_mv.sad < found_sad / 4 {
                        // stop search if good MV is found
                        break;
                    }
                }
            }

            for i in 1..(1 << LOG_PEL) {
                // small radius
                self.expanding_search::<DCT_MODE, LOG_PEL>(i, 1, self.best_mv.x, self.best_mv.y);
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

    fn sad_luma<const DCT_MODE: u8>(
        &self,
        src_plane: &[T],
        src_pitch: NonZeroUsize,
        zero_mv_blocks: &[T],
        ref_pitch: NonZeroUsize,
    ) -> u64 {
        todo!()
    }

    fn sad_chroma(
        &self,
        src_plane: &[T],
        src_pitch: NonZeroUsize,
        zero_mv_blocks: &[T],
        ref_pitch: NonZeroUsize,
    ) -> u64 {
        todo!()
    }

    fn refine<const DCT_MODE: u8, const LOG_PEL: usize>(&mut self) {
        todo!()
    }

    fn check_mv0<const DCT_MODE: u8, const LOG_PEL: usize>(&mut self, x: isize, y: isize) {
        todo!()
    }

    fn umh_search<const DCT_MODE: u8, const LOG_PEL: usize>(
        &mut self,
        me_range: isize,
        omx: isize,
        omy: isize,
    ) {
        todo!()
    }

    fn expanding_search<const DCT_MODE: u8, const LOG_PEL: usize>(
        &mut self,
        r: isize,
        s: isize,
        mvx: isize,
        mvy: isize,
    ) {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct MvsOutput {
    pub validity: bool,
    pub blocks: Vec<Vec<MotionVector>>,
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
