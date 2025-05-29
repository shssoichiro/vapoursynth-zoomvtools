use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::Result;
use smallvec::SmallVec;
use vapoursynth::frame::Frame;

use crate::{
    mv_plane::{MVPlane, MVPlaneSet},
    params::{ReduceFilter, Subpel},
    util::Pixel,
};

#[derive(Debug, Clone)]
pub struct MVFrame {
    pub planes: SmallVec<[MVPlane; 3]>,
    yuv_mode: MVPlaneSet,
}

impl MVFrame {
    pub fn new(
        width: NonZeroUsize,
        height: NonZeroUsize,
        pel: Subpel,
        hpad: usize,
        vpad: usize,
        yuv_mode: MVPlaneSet,
        x_ratio_uv: NonZeroUsize,
        y_ratio_uv: NonZeroUsize,
        bits_per_sample: NonZeroU8,
        plane_offsets: &SmallVec<[usize; 3]>,
        pitch: &[NonZeroUsize; 3],
    ) -> Result<Self> {
        // SAFETY: Width must be at least the value of its ratio
        let chroma_width = unsafe { NonZeroUsize::new_unchecked(width.get() / x_ratio_uv.get()) };
        // SAFETY: Height must be at least the value of its ratio
        let chroma_height = unsafe { NonZeroUsize::new_unchecked(height.get() / y_ratio_uv.get()) };
        let chroma_hpad = hpad / x_ratio_uv.get();
        let chroma_vpad = vpad / y_ratio_uv.get();

        let width = [width, chroma_width, chroma_width];
        let height = [height, chroma_height, chroma_height];
        let hpad = [hpad, chroma_hpad, chroma_hpad];
        let vpad = [vpad, chroma_vpad, chroma_vpad];

        let mut planes = SmallVec::new();
        for i in 0..3 {
            if (yuv_mode.bits() & (1 << i)) == 0 {
                continue;
            }

            let plane = MVPlane::new(
                width[i],
                height[i],
                pel,
                hpad[i],
                vpad[i],
                bits_per_sample,
                plane_offsets[i],
                pitch[i],
            )?;
            planes.push(plane);
        }

        Ok(Self { planes, yuv_mode })
    }

    pub(crate) fn reduce_to<T: Pixel>(
        &self,
        reduced_frame: &mut MVFrame,
        mode: MVPlaneSet,
        filter: ReduceFilter,
        frame: &mut Frame,
    ) {
        for i in 0..3 {
            if let Some(plane) = self.planes.get(i)
                && (mode.bits() & (1 << i)) > 0
            {
                let reduced_pitch = reduced_frame.planes[i].pitch;
                let (width, height) = (
                    reduced_frame.planes[i].width,
                    reduced_frame.planes[i].height,
                );
                let dest_offset = reduced_frame.planes[i].subpel_window_offsets[0]
                    + reduced_frame.planes[i].offset_padding;
                let src_offset = plane.subpel_window_offsets[0] + plane.offset_padding;
                // FIXME: Having to clone the source data is not ideal.
                let src = &frame
                    .plane(i)
                    .expect("Super: source plane should exist but does not")[src_offset..]
                    .to_vec();
                let dest = &mut frame
                    .plane_mut(i)
                    .expect("Super: dest plane should exist but does not")[dest_offset..];
                plane.reduce_to::<T>(
                    &mut reduced_frame.planes[i],
                    filter,
                    dest,
                    src,
                    reduced_pitch,
                    self.planes[i].pitch,
                    width,
                    height,
                );
            }
        }
    }

    pub(crate) fn pad(&self, mode: MVPlaneSet) {
        todo!()
    }
}
