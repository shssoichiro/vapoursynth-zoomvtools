//! Group of Frames code for managing a hierarchical frame structure

#[cfg(test)]
mod tests;

use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::Result;
use smallvec::SmallVec;
use vapoursynth::{format::Format, frame::Frame};

use crate::{
    mv_frame::MVFrame,
    mv_plane::{plane_height_luma, plane_super_offset, plane_width_luma},
    params::{MVPlaneSet, ReduceFilter, Subpel, SubpelMethod},
    util::Pixel,
};

#[derive(Debug, Clone)]
pub struct MVGroupOfFrames {
    level_count: usize,
    width: [NonZeroUsize; 3],
    height: [NonZeroUsize; 3],
    pel: Subpel,
    hpad: [usize; 3],
    vpad: [usize; 3],
    x_ratio_uv: NonZeroU8,
    y_ratio_uv: NonZeroU8,
    pub frames: Box<[MVFrame]>,
}

impl MVGroupOfFrames {
    pub fn new(
        level_count: usize,
        width: NonZeroUsize,
        height: NonZeroUsize,
        pel: Subpel,
        hpad: usize,
        vpad: usize,
        yuv_mode: MVPlaneSet,
        x_ratio_uv: NonZeroU8,
        y_ratio_uv: NonZeroU8,
        bits_per_sample: NonZeroU8,
        pitch: &[NonZeroUsize; 3],
        format: Format,
    ) -> Result<Self> {
        // SAFETY: Width must be at least the value of its ratio
        let chroma_width =
            unsafe { NonZeroUsize::new_unchecked(width.get() / x_ratio_uv.get() as usize) };
        // SAFETY: Height must be at least the value of its ratio
        let chroma_height =
            unsafe { NonZeroUsize::new_unchecked(height.get() / y_ratio_uv.get() as usize) };
        let chroma_hpad = hpad / x_ratio_uv.get() as usize;
        let chroma_vpad = vpad / y_ratio_uv.get() as usize;

        let mut this = Self {
            level_count,
            width: [width, chroma_width, chroma_width],
            height: [height, chroma_height, chroma_height],
            pel,
            hpad: [hpad, chroma_hpad, chroma_hpad],
            vpad: [vpad, chroma_vpad, chroma_vpad],
            x_ratio_uv,
            y_ratio_uv,
            frames: Default::default(),
        };

        let mut frames = Vec::with_capacity(level_count);

        for i in 0..level_count {
            let width_i = plane_width_luma(this.width[0], i, this.x_ratio_uv, this.hpad[0]);
            let height_i = plane_height_luma(this.height[0], i, this.y_ratio_uv, this.vpad[0]);
            let mut plane_offsets = SmallVec::with_capacity(3);
            for plane in 0..format.plane_count() {
                let offset = plane_super_offset(
                    plane > 0,
                    this.height[plane],
                    i,
                    this.pel,
                    this.vpad[plane],
                    pitch[plane],
                    this.y_ratio_uv,
                );
                plane_offsets.push(offset);
            }

            frames.push(MVFrame::new(
                width_i,
                height_i,
                if i == 0 { pel } else { Subpel::Full },
                this.hpad[0],
                this.vpad[0],
                yuv_mode,
                this.x_ratio_uv,
                this.y_ratio_uv,
                bits_per_sample,
                &plane_offsets,
                pitch,
            )?);
        }

        this.frames = frames.into_boxed_slice();

        Ok(this)
    }

    pub fn reduce<T: Pixel>(&mut self, mode: MVPlaneSet, filter: ReduceFilter, frame: &mut Frame) {
        for i in 0..(self.level_count - 1) {
            self.frames[i]
                .clone()
                .reduce_to::<T>(&mut self.frames[i + 1], mode, filter, frame);
            self.frames[i + 1].pad::<T>(MVPlaneSet::YUVPLANES, frame);
        }
    }

    pub fn pad<T: Pixel>(&mut self, mode: MVPlaneSet, frame: &mut Frame) {
        self.frames[0].pad::<T>(mode, frame);
    }

    pub fn refine<T: Pixel>(&mut self, mode: MVPlaneSet, subpel: SubpelMethod, frame: &mut Frame) {
        self.frames[0].refine::<T>(mode, subpel, frame);
    }
}
