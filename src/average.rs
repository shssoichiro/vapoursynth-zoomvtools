mod rust;

#[cfg(test)]
mod tests;

use std::num::NonZeroUsize;

use crate::util::Pixel;

/// Averages two images pixel by pixel, blending them together.
///
/// This function takes two source images and computes the average of corresponding
/// pixels, storing the result in the destination buffer. The averaging uses ceiling
/// division to ensure proper rounding for integer pixel values.
///
/// # Parameters
/// - `src1`: First source image buffer
/// - `src2`: Second source image buffer
/// - `dest`: Destination buffer to store the averaged result
/// - `pitch`: Number of pixels per row (including any padding)
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
pub fn average2<T: Pixel>(
    dest: &mut [T],
    src1: &[T],
    src2: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    let max_offset = (height.get() - 1) * pitch.get() + width.get();
    debug_assert!(src1.len() >= max_offset);
    debug_assert!(src2.len() >= max_offset);
    debug_assert!(dest.len() >= max_offset);

    rust::average2(dest, src1, src2, pitch, width, height);
}
