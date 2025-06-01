#![allow(clippy::undocumented_unsafe_blocks)]

use std::num::{NonZeroU8, NonZeroUsize};

use crate::util::Pixel;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

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
#[target_feature(enable = "avx2")]
pub fn refine_horizontal_bilinear<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    _bits_per_sample: NonZeroU8,
) {
    match size_of::<T>() {
        1 => unsafe {
            refine_horizontal_bilinear_u8(src, dest, pitch, width, height);
        },
        2 => unsafe {
            refine_horizontal_bilinear_u16(src, dest, pitch, width, height);
        },
        _ => unreachable!(),
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
#[target_feature(enable = "avx2")]
pub fn refine_vertical_bilinear<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    _bits_per_sample: NonZeroU8,
) {
    match size_of::<T>() {
        1 => unsafe {
            refine_vertical_bilinear_u8(src, dest, pitch, width, height);
        },
        2 => unsafe {
            refine_vertical_bilinear_u16(src, dest, pitch, width, height);
        },
        _ => unreachable!(),
    }
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
#[target_feature(enable = "avx2")]
pub fn refine_diagonal_bilinear<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    _bits_per_sample: NonZeroU8,
) {
    match size_of::<T>() {
        1 => unsafe {
            refine_diagonal_bilinear_u8(src, dest, pitch, width, height);
        },
        2 => unsafe {
            refine_diagonal_bilinear_u16(src, dest, pitch, width, height);
        },
        _ => unreachable!(),
    }
}

#[target_feature(enable = "avx2")]
unsafe fn refine_horizontal_bilinear_u8<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    let src_ptr = src.as_ptr() as *const u8;
    let dest_ptr = dest.as_mut_ptr() as *mut u8;
    let pitch = pitch.get();
    let width = width.get();
    let height = height.get();

    for j in 0..height {
        let row_offset = j * pitch;
        let mut i = 0;

        // Process 32 pixels at a time (AVX2 register size for u8)
        while i + 32 < width {
            let current = _mm256_loadu_si256((src_ptr.add(row_offset + i)) as *const __m256i);
            let next = _mm256_loadu_si256((src_ptr.add(row_offset + i + 1)) as *const __m256i);
            let result = _mm256_avg_epu8(current, next);
            _mm256_storeu_si256((dest_ptr.add(row_offset + i)) as *mut __m256i, result);
            i += 32;
        }

        // Process remaining pixels with scalar code
        while i < width - 1 {
            let a = *src_ptr.add(row_offset + i) as u32;
            let b = *src_ptr.add(row_offset + i + 1) as u32;
            *dest_ptr.add(row_offset + i) = ((a + b + 1) / 2) as u8;
            i += 1;
        }

        // Copy last column
        if width > 0 {
            *dest_ptr.add(row_offset + width - 1) = *src_ptr.add(row_offset + width - 1);
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn refine_horizontal_bilinear_u16<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    let src_ptr = src.as_ptr() as *const u16;
    let dest_ptr = dest.as_mut_ptr() as *mut u16;
    let pitch = pitch.get();
    let width = width.get();
    let height = height.get();

    for j in 0..height {
        let row_offset = j * pitch;
        let mut i = 0;

        // Process 16 pixels at a time (AVX2 register size for u16)
        while i + 16 < width {
            let current = _mm256_loadu_si256((src_ptr.add(row_offset + i)) as *const __m256i);
            let next = _mm256_loadu_si256((src_ptr.add(row_offset + i + 1)) as *const __m256i);
            let result = _mm256_avg_epu16(current, next);
            _mm256_storeu_si256((dest_ptr.add(row_offset + i)) as *mut __m256i, result);
            i += 16;
        }

        // Process remaining pixels with scalar code
        while i < width - 1 {
            let a = *src_ptr.add(row_offset + i) as u32;
            let b = *src_ptr.add(row_offset + i + 1) as u32;
            *dest_ptr.add(row_offset + i) = ((a + b + 1) / 2) as u16;
            i += 1;
        }

        // Copy last column
        if width > 0 {
            *dest_ptr.add(row_offset + width - 1) = *src_ptr.add(row_offset + width - 1);
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn refine_vertical_bilinear_u8<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    let src_ptr = src.as_ptr() as *const u8;
    let dest_ptr = dest.as_mut_ptr() as *mut u8;
    let pitch = pitch.get();
    let width = width.get();
    let height = height.get();

    for j in 0..height - 1 {
        let row_offset = j * pitch;
        let mut i = 0;

        // Process 32 pixels at a time
        while i + 32 <= width {
            let current = _mm256_loadu_si256((src_ptr.add(row_offset + i)) as *const __m256i);
            let next = _mm256_loadu_si256((src_ptr.add(row_offset + pitch + i)) as *const __m256i);
            let result = _mm256_avg_epu8(current, next);
            _mm256_storeu_si256((dest_ptr.add(row_offset + i)) as *mut __m256i, result);
            i += 32;
        }

        // Process remaining pixels with scalar code
        while i < width {
            let a = *src_ptr.add(row_offset + i) as u32;
            let b = *src_ptr.add(row_offset + pitch + i) as u32;
            *dest_ptr.add(row_offset + i) = ((a + b + 1) / 2) as u8;
            i += 1;
        }
    }

    // Copy last row
    if height > 0 {
        let last_row_offset = (height - 1) * pitch;
        std::ptr::copy_nonoverlapping(
            src_ptr.add(last_row_offset),
            dest_ptr.add(last_row_offset),
            width,
        );
    }
}

#[target_feature(enable = "avx2")]
unsafe fn refine_vertical_bilinear_u16<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    let src_ptr = src.as_ptr() as *const u16;
    let dest_ptr = dest.as_mut_ptr() as *mut u16;
    let pitch = pitch.get();
    let width = width.get();
    let height = height.get();

    for j in 0..height - 1 {
        let row_offset = j * pitch;
        let mut i = 0;

        // Process 16 pixels at a time
        while i + 16 <= width {
            let current = _mm256_loadu_si256((src_ptr.add(row_offset + i)) as *const __m256i);
            let next = _mm256_loadu_si256((src_ptr.add(row_offset + pitch + i)) as *const __m256i);
            let result = _mm256_avg_epu16(current, next);
            _mm256_storeu_si256((dest_ptr.add(row_offset + i)) as *mut __m256i, result);
            i += 16;
        }

        // Process remaining pixels with scalar code
        while i < width {
            let a = *src_ptr.add(row_offset + i) as u32;
            let b = *src_ptr.add(row_offset + pitch + i) as u32;
            *dest_ptr.add(row_offset + i) = ((a + b + 1) / 2) as u16;
            i += 1;
        }
    }

    // Copy last row
    if height > 0 {
        let last_row_offset = (height - 1) * pitch;
        std::ptr::copy_nonoverlapping(
            src_ptr.add(last_row_offset),
            dest_ptr.add(last_row_offset),
            width,
        );
    }
}

#[target_feature(enable = "avx2")]
unsafe fn refine_diagonal_bilinear_u8<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    let src_ptr = src.as_ptr() as *const u8;
    let dest_ptr = dest.as_mut_ptr() as *mut u8;
    let pitch = pitch.get();
    let width = width.get();
    let height = height.get();

    let mut offset = 0;

    for _j in 0..height {
        // Main loop for each row
        for i in 0..width {
            let a = *src_ptr.add(offset + i) as u32;
            let b = *src_ptr.add(offset + i + 1) as u32;
            let c = *src_ptr.add(offset + i + pitch) as u32;
            let d = *src_ptr.add(offset + i + pitch + 1) as u32;

            *dest_ptr.add(offset + i) = ((a + b + c + d + 2) / 4) as u8;
        }

        // Handle last column separately (2-tap vertical)
        if width > 0 {
            let a = *src_ptr.add(offset + width - 1) as u32;
            let b = *src_ptr.add(offset + width - 1 + pitch) as u32;
            *dest_ptr.add(offset + width - 1) = ((a + b + 1) / 2) as u8;
        }

        offset += pitch;
    }

    // Handle last row separately (2-tap horizontal)
    for i in 0..width - 1 {
        let a = *src_ptr.add(offset + i) as u32;
        let b = *src_ptr.add(offset + i + 1) as u32;
        *dest_ptr.add(offset + i) = ((a + b + 1) / 2) as u8;
    }
    // Last pixel - copy directly
    if width > 0 {
        *dest_ptr.add(offset + width - 1) = *src_ptr.add(offset + width - 1);
    }
}

#[target_feature(enable = "avx2")]
unsafe fn refine_diagonal_bilinear_u16<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    let src_ptr = src.as_ptr() as *const u16;
    let dest_ptr = dest.as_mut_ptr() as *mut u16;
    let pitch = pitch.get();
    let width = width.get();
    let height = height.get();

    let mut offset = 0;

    for _j in 0..height {
        // Main loop for each row
        for i in 0..width {
            let a = *src_ptr.add(offset + i) as u32;
            let b = *src_ptr.add(offset + i + 1) as u32;
            let c = *src_ptr.add(offset + i + pitch) as u32;
            let d = *src_ptr.add(offset + i + pitch + 1) as u32;

            *dest_ptr.add(offset + i) = ((a + b + c + d + 2) / 4) as u16;
        }

        // Handle last column separately (2-tap vertical)
        if width > 0 {
            let a = *src_ptr.add(offset + width - 1) as u32;
            let b = *src_ptr.add(offset + width - 1 + pitch) as u32;
            *dest_ptr.add(offset + width - 1) = ((a + b + 1) / 2) as u16;
        }

        offset += pitch;
    }

    // Handle last row separately (2-tap horizontal)
    for i in 0..width - 1 {
        let a = *src_ptr.add(offset + i) as u32;
        let b = *src_ptr.add(offset + i + 1) as u32;
        *dest_ptr.add(offset + i) = ((a + b + 1) / 2) as u16;
    }
    // Last pixel - copy directly
    if width > 0 {
        *dest_ptr.add(offset + width - 1) = *src_ptr.add(offset + width - 1);
    }
}
