use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::Result;
use smallvec::SmallVec;

use crate::params::Subpel;

pub struct MVFrame<'a> {
    planes: SmallVec<[MVPlane<'a>; 3]>,
    chroma: bool,
}

impl<'a> MVFrame<'a> {
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
        src_planes: SmallVec<[&'a [u8]; 3]>,
        pitch: [NonZeroUsize; 3],
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
        for i in 0..(if chroma { 3 } else { 1 }) {
            planes.push(MVPlane::new(
                width[i],
                height[i],
                pel,
                hpad[i],
                vpad[i],
                bits_per_sample,
                src_planes[i],
                pitch[i],
            )?);
        }

        Ok(Self { planes, chroma })
    }
}

pub struct MVPlane<'a> {
    plane: SmallVec<[&'a [u8]; 16]>,
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

impl<'a> MVPlane<'a> {
    pub fn new(
        width: NonZeroUsize,
        height: NonZeroUsize,
        pel: Subpel,
        hpad: usize,
        vpad: usize,
        bits_per_sample: NonZeroU8,
        src_plane: &'a [u8],
        pitch: NonZeroUsize,
    ) -> Result<Self> {
        let pel_val = usize::from(pel);
        let padded_width = width.saturating_add(2 * hpad);
        let padded_height = height.saturating_add(2 * vpad);
        let bytes_per_sample = NonZeroU8::try_from(bits_per_sample.saturating_add(7).get() / 8)?;
        let offset_padding = pitch.get() * vpad + hpad * bytes_per_sample.get() as usize;

        let mut plane = SmallVec::with_capacity(pel_val * pel_val);
        for i in 0..(pel_val * pel_val) {
            let offset = i * pitch.get() * padded_height.get();
            plane.push(&src_plane[offset..]);
        }

        Ok(Self {
            plane,
            width,
            height,
            padded_width,
            padded_height,
            hpad,
            vpad,
            hpad_pel: hpad * pel_val,
            vpad_pel: vpad * pel_val,
            bits_per_sample,
            bytes_per_sample,
            pel,
            pitch,
            offset_padding,
            is_padded: false,
            is_refined: false,
            is_filled: false,
        })
    }
}

pub fn plane_height_luma(
    src_height: NonZeroUsize,
    level: u16,
    y_ratio_uv: NonZeroUsize,
    vpad: usize,
) -> NonZeroUsize {
    let mut height = src_height.get();
    let y_ratio_uv_val = y_ratio_uv.get();

    for _i in 1..=level {
        height = if vpad >= y_ratio_uv_val {
            (height / y_ratio_uv_val).div_ceil(2) * y_ratio_uv_val
        } else {
            ((height / y_ratio_uv_val) / 2) * y_ratio_uv_val
        };
    }

    // SAFETY: must be non-zero because `height` is at least equal to its ratio
    unsafe { NonZeroUsize::new_unchecked(height) }
}

pub fn plane_width_luma(
    src_width: NonZeroUsize,
    level: u16,
    x_ratio_uv: NonZeroUsize,
    hpad: usize,
) -> NonZeroUsize {
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

    // SAFETY: must be non-zero because `width` is at least equal to its ratio
    unsafe { NonZeroUsize::new_unchecked(width) }
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
    let mut height; // luma or chroma

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
                )
                .get()
                    / y_ratio_uv_val
            } else {
                plane_height_luma(src_height, i, y_ratio_uv, vpad).get()
            };

            offset += plane_pitch_val * (height + vpad * 2);
        }
    }

    offset
}
