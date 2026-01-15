#[cfg(target_arch = "x86_64")]
mod avx2;
mod rust;

#[cfg(test)]
mod tests;

use std::num::{NonZeroU8, NonZeroUsize};

use cfg_if::cfg_if;

use crate::util::Pixel;

/// Performs horizontal bicubic interpolation for sub-pixel motion estimation refinement.
///
/// This function applies bicubic interpolation horizontally to create sub-pixel samples
/// between existing pixels. Bicubic interpolation uses a 4-tap kernel that considers
/// 4 horizontal neighbors, providing smooth and high-quality interpolation suitable
/// for motion estimation with sub-pixel accuracy.
///
/// Edge pixels use simple averaging due to insufficient neighbors for the full kernel.
///
/// # Parameters
/// - `src`: Source image buffer
/// - `dest`: Destination buffer for interpolated results
/// - `pitch`: Number of pixels per row in both buffers
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
/// - `bits_per_sample`: Bit depth of the pixel format for clamping
pub fn refine_horizontal_bicubic<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    debug_assert!(
        bits_per_sample.get() as usize > (size_of::<T>() - 1) * 8
            && (bits_per_sample.get() as usize <= size_of::<T>() * 8)
    );

    cfg_if! {
        if #[cfg(all(target_arch = "x86_64", not(feature = "no_simd")))] {
            if crate::util::has_avx2() {
                // SAFETY: We check for AVX2 first
                unsafe {
                    avx2::refine_horizontal_bicubic(dest, src, pitch, width, height, bits_per_sample);
                }
                return;
            }
        }
    }

    rust::refine_horizontal_bicubic(dest, src, pitch, width, height, bits_per_sample);
}

/// Performs vertical bicubic interpolation for sub-pixel motion estimation refinement.
///
/// This function applies bicubic interpolation vertically to create sub-pixel samples
/// between existing pixels. Bicubic interpolation uses a 4-tap kernel that considers
/// 4 vertical neighbors, providing smooth and high-quality interpolation suitable
/// for motion estimation with sub-pixel accuracy.
///
/// Edge rows use simple averaging due to insufficient neighbors for the full kernel,
/// and the last row is copied directly from the source.
///
/// # Parameters
/// - `src`: Source image buffer
/// - `dest`: Destination buffer for interpolated results
/// - `pitch`: Number of pixels per row in both buffers
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
/// - `bits_per_sample`: Bit depth of the pixel format for clamping
pub fn refine_vertical_bicubic<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    debug_assert!(
        bits_per_sample.get() as usize > (size_of::<T>() - 1) * 8
            && (bits_per_sample.get() as usize <= size_of::<T>() * 8)
    );

    cfg_if! {
        if #[cfg(all(target_arch = "x86_64", not(feature = "no_simd")))] {
            if crate::util::has_avx2() {
                // SAFETY: We check for AVX2 first
                unsafe {
                    avx2::refine_vertical_bicubic(dest, src, pitch, width, height, bits_per_sample);
                }
                return;
            }
        }
    }

    rust::refine_vertical_bicubic(dest, src, pitch, width, height, bits_per_sample);
}
