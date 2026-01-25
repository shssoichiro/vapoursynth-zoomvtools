#![allow(clippy::undocumented_unsafe_blocks)]

use std::{arch::x86_64::*, num::NonZeroUsize};

use crate::util::Pixel;

/// Downscales an image by 2x using simple averaging of 2x2 pixel blocks.
///
/// This function reduces both the width and height of the source image by half
/// by averaging each 2x2 block of pixels into a single output pixel. The averaging
/// uses proper rounding by adding 2 before dividing by 4, ensuring accurate
/// color representation in the downscaled result.
///
/// # Parameters
/// - `dest`: Destination buffer to store the downscaled image
/// - `src`: Source image buffer to downscale
/// - `dest_pitch`: Number of pixels per row in the destination buffer
/// - `src_pitch`: Number of pixels per row in the source buffer
/// - `dest_width`: Width of the destination image (half of source width)
/// - `dest_height`: Height of the destination image (half of source height)
#[target_feature(enable = "avx2")]
pub fn reduce_average<T: Pixel>(
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
            reduce_average_u8(
                dest.as_mut_ptr() as *mut u8,
                src.as_ptr() as *const u8,
                dest_pitch,
                src_pitch,
                dest_width,
                dest_height,
            );
        },
        2 => unsafe {
            reduce_average_u16(
                dest.as_mut_ptr() as *mut u16,
                src.as_ptr() as *const u16,
                dest_pitch,
                src_pitch,
                dest_width,
                dest_height,
            );
        },
        _ => unreachable!(),
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_average_u8(
    dest: *mut u8,
    src: *const u8,
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let dest_width = dest_width.get();
    let dest_height = dest_height.get();
    let dest_pitch = dest_pitch.get();
    let src_pitch = src_pitch.get();

    // Process 16 destination pixels at a time (requires 32 source pixels per row)
    let simd_width = 16;
    let rounding = _mm256_set1_epi16(2);

    for y in 0..dest_height {
        let dest_row = dest.add(y * dest_pitch);
        let src_row1 = src.add(y * 2 * src_pitch);
        let src_row2 = src.add((y * 2 + 1) * src_pitch);

        let mut x = 0;

        // Process SIMD chunks
        while x + simd_width <= dest_width {
            // Load 32 bytes from each of two rows (32 source pixels per row)
            let src1 = _mm256_loadu_si256(src_row1.add(x * 2) as *const __m256i);
            let src2 = _mm256_loadu_si256(src_row2.add(x * 2) as *const __m256i);

            // Convert to u16 to prevent overflow during addition
            let src1_lo = _mm256_unpacklo_epi8(src1, _mm256_setzero_si256());
            let src1_hi = _mm256_unpackhi_epi8(src1, _mm256_setzero_si256());
            let src2_lo = _mm256_unpacklo_epi8(src2, _mm256_setzero_si256());
            let src2_hi = _mm256_unpackhi_epi8(src2, _mm256_setzero_si256());

            // Add horizontal pairs: (a + b) and (c + d) for each 2x2 block
            let pairs1_lo = _mm256_hadd_epi16(src1_lo, src1_hi);
            let pairs2_lo = _mm256_hadd_epi16(src2_lo, src2_hi);

            // Add vertical pairs: (a+b) + (c+d) for each 2x2 block
            let block_sums = _mm256_add_epi16(_mm256_add_epi16(pairs1_lo, pairs2_lo), rounding);

            // Divide by 4 (right shift by 2)
            let result = _mm256_srli_epi16(block_sums, 2);

            // Pack back to u8
            let packed = _mm256_packus_epi16(result, result);

            // Extract the lower 128 bits and permute to fix ordering
            let final_result = _mm256_permute4x64_epi64(packed, 0b11011000);

            _mm_storeu_si128(
                dest_row.add(x) as *mut __m128i,
                _mm256_extracti128_si256(final_result, 0),
            );

            x += simd_width;
        }

        // Handle remaining pixels with scalar code
        while x < dest_width {
            let src_x = x * 2;
            let a = *src_row1.add(src_x) as u16;
            let b = *src_row1.add(src_x + 1) as u16;
            let c = *src_row2.add(src_x) as u16;
            let d = *src_row2.add(src_x + 1) as u16;

            let avg = ((a + b + c + d + 2) / 4) as u8;
            *dest_row.add(x) = avg;
            x += 1;
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_average_u16(
    dest: *mut u16,
    src: *const u16,
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let dest_width = dest_width.get();
    let dest_height = dest_height.get();
    let dest_pitch = dest_pitch.get();
    let src_pitch = src_pitch.get();

    // Process 8 destination pixels at a time (requires 16 source pixels per row)
    let simd_width = 8;
    let rounding = _mm256_set1_epi32(2);

    for y in 0..dest_height {
        let dest_row = dest.add(y * dest_pitch);
        let src_row1 = src.add(y * 2 * src_pitch);
        let src_row2 = src.add((y * 2 + 1) * src_pitch);

        let mut x = 0;

        // Process SIMD chunks
        while x + simd_width <= dest_width {
            // Load 16 u16 values from each row (16 source pixels per row)
            let src1 = _mm256_loadu_si256(src_row1.add(x * 2) as *const __m256i);
            let src2 = _mm256_loadu_si256(src_row2.add(x * 2) as *const __m256i);

            // Convert to u32 for intermediate calculations to prevent overflow
            let src1_lo = _mm256_unpacklo_epi16(src1, _mm256_setzero_si256());
            let src1_hi = _mm256_unpackhi_epi16(src1, _mm256_setzero_si256());
            let src2_lo = _mm256_unpacklo_epi16(src2, _mm256_setzero_si256());
            let src2_hi = _mm256_unpackhi_epi16(src2, _mm256_setzero_si256());

            // Add horizontal pairs: (a + b) and (c + d) for each 2x2 block
            let pairs1_lo = _mm256_hadd_epi32(src1_lo, src1_hi);
            let pairs2_lo = _mm256_hadd_epi32(src2_lo, src2_hi);

            // Add vertical pairs: (a+b) + (c+d) for each 2x2 block
            let block_sums = _mm256_add_epi32(_mm256_add_epi32(pairs1_lo, pairs2_lo), rounding);

            // Divide by 4 (right shift by 2)
            let result = _mm256_srli_epi32(block_sums, 2);

            // Pack back to u16
            let packed = _mm256_packus_epi32(result, result);

            // Extract the lower 128 bits and permute to fix ordering
            let final_result = _mm256_permute4x64_epi64(packed, 0b11011000);

            _mm_storeu_si128(
                dest_row.add(x) as *mut __m128i,
                _mm256_extracti128_si256(final_result, 0),
            );

            x += simd_width;
        }

        // Handle remaining pixels with scalar code
        while x < dest_width {
            let src_x = x * 2;
            let a = *src_row1.add(src_x) as u32;
            let b = *src_row1.add(src_x + 1) as u32;
            let c = *src_row2.add(src_x) as u32;
            let d = *src_row2.add(src_x + 1) as u32;

            let avg = ((a + b + c + d + 2) / 4) as u16;
            *dest_row.add(x) = avg;
            x += 1;
        }
    }
}
