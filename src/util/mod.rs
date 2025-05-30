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
    + Into<i32>
    + Into<u32>
    + Into<u64>
    + From<u8>
    + TryFrom<u16>
    + TryFrom<u32>
    + TryFrom<u64>
    + MaxValue
    + PartialOrd
    + Ord
    + PartialEq
    + Eq
{
    fn from_or_max(value: u32) -> Self;
}

impl<T> Pixel for T
where
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
        + Into<i32>
        + Into<u32>
        + Into<u64>
        + From<u8>
        + TryFrom<u16>
        + TryFrom<u32>
        + TryFrom<u64>
        + MaxValue
        + PartialOrd
        + Ord
        + PartialEq
        + Eq,
{
    fn from_or_max(value: u32) -> Self {
        Self::try_from(value).unwrap_or_else(|_| {
            // If conversion fails (shouldn't happen with our inputs), fallback to max
            Self::max_value()
        })
    }
}

pub trait MaxValue {
    fn max_value() -> Self;
}

impl MaxValue for u8 {
    fn max_value() -> Self {
        u8::MAX
    }
}

impl MaxValue for u16 {
    fn max_value() -> Self {
        u16::MAX
    }
}

/// Performs optimized bit block transfer (bitblt) between pixel buffers.
///
/// This function efficiently copies pixel data from a source buffer to a destination
/// buffer, handling different stride sizes and memory layouts. It includes an
/// optimization for the common case where both buffers have matching strides that
/// equal the row size, allowing for a single bulk copy operation.
///
/// The function is particularly useful for copying image data between buffers with
/// different padding or when extracting rectangular regions from larger images.
/// It handles the complexity of different row strides automatically.
///
/// # Parameters
/// - `dest`: Destination buffer to copy pixels into
/// - `dest_stride`: Number of pixels per row in the destination buffer (including padding)
/// - `src`: Source buffer to copy pixels from
/// - `src_stride`: Number of pixels per row in the source buffer (including padding)
/// - `row_size`: Number of pixels to copy per row (the actual image width)
/// - `height`: Number of rows to copy (the image height)
///
/// # Performance
/// - **Fast path**: When `src_stride == dest_stride == row_size`, uses single bulk copy
/// - **Standard path**: Copies row by row when strides differ, handling padding correctly
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
