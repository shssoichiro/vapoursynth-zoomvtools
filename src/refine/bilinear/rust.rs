use std::num::{NonZeroU8, NonZeroUsize};

use crate::util::Pixel;

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
    let mut offset = 0;
    for _j in 0..height.get() {
        let src_row = &src[offset..][..width.get()];
        let dest_row = &mut dest[offset..][..width.get()];

        for i in 0..width.get() - 1 {
            let a: u32 = src_row[i].into();
            let b: u32 = src_row[i + 1].into();
            dest_row[i] = T::from_or_max((a + b + 1) / 2);
        }
        // last column
        dest_row[width.get() - 1] = src_row[width.get() - 1];

        offset += pitch.get();
    }
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
    let mut offset = 0;
    for _j in 0..height.get() - 1 {
        for i in 0..width.get() {
            let a: u32 = src[offset + i].into();
            let b: u32 = src[offset + i + pitch.get()].into();
            dest[offset + i] = T::from_or_max((a + b + 1) / 2);
        }
        offset += pitch.get();
    }

    // last row
    dest[offset..offset + width.get()].copy_from_slice(&src[offset..offset + width.get()]);
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
    let mut offset = 0;

    for _j in 0..height.get() {
        for i in 0..width.get() {
            let a: u32 = src[offset + i].into();
            let b: u32 = src[offset + i + 1].into();
            let c: u32 = src[offset + i + pitch.get()].into();
            let d: u32 = src[offset + i + pitch.get() + 1].into();

            dest[offset + i] = T::from_or_max((a + b + c + d + 2) / 4);
        }
        // last column
        let a: u32 = src[offset + width.get() - 1].into();
        let b: u32 = src[offset + width.get() - 1 + pitch.get()].into();
        dest[offset + width.get() - 1] = T::from_or_max((a + b + 1) / 2);

        offset += pitch.get();
    }

    // last row
    for i in 0..width.get() - 1 {
        let a: u32 = src[offset + i].into();
        let b: u32 = src[offset + i + 1].into();
        dest[offset + i] = T::from_or_max((a + b + 1) / 2);
    }
    // last pixel
    dest[offset + width.get() - 1] = src[offset + width.get() - 1];
}
