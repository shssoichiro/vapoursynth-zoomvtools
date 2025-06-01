#![allow(clippy::undocumented_unsafe_blocks)]

use std::arch::x86_64::*;
use std::num::NonZeroUsize;

use crate::util::Pixel;

/// Downscales an image by 2x using quadratic interpolation.
///
/// This function reduces both the width and height of the source image by half
/// using a two-pass quadratic filtering approach. First, vertical filtering is
/// applied to reduce the height, then horizontal filtering is applied in-place
/// to reduce the width. Quadratic interpolation provides a balance between
/// computational efficiency and image quality, using a 6-tap kernel with
/// quadratic weighting functions.
///
/// The quadratic filter uses different weights than cubic interpolation,
/// optimized for smooth gradients while maintaining sharpness in details.
///
/// # Parameters
/// - `dest`: Destination buffer to store the downscaled image
/// - `src`: Source image buffer to downscale
/// - `dest_pitch`: Number of pixels per row in the destination buffer
/// - `src_pitch`: Number of pixels per row in the source buffer
/// - `dest_width`: Width of the destination image (half of source width)
/// - `dest_height`: Height of the destination image (half of source height)
#[target_feature(enable = "avx2")]
pub fn reduce_quadratic<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    match size_of::<T>() {
        1 => unsafe {
            reduce_quadratic_vertical_u8(
                dest.as_mut_ptr() as *mut u8,
                src.as_ptr() as *const u8,
                dest_pitch,
                src_pitch,
                // SAFETY: non-zero constant
                dest_width.saturating_mul(NonZeroUsize::new_unchecked(2)),
                dest_height,
            );
            reduce_quadratic_horizontal_inplace_u8(
                dest.as_mut_ptr() as *mut u8,
                dest_pitch,
                dest_width,
                dest_height,
            );
        },
        2 => unsafe {
            reduce_quadratic_vertical_u16(
                dest.as_mut_ptr() as *mut u16,
                src.as_ptr() as *const u16,
                dest_pitch,
                src_pitch,
                // SAFETY: non-zero constant
                dest_width.saturating_mul(NonZeroUsize::new_unchecked(2)),
                dest_height,
            );
            reduce_quadratic_horizontal_inplace_u16(
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
unsafe fn reduce_quadratic_vertical_u8(
    dest: *mut u8,
    src: *const u8,
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let dest_pitch_val = dest_pitch.get();
    let src_pitch_val = src_pitch.get();
    let dest_width_val = dest_width.get();
    let dest_height_val = dest_height.get();

    // Special case for first line - simple averaging
    {
        let dest_row = dest;
        let src_row0 = src;
        let src_row1 = src.add(src_pitch_val);

        let mut x = 0;
        // Process 32 pixels at a time
        while x + 32 <= dest_width_val {
            let a = _mm256_loadu_si256(src_row0.add(x) as *const __m256i);
            let b = _mm256_loadu_si256(src_row1.add(x) as *const __m256i);

            // Convert to u16 for calculation
            let a_lo = _mm256_unpacklo_epi8(a, _mm256_setzero_si256());
            let a_hi = _mm256_unpackhi_epi8(a, _mm256_setzero_si256());
            let b_lo = _mm256_unpacklo_epi8(b, _mm256_setzero_si256());
            let b_hi = _mm256_unpackhi_epi8(b, _mm256_setzero_si256());

            // Add with rounding: (a + b + 1) / 2
            let sum_lo = _mm256_add_epi16(a_lo, b_lo);
            let sum_hi = _mm256_add_epi16(a_hi, b_hi);
            let one = _mm256_set1_epi16(1);
            let sum_lo_rounded = _mm256_add_epi16(sum_lo, one);
            let sum_hi_rounded = _mm256_add_epi16(sum_hi, one);
            let result_lo = _mm256_srli_epi16(sum_lo_rounded, 1);
            let result_hi = _mm256_srli_epi16(sum_hi_rounded, 1);

            // Pack back to u8
            let result = _mm256_packus_epi16(result_lo, result_hi);
            _mm256_storeu_si256(dest_row.add(x) as *mut __m256i, result);

            x += 32;
        }

        // Handle remaining pixels
        while x < dest_width_val {
            let a = *src_row0.add(x) as u32;
            let b = *src_row1.add(x) as u32;
            *dest_row.add(x) = ((a + b + 1) / 2) as u8;
            x += 1;
        }
    }

    // Middle lines - full quadratic filter
    for y in 1..(dest_height_val - 1) {
        let dest_row = dest.add(y * dest_pitch_val);
        let src_row_offset = y * 2 * src_pitch_val;
        let src_m2 = src.add(src_row_offset - src_pitch_val * 2);
        let src_m1 = src.add(src_row_offset - src_pitch_val);
        let src_p0 = src.add(src_row_offset);
        let src_p1 = src.add(src_row_offset + src_pitch_val);
        let src_p2 = src.add(src_row_offset + src_pitch_val * 2);
        let src_p3 = src.add(src_row_offset + src_pitch_val * 3);

        let mut x = 0;
        // Process 16 pixels at a time (since we need u16 for calculations)
        while x + 16 <= dest_width_val {
            // Load 16 u8 values from each row
            let m0 = _mm_loadu_si128(src_m2.add(x) as *const __m128i);
            let m1 = _mm_loadu_si128(src_m1.add(x) as *const __m128i);
            let m2 = _mm_loadu_si128(src_p0.add(x) as *const __m128i);
            let m3 = _mm_loadu_si128(src_p1.add(x) as *const __m128i);
            let m4 = _mm_loadu_si128(src_p2.add(x) as *const __m128i);
            let m5 = _mm_loadu_si128(src_p3.add(x) as *const __m128i);

            // Convert to u16
            let m0_16 = _mm256_cvtepu8_epi16(m0);
            let m1_16 = _mm256_cvtepu8_epi16(m1);
            let m2_16 = _mm256_cvtepu8_epi16(m2);
            let m3_16 = _mm256_cvtepu8_epi16(m3);
            let m4_16 = _mm256_cvtepu8_epi16(m4);
            let m5_16 = _mm256_cvtepu8_epi16(m5);

            // Apply quadratic filter: (m0 + m5 + 9*(m1 + m4) + 22*(m2 + m3) + 32) >> 6
            let sum_m2_m3 = _mm256_add_epi16(m2_16, m3_16);
            let mul_22 = _mm256_mullo_epi16(sum_m2_m3, _mm256_set1_epi16(22));

            let sum_m1_m4 = _mm256_add_epi16(m1_16, m4_16);
            let mul_9 = _mm256_mullo_epi16(sum_m1_m4, _mm256_set1_epi16(9));

            let sum_m0_m5 = _mm256_add_epi16(m0_16, m5_16);
            let bias = _mm256_set1_epi16(32);

            let result = _mm256_add_epi16(
                sum_m0_m5,
                _mm256_add_epi16(mul_9, _mm256_add_epi16(mul_22, bias)),
            );
            let final_result = _mm256_srli_epi16(result, 6);

            // Pack back to u8 with saturation
            let packed_result = _mm256_packus_epi16(final_result, _mm256_setzero_si256());
            _mm_storeu_si128(
                dest_row.add(x) as *mut __m128i,
                _mm256_extracti128_si256(packed_result, 0),
            );

            x += 16;
        }

        // Handle remaining pixels
        while x < dest_width_val {
            let mut m0 = *src_m2.add(x) as u32;
            let mut m1 = *src_m1.add(x) as u32;
            let mut m2 = *src_p0.add(x) as u32;
            let m3 = *src_p1.add(x) as u32;
            let m4 = *src_p2.add(x) as u32;
            let m5 = *src_p3.add(x) as u32;

            m2 = (m2 + m3) * 22;
            m1 = (m1 + m4) * 9;
            m0 += m5 + m2 + m1 + 32;
            m0 >>= 6;

            *dest_row.add(x) = (m0.min(255)) as u8;
            x += 1;
        }
    }

    // Special case for last line - simple averaging
    if dest_height_val > 1 {
        let dest_row = dest.add((dest_height_val - 1) * dest_pitch_val);
        let src_row_offset = (dest_height_val - 1) * 2 * src_pitch_val;
        let src_row0 = src.add(src_row_offset);
        let src_row1 = src.add(src_row_offset + src_pitch_val);

        let mut x = 0;
        // Process 32 pixels at a time
        while x + 32 <= dest_width_val {
            let a = _mm256_loadu_si256(src_row0.add(x) as *const __m256i);
            let b = _mm256_loadu_si256(src_row1.add(x) as *const __m256i);

            // Convert to u16 for calculation
            let a_lo = _mm256_unpacklo_epi8(a, _mm256_setzero_si256());
            let a_hi = _mm256_unpackhi_epi8(a, _mm256_setzero_si256());
            let b_lo = _mm256_unpacklo_epi8(b, _mm256_setzero_si256());
            let b_hi = _mm256_unpackhi_epi8(b, _mm256_setzero_si256());

            // Add with rounding: (a + b + 1) / 2
            let sum_lo = _mm256_add_epi16(a_lo, b_lo);
            let sum_hi = _mm256_add_epi16(a_hi, b_hi);
            let one = _mm256_set1_epi16(1);
            let sum_lo_rounded = _mm256_add_epi16(sum_lo, one);
            let sum_hi_rounded = _mm256_add_epi16(sum_hi, one);
            let result_lo = _mm256_srli_epi16(sum_lo_rounded, 1);
            let result_hi = _mm256_srli_epi16(sum_hi_rounded, 1);

            // Pack back to u8
            let result = _mm256_packus_epi16(result_lo, result_hi);
            _mm256_storeu_si256(dest_row.add(x) as *mut __m256i, result);

            x += 32;
        }

        // Handle remaining pixels
        while x < dest_width_val {
            let a = *src_row0.add(x) as u32;
            let b = *src_row1.add(x) as u32;
            *dest_row.add(x) = ((a + b + 1) / 2) as u8;
            x += 1;
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_quadratic_horizontal_inplace_u8(
    dest: *mut u8,
    dest_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let dest_pitch_val = dest_pitch.get();
    let dest_width_val = dest_width.get();
    let dest_height_val = dest_height.get();

    for y in 0..dest_height_val {
        let dest_row = dest.add(y * dest_pitch_val);

        // Special case start of line
        let a = *dest_row as u32;
        let b = *dest_row.add(1) as u32;
        let src0 = ((a + b + 1) / 2) as u8;

        // Middle of line - process multiple pixels with SIMD where possible
        for x in 1..(dest_width_val - 1) {
            let mut m0 = *dest_row.add(x * 2 - 2) as u32;
            let mut m1 = *dest_row.add(x * 2 - 1) as u32;
            let mut m2 = *dest_row.add(x * 2) as u32;
            let m3 = *dest_row.add(x * 2 + 1) as u32;
            let m4 = *dest_row.add(x * 2 + 2) as u32;
            let m5 = *dest_row.add(x * 2 + 3) as u32;

            m2 = (m2 + m3) * 22;
            m1 = (m1 + m4) * 9;
            m0 += m5 + m2 + m1 + 32;
            m0 >>= 6;

            *dest_row.add(x) = (m0.min(255)) as u8;
        }

        *dest_row = src0;

        // Special case end of line
        if dest_width_val > 1 {
            let x = dest_width_val - 1;
            let a = *dest_row.add(x * 2) as u32;
            let b = *dest_row.add(x * 2 + 1) as u32;
            *dest_row.add(x) = ((a + b + 1) / 2) as u8;
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_quadratic_vertical_u16(
    dest: *mut u16,
    src: *const u16,
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let dest_pitch_val = dest_pitch.get();
    let src_pitch_val = src_pitch.get();
    let dest_width_val = dest_width.get();
    let dest_height_val = dest_height.get();

    // Special case for first line - simple averaging
    {
        let dest_row = dest;
        let src_row0 = src;
        let src_row1 = src.add(src_pitch_val);

        let mut x = 0;
        // Process 16 pixels at a time
        while x + 16 <= dest_width_val {
            let a = _mm256_loadu_si256(src_row0.add(x) as *const __m256i);
            let b = _mm256_loadu_si256(src_row1.add(x) as *const __m256i);

            // Convert to u32 for calculation to avoid overflow
            let a_lo = _mm256_unpacklo_epi16(a, _mm256_setzero_si256());
            let a_hi = _mm256_unpackhi_epi16(a, _mm256_setzero_si256());
            let b_lo = _mm256_unpacklo_epi16(b, _mm256_setzero_si256());
            let b_hi = _mm256_unpackhi_epi16(b, _mm256_setzero_si256());

            // Add with rounding: (a + b + 1) / 2
            let sum_lo = _mm256_add_epi32(a_lo, b_lo);
            let sum_hi = _mm256_add_epi32(a_hi, b_hi);
            let one = _mm256_set1_epi32(1);
            let sum_lo_rounded = _mm256_add_epi32(sum_lo, one);
            let sum_hi_rounded = _mm256_add_epi32(sum_hi, one);
            let result_lo = _mm256_srli_epi32(sum_lo_rounded, 1);
            let result_hi = _mm256_srli_epi32(sum_hi_rounded, 1);

            // Pack back to u16
            let result = _mm256_packus_epi32(result_lo, result_hi);
            _mm256_storeu_si256(dest_row.add(x) as *mut __m256i, result);

            x += 16;
        }

        // Handle remaining pixels
        while x < dest_width_val {
            let a = *src_row0.add(x) as u32;
            let b = *src_row1.add(x) as u32;
            *dest_row.add(x) = ((a + b + 1) / 2) as u16;
            x += 1;
        }
    }

    // Middle lines - full quadratic filter
    for y in 1..(dest_height_val - 1) {
        let dest_row = dest.add(y * dest_pitch_val);
        let src_row_offset = y * 2 * src_pitch_val;
        let src_m2 = src.add(src_row_offset - src_pitch_val * 2);
        let src_m1 = src.add(src_row_offset - src_pitch_val);
        let src_p0 = src.add(src_row_offset);
        let src_p1 = src.add(src_row_offset + src_pitch_val);
        let src_p2 = src.add(src_row_offset + src_pitch_val * 2);
        let src_p3 = src.add(src_row_offset + src_pitch_val * 3);

        let mut x = 0;
        // Process 8 pixels at a time (since we need u32 for calculations)
        while x + 8 <= dest_width_val {
            // Load 8 u16 values from each row
            let m0 = _mm_loadu_si128(src_m2.add(x) as *const __m128i);
            let m1 = _mm_loadu_si128(src_m1.add(x) as *const __m128i);
            let m2 = _mm_loadu_si128(src_p0.add(x) as *const __m128i);
            let m3 = _mm_loadu_si128(src_p1.add(x) as *const __m128i);
            let m4 = _mm_loadu_si128(src_p2.add(x) as *const __m128i);
            let m5 = _mm_loadu_si128(src_p3.add(x) as *const __m128i);

            // Convert to u32
            let m0_32 = _mm256_cvtepu16_epi32(m0);
            let m1_32 = _mm256_cvtepu16_epi32(m1);
            let m2_32 = _mm256_cvtepu16_epi32(m2);
            let m3_32 = _mm256_cvtepu16_epi32(m3);
            let m4_32 = _mm256_cvtepu16_epi32(m4);
            let m5_32 = _mm256_cvtepu16_epi32(m5);

            // Apply quadratic filter: (m0 + m5 + 9*(m1 + m4) + 22*(m2 + m3) + 32) >> 6
            let sum_m2_m3 = _mm256_add_epi32(m2_32, m3_32);
            let mul_22 = _mm256_mullo_epi32(sum_m2_m3, _mm256_set1_epi32(22));

            let sum_m1_m4 = _mm256_add_epi32(m1_32, m4_32);
            let mul_9 = _mm256_mullo_epi32(sum_m1_m4, _mm256_set1_epi32(9));

            let sum_m0_m5 = _mm256_add_epi32(m0_32, m5_32);
            let bias = _mm256_set1_epi32(32);

            let result = _mm256_add_epi32(
                sum_m0_m5,
                _mm256_add_epi32(mul_9, _mm256_add_epi32(mul_22, bias)),
            );
            let final_result = _mm256_srli_epi32(result, 6);

            // Pack back to u16 with saturation
            let packed_result = _mm256_packus_epi32(final_result, _mm256_setzero_si256());
            _mm_storeu_si128(
                dest_row.add(x) as *mut __m128i,
                _mm256_extracti128_si256(packed_result, 0),
            );

            x += 8;
        }

        // Handle remaining pixels
        while x < dest_width_val {
            let mut m0 = *src_m2.add(x) as u32;
            let mut m1 = *src_m1.add(x) as u32;
            let mut m2 = *src_p0.add(x) as u32;
            let m3 = *src_p1.add(x) as u32;
            let m4 = *src_p2.add(x) as u32;
            let m5 = *src_p3.add(x) as u32;

            m2 = (m2 + m3) * 22;
            m1 = (m1 + m4) * 9;
            m0 += m5 + m2 + m1 + 32;
            m0 >>= 6;

            *dest_row.add(x) = (m0.min(65535)) as u16;
            x += 1;
        }
    }

    // Special case for last line - simple averaging
    if dest_height_val > 1 {
        let dest_row = dest.add((dest_height_val - 1) * dest_pitch_val);
        let src_row_offset = (dest_height_val - 1) * 2 * src_pitch_val;
        let src_row0 = src.add(src_row_offset);
        let src_row1 = src.add(src_row_offset + src_pitch_val);

        let mut x = 0;
        // Process 16 pixels at a time
        while x + 16 <= dest_width_val {
            let a = _mm256_loadu_si256(src_row0.add(x) as *const __m256i);
            let b = _mm256_loadu_si256(src_row1.add(x) as *const __m256i);

            // Convert to u32 for calculation to avoid overflow
            let a_lo = _mm256_unpacklo_epi16(a, _mm256_setzero_si256());
            let a_hi = _mm256_unpackhi_epi16(a, _mm256_setzero_si256());
            let b_lo = _mm256_unpacklo_epi16(b, _mm256_setzero_si256());
            let b_hi = _mm256_unpackhi_epi16(b, _mm256_setzero_si256());

            // Add with rounding: (a + b + 1) / 2
            let sum_lo = _mm256_add_epi32(a_lo, b_lo);
            let sum_hi = _mm256_add_epi32(a_hi, b_hi);
            let one = _mm256_set1_epi32(1);
            let sum_lo_rounded = _mm256_add_epi32(sum_lo, one);
            let sum_hi_rounded = _mm256_add_epi32(sum_hi, one);
            let result_lo = _mm256_srli_epi32(sum_lo_rounded, 1);
            let result_hi = _mm256_srli_epi32(sum_hi_rounded, 1);

            // Pack back to u16
            let result = _mm256_packus_epi32(result_lo, result_hi);
            _mm256_storeu_si256(dest_row.add(x) as *mut __m256i, result);

            x += 16;
        }

        // Handle remaining pixels
        while x < dest_width_val {
            let a = *src_row0.add(x) as u32;
            let b = *src_row1.add(x) as u32;
            *dest_row.add(x) = ((a + b + 1) / 2) as u16;
            x += 1;
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_quadratic_horizontal_inplace_u16(
    dest: *mut u16,
    dest_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let dest_pitch_val = dest_pitch.get();
    let dest_width_val = dest_width.get();
    let dest_height_val = dest_height.get();

    for y in 0..dest_height_val {
        let dest_row = dest.add(y * dest_pitch_val);

        // Special case start of line
        let a = *dest_row as u32;
        let b = *dest_row.add(1) as u32;
        let src0 = ((a + b + 1) / 2) as u16;

        // Middle of line - process pixels individually due to dependencies
        for x in 1..(dest_width_val - 1) {
            let mut m0 = *dest_row.add(x * 2 - 2) as u32;
            let mut m1 = *dest_row.add(x * 2 - 1) as u32;
            let mut m2 = *dest_row.add(x * 2) as u32;
            let m3 = *dest_row.add(x * 2 + 1) as u32;
            let m4 = *dest_row.add(x * 2 + 2) as u32;
            let m5 = *dest_row.add(x * 2 + 3) as u32;

            m2 = (m2 + m3) * 22;
            m1 = (m1 + m4) * 9;
            m0 += m5 + m2 + m1 + 32;
            m0 >>= 6;

            *dest_row.add(x) = (m0.min(65535)) as u16;
        }

        *dest_row = src0;

        // Special case end of line
        if dest_width_val > 1 {
            let x = dest_width_val - 1;
            let a = *dest_row.add(x * 2) as u32;
            let b = *dest_row.add(x * 2 + 1) as u32;
            *dest_row.add(x) = ((a + b + 1) / 2) as u16;
        }
    }
}
