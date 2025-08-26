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
    src1: &[T],
    src2: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    let mut offset = 0;
    for _j in 0..height.get() {
        for i in 0..width.get() {
            let a: u32 = src1[offset + i].into();
            let b: u32 = src2[offset + i].into();
            dest[offset + i] = T::from_u32_or_max_value((a + b + 1) / 2);
        }
        offset += pitch.get();
    }
}
