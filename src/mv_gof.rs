//! Group of Frames code for managing a hierarchical frame structure

use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::Result;
use vapoursynth::frame::{Frame, FrameRef, FrameRefMut};

use crate::{
    mv_frame::{MVFrame, MVPlane, plane_height_luma, plane_width_luma},
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
    frames: Box<[MVFrame]>,
}

impl MVGroupOfFrames {
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
        src: &Frame<'core>,
        dest: &Frame<'core>,
    ) -> Result<Self> {
        let chroma_width = NonZeroUsize::try_from(width.get() / x_ratio_uv.get())?;
        let chroma_height = NonZeroUsize::try_from(height.get() / y_ratio_uv.get())?;
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
        frames.push(MVFrame::new(
            this.width[0],
            this.height[0],
            this.pel,
            this.hpad[0],
            this.vpad[0],
            chroma,
            this.x_ratio_uv,
            this.y_ratio_uv,
            bits_per_sample,
        )?);

        for i in 1..level_count {
            let width_i = plane_width_luma(this.width[0], i, this.x_ratio_uv, this.hpad[0]);
            let height_i = plane_height_luma(this.height[0], i, this.y_ratio_uv, this.vpad[0]);

            frames[i as usize] = MVFrame::new(
                NonZeroUsize::try_from(width_i)?,
                NonZeroUsize::try_from(height_i)?,
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
        // TODO: mvgofUpdate(&pSrcGOF, pDst, nDstPitch);
        Ok(this)
    }
}
