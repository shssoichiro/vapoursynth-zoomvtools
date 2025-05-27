//! Group of Frames code for managing a hierarchical frame structure

use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::Result;
use smallvec::SmallVec;
use vapoursynth::{frame::Frame, prelude::Component};

use crate::{
    mv_frame::{MVFrame, plane_height_luma, plane_super_offset, plane_width_luma},
    params::Subpel,
};

pub struct MVGroupOfFrames<'a, T: Component> {
    level_count: u16,
    width: [NonZeroUsize; 3],
    height: [NonZeroUsize; 3],
    pel: Subpel,
    hpad: [usize; 3],
    vpad: [usize; 3],
    x_ratio_uv: NonZeroUsize,
    y_ratio_uv: NonZeroUsize,
    pub frames: Box<[MVFrame<'a, T>]>,
}

impl<'a, T: Component> MVGroupOfFrames<'a, T> {
    pub fn new<'core>(
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
        src: &'a Frame<'core>,
        dest: &Frame<'core>,
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
            frames: Default::default(),
        };

        let mut frames = Vec::with_capacity(level_count as usize);

        for i in 0..level_count {
            let width_i = plane_width_luma(this.width[0], i, this.x_ratio_uv, this.hpad[0]);
            let height_i = plane_height_luma(this.height[0], i, this.y_ratio_uv, this.vpad[0]);

            let mut planes: SmallVec<[&[T]; 3]> = SmallVec::with_capacity(3);
            // SAFETY: constant is not zero.
            let mut dest_pitch = [unsafe { NonZeroUsize::new_unchecked(1) }; 3];
            #[allow(clippy::needless_range_loop)]
            for plane in 0..(if chroma { 1 } else { 3 }) {
                // SAFETY: stride must be at least width and non-zero
                dest_pitch[plane] = unsafe { NonZeroUsize::new_unchecked(dest.stride(plane)) };
                let plane_src = src
                    .plane(plane)
                    .expect("Super: plane should exist but does not");
                let offset = plane_super_offset(
                    plane > 0,
                    this.height[plane],
                    i,
                    this.pel,
                    this.vpad[plane],
                    dest_pitch[plane],
                    this.y_ratio_uv,
                );
                planes.push(&plane_src[offset..]);
            }

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
                planes,
                dest_pitch,
            )?;
        }

        this.frames = frames.into_boxed_slice();

        Ok(this)
    }
}
