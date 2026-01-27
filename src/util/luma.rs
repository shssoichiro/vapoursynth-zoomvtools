mod rust;

#[cfg(test)]
mod tests;

use std::num::NonZeroUsize;

use crate::util::Pixel;

/// Calculates the sum of luminance values in a rectangular block of pixels.
///
/// This function computes the total sum of all pixel values within a specified
/// rectangular region. It's commonly used in video processing algorithms such as
/// motion estimation, where the sum of absolute differences (SAD) or similar
/// metrics require efficient block summation.
///
/// The function is highly optimized using const generics for a predefined set of
/// common block sizes, allowing the compiler to generate specialized code for each
/// supported dimension combination.
///
/// # Parameters
/// - `width`: Width of the block in pixels (must be supported size)
/// - `height`: Height of the block in pixels (must be supported size)
/// - `src`: Source pixel buffer containing the image data
/// - `src_pitch`: Number of pixels per row in the source buffer (stride), including any padding
///
/// # Returns
/// The sum of all pixel values in the specified block as a `u64`. The wide integer
/// type prevents overflow even for large blocks with high bit-depth pixels.
///
/// # Supported Block Sizes
/// This function supports the following (width, height) combinations:
/// - `(4, 4)`, `(8, 4)`, `(8, 8)`
/// - `(16, 2)`, `(16, 8)`, `(16, 16)`
/// - `(32, 16)`, `(32, 32)`
/// - `(64, 32)`, `(64, 64)`
/// - `(128, 64)`, `(128, 128)`
///
/// # Panics
/// Panics if the `(width, height)` combination is not in the supported list above.
/// The function will call `unreachable!()` for unsupported block sizes.
///
/// # Performance
/// The use of const generics allows the compiler to unroll loops and optimize
/// memory access patterns for each specific block size, providing better
/// performance than a generic implementation.
///
/// # Example
/// ```rust,ignore
/// use std::num::NonZeroUsize;
///
/// let width = NonZeroUsize::new(8).unwrap();
/// let height = NonZeroUsize::new(8).unwrap();
/// let src_pitch = NonZeroUsize::new(16).unwrap(); // 16 pixels per row
/// let pixels: Vec<u8> = vec![128; 16 * 8]; // 8 rows of 16 pixels each
///
/// let sum = luma_sum(width, height, &pixels, src_pitch);
/// // sum = 128 * 8 * 8 = 8192 for this 8x8 block
/// ```
#[must_use]
pub fn luma_sum<T: Pixel>(
    width: NonZeroUsize,
    height: NonZeroUsize,
    src: &[T],
    src_pitch: NonZeroUsize,
) -> u64 {
    rust::luma_sum(width, height, src, src_pitch)
}
