#![allow(clippy::undocumented_unsafe_blocks)]

use std::num::NonZeroUsize;

use crate::util::Pixel;

/// Downscales an image by 2x using cubic interpolation.
///
/// This function reduces both the width and height of the source image by half
/// using a two-pass cubic filtering approach. First, vertical filtering is
/// applied to reduce the height, then horizontal filtering is applied in-place
/// to reduce the width. Cubic interpolation provides higher quality than bilinear
/// by using a wider kernel that considers more neighboring pixels for smoother results.
///
/// The cubic filter uses a 6-tap kernel with specific weights optimized for
/// downscaling while preserving image details and reducing artifacts.
///
/// # Parameters
/// - `dest`: Destination buffer to store the downscaled image
/// - `src`: Source image buffer to downscale
/// - `dest_pitch`: Number of pixels per row in the destination buffer
/// - `src_pitch`: Number of pixels per row in the source buffer
/// - `dest_width`: Width of the destination image (half of source width)
/// - `dest_height`: Height of the destination image (half of source height)
#[target_feature(enable = "avx2")]
pub fn reduce_cubic<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    match size_of::<T>() {
        1 => unsafe {
            reduce_cubic_vertical_u8(
                dest.as_mut_ptr() as *mut u8,
                src.as_ptr() as *const u8,
                dest_pitch,
                src_pitch,
                // SAFETY: non-zero constant
                dest_width.saturating_mul(NonZeroUsize::new_unchecked(2)),
                dest_height,
            );
            reduce_cubic_horizontal_inplace_u8(
                dest.as_mut_ptr() as *mut u8,
                dest_pitch,
                dest_width,
                dest_height,
            );
        },
        2 => unsafe {
            reduce_cubic_vertical_u16(
                dest.as_mut_ptr() as *mut u16,
                src.as_ptr() as *const u16,
                dest_pitch,
                src_pitch,
                // SAFETY: non-zero constant
                dest_width.saturating_mul(NonZeroUsize::new_unchecked(2)),
                dest_height,
            );
            reduce_cubic_horizontal_inplace_u16(
                dest.as_mut_ptr() as *mut u16,
                dest_pitch,
                dest_width,
                dest_height,
            );
        },
        _ => unreachable!(),
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_cubic_vertical_u8(
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
unsafe fn reduce_cubic_horizontal_inplace_u8(
    dest: *mut u8,
    dest_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    todo!()
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_cubic_vertical_u16(
    dest: *mut u16,
    src: *const u16,
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    todo!()
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_cubic_horizontal_inplace_u16(
    dest: *mut u16,
    dest_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    todo!()
}
