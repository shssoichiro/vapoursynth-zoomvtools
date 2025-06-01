#![allow(clippy::undocumented_unsafe_blocks)]

use std::num::NonZeroUsize;

use crate::util::Pixel;

/// Downscales an image by 2x using simple averaging of 2x2 pixel blocks.
///
/// This function reduces both the width and height of the source image by half
/// by averaging each 2x2 block of pixels into a single output pixel. The averaging
/// uses proper rounding by adding 2 before dividing by 4, ensuring accurate
/// color representation in the downscaled result.
///
/// # Parameters
/// - `dest`: Destination buffer to store the downscaled image
/// - `src`: Source image buffer to downscale
/// - `dest_pitch`: Number of pixels per row in the destination buffer
/// - `src_pitch`: Number of pixels per row in the source buffer
/// - `dest_width`: Width of the destination image (half of source width)
/// - `dest_height`: Height of the destination image (half of source height)
#[target_feature(enable = "avx2")]
pub fn reduce_average<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    match size_of::<T>() {
        1 => unsafe {
            reduce_average_u8(
                dest.as_mut_ptr() as *mut u8,
                src.as_ptr() as *const u8,
                dest_pitch,
                src_pitch,
                dest_width,
                dest_height,
            )
        },
        2 => unsafe {
            reduce_average_u16(
                dest.as_mut_ptr() as *mut u16,
                src.as_ptr() as *const u16,
                dest_pitch,
                src_pitch,
                dest_width,
                dest_height,
            )
        },
        _ => unreachable!(),
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_average_u8(
    dest: *mut u8,
    src: *const u8,
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    todo!()
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_average_u16(
    dest: *mut u16,
    src: *const u16,
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    todo!()
}
