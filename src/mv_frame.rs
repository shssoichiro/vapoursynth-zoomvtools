use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::Result;
use smallvec::SmallVec;

use crate::params::Subpel;

pub struct MVFrame {
    planes: SmallVec<[MVPlane; 3]>,
    chroma: bool,
}

impl MVFrame {
    pub fn new(
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
        let chroma_width = NonZeroUsize::try_from(width.get() / x_ratio_uv.get())?;
        let chroma_height = NonZeroUsize::try_from(height.get() / y_ratio_uv.get())?;
        let chroma_hpad = hpad / x_ratio_uv.get();
        let chroma_vpad = vpad / y_ratio_uv.get();

        let width = [width, chroma_width, chroma_width];
        let height = [height, chroma_height, chroma_height];
        let hpad = [hpad, chroma_hpad, chroma_hpad];
        let vpad = [vpad, chroma_vpad, chroma_vpad];

        let mut planes = SmallVec::new();
        for i in 0..(if chroma { 3 } else { 1 }) {
            planes.push(MVPlane::new(
                width[i],
                height[i],
                pel,
                hpad[i],
                vpad[i],
                bits_per_sample,
            )?);
        }

        // TODO: mvfupdate

        Ok(Self { planes, chroma })
    }
}

pub struct MVPlane {
    plane: SmallVec<[Box<[u8]>; 16]>,
    width: NonZeroUsize,
    height: NonZeroUsize,
    padded_width: NonZeroUsize,
    padded_height: NonZeroUsize,
    pitch: NonZeroUsize,
    hpad: usize,
    vpad: usize,
    offset_padding: usize,
    hpad_pel: usize,
    vpad_pel: usize,
    bits_per_sample: NonZeroU8,
    bytes_per_sample: NonZeroU8,
    pel: Subpel,
    is_padded: bool,
    is_refined: bool,
    is_filled: bool,
}

impl MVPlane {
    pub fn new(
        width: NonZeroUsize,
        height: NonZeroUsize,
        pel: Subpel,
        hpad: usize,
        vpad: usize,
        bits_per_sample: NonZeroU8,
    ) -> Result<Self> {
        let pel_val = usize::from(pel);
        let padded_width = width.saturating_add(2 * hpad);
        let padded_height = height.saturating_add(2 * vpad);

        // TODO: mvpupdate
        Ok(Self {
            plane: SmallVec::from_elem(Box::new([]), pel_val * pel_val),
            width,
            height,
            padded_width,
            padded_height,
            hpad,
            vpad,
            hpad_pel: hpad * pel_val,
            vpad_pel: vpad * pel_val,
            bits_per_sample,
            bytes_per_sample: NonZeroU8::try_from(bits_per_sample.saturating_add(7).get() / 8)?,
            pel,
            pitch: width,
            offset_padding: Default::default(),
            is_padded: Default::default(),
            is_refined: Default::default(),
            is_filled: Default::default(),
        })
    }
}

pub fn plane_height_luma(
    src_height: NonZeroUsize,
    level: u16,
    y_ratio_uv: NonZeroUsize,
    vpad: usize,
) -> usize {
    // The result should be non-zero because `y_ratio_uv` is between 1 and 4,
    // but we cannot guarantee that with current APIs.
    let mut height = src_height.get();
    let y_ratio_uv_val = y_ratio_uv.get();

    for _i in 1..=level {
        height = if vpad >= y_ratio_uv_val {
            (height / y_ratio_uv_val).div_ceil(2) * y_ratio_uv_val
        } else {
            ((height / y_ratio_uv_val) / 2) * y_ratio_uv_val
        };
    }

    height
}

pub fn plane_width_luma(
    src_width: NonZeroUsize,
    level: u16,
    x_ratio_uv: NonZeroUsize,
    hpad: usize,
) -> usize {
    // The result should be non-zero because `x_ratio_uv` is between 1 and 4,
    // but we cannot guarantee that with current APIs.
    let mut width = src_width.get();
    let x_ratio_uv_val = x_ratio_uv.get();

    for _i in 1..=level {
        width = if hpad >= x_ratio_uv_val {
            (width / x_ratio_uv_val).div_ceil(2) * x_ratio_uv_val
        } else {
            ((width / x_ratio_uv_val) / 2) * x_ratio_uv_val
        };
    }

    width
}

pub fn plane_super_offset(
    chroma: bool,
    src_height: NonZeroUsize,
    level: u16,
    pel: Subpel,
    vpad: usize,
    plane_pitch: NonZeroUsize,
    y_ratio_uv: NonZeroUsize,
) -> usize {
    // storing subplanes in superframes may be implemented by various ways
    let mut height = src_height.get(); // luma or chroma

    let mut offset;

    if level == 0 {
        offset = 0;
    } else {
        let pel = usize::from(pel);
        let plane_pitch_val = plane_pitch.get();
        let src_height_val = src_height.get();
        let y_ratio_uv_val = y_ratio_uv.get();
        offset = pel * pel * plane_pitch_val * (src_height_val + vpad * 2);

        for i in 1..level {
            // FIXME: Are we sure this should pass `src_height` and not `height?`
            height = if chroma {
                plane_height_luma(
                    src_height.saturating_mul(y_ratio_uv),
                    i,
                    y_ratio_uv,
                    vpad * y_ratio_uv_val,
                ) / y_ratio_uv_val
            } else {
                plane_height_luma(src_height, i, y_ratio_uv, vpad)
            };

            offset += plane_pitch_val * (height + vpad * 2);
        }
    }

    offset
}
