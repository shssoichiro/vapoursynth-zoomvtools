#[cfg(test)]
mod tests;

use std::num::{NonZeroU8, NonZeroUsize};

use crate::util::Pixel;

pub fn refine_horizontal_bilinear<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    _bits_per_sample: NonZeroU8,
) {
    let mut offset = 0;
    for _j in 0..height.get() {
        for i in 0..width.get() - 1 {
            let a: u32 = src[offset + i].into();
            let b: u32 = src[offset + i + 1].into();
            dest[offset + i] = T::from_or_max((a + b).div_ceil(2));
        }
        // last column
        dest[offset + width.get() - 1] = src[offset + width.get() - 1];

        offset += pitch.get();
    }
}

pub fn refine_vertical_bilinear<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    _bits_per_sample: NonZeroU8,
) {
    let mut offset = 0;
    for _j in 0..height.get() - 1 {
        for i in 0..width.get() {
            let a: u32 = src[offset + i].into();
            let b: u32 = src[offset + i + pitch.get()].into();
            dest[offset + i] = T::from_or_max((a + b).div_ceil(2));
        }
        offset += pitch.get();
    }

    // last row
    dest[offset..offset + width.get()].copy_from_slice(&src[offset..offset + width.get()]);
}

pub fn refine_diagonal_bilinear<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    _bits_per_sample: NonZeroU8,
) {
    let mut offset = 0;

    for _j in 0..height.get() {
        for i in 0..width.get() {
            let a: u32 = src[offset + i].into();
            let b: u32 = src[offset + i + 1].into();
            let c: u32 = src[offset + i + pitch.get()].into();
            let d: u32 = src[offset + i + pitch.get() + 1].into();

            dest[offset + i] = T::from_or_max((a + b + c + d + 2) / 4);
        }
        // last column
        let a: u32 = src[offset + width.get() - 1].into();
        let b: u32 = src[offset + width.get() - 1 + pitch.get()].into();
        dest[offset + width.get() - 1] = T::from_or_max((a + b).div_ceil(2));

        offset += pitch.get();
    }

    // last row
    for i in 0..width.get() - 1 {
        let a: u32 = src[offset + i].into();
        let b: u32 = src[offset + i + pitch.get()].into();
        dest[offset + i] = T::from_or_max((a + b).div_ceil(2));
    }
    // last pixel
    dest[offset + width.get() - 1] = src[offset + width.get() - 1];
}
