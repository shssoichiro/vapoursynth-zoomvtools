#[cfg(test)]
mod tests;

use std::{
    convert::TryFrom,
    num::NonZeroUsize,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use vapoursynth::prelude::Component;

pub trait Pixel:
    Component
    + Clone
    + Copy
    + Add<Self>
    + AddAssign<Self>
    + Sub<Self>
    + SubAssign<Self>
    + Mul<Self>
    + MulAssign<Self>
    + Div<Self>
    + DivAssign<Self>
    + Into<u16>
    + Into<u32>
    + Into<u64>
    + From<u8>
    + TryFrom<u16>
    + TryFrom<u32>
    + TryFrom<u64>
{
}

impl<T> Pixel for T where
    T: Component
        + Clone
        + Copy
        + Add<Self>
        + AddAssign<Self>
        + Sub<Self>
        + SubAssign<Self>
        + Mul<Self>
        + MulAssign<Self>
        + Div<Self>
        + DivAssign<Self>
        + Into<u16>
        + Into<u32>
        + Into<u64>
        + From<u8>
        + TryFrom<u16>
        + TryFrom<u32>
        + TryFrom<u64>
{
}

pub fn vs_bitblt<T: Pixel>(
    dest: &mut [T],
    dest_stride: NonZeroUsize,
    src: &[T],
    src_stride: NonZeroUsize,
    row_size: NonZeroUsize,
    height: NonZeroUsize,
) {
    let height = height.get();
    let row_size = row_size.get();
    let src_stride = src_stride.get();
    let dest_stride = dest_stride.get();

    if src_stride == dest_stride && src_stride == row_size {
        // Fast path: single copy when strides match row size
        dest[..row_size * height].copy_from_slice(&src[..row_size * height]);
    } else {
        // Copy row by row when strides differ
        for i in 0..height {
            let src_start = i * src_stride;
            let dest_start = i * dest_stride;
            dest[dest_start..dest_start + row_size]
                .copy_from_slice(&src[src_start..src_start + row_size]);
        }
    }
}
