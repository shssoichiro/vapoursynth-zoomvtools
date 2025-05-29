//! Group of Frames code for managing a hierarchical frame structure

use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::Result;
use smallvec::SmallVec;
use vapoursynth::{format::Format, frame::Frame};

use crate::{
    mv_frame::MVFrame,
    mv_plane::{MVPlaneSet, plane_height_luma, plane_super_offset, plane_width_luma},
    params::{ReduceFilter, Subpel, SubpelMethod},
    util::Pixel,
};

#[derive(Debug, Clone)]
pub struct MVGroupOfFrames {
    level_count: u16,
    width: [NonZeroUsize; 3],
    height: [NonZeroUsize; 3],
    pel: Subpel,
    hpad: [usize; 3],
    vpad: [usize; 3],
    x_ratio_uv: NonZeroUsize,
    y_ratio_uv: NonZeroUsize,
    yuv_mode: MVPlaneSet,
    pub frames: Box<[MVFrame]>,
}

impl MVGroupOfFrames {
    pub fn new(
        level_count: u16,
        width: NonZeroUsize,
        height: NonZeroUsize,
        pel: Subpel,
        hpad: usize,
        vpad: usize,
        yuv_mode: MVPlaneSet,
        x_ratio_uv: NonZeroUsize,
        y_ratio_uv: NonZeroUsize,
        bits_per_sample: NonZeroU8,
        pitch: &[NonZeroUsize; 3],
        format: Format,
    ) -> Result<Self> {
        // SAFETY: Width must be at least the value of its ratio
        let chroma_width = unsafe { NonZeroUsize::new_unchecked(width.get() / x_ratio_uv.get()) };
        // SAFETY: Height must be at least the value of its ratio
        let chroma_height = unsafe { NonZeroUsize::new_unchecked(height.get() / y_ratio_uv.get()) };
        let chroma_hpad = hpad / x_ratio_uv.get();
        let chroma_vpad = vpad / y_ratio_uv.get();

        let mut this = Self {
            level_count,
            width: [width, chroma_width, chroma_width],
            height: [height, chroma_height, chroma_height],
            pel,
            hpad: [hpad, chroma_hpad, chroma_hpad],
            vpad: [vpad, chroma_vpad, chroma_vpad],
            x_ratio_uv,
            y_ratio_uv,
            yuv_mode,
            frames: Default::default(),
        };

        let mut frames = Vec::with_capacity(level_count as usize);

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
                plane_offsets[plane] = offset;
            }

            frames[i as usize] = MVFrame::new(
                width_i,
                height_i,
                Subpel::Full,
                this.hpad[0],
                this.vpad[0],
                yuv_mode,
                this.x_ratio_uv,
                this.y_ratio_uv,
                bits_per_sample,
                &plane_offsets,
                pitch,
            )?;
        }

        this.frames = frames.into_boxed_slice();

        Ok(this)
    }

    pub fn reduce<T: Pixel>(&mut self, mode: MVPlaneSet, filter: ReduceFilter, frame: &mut Frame) {
        for i in 0..(self.level_count as usize - 1) {
            self.frames[i]
                .clone()
                .reduce_to::<T>(&mut self.frames[i + 1], mode, filter, frame);
            self.frames[i + 1].pad(MVPlaneSet::YUVPLANES);
        }
    }

    pub fn pad(&mut self, mode: MVPlaneSet) {
        todo!()
    }

    pub fn refine(&mut self, mode: MVPlaneSet, subpel: SubpelMethod) {
        todo!()
    }
}
