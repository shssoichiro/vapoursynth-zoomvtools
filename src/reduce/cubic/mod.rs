#[cfg(target_arch = "x86_64")]
mod avx2;
mod rust;

#[cfg(test)]
mod tests;

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
pub fn reduce_cubic<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    #[cfg(target_arch = "x86_64")]
    if crate::util::has_avx2() {
        // SAFETY: We check for AVX2 first
        unsafe {
            avx2::reduce_cubic(dest, src, dest_pitch, src_pitch, dest_width, dest_height);
        }
    } else {
        rust::reduce_cubic(dest, src, dest_pitch, src_pitch, dest_width, dest_height);
    }

    #[cfg(not(target_arch = "x86_64"))]
    rust::reduce_cubic(dest, src, dest_pitch, src_pitch, dest_width, dest_height);
}
