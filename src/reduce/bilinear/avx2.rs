#![allow(clippy::undocumented_unsafe_blocks)]

use std::mem::size_of;
use std::num::NonZeroUsize;

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::util::Pixel;

/// Downscales an image by 2x using bilinear interpolation.
///
/// This function reduces both the width and height of the source image by half
/// using a two-pass bilinear filtering approach. First, vertical filtering is
/// applied to reduce the height, then horizontal filtering is applied in-place
/// to reduce the width. This produces higher quality results than simple averaging
/// by using weighted interpolation that considers neighboring pixels.
///
/// # Parameters
/// - `dest`: Destination buffer to store the downscaled image
/// - `src`: Source image buffer to downscale
/// - `dest_pitch`: Number of pixels per row in the destination buffer
/// - `src_pitch`: Number of pixels per row in the source buffer
/// - `dest_width`: Width of the destination image (half of source width)
/// - `dest_height`: Height of the destination image (half of source height)
#[target_feature(enable = "avx2")]
pub fn reduce_bilinear<T: Pixel>(
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
            reduce_bilinear_vertical_u8(
                dest.as_mut_ptr() as *mut u8,
                src.as_ptr() as *const u8,
                dest_pitch,
                src_pitch,
                // SAFETY: non-zero constant
                dest_width.saturating_mul(NonZeroUsize::new_unchecked(2)),
                dest_height,
            );
            reduce_bilinear_horizontal_inplace_u8(
                dest.as_mut_ptr() as *mut u8,
                dest_pitch,
                dest_width,
                dest_height,
            );
        },
        2 => unsafe {
            reduce_bilinear_vertical_u16(
                dest.as_mut_ptr() as *mut u16,
                src.as_ptr() as *const u16,
                dest_pitch,
                src_pitch,
                // SAFETY: non-zero constant
                dest_width.saturating_mul(NonZeroUsize::new_unchecked(2)),
                dest_height,
            );
            reduce_bilinear_horizontal_inplace_u16(
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
unsafe fn reduce_bilinear_vertical_u8(
    dest: *mut u8,
    src: *const u8,
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let dest_pitch = dest_pitch.get();
    let src_pitch = src_pitch.get();
    let dest_width = dest_width.get();
    let dest_height = dest_height.get();

    let mut dest_ptr = dest;
    let src_ptr = src;

    // Special case for first line: (a + b + 1) / 2
    let mut x = 0;
    while x + 32 <= dest_width {
        let a = _mm256_loadu_si256(src_ptr.add(x) as *const __m256i);
        let b = _mm256_loadu_si256(src_ptr.add(x + src_pitch) as *const __m256i);

        // Convert to 16-bit for arithmetic
        let a_lo = _mm256_unpacklo_epi8(a, _mm256_setzero_si256());
        let a_hi = _mm256_unpackhi_epi8(a, _mm256_setzero_si256());
        let b_lo = _mm256_unpacklo_epi8(b, _mm256_setzero_si256());
        let b_hi = _mm256_unpackhi_epi8(b, _mm256_setzero_si256());

        // (a + b + 1) / 2
        let ones = _mm256_set1_epi16(1);
        let sum_lo = _mm256_add_epi16(_mm256_add_epi16(a_lo, b_lo), ones);
        let sum_hi = _mm256_add_epi16(_mm256_add_epi16(a_hi, b_hi), ones);
        let result_lo = _mm256_srli_epi16(sum_lo, 1);
        let result_hi = _mm256_srli_epi16(sum_hi, 1);

        // Pack back to 8-bit
        let result = _mm256_packus_epi16(result_lo, result_hi);
        _mm256_storeu_si256(dest_ptr.add(x) as *mut __m256i, result);

        x += 32;
    }

    // Handle remaining pixels
    while x < dest_width {
        let a = *src_ptr.add(x) as u16;
        let b = *src_ptr.add(x + src_pitch) as u16;
        *dest_ptr.add(x) = ((a + b + 1) / 2) as u8;
        x += 1;
    }

    dest_ptr = dest_ptr.add(dest_pitch);

    // Middle lines: (a + (b + c) * 3 + d + 4) / 8
    for y in 1..(dest_height - 1) {
        let src_row_offset = y * 2 * src_pitch;

        let mut x = 0;
        while x + 32 <= dest_width {
            let a =
                _mm256_loadu_si256(src_ptr.add(src_row_offset + x - src_pitch) as *const __m256i);
            let b = _mm256_loadu_si256(src_ptr.add(src_row_offset + x) as *const __m256i);
            let c =
                _mm256_loadu_si256(src_ptr.add(src_row_offset + x + src_pitch) as *const __m256i);
            let d = _mm256_loadu_si256(
                src_ptr.add(src_row_offset + x + src_pitch * 2) as *const __m256i
            );

            // Convert to 16-bit for arithmetic
            let a_lo = _mm256_unpacklo_epi8(a, _mm256_setzero_si256());
            let a_hi = _mm256_unpackhi_epi8(a, _mm256_setzero_si256());
            let b_lo = _mm256_unpacklo_epi8(b, _mm256_setzero_si256());
            let b_hi = _mm256_unpackhi_epi8(b, _mm256_setzero_si256());
            let c_lo = _mm256_unpacklo_epi8(c, _mm256_setzero_si256());
            let c_hi = _mm256_unpackhi_epi8(c, _mm256_setzero_si256());
            let d_lo = _mm256_unpacklo_epi8(d, _mm256_setzero_si256());
            let d_hi = _mm256_unpackhi_epi8(d, _mm256_setzero_si256());

            // (b + c) * 3
            let bc_lo = _mm256_add_epi16(b_lo, c_lo);
            let bc_hi = _mm256_add_epi16(b_hi, c_hi);
            let bc3_lo = _mm256_add_epi16(_mm256_add_epi16(bc_lo, bc_lo), bc_lo);
            let bc3_hi = _mm256_add_epi16(_mm256_add_epi16(bc_hi, bc_hi), bc_hi);

            // a + (b + c) * 3 + d + 4
            let fours = _mm256_set1_epi16(4);
            let sum_lo = _mm256_add_epi16(
                _mm256_add_epi16(a_lo, bc3_lo),
                _mm256_add_epi16(d_lo, fours),
            );
            let sum_hi = _mm256_add_epi16(
                _mm256_add_epi16(a_hi, bc3_hi),
                _mm256_add_epi16(d_hi, fours),
            );

            // Divide by 8
            let result_lo = _mm256_srli_epi16(sum_lo, 3);
            let result_hi = _mm256_srli_epi16(sum_hi, 3);

            // Pack back to 8-bit
            let result = _mm256_packus_epi16(result_lo, result_hi);
            _mm256_storeu_si256(dest_ptr.add(x) as *mut __m256i, result);

            x += 32;
        }

        // Handle remaining pixels
        while x < dest_width {
            let a = *src_ptr.add(src_row_offset + x - src_pitch) as u16;
            let b = *src_ptr.add(src_row_offset + x) as u16;
            let c = *src_ptr.add(src_row_offset + x + src_pitch) as u16;
            let d = *src_ptr.add(src_row_offset + x + src_pitch * 2) as u16;
            *dest_ptr.add(x) = ((a + (b + c) * 3 + d + 4) / 8) as u8;
            x += 1;
        }

        dest_ptr = dest_ptr.add(dest_pitch);
    }

    // Special case for last line: (a + b + 1) / 2
    if dest_height > 1 {
        let src_row_offset = (dest_height - 1) * 2 * src_pitch;

        let mut x = 0;
        while x + 32 <= dest_width {
            let a = _mm256_loadu_si256(src_ptr.add(src_row_offset + x) as *const __m256i);
            let b =
                _mm256_loadu_si256(src_ptr.add(src_row_offset + x + src_pitch) as *const __m256i);

            // Convert to 16-bit for arithmetic
            let a_lo = _mm256_unpacklo_epi8(a, _mm256_setzero_si256());
            let a_hi = _mm256_unpackhi_epi8(a, _mm256_setzero_si256());
            let b_lo = _mm256_unpacklo_epi8(b, _mm256_setzero_si256());
            let b_hi = _mm256_unpackhi_epi8(b, _mm256_setzero_si256());

            // (a + b + 1) / 2
            let ones = _mm256_set1_epi16(1);
            let sum_lo = _mm256_add_epi16(_mm256_add_epi16(a_lo, b_lo), ones);
            let sum_hi = _mm256_add_epi16(_mm256_add_epi16(a_hi, b_hi), ones);
            let result_lo = _mm256_srli_epi16(sum_lo, 1);
            let result_hi = _mm256_srli_epi16(sum_hi, 1);

            // Pack back to 8-bit
            let result = _mm256_packus_epi16(result_lo, result_hi);
            _mm256_storeu_si256(dest_ptr.add(x) as *mut __m256i, result);

            x += 32;
        }

        // Handle remaining pixels
        while x < dest_width {
            let a = *src_ptr.add(src_row_offset + x) as u16;
            let b = *src_ptr.add(src_row_offset + x + src_pitch) as u16;
            *dest_ptr.add(x) = ((a + b + 1) / 2) as u8;
            x += 1;
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_bilinear_horizontal_inplace_u8(
    dest: *mut u8,
    dest_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let dest_pitch = dest_pitch.get();
    let dest_width = dest_width.get();
    let dest_height = dest_height.get();

    let mut dest_ptr = dest;

    for _y in 0..dest_height {
        // Special case start of line: (a + b + 1) / 2
        let a = *dest_ptr as u16;
        let b = *dest_ptr.add(1) as u16;
        let src0 = ((a + b + 1) / 2) as u8;

        // Middle of line: (a + (b + c) * 3 + d + 4) / 8
        // Process in groups that fit AVX2 registers
        let mut x = 1;

        // We can process 16 output pixels at a time (64 input pixels)
        while x + 16 <= dest_width - 1 {
            // Load 64 input pixels for 16 output pixels
            // We need to load with proper alignment for shuffles
            let mut results = [0u8; 16];

            for i in 0..16 {
                let idx = x + i;
                let a = *dest_ptr.add(idx * 2 - 1) as u16;
                let b = *dest_ptr.add(idx * 2) as u16;
                let c = *dest_ptr.add(idx * 2 + 1) as u16;
                let d = *dest_ptr.add(idx * 2 + 2) as u16;
                results[i] = ((a + (b + c) * 3 + d + 4) / 8) as u8;
            }

            // Store results
            for i in 0..16 {
                *dest_ptr.add(x + i) = results[i];
            }

            x += 16;
        }

        // Handle remaining middle pixels
        while x < dest_width - 1 {
            let a = *dest_ptr.add(x * 2 - 1) as u16;
            let b = *dest_ptr.add(x * 2) as u16;
            let c = *dest_ptr.add(x * 2 + 1) as u16;
            let d = *dest_ptr.add(x * 2 + 2) as u16;
            *dest_ptr.add(x) = ((a + (b + c) * 3 + d + 4) / 8) as u8;
            x += 1;
        }

        *dest_ptr = src0;

        // Special case end of line: (a + b + 1) / 2
        if dest_width > 1 {
            let x = dest_width - 1;
            let a = *dest_ptr.add(x * 2) as u16;
            let b = *dest_ptr.add(x * 2 + 1) as u16;
            *dest_ptr.add(x) = ((a + b + 1) / 2) as u8;
        }

        dest_ptr = dest_ptr.add(dest_pitch);
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_bilinear_vertical_u16(
    dest: *mut u16,
    src: *const u16,
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let dest_pitch = dest_pitch.get();
    let src_pitch = src_pitch.get();
    let dest_width = dest_width.get();
    let dest_height = dest_height.get();

    let mut dest_ptr = dest;
    let src_ptr = src;

    // Special case for first line: (a + b + 1) / 2
    let mut x = 0;
    while x + 16 <= dest_width {
        let a = _mm256_loadu_si256(src_ptr.add(x) as *const __m256i);
        let b = _mm256_loadu_si256(src_ptr.add(x + src_pitch) as *const __m256i);

        // Convert to 32-bit for arithmetic to avoid overflow
        let a_lo = _mm256_unpacklo_epi16(a, _mm256_setzero_si256());
        let a_hi = _mm256_unpackhi_epi16(a, _mm256_setzero_si256());
        let b_lo = _mm256_unpacklo_epi16(b, _mm256_setzero_si256());
        let b_hi = _mm256_unpackhi_epi16(b, _mm256_setzero_si256());

        // (a + b + 1) / 2
        let ones = _mm256_set1_epi32(1);
        let sum_lo = _mm256_add_epi32(_mm256_add_epi32(a_lo, b_lo), ones);
        let sum_hi = _mm256_add_epi32(_mm256_add_epi32(a_hi, b_hi), ones);
        let result_lo = _mm256_srli_epi32(sum_lo, 1);
        let result_hi = _mm256_srli_epi32(sum_hi, 1);

        // Pack back to 16-bit
        let result = _mm256_packus_epi32(result_lo, result_hi);
        _mm256_storeu_si256(dest_ptr.add(x) as *mut __m256i, result);

        x += 16;
    }

    // Handle remaining pixels
    while x < dest_width {
        let a = *src_ptr.add(x) as u32;
        let b = *src_ptr.add(x + src_pitch) as u32;
        *dest_ptr.add(x) = ((a + b + 1) / 2) as u16;
        x += 1;
    }

    dest_ptr = dest_ptr.add(dest_pitch);

    // Middle lines: (a + (b + c) * 3 + d + 4) / 8
    for y in 1..(dest_height - 1) {
        let src_row_offset = y * 2 * src_pitch;

        let mut x = 0;
        while x + 16 <= dest_width {
            let a =
                _mm256_loadu_si256(src_ptr.add(src_row_offset + x - src_pitch) as *const __m256i);
            let b = _mm256_loadu_si256(src_ptr.add(src_row_offset + x) as *const __m256i);
            let c =
                _mm256_loadu_si256(src_ptr.add(src_row_offset + x + src_pitch) as *const __m256i);
            let d = _mm256_loadu_si256(
                src_ptr.add(src_row_offset + x + src_pitch * 2) as *const __m256i
            );

            // Convert to 32-bit for arithmetic
            let a_lo = _mm256_unpacklo_epi16(a, _mm256_setzero_si256());
            let a_hi = _mm256_unpackhi_epi16(a, _mm256_setzero_si256());
            let b_lo = _mm256_unpacklo_epi16(b, _mm256_setzero_si256());
            let b_hi = _mm256_unpackhi_epi16(b, _mm256_setzero_si256());
            let c_lo = _mm256_unpacklo_epi16(c, _mm256_setzero_si256());
            let c_hi = _mm256_unpackhi_epi16(c, _mm256_setzero_si256());
            let d_lo = _mm256_unpacklo_epi16(d, _mm256_setzero_si256());
            let d_hi = _mm256_unpackhi_epi16(d, _mm256_setzero_si256());

            // (b + c) * 3
            let bc_lo = _mm256_add_epi32(b_lo, c_lo);
            let bc_hi = _mm256_add_epi32(b_hi, c_hi);
            let bc3_lo = _mm256_add_epi32(_mm256_add_epi32(bc_lo, bc_lo), bc_lo);
            let bc3_hi = _mm256_add_epi32(_mm256_add_epi32(bc_hi, bc_hi), bc_hi);

            // a + (b + c) * 3 + d + 4
            let fours = _mm256_set1_epi32(4);
            let sum_lo = _mm256_add_epi32(
                _mm256_add_epi32(a_lo, bc3_lo),
                _mm256_add_epi32(d_lo, fours),
            );
            let sum_hi = _mm256_add_epi32(
                _mm256_add_epi32(a_hi, bc3_hi),
                _mm256_add_epi32(d_hi, fours),
            );

            // Divide by 8
            let result_lo = _mm256_srli_epi32(sum_lo, 3);
            let result_hi = _mm256_srli_epi32(sum_hi, 3);

            // Pack back to 16-bit
            let result = _mm256_packus_epi32(result_lo, result_hi);
            _mm256_storeu_si256(dest_ptr.add(x) as *mut __m256i, result);

            x += 16;
        }

        // Handle remaining pixels
        while x < dest_width {
            let a = *src_ptr.add(src_row_offset + x - src_pitch) as u32;
            let b = *src_ptr.add(src_row_offset + x) as u32;
            let c = *src_ptr.add(src_row_offset + x + src_pitch) as u32;
            let d = *src_ptr.add(src_row_offset + x + src_pitch * 2) as u32;
            *dest_ptr.add(x) = ((a + (b + c) * 3 + d + 4) / 8) as u16;
            x += 1;
        }

        dest_ptr = dest_ptr.add(dest_pitch);
    }

    // Special case for last line: (a + b + 1) / 2
    if dest_height > 1 {
        let src_row_offset = (dest_height - 1) * 2 * src_pitch;

        let mut x = 0;
        while x + 16 <= dest_width {
            let a = _mm256_loadu_si256(src_ptr.add(src_row_offset + x) as *const __m256i);
            let b =
                _mm256_loadu_si256(src_ptr.add(src_row_offset + x + src_pitch) as *const __m256i);

            // Convert to 32-bit for arithmetic
            let a_lo = _mm256_unpacklo_epi16(a, _mm256_setzero_si256());
            let a_hi = _mm256_unpackhi_epi16(a, _mm256_setzero_si256());
            let b_lo = _mm256_unpacklo_epi16(b, _mm256_setzero_si256());
            let b_hi = _mm256_unpackhi_epi16(b, _mm256_setzero_si256());

            // (a + b + 1) / 2
            let ones = _mm256_set1_epi32(1);
            let sum_lo = _mm256_add_epi32(_mm256_add_epi32(a_lo, b_lo), ones);
            let sum_hi = _mm256_add_epi32(_mm256_add_epi32(a_hi, b_hi), ones);
            let result_lo = _mm256_srli_epi32(sum_lo, 1);
            let result_hi = _mm256_srli_epi32(sum_hi, 1);

            // Pack back to 16-bit
            let result = _mm256_packus_epi32(result_lo, result_hi);
            _mm256_storeu_si256(dest_ptr.add(x) as *mut __m256i, result);

            x += 16;
        }

        // Handle remaining pixels
        while x < dest_width {
            let a = *src_ptr.add(src_row_offset + x) as u32;
            let b = *src_ptr.add(src_row_offset + x + src_pitch) as u32;
            *dest_ptr.add(x) = ((a + b + 1) / 2) as u16;
            x += 1;
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_bilinear_horizontal_inplace_u16(
    dest: *mut u16,
    dest_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let dest_pitch = dest_pitch.get();
    let dest_width = dest_width.get();
    let dest_height = dest_height.get();

    let mut dest_ptr = dest;

    for _y in 0..dest_height {
        // Special case start of line: (a + b + 1) / 2
        let a = *dest_ptr as u32;
        let b = *dest_ptr.add(1) as u32;
        let src0 = ((a + b + 1) / 2) as u16;

        // Middle of line: (a + (b + c) * 3 + d + 4) / 8
        let mut x = 1;

        // Process 8 output pixels at a time (32 input pixels)
        while x + 8 <= dest_width - 1 {
            let mut results = [0u16; 8];

            for i in 0..8 {
                let idx = x + i;
                let a = *dest_ptr.add(idx * 2 - 1) as u32;
                let b = *dest_ptr.add(idx * 2) as u32;
                let c = *dest_ptr.add(idx * 2 + 1) as u32;
                let d = *dest_ptr.add(idx * 2 + 2) as u32;
                results[i] = ((a + (b + c) * 3 + d + 4) / 8) as u16;
            }

            // Store results
            for i in 0..8 {
                *dest_ptr.add(x + i) = results[i];
            }

            x += 8;
        }

        // Handle remaining middle pixels
        while x < dest_width - 1 {
            let a = *dest_ptr.add(x * 2 - 1) as u32;
            let b = *dest_ptr.add(x * 2) as u32;
            let c = *dest_ptr.add(x * 2 + 1) as u32;
            let d = *dest_ptr.add(x * 2 + 2) as u32;
            *dest_ptr.add(x) = ((a + (b + c) * 3 + d + 4) / 8) as u16;
            x += 1;
        }

        *dest_ptr = src0;

        // Special case end of line: (a + b + 1) / 2
        if dest_width > 1 {
            let x = dest_width - 1;
            let a = *dest_ptr.add(x * 2) as u32;
            let b = *dest_ptr.add(x * 2 + 1) as u32;
            *dest_ptr.add(x) = ((a + b + 1) / 2) as u16;
        }

        dest_ptr = dest_ptr.add(dest_pitch);
    }
}
