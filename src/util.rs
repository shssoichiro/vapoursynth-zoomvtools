use anyhow::{Result, bail};
use core::slice;
use std::{
    convert::TryFrom,
    mem::transmute,
    num::NonZeroUsize,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use vapoursynth::{frame::Frame, prelude::Component};

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

/// Gets a slice to the plane's data including its padding.
/// The `plane` function in Vapoursynth fails if a plane has padding,
/// but we need access to the padding, so we use this function to do so.
pub fn plane_with_padding<'a, T: Pixel>(frame: &'a Frame, plane: usize) -> Result<&'a [T]> {
    if frame.format().plane_count() < plane + 1 {
        bail!("Tried to get plane not present in frame");
    }

    let data_ptr = frame.data_ptr(plane);
    let stride = frame.stride(plane);
    let height = frame.height(plane);
    let bytes_per_pixel = size_of::<T>();

    // SAFETY: We know the layout of the plane
    Ok(unsafe {
        slice::from_raw_parts(
            transmute::<*const u8, *const T>(data_ptr),
            stride * height / bytes_per_pixel,
        )
    })
}

/// Gets a slice to the plane's data including its padding.
/// The `plane` function in Vapoursynth fails if a plane has padding,
/// but we need access to the padding, so we use this function to do so.
pub fn plane_with_padding_mut<'a, T: Pixel>(
    frame: &'a mut Frame,
    plane: usize,
) -> Result<&'a mut [T]> {
    if frame.format().plane_count() < plane + 1 {
        bail!("Tried to get plane not present in frame");
    }

    let data_ptr = frame.data_ptr_mut(plane);
    let stride = frame.stride(plane);
    let height = frame.height(plane);
    let bytes_per_pixel = size_of::<T>();

    // SAFETY: We know the layout of the plane
    Ok(unsafe {
        slice::from_raw_parts_mut(
            transmute::<*mut u8, *mut T>(data_ptr),
            stride * height / bytes_per_pixel,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vs_bitblt_same_stride() {
        // Test case where src_stride == dst_stride == row_size
        let src = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut dest = vec![0u8; 9];
        let stride = NonZeroUsize::new(3).unwrap();
        let row_size = NonZeroUsize::new(3).unwrap();
        let height = NonZeroUsize::new(3).unwrap();

        vs_bitblt(&mut dest, stride, &src, stride, row_size, height);

        assert_eq!(dest, src, "Entire buffer should be copied exactly");
    }

    #[test]
    fn test_vs_bitblt_different_stride() {
        // Test case where strides are larger than row_size
        let src = vec![
            1u8, 2, 3, 0, 0, // src_stride = 5
            4, 5, 6, 0, 0, 7, 8, 9, 0, 0,
        ];
        let mut dest = vec![
            0u8, 0, 0, 0, 0, 0, // dest_stride = 6
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let src_stride = NonZeroUsize::new(5).unwrap();
        let dest_stride = NonZeroUsize::new(6).unwrap();
        let row_size = NonZeroUsize::new(3).unwrap();
        let height = NonZeroUsize::new(3).unwrap();

        vs_bitblt(&mut dest, dest_stride, &src, src_stride, row_size, height);

        // Check that each row was copied correctly
        assert_eq!(&dest[0..3], &[1, 2, 3], "First row should match");
        assert_eq!(&dest[6..9], &[4, 5, 6], "Second row should match");
        assert_eq!(&dest[12..15], &[7, 8, 9], "Third row should match");

        // Check that padding remains untouched
        assert_eq!(
            &dest[3..6],
            &[0, 0, 0],
            "First row padding should be unchanged"
        );
        assert_eq!(
            &dest[9..12],
            &[0, 0, 0],
            "Second row padding should be unchanged"
        );
        assert_eq!(
            &dest[15..18],
            &[0, 0, 0],
            "Third row padding should be unchanged"
        );
    }
}
