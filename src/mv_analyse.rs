#[cfg(test)]
mod tests;

use std::{
    cmp::min,
    num::{NonZeroU8, NonZeroUsize},
};

use anyhow::{Result, anyhow, bail};
use vapoursynth::{
    format::{ColorFamily, Format, SampleType},
    frame::FrameRef,
    node::Node,
    plugins::Filter,
};

use crate::{
    params::MVPlaneSet,
    params::{DctMode, DivideMode, MotionFlags, PenaltyScaling, SearchType, Subpel},
    util::Pixel,
};

#[derive(Debug)]
pub struct Analyse<'core> {
    /// Super clip
    node: Node<'core>,
    /// Number of hierarchical levels in super clip frames. `Analyse` is needed
    /// in all levels, but for other client functions single finest level is
    /// enough (coarser levels are not used).
    ///
    /// Default `0` = auto, all levels are produced
    levels: u16,
    /// search type chosen for refinement in the EPZ
    search_type: SearchType,
    search_type_coarse: SearchType,
    search_param: usize,
    /// search radius at finest level
    pel_search: usize,
    /// set to true, it allows to take chroma into account when doing the motion estimation
    chroma: bool,
    /// A preset of these parameters values.
    /// It allows easy to switch default values of all "true motion" parameters at once.
    /// Set it true for true motion search (high vector coherence),
    /// set it false to search motion vectors with best SAD.
    truemotion: bool,
    /// Motion vector cost factor
    lambda: u32,
    /// SAD limit for lambda usage
    lambda_sad: u32,
    /// Penalty factors level scaling
    penalty_level: PenaltyScaling,
    /// Use global motion predictor
    global: bool,
    /// Penalty for New MV cost
    penalty_new: u16,
    /// Penalty for Zero MV cost
    penalty_zero: u16,
    /// Penalty for Global MV cost
    penalty_global: u16,
    /// Usage of block DCT (frequency spectrum) for block difference (SAD) calculation.
    /// In particular it can improve motion vector estimation around luma flicker and fades.
    dctmode: DctMode,
    /// Divice blocks on subblocks with median motion
    divide_extra: DivideMode,
    /// SAD threshold to make more wide second search for bad vectors.
    /// Value is scaled to block size 8x8.
    /// Default is 10000 (disabling value), recommended is about 1000-2000.
    bad_sad: u32,
    /// the range (radius) of wide search for bad blocks.
    /// Default is 24 (image pixel units).
    /// Use positive value for UMH search and negative for Exhaustive search.
    bad_range: usize,
    /// Alternate blocks scan in rows from left to right and from right to left. Default is True
    meander: bool,
    /// try to start searches around many predictors. Default is false.
    try_many: bool,
    fields: bool,
    tff: Option<bool>,

    // Internal fields
    analysis_data: MVAnalysisData,
    analysis_data_divided: Option<MVAnalysisData>,
    format: Format<'core>,
    yuv_mode: MVPlaneSet,
    super_hpad: usize,
    super_vpad: usize,
    super_pel: Subpel,
    super_mode_yuv: MVPlaneSet,
    super_levels: u16,
}

#[derive(Debug, Clone)]
struct MVAnalysisData {
    // TODO: Are these top two fields even used for anything at all?
    /// Unique identifier, not very useful
    pub magic_key: i32,
    /// MVAnalysisData and outfile format version
    pub version: i32,
    /// horizontal block size in pixels
    pub blk_size_x: NonZeroUsize,
    /// vertical block size in pixels
    pub blk_size_y: NonZeroUsize,
    /// pixel refinement of the motion estimation
    pub pel: Subpel,
    /// number of level for the hierarchal search
    pub level_count: u16,
    /// difference between the index of the reference and the index of the current frame
    pub delta_frame: isize,
    /// direction of the search ( forward / backward )
    pub is_backward: bool,
    /// diverse flags to set up the search
    pub motion_flags: u8,
    /// Width of the frame
    pub width: NonZeroUsize,
    /// Height of the frame
    pub height: NonZeroUsize,
    /// overlap block size
    pub overlap_x: usize,
    /// vertical overlap
    pub overlap_y: usize,
    /// number of blocks along X
    pub blk_x: NonZeroUsize,
    /// number of blocks along Y
    pub blk_y: NonZeroUsize,
    /// Number of bits per pixel
    pub bits_per_sample: NonZeroU8,
    /// ratio of luma plane height to chroma plane height
    pub y_ratio_uv: NonZeroUsize,
    /// ratio of luma plane width to chroma plane width
    pub x_ratio_uv: NonZeroUsize,
    /// Horizontal padding
    pub h_padding: usize,
    /// Vertical padding
    pub v_padding: usize,
}

impl<'core> Analyse<'core> {
    pub fn new(
        super_: Node<'core>,
        blksize: Option<i64>,
        blksizev: Option<i64>,
        levels: Option<i64>,
        search: Option<i64>,
        searchparam: Option<i64>,
        pelsearch: Option<i64>,
        isb: Option<i64>,
        lambda: Option<i64>,
        chroma: Option<i64>,
        delta: Option<i64>,
        truemotion: Option<i64>,
        lsad: Option<i64>,
        plevel: Option<i64>,
        global: Option<i64>,
        pnew: Option<i64>,
        pzero: Option<i64>,
        pglobal: Option<i64>,
        overlap: Option<i64>,
        overlapv: Option<i64>,
        divide: Option<i64>,
        badsad: Option<i64>,
        badrange: Option<i64>,
        meander: Option<i64>,
        trymany: Option<i64>,
        fields: Option<i64>,
        tff: Option<i64>,
        search_coarse: Option<i64>,
        dct: Option<i64>,
    ) -> Result<Self> {
        let blk_size_x = blksize.map(usize::try_from).unwrap_or(Ok(8))?;
        let blk_size_y = blksizev.map(usize::try_from).unwrap_or(Ok(blk_size_x))?;
        let overlap_x = overlap.map(usize::try_from).unwrap_or(Ok(0))?;
        let overlap_y = overlapv.map(usize::try_from).unwrap_or(Ok(overlap_x))?;
        let truemotion = truemotion.map(|truemotion| truemotion > 0).unwrap_or(true);
        let penalty_new = pnew
            .map(u16::try_from)
            .unwrap_or(Ok(if truemotion { 50 } else { 0 }))?;
        let penalty_zero = pzero.map(u16::try_from).unwrap_or(Ok(penalty_new))?;
        let penalty_global = pglobal.map(u16::try_from).unwrap_or(Ok(0))?;
        let dctmode = dct.map(DctMode::try_from).unwrap_or(Ok(DctMode::Spatial))?;
        let search_type = search
            .map(SearchType::try_from)
            .unwrap_or(Ok(SearchType::Hex2))?;
        let mut search_param = searchparam.map(isize::try_from).unwrap_or(Ok(2))?;
        let divide_extra = divide
            .map(DivideMode::try_from)
            .unwrap_or(Ok(DivideMode::None))?;
        let mut chroma = chroma.map(|chroma| chroma > 0).unwrap_or(true);
        let mut lambda = lambda.map(u32::try_from).unwrap_or(Ok(if truemotion {
            (1000 * blk_size_x * blk_size_y / 64) as u32
        } else {
            0
        }))?;
        let mut lambda_sad =
            lsad.map(u32::try_from)
                .unwrap_or(Ok(if truemotion { 1200 } else { 400 }))?;
        let mut bad_sad = badsad.map(u32::try_from).unwrap_or(Ok(10_000))?;
        let is_backward = isb.map(|isb| isb > 0).unwrap_or(false);
        let delta_frame = delta.map(isize::try_from).unwrap_or(Ok(1))?;
        let mut pel_search = pelsearch.map(usize::try_from).unwrap_or(Ok(0))?;

        if dctmode.uses_satd() && blk_size_x == 16 && blk_size_y == 2 {
            bail!("Analyse: dct 5-10 cannot work with 16x2 blocks");
        }
        match (blk_size_x, blk_size_y) {
            // Allowed block sizes
            (4, 4)
            | (8, 4)
            | (8, 8)
            | (16, 2)
            | (16, 8)
            | (16, 16)
            | (32, 16)
            | (32, 32)
            | (64, 32)
            | (64, 64)
            | (128, 64)
            | (128, 128) => (),
            _ => bail!(
                "Analyse: the block size must be 4x4, 8x4, 8x8, 16x2, 16x8, 16x16, 32x16, 32x32, 64x32, 64x64, 128x64, or 128x128."
            ),
        }

        if penalty_new > 256 {
            bail!("Analyse: pnew must be between 0 and 256 (inclusive).");
        }
        if penalty_zero > 256 {
            bail!("Analyse: pzero must be between 0 and 256 (inclusive).");
        }
        if penalty_global > 256 {
            bail!("Analyse: pglobal must be between 0 and 256 (inclusive).");
        }

        if overlap_x > blk_size_x / 2 || overlap_y > blk_size_y / 2 {
            bail!(
                "Analyse: overlap must be at most half of blksize, and overlapv must be at most half of blksizev"
            );
        }

        if divide_extra != DivideMode::None && (blk_size_x < 8 || blk_size_y < 8) {
            bail!("Analyse: blksize and blksizev must be at least 8 when divide=True.");
        }

        if search_type == SearchType::Nstep {
            if search_param < 0 {
                search_param = 0;
            }
        } else if search_param < 1 {
            search_param = 1;
        }

        let info = super_.info();
        let format = match info.format {
            vapoursynth::prelude::Property::Variable => {
                bail!("Analyse: variable format input clips are not supported")
            }
            vapoursynth::prelude::Property::Constant(format) => format,
        };
        if format.bits_per_sample() > 16 {
            bail!("Analyse: input clip must be 8-16 bits");
        }
        if format.sample_type() != SampleType::Integer {
            bail!("Analyse: input clip must be integer super_format");
        }
        if ![ColorFamily::YUV, ColorFamily::Gray].contains(&format.color_family())
            || format.sub_sampling_w() > 1
            || format.sub_sampling_h() > 1
        {
            bail!("Analyse: input clip must be GRAY, 420, 422, 440, or 444");
        }

        if format.color_family() == ColorFamily::Gray {
            chroma = false;
        }
        let bits_per_sample = NonZeroU8::new(format.bits_per_sample()).unwrap();
        let yuv_mode = if chroma {
            MVPlaneSet::YUVPLANES
        } else {
            MVPlaneSet::YPLANE
        };
        let pixel_max = (1u32 << bits_per_sample.get()) - 1;
        lambda_sad = (lambda_sad as f32 * pixel_max as f32 / 255.0 + 0.5) as u32;
        bad_sad = (bad_sad as f32 * pixel_max as f32 / 255.0 + 0.5) as u32;
        lambda = (lambda as f32 * pixel_max as f32 / 255.0 + 0.5) as u32;
        lambda_sad = (lambda_sad as usize * (blk_size_x * blk_size_y) / 64) as u32;
        bad_sad = (bad_sad as usize * (blk_size_x * blk_size_y) / 64) as u32;

        // TODO: Why are we using this instead of just checking the variables directly?
        let mut motion_flags = 0;
        if is_backward {
            motion_flags |= MotionFlags::IS_BACKWARD.bits();
        }
        if chroma {
            motion_flags |= MotionFlags::USE_CHROMA_MOTION.bits();
        }

        let mode_yuv = if chroma {
            MVPlaneSet::YUVPLANES
        } else {
            MVPlaneSet::YPLANE
        };

        if overlap_x % (1 << format.sub_sampling_w()) > 0
            || overlap_y % (1 << format.sub_sampling_h()) > 0
        {
            bail!(
                "Analyse: the requested overlap is incompatible with the super clip's subsampling."
            );
        }
        if divide_extra != DivideMode::None
            && (overlap_x % (2 << format.sub_sampling_w()) > 0
                || overlap_y % (2 << format.sub_sampling_h()) > 0)
        {
            bail!(
                "Analyse: overlap and overlapv must be multiples of 2 or 4 when divide=True, depending on the super clip's subsampling."
            );
        }
        if delta_frame <= 0 && (-delta_frame) >= info.num_frames as isize {
            bail!("Analyse: delta points to frame past the input clip's end.");
        }

        let (width, _height) = match info.resolution {
            vapoursynth::prelude::Property::Variable => {
                bail!("Analyse: variable resolution input clips are not supported")
            }
            // SAFETY: width and height must be positive
            vapoursynth::prelude::Property::Constant(resolution) => unsafe {
                (
                    NonZeroUsize::new_unchecked(resolution.width),
                    NonZeroUsize::new_unchecked(resolution.height),
                )
            },
        };

        let x_ratio_uv = NonZeroUsize::new(1 << format.sub_sampling_w()).unwrap();
        let y_ratio_uv = NonZeroUsize::new(1 << format.sub_sampling_h()).unwrap();

        // I like that this is called `evil`, but I don't really know a better way to handle it.
        let evil = match super_.get_frame(0) {
            Ok(frame) => frame,
            Err(e) => {
                bail!(
                    "Analyse: failed to retrieve first frame from super clip. Error message: {e}"
                );
            }
        };
        let super_props = evil.props();
        let super_props_err = "Analyse: required properties not found in first frame of super clip. Maybe clip didn't come from mv.Super? Was the first frame trimmed away?";
        let super_props_err2 = "Analyse: parameters from super clip appear to be wrong.";
        let super_height = NonZeroUsize::new(
            usize::try_from(
                super_props
                    .get_int("Super_height")
                    .map_err(|_| anyhow!(super_props_err))?,
            )
            .map_err(|_| anyhow!(super_props_err2))?,
        )
        .ok_or_else(|| anyhow!(super_props_err2))?;
        let super_hpad = usize::try_from(
            super_props
                .get_int("Super_hpad")
                .map_err(|_| anyhow!(super_props_err))?,
        )
        .map_err(|_| anyhow!(super_props_err2))?;
        let super_vpad = usize::try_from(
            super_props
                .get_int("Super_vpad")
                .map_err(|_| anyhow!(super_props_err))?,
        )
        .map_err(|_| anyhow!(super_props_err2))?;
        let super_pel = Subpel::try_from(
            super_props
                .get_int("Super_pel")
                .map_err(|_| anyhow!(super_props_err))?,
        )
        .map_err(|_| anyhow!(super_props_err2))?;
        let super_mode_yuv = MVPlaneSet::from_bits(
            u8::try_from(
                super_props
                    .get_int("Super_modeyuv")
                    .map_err(|_| anyhow!(super_props_err))?,
            )
            .map_err(|_| anyhow!(super_props_err2))?,
        )
        .unwrap();
        let super_levels = u16::try_from(
            super_props
                .get_int("Super_levels")
                .map_err(|_| anyhow!(super_props_err))?,
        )
        .map_err(|_| anyhow!(super_props_err2))?;
        if super_hpad >= super_height.get() / 2 {
            bail!(super_props_err2);
        }
        // I don't know why `bitflags` had to give its methods complicated names
        // instead of just naming them "and" and "or".
        if mode_yuv.bits() & super_mode_yuv.bits() != mode_yuv.bits() {
            bail!("Analyse: super clip does not contain needed colour data.");
        }

        let super_width = NonZeroUsize::new(width.get() - super_hpad * 2).unwrap();
        let blk_x = (super_width.get() - overlap_x) / (blk_size_x - overlap_x);
        let blk_y = (super_height.get() - overlap_y) / (blk_size_y - overlap_y);
        let width_b = (blk_size_x - overlap_x) + blk_x + overlap_x;
        let height_b = (blk_size_y - overlap_y) + blk_y + overlap_y;

        // calculate valid levels
        let mut levels_max = 0;
        while ((width_b >> levels_max) - overlap_x) / (blk_size_x - overlap_x) > 0
            && ((height_b >> levels_max) - overlap_y) / (blk_size_y - overlap_y) > 0
        {
            levels_max += 1;
        }
        let level_count = match levels.filter(|l| *l > 0) {
            Some(levels) => min(levels_max, levels as u16),
            None => levels_max,
        };
        debug_assert!(level_count > 0);

        if level_count > super_levels {
            bail!(
                "Analyse: super clip has {} levels. Analyse needs {} levels.",
                super_levels,
                level_count
            );
        }

        if pel_search == 0 {
            pel_search = super_pel as usize;
        }

        let analysis_data = MVAnalysisData {
            magic_key: Default::default(),
            version: Default::default(),
            blk_size_x: NonZeroUsize::new(blk_size_x)
                .ok_or_else(|| anyhow!("Analyse: blksize must be greater than 0"))?,
            blk_size_y: NonZeroUsize::new(blk_size_y)
                .ok_or_else(|| anyhow!("Analyse: blksizev must be greater than 0"))?,
            pel: super_pel,
            level_count,
            delta_frame,
            is_backward,
            motion_flags,
            width: super_width,
            height: super_height,
            overlap_x,
            overlap_y,
            blk_x: NonZeroUsize::new(blk_x).unwrap(),
            blk_y: NonZeroUsize::new(blk_y).unwrap(),
            bits_per_sample,
            y_ratio_uv,
            x_ratio_uv,
            h_padding: super_hpad,
            v_padding: super_vpad,
        };

        let analysis_data_divided = if divide_extra != DivideMode::None {
            let mut div_data = analysis_data.clone();
            // SAFETY: constant is non-zero
            div_data.blk_x = div_data
                .blk_x
                .saturating_mul(unsafe { NonZeroUsize::new_unchecked(2) });
            // SAFETY: constant is non-zero
            div_data.blk_y = div_data
                .blk_y
                .saturating_mul(unsafe { NonZeroUsize::new_unchecked(2) });
            div_data.blk_size_x = NonZeroUsize::new(div_data.blk_size_x.get() / 2).unwrap();
            div_data.blk_size_y = NonZeroUsize::new(div_data.blk_size_y.get() / 2).unwrap();
            div_data.overlap_x /= 2;
            div_data.overlap_y /= 2;
            div_data.level_count += 1;
            Some(div_data)
        } else {
            None
        };

        Ok(Self {
            node: super_,
            levels: levels.map(u16::try_from).unwrap_or(Ok(0))?,
            search_type,
            search_type_coarse: search_coarse
                .map(SearchType::try_from)
                .unwrap_or(Ok(SearchType::Exhaustive))?,
            search_param: search_param as usize,
            pel_search,
            chroma,
            truemotion,
            lambda,
            lambda_sad,
            penalty_level: plevel.map(PenaltyScaling::try_from).unwrap_or_else(|| {
                Ok(if truemotion {
                    PenaltyScaling::Linear
                } else {
                    PenaltyScaling::None
                })
            })?,
            global: global.map(|global| global > 0).unwrap_or(truemotion),
            penalty_new,
            penalty_zero,
            penalty_global,
            dctmode,
            divide_extra,
            bad_sad,
            bad_range: badrange.map(usize::try_from).unwrap_or(Ok(24))?,
            meander: meander.map(|meander| meander > 0).unwrap_or(true),
            try_many: trymany.map(|trymany| trymany > 0).unwrap_or(false),
            fields: fields.map(|fields| fields > 0).unwrap_or(false),
            tff: tff.map(|tff| tff > 0),
            analysis_data,
            analysis_data_divided,
            format,
            yuv_mode,
            super_hpad,
            super_vpad,
            super_pel,
            super_mode_yuv,
            super_levels,
        })
    }

    fn get_frame_internal<T: Pixel>(
        &self,
        core: vapoursynth::core::CoreRef<'core>,
        context: vapoursynth::plugins::FrameContext,
        n: usize,
    ) -> Result<FrameRef<'core>> {
        todo!()
    }
}

impl<'core> Filter<'core> for Analyse<'core> {
    fn video_info(
        &self,
        _api: vapoursynth::prelude::API,
        _core: vapoursynth::core::CoreRef<'core>,
    ) -> Vec<vapoursynth::video_info::VideoInfo<'core>> {
        let info = self.node.info();
        vec![info]
    }

    fn get_frame_initial(
        &self,
        _api: vapoursynth::prelude::API,
        _core: vapoursynth::core::CoreRef<'core>,
        context: vapoursynth::plugins::FrameContext,
        n: usize,
    ) -> std::result::Result<Option<vapoursynth::prelude::FrameRef<'core>>, anyhow::Error> {
        if self.analysis_data.delta_frame > 0 {
            let offset = if self.analysis_data.is_backward {
                self.analysis_data.delta_frame
            } else {
                -self.analysis_data.delta_frame
            };
            let nref = n as isize + offset;

            if nref >= 0 && (nref as usize) < self.node.info().num_frames {
                let nref = nref as usize;
                if n < nref {
                    self.node.request_frame_filter(context, n);
                    self.node.request_frame_filter(context, nref);
                } else {
                    self.node.request_frame_filter(context, nref);
                    self.node.request_frame_filter(context, n);
                }
            } else {
                // too close to beginning/end of clip
                self.node.request_frame_filter(context, n);
            }
        } else {
            // special static mode

            // positive fixed frame number
            let nref = -self.analysis_data.delta_frame;
            debug_assert!(nref >= 0);
            let nref = nref as usize;

            if n < nref {
                self.node.request_frame_filter(context, n);
                self.node.request_frame_filter(context, nref);
            } else {
                self.node.request_frame_filter(context, nref);
                self.node.request_frame_filter(context, n);
            }
        }
        Ok(None)
    }

    fn get_frame(
        &self,
        _api: vapoursynth::prelude::API,
        core: vapoursynth::core::CoreRef<'core>,
        context: vapoursynth::plugins::FrameContext,
        n: usize,
    ) -> std::result::Result<vapoursynth::prelude::FrameRef<'core>, anyhow::Error> {
        match self.format.bytes_per_sample() {
            1 => self.get_frame_internal::<u8>(core, context, n),
            2 => self.get_frame_internal::<u16>(core, context, n),
            _ => bail!("Analyse: does not support clips greater than 16 bits"),
        }
    }
}
