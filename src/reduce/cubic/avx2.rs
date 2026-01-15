#![allow(clippy::undocumented_unsafe_blocks)]

use std::{arch::x86_64::*, num::NonZeroUsize};

use crate::util::Pixel;

/// Downscales an image by 2x using cubic interpolation.
///
/// This function reduces both the width and height of the source image by half
/// using a two-pass cubic filtering approach. First, vertical filtering is
/// applied to reduce the height, then horizontal filtering is applied in-place
/// to reduce the width. Cubic interpolation provides higher quality than bilinear
/// by using a wider kernel that considers more neighboring pixels for smoother results.
///
/// The cubic filter uses a 6-tap kernel with specific weights optimized for
/// downscaling while preserving image details and reducing artifacts.
///
/// # Parameters
/// - `dest`: Destination buffer to store the downscaled image
/// - `src`: Source image buffer to downscale
/// - `dest_pitch`: Number of pixels per row in the destination buffer
/// - `src_pitch`: Number of pixels per row in the source buffer
/// - `dest_width`: Width of the destination image (half of source width)
/// - `dest_height`: Height of the destination image (half of source height)
#[target_feature(enable = "avx2")]
pub fn reduce_cubic<T: Pixel>(
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
            reduce_cubic_vertical_u8(
                dest.as_mut_ptr() as *mut u8,
                src.as_ptr() as *const u8,
                dest_pitch,
                src_pitch,
                // SAFETY: non-zero constant
                dest_width.saturating_mul(NonZeroUsize::new_unchecked(2)),
                dest_height,
            );
            reduce_cubic_horizontal_inplace_u8(
                dest.as_mut_ptr() as *mut u8,
                dest_pitch,
                dest_width,
                dest_height,
            );
        },
        2 => unsafe {
            reduce_cubic_vertical_u16(
                dest.as_mut_ptr() as *mut u16,
                src.as_ptr() as *const u16,
                dest_pitch,
                src_pitch,
                // SAFETY: non-zero constant
                dest_width.saturating_mul(NonZeroUsize::new_unchecked(2)),
                dest_height,
            );
            reduce_cubic_horizontal_inplace_u16(
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
unsafe fn reduce_cubic_vertical_u8(
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

    // Special case for first line - simple averaging
    let mut x = 0;
    while x + 32 <= dest_width {
        let a = _mm256_loadu_si256(src.add(x) as *const __m256i);
        let b = _mm256_loadu_si256(src.add(x + src_pitch) as *const __m256i);

        // Convert to u16 for calculation
        let a_lo = _mm256_unpacklo_epi8(a, _mm256_setzero_si256());
        let a_hi = _mm256_unpackhi_epi8(a, _mm256_setzero_si256());
        let b_lo = _mm256_unpacklo_epi8(b, _mm256_setzero_si256());
        let b_hi = _mm256_unpackhi_epi8(b, _mm256_setzero_si256());

        // (a + b + 1) / 2
        let sum_lo = _mm256_add_epi16(_mm256_add_epi16(a_lo, b_lo), _mm256_set1_epi16(1));
        let sum_hi = _mm256_add_epi16(_mm256_add_epi16(a_hi, b_hi), _mm256_set1_epi16(1));
        let result_lo = _mm256_srli_epi16(sum_lo, 1);
        let result_hi = _mm256_srli_epi16(sum_hi, 1);

        let result = _mm256_packus_epi16(result_lo, result_hi);
        _mm256_storeu_si256(dest_ptr.add(x) as *mut __m256i, result);

        x += 32;
    }

    // Handle remaining pixels
    while x < dest_width {
        let a = *src.add(x) as u16;
        let b = *src.add(x + src_pitch) as u16;
        *dest_ptr.add(x) = ((a + b + 1) / 2) as u8;
        x += 1;
    }

    dest_ptr = dest_ptr.add(dest_pitch);

    // Middle lines - full cubic filter
    for y in 1..(dest_height - 1) {
        let src_row_offset = y * 2 * src_pitch;
        let mut x = 0;

        while x + 16 <= dest_width {
            // Load 6 rows of 16 u8 pixels each
            let m0 =
                _mm256_loadu_si256(src.add(src_row_offset + x - src_pitch * 2) as *const __m256i);
            let m1 = _mm256_loadu_si256(src.add(src_row_offset + x - src_pitch) as *const __m256i);
            let m2 = _mm256_loadu_si256(src.add(src_row_offset + x) as *const __m256i);
            let m3 = _mm256_loadu_si256(src.add(src_row_offset + x + src_pitch) as *const __m256i);
            let m4 =
                _mm256_loadu_si256(src.add(src_row_offset + x + src_pitch * 2) as *const __m256i);
            let m5 =
                _mm256_loadu_si256(src.add(src_row_offset + x + src_pitch * 3) as *const __m256i);

            // Process low and high halves separately to avoid overflow
            let m0_lo = _mm256_unpacklo_epi8(m0, _mm256_setzero_si256());
            let m1_lo = _mm256_unpacklo_epi8(m1, _mm256_setzero_si256());
            let m2_lo = _mm256_unpacklo_epi8(m2, _mm256_setzero_si256());
            let m3_lo = _mm256_unpacklo_epi8(m3, _mm256_setzero_si256());
            let m4_lo = _mm256_unpacklo_epi8(m4, _mm256_setzero_si256());
            let m5_lo = _mm256_unpacklo_epi8(m5, _mm256_setzero_si256());

            // Apply cubic filter: m0 + m5 + (m1 + m4) * 5 + (m2 + m3) * 10 + 16 >> 5
            let term1_lo = _mm256_add_epi16(m0_lo, m5_lo);
            let term2_lo = _mm256_mullo_epi16(_mm256_add_epi16(m1_lo, m4_lo), _mm256_set1_epi16(5));
            let term3_lo =
                _mm256_mullo_epi16(_mm256_add_epi16(m2_lo, m3_lo), _mm256_set1_epi16(10));
            let sum_lo = _mm256_add_epi16(_mm256_add_epi16(term1_lo, term2_lo), term3_lo);
            let result_lo = _mm256_srli_epi16(_mm256_add_epi16(sum_lo, _mm256_set1_epi16(16)), 5);

            // High half
            let m0_hi = _mm256_unpackhi_epi8(m0, _mm256_setzero_si256());
            let m1_hi = _mm256_unpackhi_epi8(m1, _mm256_setzero_si256());
            let m2_hi = _mm256_unpackhi_epi8(m2, _mm256_setzero_si256());
            let m3_hi = _mm256_unpackhi_epi8(m3, _mm256_setzero_si256());
            let m4_hi = _mm256_unpackhi_epi8(m4, _mm256_setzero_si256());
            let m5_hi = _mm256_unpackhi_epi8(m5, _mm256_setzero_si256());

            let term1_hi = _mm256_add_epi16(m0_hi, m5_hi);
            let term2_hi = _mm256_mullo_epi16(_mm256_add_epi16(m1_hi, m4_hi), _mm256_set1_epi16(5));
            let term3_hi =
                _mm256_mullo_epi16(_mm256_add_epi16(m2_hi, m3_hi), _mm256_set1_epi16(10));
            let sum_hi = _mm256_add_epi16(_mm256_add_epi16(term1_hi, term2_hi), term3_hi);
            let result_hi = _mm256_srli_epi16(_mm256_add_epi16(sum_hi, _mm256_set1_epi16(16)), 5);

            let result = _mm256_packus_epi16(result_lo, result_hi);
            _mm256_storeu_si256(dest_ptr.add(x) as *mut __m256i, result);

            x += 16;
        }

        // Handle remaining pixels
        while x < dest_width {
            let m0 = *src.add(src_row_offset + x - src_pitch * 2) as u16;
            let m1 = *src.add(src_row_offset + x - src_pitch) as u16;
            let m2 = *src.add(src_row_offset + x) as u16;
            let m3 = *src.add(src_row_offset + x + src_pitch) as u16;
            let m4 = *src.add(src_row_offset + x + src_pitch * 2) as u16;
            let m5 = *src.add(src_row_offset + x + src_pitch * 3) as u16;

            let result = (m0 + m5 + (m1 + m4) * 5 + (m2 + m3) * 10 + 16) >> 5;
            *dest_ptr.add(x) = result.min(255) as u8;
            x += 1;
        }

        dest_ptr = dest_ptr.add(dest_pitch);
    }

    // Special case for last line
    if dest_height > 1 {
        let src_row_offset = (dest_height - 1) * 2 * src_pitch;
        let mut x = 0;

        while x + 32 <= dest_width {
            let a = _mm256_loadu_si256(src.add(src_row_offset + x) as *const __m256i);
            let b = _mm256_loadu_si256(src.add(src_row_offset + x + src_pitch) as *const __m256i);

            let a_lo = _mm256_unpacklo_epi8(a, _mm256_setzero_si256());
            let a_hi = _mm256_unpackhi_epi8(a, _mm256_setzero_si256());
            let b_lo = _mm256_unpacklo_epi8(b, _mm256_setzero_si256());
            let b_hi = _mm256_unpackhi_epi8(b, _mm256_setzero_si256());

            let sum_lo = _mm256_add_epi16(_mm256_add_epi16(a_lo, b_lo), _mm256_set1_epi16(1));
            let sum_hi = _mm256_add_epi16(_mm256_add_epi16(a_hi, b_hi), _mm256_set1_epi16(1));
            let result_lo = _mm256_srli_epi16(sum_lo, 1);
            let result_hi = _mm256_srli_epi16(sum_hi, 1);

            let result = _mm256_packus_epi16(result_lo, result_hi);
            _mm256_storeu_si256(dest_ptr.add(x) as *mut __m256i, result);

            x += 32;
        }

        while x < dest_width {
            let a = *src.add(src_row_offset + x) as u16;
            let b = *src.add(src_row_offset + x + src_pitch) as u16;
            *dest_ptr.add(x) = ((a + b + 1) / 2) as u8;
            x += 1;
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_cubic_horizontal_inplace_u8(
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
        // Special case start of line
        let a = *dest_ptr as u16;
        let b = *dest_ptr.add(1) as u16;
        let src0 = ((a + b + 1) / 2) as u8;

        // Middle of line
        for x in 1..(dest_width - 1) {
            let m0 = *dest_ptr.add(x * 2 - 2) as u16;
            let m1 = *dest_ptr.add(x * 2 - 1) as u16;
            let m2 = *dest_ptr.add(x * 2) as u16;
            let m3 = *dest_ptr.add(x * 2 + 1) as u16;
            let m4 = *dest_ptr.add(x * 2 + 2) as u16;
            let m5 = *dest_ptr.add(x * 2 + 3) as u16;

            let result = (m0 + m5 + (m1 + m4) * 5 + (m2 + m3) * 10 + 16) >> 5;
            *dest_ptr.add(x) = result.min(255) as u8;
        }

        *dest_ptr = src0;

        // Special case end of line
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
unsafe fn reduce_cubic_vertical_u16(
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

    // Special case for first line
    let mut x = 0;
    while x + 16 <= dest_width {
        let a = _mm256_loadu_si256(src.add(x) as *const __m256i);
        let b = _mm256_loadu_si256(src.add(x + src_pitch) as *const __m256i);

        let sum = _mm256_add_epi16(_mm256_add_epi16(a, b), _mm256_set1_epi16(1));
        let result = _mm256_srli_epi16(sum, 1);

        _mm256_storeu_si256(dest_ptr.add(x) as *mut __m256i, result);
        x += 16;
    }

    while x < dest_width {
        let a = *src.add(x) as u32;
        let b = *src.add(x + src_pitch) as u32;
        *dest_ptr.add(x) = ((a + b + 1) / 2) as u16;
        x += 1;
    }

    dest_ptr = dest_ptr.add(dest_pitch);

    // Middle lines
    for y in 1..(dest_height - 1) {
        let src_row_offset = y * 2 * src_pitch;
        let mut x = 0;

        while x + 16 <= dest_width {
            let m0 =
                _mm256_loadu_si256(src.add(src_row_offset + x - src_pitch * 2) as *const __m256i);
            let m1 = _mm256_loadu_si256(src.add(src_row_offset + x - src_pitch) as *const __m256i);
            let m2 = _mm256_loadu_si256(src.add(src_row_offset + x) as *const __m256i);
            let m3 = _mm256_loadu_si256(src.add(src_row_offset + x + src_pitch) as *const __m256i);
            let m4 =
                _mm256_loadu_si256(src.add(src_row_offset + x + src_pitch * 2) as *const __m256i);
            let m5 =
                _mm256_loadu_si256(src.add(src_row_offset + x + src_pitch * 3) as *const __m256i);

            // Need to use 32-bit arithmetic to avoid overflow
            let m0_lo = _mm256_unpacklo_epi16(m0, _mm256_setzero_si256());
            let m1_lo = _mm256_unpacklo_epi16(m1, _mm256_setzero_si256());
            let m2_lo = _mm256_unpacklo_epi16(m2, _mm256_setzero_si256());
            let m3_lo = _mm256_unpacklo_epi16(m3, _mm256_setzero_si256());
            let m4_lo = _mm256_unpacklo_epi16(m4, _mm256_setzero_si256());
            let m5_lo = _mm256_unpacklo_epi16(m5, _mm256_setzero_si256());

            let term1_lo = _mm256_add_epi32(m0_lo, m5_lo);
            let term2_lo = _mm256_mullo_epi32(_mm256_add_epi32(m1_lo, m4_lo), _mm256_set1_epi32(5));
            let term3_lo =
                _mm256_mullo_epi32(_mm256_add_epi32(m2_lo, m3_lo), _mm256_set1_epi32(10));
            let sum_lo = _mm256_add_epi32(_mm256_add_epi32(term1_lo, term2_lo), term3_lo);
            let result_lo = _mm256_srli_epi32(_mm256_add_epi32(sum_lo, _mm256_set1_epi32(16)), 5);

            let m0_hi = _mm256_unpackhi_epi16(m0, _mm256_setzero_si256());
            let m1_hi = _mm256_unpackhi_epi16(m1, _mm256_setzero_si256());
            let m2_hi = _mm256_unpackhi_epi16(m2, _mm256_setzero_si256());
            let m3_hi = _mm256_unpackhi_epi16(m3, _mm256_setzero_si256());
            let m4_hi = _mm256_unpackhi_epi16(m4, _mm256_setzero_si256());
            let m5_hi = _mm256_unpackhi_epi16(m5, _mm256_setzero_si256());

            let term1_hi = _mm256_add_epi32(m0_hi, m5_hi);
            let term2_hi = _mm256_mullo_epi32(_mm256_add_epi32(m1_hi, m4_hi), _mm256_set1_epi32(5));
            let term3_hi =
                _mm256_mullo_epi32(_mm256_add_epi32(m2_hi, m3_hi), _mm256_set1_epi32(10));
            let sum_hi = _mm256_add_epi32(_mm256_add_epi32(term1_hi, term2_hi), term3_hi);
            let result_hi = _mm256_srli_epi32(_mm256_add_epi32(sum_hi, _mm256_set1_epi32(16)), 5);

            let result = _mm256_packus_epi32(result_lo, result_hi);
            _mm256_storeu_si256(dest_ptr.add(x) as *mut __m256i, result);

            x += 16;
        }

        while x < dest_width {
            let m0 = *src.add(src_row_offset + x - src_pitch * 2) as u32;
            let m1 = *src.add(src_row_offset + x - src_pitch) as u32;
            let m2 = *src.add(src_row_offset + x) as u32;
            let m3 = *src.add(src_row_offset + x + src_pitch) as u32;
            let m4 = *src.add(src_row_offset + x + src_pitch * 2) as u32;
            let m5 = *src.add(src_row_offset + x + src_pitch * 3) as u32;

            let result = (m0 + m5 + (m1 + m4) * 5 + (m2 + m3) * 10 + 16) >> 5;
            *dest_ptr.add(x) = result.min(65535) as u16;
            x += 1;
        }

        dest_ptr = dest_ptr.add(dest_pitch);
    }

    // Special case for last line
    if dest_height > 1 {
        let src_row_offset = (dest_height - 1) * 2 * src_pitch;
        let mut x = 0;

        while x + 16 <= dest_width {
            let a = _mm256_loadu_si256(src.add(src_row_offset + x) as *const __m256i);
            let b = _mm256_loadu_si256(src.add(src_row_offset + x + src_pitch) as *const __m256i);

            let sum = _mm256_add_epi16(_mm256_add_epi16(a, b), _mm256_set1_epi16(1));
            let result = _mm256_srli_epi16(sum, 1);

            _mm256_storeu_si256(dest_ptr.add(x) as *mut __m256i, result);
            x += 16;
        }

        while x < dest_width {
            let a = *src.add(src_row_offset + x) as u32;
            let b = *src.add(src_row_offset + x + src_pitch) as u32;
            *dest_ptr.add(x) = ((a + b + 1) / 2) as u16;
            x += 1;
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reduce_cubic_horizontal_inplace_u16(
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
        // Special case start of line
        let a = *dest_ptr as u32;
        let b = *dest_ptr.add(1) as u32;
        let src0 = ((a + b + 1) / 2) as u16;

        // Middle of line
        for x in 1..(dest_width - 1) {
            let m0 = *dest_ptr.add(x * 2 - 2) as u32;
            let m1 = *dest_ptr.add(x * 2 - 1) as u32;
            let m2 = *dest_ptr.add(x * 2) as u32;
            let m3 = *dest_ptr.add(x * 2 + 1) as u32;
            let m4 = *dest_ptr.add(x * 2 + 2) as u32;
            let m5 = *dest_ptr.add(x * 2 + 3) as u32;

            let result = (m0 + m5 + (m1 + m4) * 5 + (m2 + m3) * 10 + 16) >> 5;
            *dest_ptr.add(x) = result.min(65535) as u16;
        }

        *dest_ptr = src0;

        // Special case end of line
        if dest_width > 1 {
            let x = dest_width - 1;
            let a = *dest_ptr.add(x * 2) as u32;
            let b = *dest_ptr.add(x * 2 + 1) as u32;
            *dest_ptr.add(x) = ((a + b + 1) / 2) as u16;
        }

        dest_ptr = dest_ptr.add(dest_pitch);
    }
}
