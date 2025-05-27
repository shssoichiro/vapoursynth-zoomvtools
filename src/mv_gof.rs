//! Group of Frames code for managing a hierarchical frame structure

use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::Result;
use smallvec::SmallVec;

use crate::{
    mv_frame::{MVFrame, plane_height_luma, plane_super_offset, plane_width_luma},
    params::Subpel,
};

pub struct MVGroupOfFrames {
    level_count: u16,
    width: [NonZeroUsize; 3],
    height: [NonZeroUsize; 3],
    pel: Subpel,
    hpad: [usize; 3],
    vpad: [usize; 3],
    x_ratio_uv: NonZeroUsize,
    y_ratio_uv: NonZeroUsize,
    chroma: bool,
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
        chroma: bool,
        x_ratio_uv: NonZeroUsize,
        y_ratio_uv: NonZeroUsize,
        bits_per_sample: NonZeroU8,
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
            chroma,
            frames: Default::default(),
        };

        let mut frames = Vec::with_capacity(level_count as usize);

        for i in 0..level_count {
            let width_i = plane_width_luma(this.width[0], i, this.x_ratio_uv, this.hpad[0]);
            let height_i = plane_height_luma(this.height[0], i, this.y_ratio_uv, this.vpad[0]);

            frames[i as usize] = MVFrame::new(
                width_i,
                height_i,
                Subpel::Full,
                this.hpad[0],
                this.vpad[0],
                chroma,
                this.x_ratio_uv,
                this.y_ratio_uv,
                bits_per_sample,
            )?;
        }

        this.frames = frames.into_boxed_slice();

        Ok(this)
    }

    // TODO: Merge into `new`
    pub fn update(&mut self, pitch: &[NonZeroUsize; 3]) -> Result<()> {
        for i in 0..self.level_count {
            let mut planes = SmallVec::with_capacity(3);
            for plane in 0..(if self.chroma { 1 } else { 3 }) {
                let offset = plane_super_offset(
                    plane > 0,
                    self.height[plane],
                    i,
                    self.pel,
                    self.vpad[plane],
                    pitch[plane],
                    self.y_ratio_uv,
                );
                planes[plane] = offset;
            }
            self.frames[i as usize].update(&planes, pitch);
        }
        Ok(())
    }
}
