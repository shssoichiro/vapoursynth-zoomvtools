#![allow(clippy::undocumented_unsafe_blocks)]

use std::{
    arch::x86_64::*,
    num::{NonZeroU8, NonZeroUsize},
};

use crate::util::Pixel;

#[target_feature(enable = "avx2")]
pub(super) fn refine_horizontal_bilinear<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    _bits_per_sample: NonZeroU8,
) {
    match size_of::<T>() {
        1 => unsafe {
            refine_horizontal_bilinear_u8(
                src.as_ptr() as *const u8,
                dest.as_mut_ptr() as *mut u8,
                pitch,
                width,
                height,
            );
        },
        2 => unsafe {
            refine_horizontal_bilinear_u16(
                src.as_ptr() as *const u16,
                dest.as_mut_ptr() as *mut u16,
                pitch,
                width,
                height,
            );
        },
        _ => unreachable!(),
    }
}

#[target_feature(enable = "avx2")]
pub(super) fn refine_vertical_bilinear<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    _bits_per_sample: NonZeroU8,
) {
    match size_of::<T>() {
        1 => unsafe {
            refine_vertical_bilinear_u8(
                src.as_ptr() as *const u8,
                dest.as_mut_ptr() as *mut u8,
                pitch,
                width,
                height,
            );
        },
        2 => unsafe {
            refine_vertical_bilinear_u16(
                src.as_ptr() as *const u16,
                dest.as_mut_ptr() as *mut u16,
                pitch,
                width,
                height,
            );
        },
        _ => unreachable!(),
    }
}

#[target_feature(enable = "avx2")]
pub(super) fn refine_diagonal_bilinear<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    _bits_per_sample: NonZeroU8,
) {
    match size_of::<T>() {
        1 => unsafe {
            refine_diagonal_bilinear_u8(
                src.as_ptr() as *const u8,
                dest.as_mut_ptr() as *mut u8,
                pitch,
                width,
                height,
            );
        },
        2 => unsafe {
            refine_diagonal_bilinear_u16(
                src.as_ptr() as *const u16,
                dest.as_mut_ptr() as *mut u16,
                pitch,
                width,
                height,
            );
        },
        _ => unreachable!(),
    }
}

/// # Notes
/// Attempted to port the C++ version, but there were some bugs that were unmasked by the tests.
/// This is an original version.
#[target_feature(enable = "avx2")]
unsafe fn refine_horizontal_bilinear_u8(
    src: *const u8,
    dest: *mut u8,
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    let pitch = pitch.get();
    let width = width.get();
    let height = height.get();

    for j in 0..height {
        let row_offset = j * pitch;
        let mut i = 0;

        // Process 32 pixels at a time (AVX2 register size for u8)
        while i + 32 < width {
            let current = _mm256_loadu_si256((src.add(row_offset + i)) as *const __m256i);
            let next = _mm256_loadu_si256((src.add(row_offset + i + 1)) as *const __m256i);
            let result = _mm256_avg_epu8(current, next);
            _mm256_storeu_si256((dest.add(row_offset + i)) as *mut __m256i, result);
            i += 32;
        }

        // Process remaining pixels with scalar code
        while i < width - 1 {
            let a = *src.add(row_offset + i) as u16;
            let b = *src.add(row_offset + i + 1) as u16;
            *dest.add(row_offset + i) = ((a + b + 1) / 2) as u8;
            i += 1;
        }

        // Copy last column
        if width > 0 {
            *dest.add(row_offset + width - 1) = *src.add(row_offset + width - 1);
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn refine_horizontal_bilinear_u16(
    src: *const u16,
    dest: *mut u16,
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    let pitch = pitch.get();
    let width = width.get();
    let height = height.get();

    for j in 0..height {
        let row_offset = j * pitch;
        let mut i = 0;

        // Process 16 pixels at a time (AVX2 register size for u16)
        while i + 16 < width {
            let current = _mm256_loadu_si256((src.add(row_offset + i)) as *const __m256i);
            let next = _mm256_loadu_si256((src.add(row_offset + i + 1)) as *const __m256i);
            let result = _mm256_avg_epu16(current, next);
            _mm256_storeu_si256((dest.add(row_offset + i)) as *mut __m256i, result);
            i += 16;
        }

        // Process remaining pixels with scalar code
        while i < width - 1 {
            let a = *src.add(row_offset + i) as u32;
            let b = *src.add(row_offset + i + 1) as u32;
            *dest.add(row_offset + i) = ((a + b + 1) / 2) as u16;
            i += 1;
        }

        // Copy last column
        if width > 0 {
            *dest.add(row_offset + width - 1) = *src.add(row_offset + width - 1);
        }
    }
}

/// # Notes
/// This implementation is ported from C++. It is about 8% faster than our first attempt.
#[target_feature(enable = "avx2")]
unsafe fn refine_vertical_bilinear_u8(
    mut src: *const u8,
    mut dest: *mut u8,
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    let pitch = pitch.get();
    let width = width.get();
    let height = height.get();

    let simd_width_32 = width & !31; // Round down to multiple of 32

    for _y in 0..(height - 1) {
        for x in (0..simd_width_32).step_by(32) {
            let mut m0 = _mm256_loadu_si256(src.add(x).cast());
            let m1 = _mm256_loadu_si256(src.add(x + pitch).cast());

            m0 = _mm256_avg_epu8(m0, m1);
            _mm256_storeu_si256(dest.add(x).cast(), m0);
        }

        // Scalar fallback for remaining pixels
        for x in simd_width_32..width {
            *dest.add(x) = ((*src.add(x) as u16 + *src.add(x + pitch) as u16 + 1) >> 1) as u8;
        }

        src = src.add(pitch);
        dest = dest.add(pitch);
    }

    for x in 0..width {
        *dest.add(x) = *src.add(x);
    }
}

#[target_feature(enable = "avx2")]
unsafe fn refine_vertical_bilinear_u16(
    src: *const u16,
    dest: *mut u16,
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    let pitch = pitch.get();
    let width = width.get();
    let height = height.get();

    for j in 0..height - 1 {
        let row_offset = j * pitch;
        let mut i = 0;

        // Process 16 pixels at a time
        while i + 16 <= width {
            let current = _mm256_loadu_si256((src.add(row_offset + i)) as *const __m256i);
            let next = _mm256_loadu_si256((src.add(row_offset + pitch + i)) as *const __m256i);
            let result = _mm256_avg_epu16(current, next);
            _mm256_storeu_si256((dest.add(row_offset + i)) as *mut __m256i, result);
            i += 16;
        }

        // Process remaining pixels with scalar code
        while i < width {
            let a = *src.add(row_offset + i) as u32;
            let b = *src.add(row_offset + pitch + i) as u32;
            *dest.add(row_offset + i) = ((a + b + 1) / 2) as u16;
            i += 1;
        }
    }

    // Copy last row
    if height > 0 {
        let last_row_offset = (height - 1) * pitch;
        std::ptr::copy_nonoverlapping(src.add(last_row_offset), dest.add(last_row_offset), width);
    }
}

/// # Notes
/// Attempted to port the C++ version, but there were some bugs that were unmasked by the tests.
/// This is an original version.
#[target_feature(enable = "avx2")]
unsafe fn refine_diagonal_bilinear_u8(
    src: *const u8,
    dest: *mut u8,
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    let pitch = pitch.get();
    let width = width.get();
    let height = height.get();

    let mut offset = 0;

    for _j in 0..height {
        // Main loop for each row
        for i in 0..width {
            let a = *src.add(offset + i) as u16;
            let b = *src.add(offset + i + 1) as u16;
            let c = *src.add(offset + i + pitch) as u16;
            let d = *src.add(offset + i + pitch + 1) as u16;

            *dest.add(offset + i) = ((a + b + c + d + 2) / 4) as u8;
        }

        // Handle last column separately (2-tap vertical)
        if width > 0 {
            let a = *src.add(offset + width - 1) as u16;
            let b = *src.add(offset + width - 1 + pitch) as u16;
            *dest.add(offset + width - 1) = ((a + b + 1) / 2) as u8;
        }

        offset += pitch;
    }

    // Handle last row separately (2-tap horizontal)
    for i in 0..width - 1 {
        let a = *src.add(offset + i) as u16;
        let b = *src.add(offset + i + 1) as u16;
        *dest.add(offset + i) = ((a + b + 1) / 2) as u8;
    }
    // Last pixel - copy directly
    if width > 0 {
        *dest.add(offset + width - 1) = *src.add(offset + width - 1);
    }
}

#[target_feature(enable = "avx2")]
unsafe fn refine_diagonal_bilinear_u16(
    src: *const u16,
    dest: *mut u16,
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    let pitch = pitch.get();
    let width = width.get();
    let height = height.get();

    let mut offset = 0;

    for _j in 0..height {
        // Main loop for each row
        for i in 0..width {
            let a = *src.add(offset + i) as u32;
            let b = *src.add(offset + i + 1) as u32;
            let c = *src.add(offset + i + pitch) as u32;
            let d = *src.add(offset + i + pitch + 1) as u32;

            *dest.add(offset + i) = ((a + b + c + d + 2) / 4) as u16;
        }

        // Handle last column separately (2-tap vertical)
        if width > 0 {
            let a = *src.add(offset + width - 1) as u32;
            let b = *src.add(offset + width - 1 + pitch) as u32;
            *dest.add(offset + width - 1) = ((a + b + 1) / 2) as u16;
        }

        offset += pitch;
    }

    // Handle last row separately (2-tap horizontal)
    for i in 0..width - 1 {
        let a = *src.add(offset + i) as u32;
        let b = *src.add(offset + i + 1) as u32;
        *dest.add(offset + i) = ((a + b + 1) / 2) as u16;
    }
    // Last pixel - copy directly
    if width > 0 {
        *dest.add(offset + width - 1) = *src.add(offset + width - 1);
    }
}
