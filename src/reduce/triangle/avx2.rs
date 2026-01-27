#![allow(clippy::undocumented_unsafe_blocks)]

use std::{arch::x86_64::*, num::NonZeroUsize};

use crate::util::Pixel;

#[target_feature(enable = "avx2")]
pub(super) fn reduce_triangle<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    // Check the array bounds once at the start of the loop.
    assert!(src.len() >= src_pitch.get() * dest_height.get() * 2);
    assert!(dest.len() >= dest_pitch.get() * dest_height.get());

    match size_of::<T>() {
        1 => unsafe {
            reduce_triangle_vertical_u8(
                dest.as_mut_ptr() as *mut u8,
                src.as_ptr() as *const u8,
                dest_pitch,
                src_pitch,
                // SAFETY: non-zero constant
                dest_width.saturating_mul(NonZeroUsize::new_unchecked(2)),
                dest_height,
            );
            reduce_triangle_horizontal_inplace_u8(
                dest.as_mut_ptr() as *mut u8,
                dest_pitch,
                dest_width,
                dest_height,
            );
        },
        2 => unsafe {
            reduce_triangle_vertical_u16(
                dest.as_mut_ptr() as *mut u16,
                src.as_ptr() as *const u16,
                dest_pitch,
                src_pitch,
                // SAFETY: non-zero constant
                dest_width.saturating_mul(NonZeroUsize::new_unchecked(2)),
                dest_height,
            );
            reduce_triangle_horizontal_inplace_u16(
                dest.as_mut_ptr() as *mut u16,
                dest_pitch,
                dest_width,
                dest_height,
            );
        },
        _ => unreachable!(),
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_triangle_vertical_u8(
    dest: *mut u8,
    src: *const u8,
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let width_usize = dest_width.get();
    let height_usize = dest_height.get();
    let src_pitch_usize = src_pitch.get();
    let dest_pitch_usize = dest_pitch.get();

    // Process first output row: average of first two input rows
    let mut x = 0;
    while x + 32 <= width_usize {
        let a = _mm256_loadu_si256(src.add(x) as *const __m256i);
        let b = _mm256_loadu_si256(src.add(x + src_pitch_usize) as *const __m256i);

        // Convert to u16 for intermediate calculations
        let a_lo = _mm256_unpacklo_epi8(a, _mm256_setzero_si256());
        let a_hi = _mm256_unpackhi_epi8(a, _mm256_setzero_si256());
        let b_lo = _mm256_unpacklo_epi8(b, _mm256_setzero_si256());
        let b_hi = _mm256_unpackhi_epi8(b, _mm256_setzero_si256());

        // Calculate (a + b + 1) / 2
        let ones = _mm256_set1_epi16(1);
        let sum_lo = _mm256_add_epi16(_mm256_add_epi16(a_lo, b_lo), ones);
        let sum_hi = _mm256_add_epi16(_mm256_add_epi16(a_hi, b_hi), ones);
        let result_lo = _mm256_srli_epi16(sum_lo, 1);
        let result_hi = _mm256_srli_epi16(sum_hi, 1);

        // Pack back to u8
        let result = _mm256_packus_epi16(result_lo, result_hi);
        _mm256_storeu_si256(dest.add(x) as *mut __m256i, result);

        x += 32;
    }

    // Handle remaining pixels
    while x < width_usize {
        let a = *src.add(x) as u16;
        let b = *src.add(x + src_pitch_usize) as u16;
        *dest.add(x) = ((a + b + 1) / 2) as u8;
        x += 1;
    }

    // Process remaining output rows: 1/4, 1/2, 1/4 filter
    for y in 1..height_usize {
        let dest_offset = y * dest_pitch_usize;
        let src_offset = y * 2 * src_pitch_usize;

        let mut x = 0;
        while x + 32 <= width_usize {
            let a = _mm256_loadu_si256(src.add(src_offset + x - src_pitch_usize) as *const __m256i);
            let b = _mm256_loadu_si256(src.add(src_offset + x) as *const __m256i);
            let c = _mm256_loadu_si256(src.add(src_offset + x + src_pitch_usize) as *const __m256i);

            // Convert to u16 for intermediate calculations
            let a_lo = _mm256_unpacklo_epi8(a, _mm256_setzero_si256());
            let a_hi = _mm256_unpackhi_epi8(a, _mm256_setzero_si256());
            let b_lo = _mm256_unpacklo_epi8(b, _mm256_setzero_si256());
            let b_hi = _mm256_unpackhi_epi8(b, _mm256_setzero_si256());
            let c_lo = _mm256_unpacklo_epi8(c, _mm256_setzero_si256());
            let c_hi = _mm256_unpackhi_epi8(c, _mm256_setzero_si256());

            // Calculate (a + b * 2 + c + 2) / 4
            let twos = _mm256_set1_epi16(2);
            let b2_lo = _mm256_slli_epi16(b_lo, 1);
            let b2_hi = _mm256_slli_epi16(b_hi, 1);
            let sum_lo =
                _mm256_add_epi16(_mm256_add_epi16(_mm256_add_epi16(a_lo, b2_lo), c_lo), twos);
            let sum_hi =
                _mm256_add_epi16(_mm256_add_epi16(_mm256_add_epi16(a_hi, b2_hi), c_hi), twos);
            let result_lo = _mm256_srli_epi16(sum_lo, 2);
            let result_hi = _mm256_srli_epi16(sum_hi, 2);

            // Pack back to u8
            let result = _mm256_packus_epi16(result_lo, result_hi);
            _mm256_storeu_si256(dest.add(dest_offset + x) as *mut __m256i, result);

            x += 32;
        }

        // Handle remaining pixels
        while x < width_usize {
            let a = *src.add(src_offset + x - src_pitch_usize) as u16;
            let b = *src.add(src_offset + x) as u16;
            let c = *src.add(src_offset + x + src_pitch_usize) as u16;
            *dest.add(dest_offset + x) = ((a + b * 2 + c + 2) / 4) as u8;
            x += 1;
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_triangle_horizontal_inplace_u8(
    dest: *mut u8,
    dest_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let width_usize = dest_width.get();
    let height_usize = dest_height.get();
    let dest_pitch_usize = dest_pitch.get();

    for y in 0..height_usize {
        let row_offset = y * dest_pitch_usize;

        // First pixel: simple average
        let b = *dest.add(row_offset) as u16;
        let c = *dest.add(row_offset + 1) as u16;
        let src0 = ((b + c + 1) / 2) as u8;

        // Process remaining pixels with triangle filter
        let mut x = 1;
        while x + 16 <= width_usize {
            // Load 16 sets of 3 consecutive pixels (a, b, c)
            let data_offset = row_offset + x * 2 - 1;

            // We need to handle this carefully as we're reading overlapping data
            // Load in chunks and process individually to avoid complex shuffles
            for i in 0..16.min(width_usize - x) {
                let pixel_offset = data_offset + i * 2;
                let a = *dest.add(pixel_offset) as u16;
                let b = *dest.add(pixel_offset + 1) as u16;
                let c = *dest.add(pixel_offset + 2) as u16;
                *dest.add(row_offset + x + i) = ((a + b * 2 + c + 2) / 4) as u8;
            }
            x += 16;
        }

        // Handle remaining pixels
        while x < width_usize {
            let pixel_offset = row_offset + x * 2 - 1;
            let a = *dest.add(pixel_offset) as u16;
            let b = *dest.add(pixel_offset + 1) as u16;
            let c = *dest.add(pixel_offset + 2) as u16;
            *dest.add(row_offset + x) = ((a + b * 2 + c + 2) / 4) as u8;
            x += 1;
        }

        // Store the first pixel
        *dest.add(row_offset) = src0;
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_triangle_vertical_u16(
    dest: *mut u16,
    src: *const u16,
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let width_usize = dest_width.get();
    let height_usize = dest_height.get();
    let src_pitch_usize = src_pitch.get();
    let dest_pitch_usize = dest_pitch.get();

    // Process first output row: average of first two input rows
    let mut x = 0;
    while x + 16 <= width_usize {
        let a = _mm256_loadu_si256(src.add(x) as *const __m256i);
        let b = _mm256_loadu_si256(src.add(x + src_pitch_usize) as *const __m256i);

        // Convert to u32 for intermediate calculations
        let a_lo = _mm256_unpacklo_epi16(a, _mm256_setzero_si256());
        let a_hi = _mm256_unpackhi_epi16(a, _mm256_setzero_si256());
        let b_lo = _mm256_unpacklo_epi16(b, _mm256_setzero_si256());
        let b_hi = _mm256_unpackhi_epi16(b, _mm256_setzero_si256());

        // Calculate (a + b + 1) / 2
        let ones = _mm256_set1_epi32(1);
        let sum_lo = _mm256_add_epi32(_mm256_add_epi32(a_lo, b_lo), ones);
        let sum_hi = _mm256_add_epi32(_mm256_add_epi32(a_hi, b_hi), ones);
        let result_lo = _mm256_srli_epi32(sum_lo, 1);
        let result_hi = _mm256_srli_epi32(sum_hi, 1);

        // Pack back to u16
        let result = _mm256_packus_epi32(result_lo, result_hi);
        _mm256_storeu_si256(dest.add(x) as *mut __m256i, result);

        x += 16;
    }

    // Handle remaining pixels
    while x < width_usize {
        let a = *src.add(x) as u32;
        let b = *src.add(x + src_pitch_usize) as u32;
        *dest.add(x) = ((a + b + 1) / 2) as u16;
        x += 1;
    }

    // Process remaining output rows: 1/4, 1/2, 1/4 filter
    for y in 1..height_usize {
        let dest_offset = y * dest_pitch_usize;
        let src_offset = y * 2 * src_pitch_usize;

        let mut x = 0;
        while x + 16 <= width_usize {
            let a = _mm256_loadu_si256(src.add(src_offset + x - src_pitch_usize) as *const __m256i);
            let b = _mm256_loadu_si256(src.add(src_offset + x) as *const __m256i);
            let c = _mm256_loadu_si256(src.add(src_offset + x + src_pitch_usize) as *const __m256i);

            // Convert to u32 for intermediate calculations
            let a_lo = _mm256_unpacklo_epi16(a, _mm256_setzero_si256());
            let a_hi = _mm256_unpackhi_epi16(a, _mm256_setzero_si256());
            let b_lo = _mm256_unpacklo_epi16(b, _mm256_setzero_si256());
            let b_hi = _mm256_unpackhi_epi16(b, _mm256_setzero_si256());
            let c_lo = _mm256_unpacklo_epi16(c, _mm256_setzero_si256());
            let c_hi = _mm256_unpackhi_epi16(c, _mm256_setzero_si256());

            // Calculate (a + b * 2 + c + 2) / 4
            let twos = _mm256_set1_epi32(2);
            let b2_lo = _mm256_slli_epi32(b_lo, 1);
            let b2_hi = _mm256_slli_epi32(b_hi, 1);
            let sum_lo =
                _mm256_add_epi32(_mm256_add_epi32(_mm256_add_epi32(a_lo, b2_lo), c_lo), twos);
            let sum_hi =
                _mm256_add_epi32(_mm256_add_epi32(_mm256_add_epi32(a_hi, b2_hi), c_hi), twos);
            let result_lo = _mm256_srli_epi32(sum_lo, 2);
            let result_hi = _mm256_srli_epi32(sum_hi, 2);

            // Pack back to u16
            let result = _mm256_packus_epi32(result_lo, result_hi);
            _mm256_storeu_si256(dest.add(dest_offset + x) as *mut __m256i, result);

            x += 16;
        }

        // Handle remaining pixels
        while x < width_usize {
            let a = *src.add(src_offset + x - src_pitch_usize) as u32;
            let b = *src.add(src_offset + x) as u32;
            let c = *src.add(src_offset + x + src_pitch_usize) as u32;
            *dest.add(dest_offset + x) = ((a + b * 2 + c + 2) / 4) as u16;
            x += 1;
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_triangle_horizontal_inplace_u16(
    dest: *mut u16,
    dest_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let width_usize = dest_width.get();
    let height_usize = dest_height.get();
    let dest_pitch_usize = dest_pitch.get();

    for y in 0..height_usize {
        let row_offset = y * dest_pitch_usize;

        // First pixel: simple average
        let b = *dest.add(row_offset) as u32;
        let c = *dest.add(row_offset + 1) as u32;
        let src0 = ((b + c + 1) / 2) as u16;

        // Process remaining pixels with triangle filter
        let mut x = 1;
        while x + 8 <= width_usize {
            // Process 8 pixels at a time
            for i in 0..8.min(width_usize - x) {
                let pixel_offset = row_offset + (x + i) * 2 - 1;
                let a = *dest.add(pixel_offset) as u32;
                let b = *dest.add(pixel_offset + 1) as u32;
                let c = *dest.add(pixel_offset + 2) as u32;
                *dest.add(row_offset + x + i) = ((a + b * 2 + c + 2) / 4) as u16;
            }
            x += 8;
        }

        // Handle remaining pixels
        while x < width_usize {
            let pixel_offset = row_offset + x * 2 - 1;
            let a = *dest.add(pixel_offset) as u32;
            let b = *dest.add(pixel_offset + 1) as u32;
            let c = *dest.add(pixel_offset + 2) as u32;
            *dest.add(row_offset + x) = ((a + b * 2 + c + 2) / 4) as u16;
            x += 1;
        }

        // Store the first pixel
        *dest.add(row_offset) = src0;
    }
}
