#[cfg(target_arch = "x86_64")]
mod avx2;
mod rust;

#[cfg(test)]
mod tests;

use crate::util::Pixel;
use cfg_if::cfg_if;
use std::num::{NonZeroU8, NonZeroUsize};

/// Performs horizontal bilinear interpolation for sub-pixel motion estimation refinement.
///
/// This function applies bilinear interpolation horizontally to create sub-pixel samples
/// between existing pixels. Bilinear interpolation uses a simple 2-tap kernel that
/// averages adjacent horizontal pixels, providing fast and reasonably smooth interpolation
/// for motion estimation with sub-pixel accuracy.
///
/// The last column is copied directly from the source since there's no right neighbor.
///
/// # Parameters
/// - `src`: Source image buffer
/// - `dest`: Destination buffer for interpolated results
/// - `pitch`: Number of pixels per row in both buffers
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
/// - `_bits_per_sample`: Unused parameter for API consistency
pub fn refine_horizontal_bilinear<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    _bits_per_sample: NonZeroU8,
) {
    cfg_if! {
        if #[cfg(all(target_arch = "x86_64", not(feature = "no_simd")))] {
            if crate::util::has_avx2() {
                // SAFETY: We check for AVX2 first
                unsafe {
                    avx2::refine_horizontal_bilinear(dest, src, pitch, width, height, _bits_per_sample);
                }
                return;
            }
        }
    }

    rust::refine_horizontal_bilinear(dest, src, pitch, width, height, _bits_per_sample);
}

/// Performs vertical bilinear interpolation for sub-pixel motion estimation refinement.
///
/// This function applies bilinear interpolation vertically to create sub-pixel samples
/// between existing pixels. Bilinear interpolation uses a simple 2-tap kernel that
/// averages adjacent vertical pixels, providing fast and reasonably smooth interpolation
/// for motion estimation with sub-pixel accuracy.
///
/// The last row is copied directly from the source since there's no bottom neighbor.
///
/// # Parameters
/// - `src`: Source image buffer
/// - `dest`: Destination buffer for interpolated results
/// - `pitch`: Number of pixels per row in both buffers
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
/// - `_bits_per_sample`: Unused parameter for API consistency
pub fn refine_vertical_bilinear<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    _bits_per_sample: NonZeroU8,
) {
    cfg_if! {
        if #[cfg(all(target_arch = "x86_64", not(feature = "no_simd")))] {
            if crate::util::has_avx2() {
                // SAFETY: We check for AVX2 first
                unsafe {
                    avx2::refine_vertical_bilinear(dest, src, pitch, width, height, _bits_per_sample);
                }
                return;
            }
        }
    }

    rust::refine_vertical_bilinear(dest, src, pitch, width, height, _bits_per_sample);
}

/// Performs diagonal bilinear interpolation for sub-pixel motion estimation refinement.
///
/// This function applies bilinear interpolation diagonally to create sub-pixel samples
/// at quarter-pixel positions. It averages 2x2 blocks of pixels to create interpolated
/// values at diagonal positions, which is essential for sub-pixel motion estimation
/// that requires samples at positions like (0.5, 0.5).
///
/// Edge pixels and the last row/column use simplified interpolation due to missing neighbors.
///
/// # Parameters
/// - `src`: Source image buffer
/// - `dest`: Destination buffer for interpolated results
/// - `pitch`: Number of pixels per row in both buffers
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
/// - `_bits_per_sample`: Unused parameter for API consistency
pub fn refine_diagonal_bilinear<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    _bits_per_sample: NonZeroU8,
) {
    cfg_if! {
        if #[cfg(all(target_arch = "x86_64", not(feature = "no_simd")))] {
            if crate::util::has_avx2() {
                // SAFETY: We check for AVX2 first
                unsafe {
                    avx2::refine_diagonal_bilinear(dest, src, pitch, width, height, _bits_per_sample);
                }
                return;
            }
        }
    }

    rust::refine_diagonal_bilinear(dest, src, pitch, width, height, _bits_per_sample);
}
