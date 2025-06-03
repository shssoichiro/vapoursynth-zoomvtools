#[cfg(test)]
mod tests;

use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::Result;
use smallvec::SmallVec;
use vapoursynth::frame::Frame;

use crate::{
    mv_plane::MVPlane,
    params::{MVPlaneSet, ReduceFilter, Subpel, SubpelMethod},
    util::{Pixel, plane_with_padding_mut, plane_with_padding_split},
};

#[derive(Debug, Clone)]
pub struct MVFrame {
    pub planes: SmallVec<[MVPlane; 3]>,
}

impl MVFrame {
    pub fn new(
        width: NonZeroUsize,
        height: NonZeroUsize,
        pel: Subpel,
        hpad: usize,
        vpad: usize,
        yuv_mode: MVPlaneSet,
        x_ratio_uv: NonZeroU8,
        y_ratio_uv: NonZeroU8,
        bits_per_sample: NonZeroU8,
        plane_offsets: &SmallVec<[usize; 3]>,
        pitch: &[NonZeroUsize; 3],
    ) -> Result<Self> {
        // SAFETY: Width must be at least the value of its ratio
        let chroma_width =
            unsafe { NonZeroUsize::new_unchecked(width.get() / x_ratio_uv.get() as usize) };
        // SAFETY: Height must be at least the value of its ratio
        let chroma_height =
            unsafe { NonZeroUsize::new_unchecked(height.get() / y_ratio_uv.get() as usize) };
        let chroma_hpad = hpad / x_ratio_uv.get() as usize;
        let chroma_vpad = vpad / y_ratio_uv.get() as usize;

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

        Ok(Self { planes })
    }

    pub(crate) fn reduce_to<T: Pixel>(
        &self,
        reduced_frame: &mut MVFrame,
        mode: MVPlaneSet,
        filter: ReduceFilter,
        frame: &mut Frame,
    ) {
        for i in 0..3 {
            if let Some(plane) = self.planes.get(i) {
                if (mode.bits() & (1 << i)) > 0 {
                    let reduced_pitch = reduced_frame.planes[i].pitch;
                    let (width, height) = (
                        reduced_frame.planes[i].width,
                        reduced_frame.planes[i].height,
                    );
                    // Use the new helper function to avoid cloning the source data
                    // SAFETY: The windows inside each plane are set up so that they do not overlap.
                    unsafe {
                        let (src, dest) = plane_with_padding_split::<T>(frame, i)
                            .expect("Super: plane should exist but does not");
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
        }
    }

    pub(crate) fn pad<T: Pixel>(&mut self, mode: MVPlaneSet, frame: &mut Frame) {
        for i in 0..3 {
            if let Some(plane) = self.planes.get_mut(i) {
                if (mode.bits() & (1 << i)) > 0 {
                    plane.pad(
                        plane_with_padding_mut::<T>(frame, i)
                            .expect("Super: source plane should exist but does not"),
                    );
                }
            }
        }
    }

    pub(crate) fn refine<T: Pixel>(
        &mut self,
        mode: MVPlaneSet,
        subpel: SubpelMethod,
        frame: &mut Frame,
    ) {
        for i in 0..3 {
            if let Some(plane) = self.planes.get_mut(i) {
                if (mode.bits() & (1 << i)) > 0 {
                    plane.refine::<T>(
                        subpel,
                        plane_with_padding_mut::<T>(frame, i)
                            .expect("Super: source plane should exist but does not"),
                    );
                }
            }
        }
    }
}
