#[cfg(test)]
mod tests;

use std::{
    cmp::min,
    num::{NonZeroU8, NonZeroUsize},
};

use anyhow::{Result, anyhow, bail};
use vapoursynth::{
    format::{ColorFamily, Format, SampleType},
    frame::{FrameRef, FrameRefMut},
    node::Node,
    plugins::Filter,
};

use crate::{
    group_of_planes::GroupOfPlanes,
    mv_gof::MVGroupOfFrames,
    params::{DctMode, DivideMode, MVPlaneSet, MotionFlags, PenaltyScaling, SearchType, Subpel},
    util::Pixel,
};

const PROP_MVANALYSISDATA: &str = "MVTools_MVAnalysisData";
const PROP_VECTORS: &str = "MVTools_vectors";

#[derive(Debug)]
#[allow(dead_code)]
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
    search_param: i32,
    /// search radius at finest level
    pel_search: i32,
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
    dct_mode: DctMode,
    /// Divice blocks on subblocks with median motion
    divide_extra: DivideMode,
    /// SAD threshold to make more wide second search for bad vectors.
    /// Value is scaled to block size 8x8.
    /// Default is 10000 (disabling value), recommended is about 1000-2000.
    bad_sad: u64,
    /// the range (radius) of wide search for bad blocks.
    /// Default is 24 (image pixel units).
    /// Use positive value for UMH search and negative for Exhaustive search.
    bad_range: i32,
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
    super_levels: usize,
}

#[derive(Debug, Clone, Copy)]
struct MVAnalysisData {
    /// horizontal block size in pixels
    pub blk_size_x: NonZeroUsize,
    /// vertical block size in pixels
    pub blk_size_y: NonZeroUsize,
    /// pixel refinement of the motion estimation
    pub pel: Subpel,
    /// number of level for the hierarchal search
    pub level_count: usize,
    /// difference between the index of the reference and the index of the current frame
    pub delta_frame: isize,
    /// direction of the search ( forward / backward )
    pub is_backward: bool,
    /// diverse flags to set up the search
    pub motion_flags: MotionFlags,
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
    pub y_ratio_uv: NonZeroU8,
    /// ratio of luma plane width to chroma plane width
    pub x_ratio_uv: NonZeroU8,
    /// Horizontal padding
    pub h_padding: usize,
    /// Vertical padding
    pub v_padding: usize,
}

impl MVAnalysisData {
    #[must_use]
    pub(crate) fn bytes(&self) -> Vec<u8> {
        let prop_data = MVAnalysisPropData::from(*self);
        // SAFETY: We've added `repr(c)` to ensure a predictable size of the struct
        unsafe {
            // convert to vec to avoid lifetime issues
            std::slice::from_raw_parts(
                &prop_data as *const MVAnalysisPropData as *const u8,
                std::mem::size_of::<MVAnalysisPropData>(),
            )
            .to_vec()
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
// This version of the struct maintains equivalent types to the C version
struct MVAnalysisPropData {
    pub blk_size_x: i32,
    pub blk_size_y: i32,
    pub pel: i32,
    pub level_count: i32,
    pub delta_frame: i32,
    pub is_backward: i32,
    pub motion_flags: i32,
    pub width: i32,
    pub height: i32,
    pub overlap_x: i32,
    pub overlap_y: i32,
    pub blk_x: i32,
    pub blk_y: i32,
    pub bits_per_sample: i32,
    pub y_ratio_uv: i32,
    pub x_ratio_uv: i32,
    pub h_padding: i32,
    pub v_padding: i32,
}

impl From<MVAnalysisData> for MVAnalysisPropData {
    fn from(value: MVAnalysisData) -> Self {
        MVAnalysisPropData {
            blk_size_x: value.blk_size_x.get() as _,
            blk_size_y: value.blk_size_y.get() as _,
            pel: value.pel as _,
            level_count: value.level_count as _,
            delta_frame: value.delta_frame as _,
            is_backward: value.is_backward as _,
            motion_flags: value.motion_flags.bits() as _,
            width: value.width.get() as _,
            height: value.height.get() as _,
            overlap_x: value.overlap_x as _,
            overlap_y: value.overlap_y as _,
            blk_x: value.blk_x.get() as _,
            blk_y: value.blk_y.get() as _,
            bits_per_sample: value.bits_per_sample.get() as _,
            y_ratio_uv: value.y_ratio_uv.get() as _,
            x_ratio_uv: value.x_ratio_uv.get() as _,
            h_padding: value.h_padding as _,
            v_padding: value.v_padding as _,
        }
    }
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
        let blk_size_x = blksize.map_or(Ok(8), usize::try_from)?;
        let blk_size_y = blksizev.map_or(Ok(blk_size_x), usize::try_from)?;
        let overlap_x = overlap.map_or(Ok(0), usize::try_from)?;
        let overlap_y = overlapv.map_or(Ok(overlap_x), usize::try_from)?;
        let truemotion = truemotion.is_none_or(|truemotion| truemotion > 0);
        let penalty_new = pnew.map_or(Ok(if truemotion { 50 } else { 0 }), u16::try_from)?;
        let penalty_zero = pzero.map_or(Ok(penalty_new), u16::try_from)?;
        let penalty_global = pglobal.map_or(Ok(0), u16::try_from)?;
        let dctmode = dct.map_or(Ok(DctMode::Spatial), DctMode::try_from)?;
        let search_type = search.map_or(Ok(SearchType::Hex2), SearchType::try_from)?;
        let mut search_param = searchparam.map_or(Ok(2), i32::try_from)?;
        let divide_extra = divide.map_or(Ok(DivideMode::None), DivideMode::try_from)?;
        let mut chroma = chroma.is_none_or(|chroma| chroma > 0);
        let mut lambda = lambda.map_or(
            Ok(if truemotion {
                (1000 * blk_size_x * blk_size_y / 64) as u32
            } else {
                0
            }),
            u32::try_from,
        )?;
        let mut lambda_sad = lsad.map_or(Ok(if truemotion { 1200 } else { 400 }), u32::try_from)?;
        let mut bad_sad = badsad.map_or(Ok(10_000), u64::try_from)?;
        let is_backward = isb.is_some_and(|isb| isb > 0);
        let delta_frame = delta.map_or(Ok(1), isize::try_from)?;
        let mut pel_search = pelsearch.map_or(Ok(0), usize::try_from)?;

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
                "Analyse: the block size must be 4x4, 8x4, 8x8, 16x2, 16x8, 16x16, 32x16, 32x32, \
                 64x32, 64x64, 128x64, or 128x128."
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
                "Analyse: overlap must be at most half of blksize, and overlapv must be at most \
                 half of blksizev"
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
        let format = info.format;
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
        let bits_per_sample =
            NonZeroU8::new(format.bits_per_sample()).expect("VS should not return 0 BPP");
        let yuv_mode = if chroma {
            MVPlaneSet::YUVPLANES
        } else {
            MVPlaneSet::YPLANE
        };
        let pixel_max = (1u32 << bits_per_sample.get()) - 1;
        lambda_sad = (lambda_sad as f32 * pixel_max as f32 / 255.0 + 0.5) as u32;
        bad_sad = (bad_sad as f32 * pixel_max as f32 / 255.0 + 0.5) as u64;
        lambda = (lambda as f32 * pixel_max as f32 / 255.0 + 0.5) as u32;
        lambda_sad = (lambda_sad as usize * (blk_size_x * blk_size_y) / 64) as u32;
        bad_sad = bad_sad * (blk_size_x * blk_size_y) as u64 / 64;

        // TODO: Why are we using this instead of just checking the variables directly?
        let mut motion_flags = MotionFlags::empty();
        if is_backward {
            motion_flags |= MotionFlags::IS_BACKWARD;
        }
        if chroma {
            motion_flags |= MotionFlags::USE_CHROMA_MOTION;
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
                "Analyse: overlap and overlapv must be multiples of 2 or 4 when divide=True, \
                 depending on the super clip's subsampling."
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

        let x_ratio_uv = NonZeroU8::new(1 << format.sub_sampling_w()).expect("cannot be zero");
        let y_ratio_uv = NonZeroU8::new(1 << format.sub_sampling_h()).expect("cannot be zero");

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
        let super_props_err = "Analyse: required properties not found in first frame of super \
                               clip. Maybe clip didn't come from mv.Super? Was the first frame \
                               trimmed away?";
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
        .ok_or_else(|| anyhow!(super_props_err2))?;
        let super_levels = usize::try_from(
            super_props
                .get_int("Super_levels")
                .map_err(|_| anyhow!(super_props_err))?,
        )
        .map_err(|_| anyhow!(super_props_err2))?;
        if super_hpad >= super_height.get() / 2 {
            bail!(super_props_err2);
        }

        if mode_yuv & super_mode_yuv != mode_yuv {
            bail!("Analyse: super clip does not contain needed colour data.");
        }

        let super_width = NonZeroUsize::new(width.get() - super_hpad * 2)
            .expect("super width should not be zero");
        let blk_x = (super_width.get() - overlap_x) / (blk_size_x - overlap_x);
        let blk_y = (super_height.get() - overlap_y) / (blk_size_y - overlap_y);
        let width_b = (blk_size_x - overlap_x) * blk_x + overlap_x;
        let height_b = (blk_size_y - overlap_y) * blk_y + overlap_y;

        // calculate valid levels
        let mut levels_max = 0;
        while ((width_b >> levels_max) - overlap_x) / (blk_size_x - overlap_x) > 0
            && ((height_b >> levels_max) - overlap_y) / (blk_size_y - overlap_y) > 0
        {
            levels_max += 1;
        }
        let level_count = levels
            .filter(|l| *l > 0)
            .map_or(levels_max, |levels| min(levels_max, levels as usize));
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
            blk_x: NonZeroUsize::new(blk_x).expect("block count should not be zero"),
            blk_y: NonZeroUsize::new(blk_y).expect("block count should not be zero"),
            bits_per_sample,
            y_ratio_uv,
            x_ratio_uv,
            h_padding: super_hpad,
            v_padding: super_vpad,
        };

        let analysis_data_divided = (divide_extra != DivideMode::None).then(|| {
            let mut div_data = analysis_data;
            // SAFETY: constant is non-zero
            div_data.blk_x = div_data
                .blk_x
                .saturating_mul(unsafe { NonZeroUsize::new_unchecked(2) });
            // SAFETY: constant is non-zero
            div_data.blk_y = div_data
                .blk_y
                .saturating_mul(unsafe { NonZeroUsize::new_unchecked(2) });
            div_data.blk_size_x = NonZeroUsize::new(div_data.blk_size_x.get() / 2)
                .expect("block size cannot not be 1");
            div_data.blk_size_y = NonZeroUsize::new(div_data.blk_size_y.get() / 2)
                .expect("block size cannot not be 1");
            div_data.overlap_x /= 2;
            div_data.overlap_y /= 2;
            div_data.level_count += 1;
            div_data
        });

        Ok(Self {
            node: super_,
            levels: levels.map_or(Ok(0), u16::try_from)?,
            search_type,
            search_type_coarse: search_coarse
                .map_or(Ok(SearchType::Exhaustive), SearchType::try_from)?,
            search_param,
            pel_search: pel_search as i32,
            chroma,
            truemotion,
            lambda,
            lambda_sad,
            penalty_level: plevel.map_or_else(
                || {
                    Ok(if truemotion {
                        PenaltyScaling::Linear
                    } else {
                        PenaltyScaling::None
                    })
                },
                PenaltyScaling::try_from,
            )?,
            global: global.map_or(truemotion, |global| global > 0),
            penalty_new,
            penalty_zero,
            penalty_global,
            dct_mode: dctmode,
            divide_extra,
            bad_sad,
            bad_range: badrange.map_or(Ok(24), i32::try_from)?,
            meander: meander.is_none_or(|meander| meander > 0),
            try_many: trymany.is_some_and(|trymany| trymany > 0),
            fields: fields.is_some_and(|fields| fields > 0),
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
        let mut vector_fields = GroupOfPlanes::<T>::new(
            self.analysis_data.blk_size_x,
            self.analysis_data.blk_size_y,
            self.analysis_data.level_count,
            self.analysis_data.pel,
            self.analysis_data.motion_flags,
            self.analysis_data.overlap_x,
            self.analysis_data.overlap_y,
            self.analysis_data.blk_x,
            self.analysis_data.blk_y,
            self.analysis_data.x_ratio_uv,
            self.analysis_data.y_ratio_uv,
            self.divide_extra,
            self.analysis_data.bits_per_sample,
        )?;

        let nref = if self.analysis_data.delta_frame > 0 {
            let offset = if self.analysis_data.is_backward {
                self.analysis_data.delta_frame
            } else {
                -self.analysis_data.delta_frame
            };
            n as isize + offset
        } else {
            // special static mode
            // positive fixed frame number
            -self.analysis_data.delta_frame
        };

        let src = self
            .node
            .get_frame_filter(context, n)
            .ok_or_else(|| anyhow!("Analyse: get_frame_filter past end of video"))?;
        let src_props = src.props();

        let mut src_top_field = match src_props.get_int("_Field") {
            Ok(field) => field > 0,
            Err(_) if self.fields && self.tff.is_none() => {
                bail!(
                    "Analyse: _Field property not found in input frame. Therefore, you must pass \
                     tff argument."
                );
            }
            _ => false,
        };
        // if tff was passed, it overrides _Field.
        if let Some(tff) = self.tff {
            src_top_field = (tff as u8 ^ (n % 2) as u8) > 0;
        }

        let vectors = if nref >= 0 && (nref as usize) < self.node.info().num_frames {
            let ref_ = self
                .node
                .get_frame_filter(context, nref as usize)
                .ok_or_else(|| anyhow!("Analyse: get_frame_filter ref past end of video"))?;
            let ref_props = ref_.props();
            let mut ref_top_field = match ref_props.get_int("_Field") {
                Ok(field) => field > 0,
                Err(_) if self.fields && self.tff.is_none() => {
                    bail!(
                        "Analyse: _Field property not found in input frame. Therefore, you must \
                         pass tff argument."
                    );
                }
                _ => false,
            };

            // if tff was passed, it overrides _Field.
            if let Some(tff) = self.tff {
                ref_top_field = (tff as u8 ^ (n % 2) as u8) > 0;
            }

            let mut field_shift = 0;
            if self.fields
                && self.analysis_data.pel > Subpel::Full
                && (self.analysis_data.delta_frame % 2) > 0
            {
                // vertical shift of fields for fieldbased video at finest level pel2
                field_shift = if src_top_field && !ref_top_field {
                    (u8::from(self.analysis_data.pel) as u32 / 2) as i32
                } else if ref_top_field && !src_top_field {
                    -((u8::from(self.analysis_data.pel) as u32 / 2) as i32)
                } else {
                    0
                };
            }

            let src_pitch = [
                // SAFETY: stride cannot be 0
                unsafe { NonZeroUsize::new_unchecked(src.stride(0) / size_of::<T>()) },
                // SAFETY: stride cannot be 0
                unsafe { NonZeroUsize::new_unchecked(src.stride(1) / size_of::<T>()) },
                // SAFETY: stride cannot be 0
                unsafe { NonZeroUsize::new_unchecked(src.stride(2) / size_of::<T>()) },
            ];
            let ref_pitch = [
                // SAFETY: stride cannot be 0
                unsafe { NonZeroUsize::new_unchecked(ref_.stride(0) / size_of::<T>()) },
                // SAFETY: stride cannot be 0
                unsafe { NonZeroUsize::new_unchecked(ref_.stride(1) / size_of::<T>()) },
                // SAFETY: stride cannot be 0
                unsafe { NonZeroUsize::new_unchecked(ref_.stride(2) / size_of::<T>()) },
            ];
            let src_gof = MVGroupOfFrames::new(
                self.super_levels,
                self.analysis_data.width,
                self.analysis_data.height,
                self.super_pel,
                self.super_hpad,
                self.super_vpad,
                self.super_mode_yuv,
                self.analysis_data.x_ratio_uv,
                self.analysis_data.y_ratio_uv,
                self.analysis_data.bits_per_sample,
                &src_pitch,
                self.format,
            )?;
            let ref_gof = MVGroupOfFrames::new(
                self.super_levels,
                self.analysis_data.width,
                self.analysis_data.height,
                self.super_pel,
                self.super_hpad,
                self.super_vpad,
                self.super_mode_yuv,
                self.analysis_data.x_ratio_uv,
                self.analysis_data.y_ratio_uv,
                self.analysis_data.bits_per_sample,
                &ref_pitch,
                self.format,
            )?;

            let mut vectors = vector_fields.search_mvs(
                &src_gof,
                &src,
                &ref_gof,
                &ref_,
                self.search_type,
                self.search_type_coarse,
                self.search_param,
                self.pel_search,
                self.lambda,
                self.lambda_sad,
                self.penalty_new,
                self.penalty_level,
                self.global,
                field_shift,
                self.dct_mode,
                self.penalty_zero,
                self.penalty_global,
                self.bad_sad,
                self.bad_range,
                self.meander,
                self.try_many,
            )?;
            if self.divide_extra != DivideMode::None {
                vector_fields.extra_divide(&mut vectors);
            }
            vectors
        } else {
            // too close to the beginning or end to do anything
            vector_fields.write_default_to_array()
        };

        let mut dest = FrameRefMut::copy_of(core, &src);
        let mut dest_props = dest.props_mut();
        dest_props.set_data(
            PROP_MVANALYSISDATA,
            &if self.divide_extra != DivideMode::None {
                self.analysis_data_divided
                    .as_ref()
                    .map_or_else(Vec::new, |data| data.bytes())
            } else {
                self.analysis_data.bytes()
            },
        )?;
        dest_props.set_data(PROP_VECTORS, &vectors.block_data)?;

        Ok(dest.into())
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
