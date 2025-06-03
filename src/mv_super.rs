#[cfg(test)]
mod tests;

use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::{Result, bail};
use vapoursynth::{
    format::{ColorFamily, Format, SampleType},
    frame::{FrameRef, FrameRefMut},
    node::Node,
    plugins::Filter,
    prelude::Property,
    video_info::Resolution,
};

use crate::{
    mv_gof::MVGroupOfFrames,
    mv_plane::{plane_height_luma, plane_super_offset, plane_width_luma},
    params::{MVPlaneSet, ReduceFilter, Subpel, SubpelMethod},
    util::{Pixel, plane_with_padding, plane_with_padding_mut},
};

/// Get source clip and prepare special "super" clip with multilevel
/// (hierarchical scaled) frames data. The super clip is used by both `MAnalyse`
/// and motion compensation (client) functions.
pub struct Super<'core> {
    /// Input clip
    clip: Node<'core>,
    /// Horizontal padding added to source frame (both left and right).
    /// Small padding is added for more correct motion estimation near frame
    /// borders.
    hpad: usize,
    /// Vertical padding added to source frame (both top and bottom).
    vpad: usize,
    /// Accuracy of the motion estimation. Value can only be 1, 2 or 4.
    ///
    /// - 1 means a precision to the pixel
    /// - 2 means a precision to half a pixel,
    /// - 4 means a precision to quarter a pixel, produced by spatial
    ///   interpolation (more accurate but slower and not always better due to
    ///   big level scale step).
    pel: Subpel,
    /// Number of hierarchical levels in super clip frames. `Analyse` is needed
    /// in all levels, but for other client functions single finest level is
    /// enough (coarser levels are not used).
    ///
    /// Default `0` = auto, all levels are produced
    levels: usize,
    /// If set to true, it allows to also prepare chroma planes in super clip.
    chroma: bool,
    /// subpixel interpolation method for pel=2,4.
    ///
    /// - 0 for soft interpolation (bilinear)
    /// - 1 for bicubic interpolation (4-tap Catmull-Rom)
    /// - 2 for sharper Wiener interpolation (6-tap, similar o Lanczos).
    sharp: SubpelMethod,
    /// Hierarchical levels smoothing and reducing (halving) filter.
    ///
    /// - 0 is simple 4 pixels averaging like unfiltered `SimpleResize`
    /// - 1 is triangle (shifted) filter like `ReduceBy2` for more smoothing
    ///   (decrease aliasing)
    /// - 2 is triangle filter like `BilinearResize` for even more smoothing
    /// - 3 is quadratic filter for even more smoothing
    /// - 4 is cubic filter like `BicubicResize(b=1, c=0)` for even more
    ///   smoothing
    rfilter: ReduceFilter,
    /// Optional upsampled source clip instead of using internal subpixel
    /// interpolation (for `pel>1`). Pixels at rows and columns positions
    /// multiple to `pel` (0,2,4,... for `pel=2`) (without padding) must be
    /// original source pixels, other pixels must be interpolated.
    ///
    /// Example for `pel=2`:
    /// `LanczosResize(width*2,height*2,src_left=0.25,src_top=0.25)`
    ///
    /// Another useful example is EEDI2 edge-directed resampling.
    pelclip: Option<Node<'core>>,

    // Internal fields
    width: NonZeroUsize,
    height: NonZeroUsize,
    format: Format<'core>,
    super_width: NonZeroUsize,
    super_height: NonZeroUsize,
    x_ratio_uv: NonZeroU8,
    y_ratio_uv: NonZeroU8,
    is_pelclip_padded: bool,
}

impl<'core> Super<'core> {
    pub fn new(
        clip: Node<'core>,
        hpad: Option<i64>,
        vpad: Option<i64>,
        pel: Option<i64>,
        levels: Option<i64>,
        chroma: Option<i64>,
        sharp: Option<i64>,
        rfilter: Option<i64>,
        pelclip: Option<Node<'core>>,
    ) -> Result<Self> {
        // Parse arguments
        let hpad = hpad.map(usize::try_from).unwrap_or(Ok(16))?;
        let vpad = vpad.map(usize::try_from).unwrap_or(Ok(16))?;
        let pel = pel.map(Subpel::try_from).unwrap_or(Ok(Subpel::Half))?;
        let mut levels = levels.map(usize::try_from).unwrap_or(Ok(0))?;
        let mut chroma = chroma.map(|chroma| chroma > 0).unwrap_or(true);
        let sharp = sharp
            .map(SubpelMethod::try_from)
            .unwrap_or(Ok(SubpelMethod::Wiener))?;
        let rfilter = rfilter
            .map(ReduceFilter::try_from)
            .unwrap_or(Ok(ReduceFilter::Bilinear))?;

        // Validate video info
        let video_info = clip.info();
        let (width, height) = match video_info.resolution {
            vapoursynth::prelude::Property::Variable => {
                bail!("Super: variable resolution input clips are not supported")
            }
            // SAFETY: width and height must be positive
            vapoursynth::prelude::Property::Constant(resolution) => unsafe {
                (
                    NonZeroUsize::new_unchecked(resolution.width),
                    NonZeroUsize::new_unchecked(resolution.height),
                )
            },
        };
        let format = match video_info.format {
            vapoursynth::prelude::Property::Variable => {
                bail!("Super: variable format input clips are not supported")
            }
            vapoursynth::prelude::Property::Constant(format) => format,
        };
        if format.bits_per_sample() > 16 {
            bail!("Super: input clip must be 8-16 bits");
        }
        if format.sample_type() != SampleType::Integer {
            bail!("Super: input clip must be integer format");
        }
        if ![ColorFamily::YUV, ColorFamily::Gray].contains(&format.color_family())
            || format.sub_sampling_w() > 1
            || format.sub_sampling_h() > 1
        {
            bail!("Super: input clip must be GRAY, 420, 422, 440, or 444");
        }

        // Compute internal parameters
        if format.color_family() == ColorFamily::Gray {
            chroma = false;
        }

        // SAFETY: operation cannot result in zero
        let (x_ratio_uv, y_ratio_uv) = unsafe {
            (
                NonZeroU8::new_unchecked(1 << format.sub_sampling_w()),
                NonZeroU8::new_unchecked(1 << format.sub_sampling_h()),
            )
        };

        let mut levels_max = 0;
        while plane_height_luma(height, levels_max, y_ratio_uv, vpad).get()
            >= y_ratio_uv.get() as usize * 2
            && plane_width_luma(width, levels_max, x_ratio_uv, hpad).get()
                >= x_ratio_uv.get() as usize * 2
        {
            levels_max += 1;
        }
        if levels == 0 || levels > levels_max {
            levels = levels_max;
        }
        debug_assert!(levels > 0);

        // Validate `pelclip` video info
        let (use_pelclip, is_pelclip_padded) = if let Some(ref pelclip) = pelclip {
            let pelclip_info = pelclip.info();
            let (pelclip_w, pelclip_h) = match pelclip_info.resolution {
                vapoursynth::prelude::Property::Variable => {
                    bail!("Super: 'pelclip' must be constant resolution")
                }
                // SAFETY: width and height must be positive
                vapoursynth::prelude::Property::Constant(resolution) => unsafe {
                    (
                        NonZeroUsize::new_unchecked(resolution.width),
                        NonZeroUsize::new_unchecked(resolution.height),
                    )
                },
            };
            match pelclip_info.format {
                vapoursynth::prelude::Property::Variable => {
                    bail!("Super: 'pelclip' must be constant format")
                }
                vapoursynth::prelude::Property::Constant(pelclip_format) => {
                    if pelclip_format != format {
                        bail!("Super: 'pelclip' must have same format as input clip");
                    }
                }
            };

            if pel >= Subpel::Half {
                let pel = NonZeroUsize::from(pel);
                if pelclip_w == width.saturating_mul(pel) && pelclip_h == height.saturating_mul(pel)
                {
                    (true, false)
                } else if pelclip_w == width.saturating_add(hpad * 2).saturating_mul(pel)
                    && pelclip_h == height.saturating_add(vpad * 2).saturating_mul(pel)
                {
                    (true, true)
                } else {
                    bail!(
                        "Super: 'pelclip' dimensions must be multiples of the input clip's \
                         dimensions"
                    );
                }
            } else {
                (false, false)
            }
        } else {
            (false, false)
        };

        let mut super_width = width.saturating_add(2 * hpad);
        // SAFETY: cannot return zero as long as `levels` is positive
        let mut super_height = unsafe {
            NonZeroUsize::new_unchecked(
                plane_super_offset(false, height, levels, pel, vpad, super_width, y_ratio_uv)
                    / super_width,
            )
        };
        if y_ratio_uv.get() == 2 && super_height.get() & 1 > 0 {
            super_height = super_height.saturating_add(1);
        }
        if x_ratio_uv.get() == 2 && super_width.get() & 1 > 0 {
            super_width = super_width.saturating_add(1);
        }

        Ok(Self {
            clip,
            hpad,
            vpad,
            pel,
            levels,
            chroma,
            sharp,
            rfilter,
            pelclip: if use_pelclip { pelclip } else { None },
            width,
            height,
            format,
            super_width,
            super_height,
            x_ratio_uv,
            y_ratio_uv,
            is_pelclip_padded,
        })
    }

    fn get_frame_internal<T: Pixel>(
        &self,
        core: vapoursynth::core::CoreRef<'core>,
        context: vapoursynth::plugins::FrameContext,
        n: usize,
    ) -> Result<FrameRef<'core>> {
        let src = self
            .clip
            .get_frame_filter(context, n)
            .expect("Super: called get_frame_filter before request_frame_filter (clip)");

        let src_pel = self.pelclip.as_ref().map(|pelclip| {
            pelclip.get_frame_filter(context, n).expect(
                "Super: called get_frame_filter before request_frame_filter (pelclip), this \
                 should not happen!",
            )
        });

        // SAFETY: We write to the planes before returning
        let mut dest = unsafe {
            let mut dest = FrameRefMut::new_uninitialized(
                core,
                Some(&src),
                self.format,
                Resolution {
                    width: self.super_width.get(),
                    height: self.super_height.get(),
                },
            );
            for plane in 0..self.format.plane_count() {
                match self.format.bytes_per_sample() {
                    1 => {
                        plane_with_padding_mut(&mut dest, plane)
                            .expect("Super: plane should exist but does not")
                            .fill(0u8);
                    }
                    2 => {
                        plane_with_padding_mut(&mut dest, plane)
                            .expect("Super: plane should exist but does not")
                            .fill(0u16);
                    }
                    _ => unreachable!("Super: does not support clips greater than 16 bits"),
                }
            }
            dest
        };

        let bytes_per_sample = size_of::<T>();
        let mode_yuv = if self.chroma {
            MVPlaneSet::YUVPLANES
        } else {
            MVPlaneSet::YPLANE
        };
        // SAFETY: strides must be at least width and non-zero
        let dest_pitch = unsafe {
            [
                NonZeroUsize::new_unchecked(dest.stride(0) / bytes_per_sample),
                NonZeroUsize::new_unchecked(dest.stride(1) / bytes_per_sample),
                NonZeroUsize::new_unchecked(dest.stride(2) / bytes_per_sample),
            ]
        };
        let mut src_gof = MVGroupOfFrames::new(
            self.levels,
            self.width,
            self.height,
            self.pel,
            self.hpad,
            self.vpad,
            mode_yuv,
            self.x_ratio_uv,
            self.y_ratio_uv,
            NonZeroU8::try_from(self.format.bits_per_sample())?,
            &dest_pitch,
            self.format,
        )?;

        for plane in 0..self.format.plane_count() {
            if let Some(plane_ref) = src_gof.frames[0].planes.get_mut(plane) {
                plane_ref.fill_plane(
                    plane_with_padding::<T>(&src, plane)
                        .expect("Super: source plane should exist but does not"),
                    // SAFETY: stride must be at least width and non-zero
                    unsafe { NonZeroUsize::new_unchecked(src.stride(plane) / bytes_per_sample) },
                    plane_with_padding_mut(&mut dest, plane)
                        .expect("Super: destination plane should exist but does not"),
                );
            }
        }

        let planes = [MVPlaneSet::YPLANE, MVPlaneSet::UPLANE, MVPlaneSet::VPLANE];
        src_gof.reduce::<T>(mode_yuv, self.rfilter, &mut dest);
        src_gof.pad::<T>(mode_yuv, &mut dest);

        if let Some(pel_clip) = src_pel.as_ref() {
            let src_frames = &mut src_gof.frames[0];

            for plane in 0..self.format.plane_count() {
                let src_pel = plane_with_padding::<T>(pel_clip, plane)
                    .expect("Super: pelclip plane should exist but does not");
                // SAFETY: stride must be at least width and non-zero
                let src_pel_pitch = unsafe {
                    NonZeroUsize::new_unchecked(pel_clip.stride(plane) / bytes_per_sample)
                };
                let src_plane = &mut src_frames.planes[plane];
                if !(mode_yuv & planes[plane]).is_empty() {
                    src_plane.refine_ext(
                        src_pel,
                        src_pel_pitch,
                        self.is_pelclip_padded,
                        plane_with_padding_mut(&mut dest, plane)
                            .expect("Super: destination plane should exist but does not"),
                    );
                }
            }
        } else {
            src_gof.refine::<T>(mode_yuv, self.sharp, &mut dest);
        }

        if n == 0 {
            // Set properties for the first frame
            let mut props = dest.props_mut();
            props.set_int("Super_height", self.height.get() as i64)?;
            props.set_int("Super_hpad", self.hpad as i64)?;
            props.set_int("Super_vpad", self.vpad as i64)?;
            props.set_int("Super_pel", usize::from(self.pel) as i64)?;
            props.set_int("Super_modeyuv", mode_yuv.bits() as i64)?;
            props.set_int("Super_levels", self.levels as i64)?;
        }

        Ok(dest.into())
    }
}

impl<'core> Filter<'core> for Super<'core> {
    fn video_info(
        &self,
        _api: vapoursynth::prelude::API,
        _core: vapoursynth::core::CoreRef<'core>,
    ) -> Vec<vapoursynth::video_info::VideoInfo<'core>> {
        let mut info = self.clip.info();
        info.resolution = Property::Constant(Resolution {
            width: self.super_width.get(),
            height: self.super_height.get(),
        });
        vec![info]
    }

    fn get_frame_initial(
        &self,
        _api: vapoursynth::prelude::API,
        _core: vapoursynth::core::CoreRef<'core>,
        context: vapoursynth::plugins::FrameContext,
        n: usize,
    ) -> std::result::Result<Option<vapoursynth::prelude::FrameRef<'core>>, anyhow::Error> {
        self.clip.request_frame_filter(context, n);
        if let Some(ref pelclip) = self.pelclip {
            pelclip.request_frame_filter(context, n);
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
            _ => bail!("Super: does not support clips greater than 16 bits"),
        }
    }
}
