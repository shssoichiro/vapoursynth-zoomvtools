#[cfg(target_arch = "x86_64")]
mod avx2;
mod rust;

#[cfg(test)]
mod tests;

use std::num::{NonZeroU8, NonZeroUsize};

use crate::util::Pixel;

/// Performs horizontal Wiener filtering for sub-pixel motion estimation refinement.
///
/// This function applies a Wiener filter horizontally to create high-quality sub-pixel
/// samples between existing pixels. The Wiener filter uses a 6-tap kernel with optimized
/// coefficients that provide excellent interpolation quality by minimizing reconstruction
/// error while preserving image details.
///
/// Edge pixels use simple averaging due to insufficient neighbors for the full kernel.
/// The Wiener filter is particularly effective for maintaining sharpness during
/// sub-pixel interpolation in motion estimation applications.
///
/// # Parameters
/// - `src`: Source image buffer
/// - `dest`: Destination buffer for interpolated results
/// - `pitch`: Number of pixels per row in both buffers
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
/// - `bits_per_sample`: Bit depth of the pixel format for clamping
pub fn refine_horizontal_wiener<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    debug_assert!(
        bits_per_sample.get() as usize > (size_of::<T>() - 1) * 8
            && (bits_per_sample.get() as usize <= size_of::<T>() * 8)
    );

    #[cfg(target_arch = "x86_64")]
    if crate::util::has_avx2() {
        // SAFETY: We check for AVX2 first
        unsafe {
            avx2::refine_horizontal_wiener(src, dest, pitch, width, height, bits_per_sample);
        }
    } else {
        rust::refine_horizontal_wiener(src, dest, pitch, width, height, bits_per_sample);
    }

    #[cfg(not(target_arch = "x86_64"))]
    rust::refine_horizontal_wiener(src, dest, pitch, width, height, bits_per_sample);
}

/// Performs vertical Wiener filtering for sub-pixel motion estimation refinement.
///
/// This function applies a Wiener filter vertically to create high-quality sub-pixel
/// samples between existing pixels. The Wiener filter uses a 6-tap kernel with optimized
/// coefficients that provide excellent interpolation quality by minimizing reconstruction
/// error while preserving image details.
///
/// Edge rows use simple averaging due to insufficient neighbors for the full kernel,
/// and the last row is copied directly from the source. The Wiener filter is
/// particularly effective for maintaining sharpness during sub-pixel interpolation.
///
/// # Parameters
/// - `src`: Source image buffer
/// - `dest`: Destination buffer for interpolated results
/// - `pitch`: Number of pixels per row in both buffers
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
/// - `bits_per_sample`: Bit depth of the pixel format for clamping
pub fn refine_vertical_wiener<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    debug_assert!(
        bits_per_sample.get() as usize > (size_of::<T>() - 1) * 8
            && (bits_per_sample.get() as usize <= size_of::<T>() * 8)
    );

    #[cfg(target_arch = "x86_64")]
    if crate::util::has_avx2() {
        // SAFETY: We check for AVX2 first
        unsafe {
            avx2::refine_vertical_wiener(src, dest, pitch, width, height, bits_per_sample);
        }
    } else {
        rust::refine_vertical_wiener(src, dest, pitch, width, height, bits_per_sample);
    }

    #[cfg(not(target_arch = "x86_64"))]
    rust::refine_vertical_wiener(src, dest, pitch, width, height, bits_per_sample);
}
