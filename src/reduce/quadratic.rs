#[cfg(target_arch = "x86_64")]
mod avx2;
mod rust;

#[cfg(test)]
mod tests;

use std::num::NonZeroUsize;

use cfg_if::cfg_if;

use crate::util::Pixel;

/// Downscales an image by 2x using quadratic interpolation.
///
/// This function reduces both the width and height of the source image by half
/// using a two-pass quadratic filtering approach. First, vertical filtering is
/// applied to reduce the height, then horizontal filtering is applied in-place
/// to reduce the width. Quadratic interpolation provides a balance between
/// computational efficiency and image quality, using a 6-tap kernel with
/// quadratic weighting functions.
///
/// The quadratic filter uses different weights than cubic interpolation,
/// optimized for smooth gradients while maintaining sharpness in details.
///
/// # Parameters
/// - `dest`: Destination buffer to store the downscaled image
/// - `src`: Source image buffer to downscale
/// - `dest_pitch`: Number of pixels per row in the destination buffer
/// - `src_pitch`: Number of pixels per row in the source buffer
/// - `dest_width`: Width of the destination image (half of source width)
/// - `dest_height`: Height of the destination image (half of source height)
pub fn reduce_quadratic<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    cfg_if! {
        if #[cfg(all(target_arch = "x86_64", not(feature = "no_simd")))] {
            if crate::util::has_avx2() {
                // SAFETY: We check for AVX2 first
                unsafe {
                    avx2::reduce_quadratic(dest, src, dest_pitch, src_pitch, dest_width, dest_height);
                }
                return;
            }
        }
    }

    rust::reduce_quadratic(dest, src, dest_pitch, src_pitch, dest_width, dest_height);
}
