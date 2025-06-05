#[cfg(test)]
mod tests;

use anyhow::{Result, bail};
use core::slice;
use std::{
    convert::TryFrom,
    mem::transmute,
    num::NonZeroUsize,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use vapoursynth::{frame::Frame, prelude::Component};

#[cfg(target_arch = "x86_64")]
cpufeatures::new!(cpuid_avx2, "avx2");

#[cfg(target_arch = "x86_64")]
pub use cpuid_avx2::get as has_avx2;

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
    #[must_use]
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
    #[must_use]
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

/// Gets both immutable and mutable slices to the same plane's data including its padding.
/// This function allows safe access to both source and destination views of the same plane
/// data without violating Rust's borrowing rules by using raw pointers internally.
///
/// This is specifically designed to eliminate the need for cloning plane data when we need
/// to read from one part and write to another part of the same plane.
///
/// # Safety
/// The caller must ensure that the returned slices do not overlap in their actual usage.
/// While both slices reference the same underlying memory, they should be used to access
/// different logical regions (e.g., source data vs destination data within the plane).
pub unsafe fn plane_with_padding_split<'a, T: Pixel>(
    frame: &'a mut Frame,
    plane: usize,
) -> Result<(&'a [T], &'a mut [T])> {
    if frame.format().plane_count() < plane + 1 {
        bail!("Tried to get plane not present in frame");
    }

    let data_ptr = frame.data_ptr_mut(plane);
    let stride = frame.stride(plane);
    let height = frame.height(plane);
    let bytes_per_pixel = size_of::<T>();
    let total_len = stride * height / bytes_per_pixel;

    // SAFETY: We create two slices from the same memory region, but the caller
    // is responsible for ensuring they don't overlap in actual usage.
    // This is similar to how split_at_mut works, but for the same logical data.
    unsafe {
        let src_slice = slice::from_raw_parts(transmute::<*mut u8, *const T>(data_ptr), total_len);
        let dest_slice =
            slice::from_raw_parts_mut(transmute::<*mut u8, *mut T>(data_ptr), total_len);
        Ok((src_slice, dest_slice))
    }
}

pub fn luma_mean<T: Pixel>(
    width: NonZeroUsize,
    height: NonZeroUsize,
    src: &[T],
    src_pitch: NonZeroUsize,
) -> u64 {
    todo!()
}
