#[cfg(target_arch = "x86_64")]
mod avx2;
mod rust;

#[cfg(test)]
mod tests;

use crate::util::Pixel;
use cfg_if::cfg_if;
use std::num::NonZeroUsize;

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
pub fn reduce_average<T: Pixel>(
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
                    avx2::reduce_average(dest, src, dest_pitch, src_pitch, dest_width, dest_height);
                }
                return;
            }
        }
    }

    rust::reduce_average(dest, src, dest_pitch, src_pitch, dest_width, dest_height);
}
