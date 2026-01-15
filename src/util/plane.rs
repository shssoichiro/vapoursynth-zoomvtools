use core::slice;
use std::mem::transmute;

use anyhow::{Result, bail};
use vapoursynth::frame::Frame;

use crate::util::Pixel;

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
