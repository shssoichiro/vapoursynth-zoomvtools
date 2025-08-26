mod math;
mod plane;
#[cfg(test)]
mod tests;

pub use math::*;
pub use plane::*;

use std::{
    convert::TryFrom,
    num::NonZeroUsize,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Shl, Shr, Sub, SubAssign},
};

use vapoursynth::prelude::Component;

#[cfg(target_arch = "x86_64")]
cpufeatures::new!(cpuid_avx2, "avx2");

#[cfg(target_arch = "x86_64")]
pub use cpuid_avx2::get as has_avx2;

pub trait Pixel:
    Component
    + Clone
    + Copy
    + Add<Self, Output = Self>
    + AddAssign<Self>
    + Sub<Self, Output = Self>
    + SubAssign<Self>
    + Mul<Self, Output = Self>
    + MulAssign<Self>
    + Div<Self, Output = Self>
    + DivAssign<Self>
    + Shl<u8, Output = Self>
    + Shr<usize, Output = Self>
    + Into<u16>
    + Into<i32>
    + Into<u32>
    + Into<u64>
    + Into<f32>
    + From<u8>
    + TryFrom<u16>
    + TryFrom<u32>
    + TryFrom<u64>
    + FromFloatLossy
    + MaxValue
    + PartialOrd
    + Ord
    + PartialEq
    + Eq
{
    #[must_use]
    fn from_or_max(value: u32) -> Self;
}

impl<T> Pixel for T
where
    T: Component
        + Clone
        + Copy
        + Add<Self, Output = Self>
        + AddAssign<Self>
        + Sub<Self, Output = Self>
        + SubAssign<Self>
        + Mul<Self, Output = Self>
        + MulAssign<Self>
        + Div<Self, Output = Self>
        + DivAssign<Self>
        + Shl<u8, Output = Self>
        + Shr<usize, Output = Self>
        + Into<u16>
        + Into<i32>
        + Into<u32>
        + Into<u64>
        + Into<f32>
        + From<u8>
        + TryFrom<u16>
        + TryFrom<u32>
        + TryFrom<u64>
        + FromFloatLossy
        + MaxValue
        + PartialOrd
        + Ord
        + PartialEq
        + Eq,
{
    fn from_or_max(value: u32) -> Self {
        Self::try_from(value).unwrap_or_else(|_| {
            // If conversion fails (shouldn't happen with our inputs), fallback to max
            Self::max_value()
        })
    }
}

pub trait MaxValue {
    #[must_use]
    fn max_value() -> Self;
}

impl MaxValue for u8 {
    fn max_value() -> Self {
        u8::MAX
    }
}

impl MaxValue for u16 {
    fn max_value() -> Self {
        u16::MAX
    }
}

pub trait FromFloatLossy {
    #[must_use]
    fn from_float_lossy(value: f32) -> Self;
}

impl FromFloatLossy for u8 {
    fn from_float_lossy(value: f32) -> Self {
        value as u8
    }
}

impl FromFloatLossy for u16 {
    fn from_float_lossy(value: f32) -> Self {
        value as u16
    }
}

/// Performs optimized bit block transfer (bitblt) between pixel buffers.
///
/// This function efficiently copies pixel data from a source buffer to a destination
/// buffer, handling different stride sizes and memory layouts. It includes an
/// optimization for the common case where both buffers have matching strides that
/// equal the row size, allowing for a single bulk copy operation.
///
/// The function is particularly useful for copying image data between buffers with
/// different padding or when extracting rectangular regions from larger images.
/// It handles the complexity of different row strides automatically.
///
/// # Parameters
/// - `dest`: Destination buffer to copy pixels into
/// - `dest_stride`: Number of pixels per row in the destination buffer (including padding)
/// - `src`: Source buffer to copy pixels from
/// - `src_stride`: Number of pixels per row in the source buffer (including padding)
/// - `row_size`: Number of pixels to copy per row (the actual image width)
/// - `height`: Number of rows to copy (the image height)
///
/// # Performance
/// - **Fast path**: When `src_stride == dest_stride == row_size`, uses single bulk copy
/// - **Standard path**: Copies row by row when strides differ, handling padding correctly
pub fn vs_bitblt<T: Pixel>(
    dest: &mut [T],
    dest_stride: NonZeroUsize,
    src: &[T],
    src_stride: NonZeroUsize,
    row_size: NonZeroUsize,
    height: NonZeroUsize,
) {
    let height = height.get();
    let row_size = row_size.get();
    let src_stride = src_stride.get();
    let dest_stride = dest_stride.get();

    if src_stride == dest_stride && src_stride == row_size {
        // Fast path: single copy when strides match row size
        dest[..row_size * height].copy_from_slice(&src[..row_size * height]);
    } else {
        // Copy row by row when strides differ
        for i in 0..height {
            let src_start = i * src_stride;
            let dest_start = i * dest_stride;
            dest[dest_start..dest_start + row_size]
                .copy_from_slice(&src[src_start..src_start + row_size]);
        }
    }
}

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
pub fn luma_sum<T: Pixel>(
    width: NonZeroUsize,
    height: NonZeroUsize,
    src: &[T],
    src_pitch: NonZeroUsize,
) -> u64 {
    match (width.get(), height.get()) {
        (4, 4) => luma_sum_impl::<T, 4, 4>(src, src_pitch),
        (8, 4) => luma_sum_impl::<T, 8, 4>(src, src_pitch),
        (8, 8) => luma_sum_impl::<T, 8, 8>(src, src_pitch),
        (16, 2) => luma_sum_impl::<T, 16, 2>(src, src_pitch),
        (16, 8) => luma_sum_impl::<T, 16, 8>(src, src_pitch),
        (16, 16) => luma_sum_impl::<T, 16, 16>(src, src_pitch),
        (32, 16) => luma_sum_impl::<T, 32, 16>(src, src_pitch),
        (32, 32) => luma_sum_impl::<T, 32, 32>(src, src_pitch),
        (64, 32) => luma_sum_impl::<T, 64, 32>(src, src_pitch),
        (64, 64) => luma_sum_impl::<T, 64, 64>(src, src_pitch),
        (128, 64) => luma_sum_impl::<T, 128, 64>(src, src_pitch),
        (128, 128) => luma_sum_impl::<T, 128, 128>(src, src_pitch),
        _ => unreachable!("unsupported block size"),
    }
}

fn luma_sum_impl<T: Pixel, const WIDTH: usize, const HEIGHT: usize>(
    src: &[T],
    src_pitch: NonZeroUsize,
) -> u64 {
    let mut luma_sum = 0u64;
    for j in 0..HEIGHT {
        let src_row = &src[j * src_pitch.get()..][..WIDTH];
        for &pix in src_row {
            let pixel_value: u64 = pix.into();
            luma_sum += pixel_value;
        }
    }
    luma_sum
}
