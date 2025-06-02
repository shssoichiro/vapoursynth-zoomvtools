#![allow(clippy::undocumented_unsafe_blocks)]
#![allow(unsafe_op_in_unsafe_fn)]

use std::arch::x86_64::*;
use std::num::{NonZeroU8, NonZeroUsize};

use crate::util::Pixel;

/// Performs horizontal Wiener filtering for sub-pixel motion estimation refinement.
///
/// This function applies a Wiener filter horizontally to create high-quality sub-pixel
/// samples between existing pixels. The Wiener filter uses a 6-tap kernel with optimized
/// coefficients that provide excellent interpolation quality by minimizing reconstruction
/// error while preserving image details.
///
/// Edge pixels use simple averaging due to insufficient neighbors for the full kernel.
/// The Wiener filter is particularly effective for maintaining sharpness during
/// sub-pixel interpolation in motion estimation applications.
///
/// # Parameters
/// - `src`: Source image buffer
/// - `dest`: Destination buffer for interpolated results
/// - `pitch`: Number of pixels per row in both buffers
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
/// - `bits_per_sample`: Bit depth of the pixel format for clamping
#[target_feature(enable = "avx2")]
pub unsafe fn refine_horizontal_wiener<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    debug_assert!(
        bits_per_sample.get() as usize > (size_of::<T>() - 1) * 8
            && (bits_per_sample.get() as usize <= size_of::<T>() * 8)
    );

    match size_of::<T>() {
        1 => refine_horizontal_wiener_u8(
            src.as_ptr() as *const u8,
            dest.as_mut_ptr() as *mut u8,
            pitch,
            width,
            height,
            bits_per_sample,
        ),
        2 => refine_horizontal_wiener_u16(
            src.as_ptr() as *const u16,
            dest.as_mut_ptr() as *mut u16,
            pitch,
            width,
            height,
            bits_per_sample,
        ),
        _ => unreachable!(),
    }
}

/// Performs vertical Wiener filtering for sub-pixel motion estimation refinement.
///
/// This function applies a Wiener filter vertically to create high-quality sub-pixel
/// samples between existing pixels. The Wiener filter uses a 6-tap kernel with optimized
/// coefficients that provide excellent interpolation quality by minimizing reconstruction
/// error while preserving image details.
///
/// Edge rows use simple averaging due to insufficient neighbors for the full kernel,
/// and the last row is copied directly from the source. The Wiener filter is
/// particularly effective for maintaining sharpness during sub-pixel interpolation.
///
/// # Parameters
/// - `src`: Source image buffer
/// - `dest`: Destination buffer for interpolated results
/// - `pitch`: Number of pixels per row in both buffers
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
/// - `bits_per_sample`: Bit depth of the pixel format for clamping
#[target_feature(enable = "avx2")]
pub unsafe fn refine_vertical_wiener<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    debug_assert!(
        bits_per_sample.get() as usize > (size_of::<T>() - 1) * 8
            && (bits_per_sample.get() as usize <= size_of::<T>() * 8)
    );

    match size_of::<T>() {
        1 => refine_vertical_wiener_u8(
            src.as_ptr() as *const u8,
            dest.as_mut_ptr() as *mut u8,
            pitch,
            width,
            height,
            bits_per_sample,
        ),
        2 => refine_vertical_wiener_u16(
            src.as_ptr() as *const u16,
            dest.as_mut_ptr() as *mut u16,
            pitch,
            width,
            height,
            bits_per_sample,
        ),
        _ => unreachable!(),
    }
}

#[target_feature(enable = "avx2")]
unsafe fn refine_horizontal_wiener_u8(
    src: *const u8,
    dest: *mut u8,
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    let pixel_max = _mm256_set1_epi16(((1i32 << bits_per_sample.get()) - 1) as i16);
    let zero = _mm256_setzero_si256();
    let _one = _mm256_set1_epi16(1);
    let _two = _mm256_set1_epi16(2);
    let four = _mm256_set1_epi16(4);
    let five = _mm256_set1_epi16(5);
    let sixteen = _mm256_set1_epi16(16);

    let mut offset = 0;

    for _j in 0..height.get() {
        // Handle first two pixels with bilinear interpolation
        if width.get() >= 2 {
            let a = *src.add(offset) as u16;
            let b = *src.add(offset + 1) as u16;
            *dest.add(offset) = ((a + b + 1) / 2) as u8;

            if width.get() >= 3 {
                let c = *src.add(offset + 2) as u16;
                *dest.add(offset + 1) = ((b + c + 1) / 2) as u8;
            }
        }

        // Process middle pixels with Wiener filter (SIMD)
        let wiener_start = 2;
        let wiener_end = if width.get() >= 4 {
            width.get() - 4
        } else {
            wiener_start
        };

        let mut i = wiener_start;
        while i + 32 <= wiener_end {
            // Load 32 pixels centered around current position
            let m0_bytes = _mm256_loadu_si256((src.add(offset + i - 2)) as *const __m256i);
            let m1_bytes = _mm256_loadu_si256((src.add(offset + i - 1)) as *const __m256i);
            let m2_bytes = _mm256_loadu_si256((src.add(offset + i)) as *const __m256i);
            let m3_bytes = _mm256_loadu_si256((src.add(offset + i + 1)) as *const __m256i);
            let m4_bytes = _mm256_loadu_si256((src.add(offset + i + 2)) as *const __m256i);
            let m5_bytes = _mm256_loadu_si256((src.add(offset + i + 3)) as *const __m256i);

            // Process first 16 pixels
            let m0_lo = _mm256_unpacklo_epi8(m0_bytes, zero);
            let m1_lo = _mm256_unpacklo_epi8(m1_bytes, zero);
            let m2_lo = _mm256_unpacklo_epi8(m2_bytes, zero);
            let m3_lo = _mm256_unpacklo_epi8(m3_bytes, zero);
            let m4_lo = _mm256_unpacklo_epi8(m4_bytes, zero);
            let m5_lo = _mm256_unpacklo_epi8(m5_bytes, zero);

            let result_lo = apply_wiener_kernel_u8(
                m0_lo, m1_lo, m2_lo, m3_lo, m4_lo, m5_lo, four, five, sixteen, pixel_max,
            );

            // Process next 16 pixels
            let m0_hi = _mm256_unpackhi_epi8(m0_bytes, zero);
            let m1_hi = _mm256_unpackhi_epi8(m1_bytes, zero);
            let m2_hi = _mm256_unpackhi_epi8(m2_bytes, zero);
            let m3_hi = _mm256_unpackhi_epi8(m3_bytes, zero);
            let m4_hi = _mm256_unpackhi_epi8(m4_bytes, zero);
            let m5_hi = _mm256_unpackhi_epi8(m5_bytes, zero);

            let result_hi = apply_wiener_kernel_u8(
                m0_hi, m1_hi, m2_hi, m3_hi, m4_hi, m5_hi, four, five, sixteen, pixel_max,
            );

            // Pack results back to bytes
            let result = _mm256_packus_epi16(result_lo, result_hi);
            _mm256_storeu_si256((dest.add(offset + i)) as *mut __m256i, result);

            i += 32;
        }

        // Handle remaining pixels with scalar code
        while i < wiener_end {
            let m0 = *src.add(offset + i - 2) as i16;
            let m1 = *src.add(offset + i - 1) as i16;
            let mut m2 = *src.add(offset + i) as i16;
            let m3 = *src.add(offset + i + 1) as i16;
            let m4 = *src.add(offset + i + 2) as i16;
            let m5 = *src.add(offset + i + 3) as i16;

            m2 = (m2 + m3) * 4;
            m2 -= m1 + m4;
            m2 *= 5;
            let result = (m0 + m5 + m2 + 16) >> 5;

            *dest.add(offset + i) = result.max(0).min((1 << bits_per_sample.get()) - 1) as u8;
            i += 1;
        }

        // Handle last few pixels with bilinear interpolation
        for i in wiener_end..(width.get() - 1).min(width.get()) {
            let a = *src.add(offset + i) as u16;
            let b = *src.add(offset + i + 1) as u16;
            *dest.add(offset + i) = ((a + b + 1) / 2) as u8;
        }

        // Copy last pixel
        if width.get() > 0 {
            *dest.add(offset + width.get() - 1) = *src.add(offset + width.get() - 1);
        }

        offset += pitch.get();
    }
}

#[target_feature(enable = "avx2")]
unsafe fn refine_horizontal_wiener_u16(
    src: *const u16,
    dest: *mut u16,
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    let pixel_max = _mm256_set1_epi32((1i32 << bits_per_sample.get()) - 1);
    let four = _mm256_set1_epi32(4);
    let five = _mm256_set1_epi32(5);
    let sixteen = _mm256_set1_epi32(16);

    let mut offset = 0;

    for _j in 0..height.get() {
        // Handle first two pixels with bilinear interpolation
        if width.get() >= 2 {
            let a = *src.add(offset) as u32;
            let b = *src.add(offset + 1) as u32;
            *dest.add(offset) = ((a + b + 1) / 2) as u16;

            if width.get() >= 3 {
                let c = *src.add(offset + 2) as u32;
                *dest.add(offset + 1) = ((b + c + 1) / 2) as u16;
            }
        }

        // Process middle pixels with Wiener filter (SIMD)
        let wiener_start = 2;
        let wiener_end = if width.get() >= 4 {
            width.get() - 4
        } else {
            wiener_start
        };

        let mut i = wiener_start;
        while i + 16 <= wiener_end {
            // Load 16 u16 pixels for each tap
            let m0_words = _mm256_loadu_si256((src.add(offset + i - 2)) as *const __m256i);
            let m1_words = _mm256_loadu_si256((src.add(offset + i - 1)) as *const __m256i);
            let m2_words = _mm256_loadu_si256((src.add(offset + i)) as *const __m256i);
            let m3_words = _mm256_loadu_si256((src.add(offset + i + 1)) as *const __m256i);
            let m4_words = _mm256_loadu_si256((src.add(offset + i + 2)) as *const __m256i);
            let m5_words = _mm256_loadu_si256((src.add(offset + i + 3)) as *const __m256i);

            // Process first 8 pixels
            let m0_lo = _mm256_unpacklo_epi16(m0_words, _mm256_setzero_si256());
            let m1_lo = _mm256_unpacklo_epi16(m1_words, _mm256_setzero_si256());
            let m2_lo = _mm256_unpacklo_epi16(m2_words, _mm256_setzero_si256());
            let m3_lo = _mm256_unpacklo_epi16(m3_words, _mm256_setzero_si256());
            let m4_lo = _mm256_unpacklo_epi16(m4_words, _mm256_setzero_si256());
            let m5_lo = _mm256_unpacklo_epi16(m5_words, _mm256_setzero_si256());

            let result_lo = apply_wiener_kernel_u16(
                m0_lo, m1_lo, m2_lo, m3_lo, m4_lo, m5_lo, four, five, sixteen, pixel_max,
            );

            // Process next 8 pixels
            let m0_hi = _mm256_unpackhi_epi16(m0_words, _mm256_setzero_si256());
            let m1_hi = _mm256_unpackhi_epi16(m1_words, _mm256_setzero_si256());
            let m2_hi = _mm256_unpackhi_epi16(m2_words, _mm256_setzero_si256());
            let m3_hi = _mm256_unpackhi_epi16(m3_words, _mm256_setzero_si256());
            let m4_hi = _mm256_unpackhi_epi16(m4_words, _mm256_setzero_si256());
            let m5_hi = _mm256_unpackhi_epi16(m5_words, _mm256_setzero_si256());

            let result_hi = apply_wiener_kernel_u16(
                m0_hi, m1_hi, m2_hi, m3_hi, m4_hi, m5_hi, four, five, sixteen, pixel_max,
            );

            // Pack results back to u16
            let result = _mm256_packus_epi32(result_lo, result_hi);
            _mm256_storeu_si256((dest.add(offset + i)) as *mut __m256i, result);

            i += 16;
        }

        // Handle remaining pixels with scalar code
        while i < wiener_end {
            let m0 = *src.add(offset + i - 2) as i32;
            let m1 = *src.add(offset + i - 1) as i32;
            let mut m2 = *src.add(offset + i) as i32;
            let m3 = *src.add(offset + i + 1) as i32;
            let m4 = *src.add(offset + i + 2) as i32;
            let m5 = *src.add(offset + i + 3) as i32;

            m2 = (m2 + m3) * 4;
            m2 -= m1 + m4;
            m2 *= 5;
            let result = (m0 + m5 + m2 + 16) >> 5;

            *dest.add(offset + i) = result.max(0).min((1 << bits_per_sample.get()) - 1) as u16;
            i += 1;
        }

        // Handle last few pixels with bilinear interpolation
        for i in wiener_end..(width.get() - 1).min(width.get()) {
            let a = *src.add(offset + i) as u32;
            let b = *src.add(offset + i + 1) as u32;
            *dest.add(offset + i) = ((a + b + 1) / 2) as u16;
        }

        // Copy last pixel
        if width.get() > 0 {
            *dest.add(offset + width.get() - 1) = *src.add(offset + width.get() - 1);
        }

        offset += pitch.get();
    }
}

#[target_feature(enable = "avx2")]
unsafe fn refine_vertical_wiener_u8(
    src: *const u8,
    dest: *mut u8,
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    let pixel_max = _mm256_set1_epi16(((1i32 << bits_per_sample.get()) - 1) as i16);
    let zero = _mm256_setzero_si256();
    let four = _mm256_set1_epi16(4);
    let five = _mm256_set1_epi16(5);
    let sixteen = _mm256_set1_epi16(16);

    let mut offset = 0;

    // Handle first two rows with bilinear interpolation
    for _j in 0..2.min(height.get() - 1) {
        let mut i = 0;
        while i + 32 <= width.get() {
            let a_bytes = _mm256_loadu_si256((src.add(offset + i)) as *const __m256i);
            let b_bytes = _mm256_loadu_si256((src.add(offset + i + pitch.get())) as *const __m256i);

            let a_lo = _mm256_unpacklo_epi8(a_bytes, zero);
            let b_lo = _mm256_unpacklo_epi8(b_bytes, zero);
            let a_hi = _mm256_unpackhi_epi8(a_bytes, zero);
            let b_hi = _mm256_unpackhi_epi8(b_bytes, zero);

            let sum_lo = _mm256_add_epi16(_mm256_add_epi16(a_lo, b_lo), _mm256_set1_epi16(1));
            let sum_hi = _mm256_add_epi16(_mm256_add_epi16(a_hi, b_hi), _mm256_set1_epi16(1));
            let avg_lo = _mm256_srli_epi16(sum_lo, 1);
            let avg_hi = _mm256_srli_epi16(sum_hi, 1);

            let result = _mm256_packus_epi16(avg_lo, avg_hi);
            _mm256_storeu_si256((dest.add(offset + i)) as *mut __m256i, result);

            i += 32;
        }

        // Handle remaining pixels
        while i < width.get() {
            let a = *src.add(offset + i) as u16;
            let b = *src.add(offset + i + pitch.get()) as u16;
            *dest.add(offset + i) = ((a + b + 1) / 2) as u8;
            i += 1;
        }

        offset += pitch.get();
    }

    // Process middle rows with Wiener filter
    for _j in 2..(height.get() - 4).max(2) {
        let mut i = 0;
        while i + 32 <= width.get() {
            let m0_bytes =
                _mm256_loadu_si256((src.add(offset + i - pitch.get() * 2)) as *const __m256i);
            let m1_bytes =
                _mm256_loadu_si256((src.add(offset + i - pitch.get())) as *const __m256i);
            let m2_bytes = _mm256_loadu_si256((src.add(offset + i)) as *const __m256i);
            let m3_bytes =
                _mm256_loadu_si256((src.add(offset + i + pitch.get())) as *const __m256i);
            let m4_bytes =
                _mm256_loadu_si256((src.add(offset + i + pitch.get() * 2)) as *const __m256i);
            let m5_bytes =
                _mm256_loadu_si256((src.add(offset + i + pitch.get() * 3)) as *const __m256i);

            // Process first 16 pixels
            let m0_lo = _mm256_unpacklo_epi8(m0_bytes, zero);
            let m1_lo = _mm256_unpacklo_epi8(m1_bytes, zero);
            let m2_lo = _mm256_unpacklo_epi8(m2_bytes, zero);
            let m3_lo = _mm256_unpacklo_epi8(m3_bytes, zero);
            let m4_lo = _mm256_unpacklo_epi8(m4_bytes, zero);
            let m5_lo = _mm256_unpacklo_epi8(m5_bytes, zero);

            let result_lo = apply_wiener_kernel_u8(
                m0_lo, m1_lo, m2_lo, m3_lo, m4_lo, m5_lo, four, five, sixteen, pixel_max,
            );

            // Process next 16 pixels
            let m0_hi = _mm256_unpackhi_epi8(m0_bytes, zero);
            let m1_hi = _mm256_unpackhi_epi8(m1_bytes, zero);
            let m2_hi = _mm256_unpackhi_epi8(m2_bytes, zero);
            let m3_hi = _mm256_unpackhi_epi8(m3_bytes, zero);
            let m4_hi = _mm256_unpackhi_epi8(m4_bytes, zero);
            let m5_hi = _mm256_unpackhi_epi8(m5_bytes, zero);

            let result_hi = apply_wiener_kernel_u8(
                m0_hi, m1_hi, m2_hi, m3_hi, m4_hi, m5_hi, four, five, sixteen, pixel_max,
            );

            let result = _mm256_packus_epi16(result_lo, result_hi);
            _mm256_storeu_si256((dest.add(offset + i)) as *mut __m256i, result);

            i += 32;
        }

        // Handle remaining pixels
        while i < width.get() {
            let m0 = *src.add(offset + i - pitch.get() * 2) as i16;
            let m1 = *src.add(offset + i - pitch.get()) as i16;
            let mut m2 = *src.add(offset + i) as i16;
            let m3 = *src.add(offset + i + pitch.get()) as i16;
            let m4 = *src.add(offset + i + pitch.get() * 2) as i16;
            let m5 = *src.add(offset + i + pitch.get() * 3) as i16;

            m2 = (m2 + m3) * 4;
            m2 -= m1 + m4;
            m2 *= 5;
            let result = (m0 + m5 + m2 + 16) >> 5;

            *dest.add(offset + i) = result.max(0).min((1 << bits_per_sample.get()) - 1) as u8;
            i += 1;
        }

        offset += pitch.get();
    }

    // Handle last few rows with bilinear interpolation
    for _j in (height.get() - 4).max(2)..(height.get() - 1) {
        let mut i = 0;
        while i + 32 <= width.get() {
            let a_bytes = _mm256_loadu_si256((src.add(offset + i)) as *const __m256i);
            let b_bytes = _mm256_loadu_si256((src.add(offset + i + pitch.get())) as *const __m256i);

            let a_lo = _mm256_unpacklo_epi8(a_bytes, zero);
            let b_lo = _mm256_unpacklo_epi8(b_bytes, zero);
            let a_hi = _mm256_unpackhi_epi8(a_bytes, zero);
            let b_hi = _mm256_unpackhi_epi8(b_bytes, zero);

            let sum_lo = _mm256_add_epi16(_mm256_add_epi16(a_lo, b_lo), _mm256_set1_epi16(1));
            let sum_hi = _mm256_add_epi16(_mm256_add_epi16(a_hi, b_hi), _mm256_set1_epi16(1));
            let avg_lo = _mm256_srli_epi16(sum_lo, 1);
            let avg_hi = _mm256_srli_epi16(sum_hi, 1);

            let result = _mm256_packus_epi16(avg_lo, avg_hi);
            _mm256_storeu_si256((dest.add(offset + i)) as *mut __m256i, result);

            i += 32;
        }

        while i < width.get() {
            let a = *src.add(offset + i) as u16;
            let b = *src.add(offset + i + pitch.get()) as u16;
            *dest.add(offset + i) = ((a + b + 1) / 2) as u8;
            i += 1;
        }

        offset += pitch.get();
    }

    // Copy last row
    if height.get() > 0 {
        std::ptr::copy_nonoverlapping(src.add(offset), dest.add(offset), width.get());
    }
}

#[target_feature(enable = "avx2")]
unsafe fn refine_vertical_wiener_u16(
    src: *const u16,
    dest: *mut u16,
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    let pixel_max = _mm256_set1_epi32((1i32 << bits_per_sample.get()) - 1);
    let four = _mm256_set1_epi32(4);
    let five = _mm256_set1_epi32(5);
    let sixteen = _mm256_set1_epi32(16);

    let mut offset = 0;

    // Handle first two rows with bilinear interpolation
    for _j in 0..2.min(height.get() - 1) {
        let mut i = 0;
        while i + 16 <= width.get() {
            let a_words = _mm256_loadu_si256((src.add(offset + i)) as *const __m256i);
            let b_words = _mm256_loadu_si256((src.add(offset + i + pitch.get())) as *const __m256i);

            let a_lo = _mm256_unpacklo_epi16(a_words, _mm256_setzero_si256());
            let b_lo = _mm256_unpacklo_epi16(b_words, _mm256_setzero_si256());
            let a_hi = _mm256_unpackhi_epi16(a_words, _mm256_setzero_si256());
            let b_hi = _mm256_unpackhi_epi16(b_words, _mm256_setzero_si256());

            let sum_lo = _mm256_add_epi32(_mm256_add_epi32(a_lo, b_lo), _mm256_set1_epi32(1));
            let sum_hi = _mm256_add_epi32(_mm256_add_epi32(a_hi, b_hi), _mm256_set1_epi32(1));
            let avg_lo = _mm256_srli_epi32(sum_lo, 1);
            let avg_hi = _mm256_srli_epi32(sum_hi, 1);

            let result = _mm256_packus_epi32(avg_lo, avg_hi);
            _mm256_storeu_si256((dest.add(offset + i)) as *mut __m256i, result);

            i += 16;
        }

        while i < width.get() {
            let a = *src.add(offset + i) as u32;
            let b = *src.add(offset + i + pitch.get()) as u32;
            *dest.add(offset + i) = ((a + b + 1) / 2) as u16;
            i += 1;
        }

        offset += pitch.get();
    }

    // Process middle rows with Wiener filter
    for _j in 2..(height.get() - 4).max(2) {
        let mut i = 0;
        while i + 16 <= width.get() {
            let m0_words =
                _mm256_loadu_si256((src.add(offset + i - pitch.get() * 2)) as *const __m256i);
            let m1_words =
                _mm256_loadu_si256((src.add(offset + i - pitch.get())) as *const __m256i);
            let m2_words = _mm256_loadu_si256((src.add(offset + i)) as *const __m256i);
            let m3_words =
                _mm256_loadu_si256((src.add(offset + i + pitch.get())) as *const __m256i);
            let m4_words =
                _mm256_loadu_si256((src.add(offset + i + pitch.get() * 2)) as *const __m256i);
            let m5_words =
                _mm256_loadu_si256((src.add(offset + i + pitch.get() * 3)) as *const __m256i);

            // Process first 8 pixels
            let m0_lo = _mm256_unpacklo_epi16(m0_words, _mm256_setzero_si256());
            let m1_lo = _mm256_unpacklo_epi16(m1_words, _mm256_setzero_si256());
            let m2_lo = _mm256_unpacklo_epi16(m2_words, _mm256_setzero_si256());
            let m3_lo = _mm256_unpacklo_epi16(m3_words, _mm256_setzero_si256());
            let m4_lo = _mm256_unpacklo_epi16(m4_words, _mm256_setzero_si256());
            let m5_lo = _mm256_unpacklo_epi16(m5_words, _mm256_setzero_si256());

            let result_lo = apply_wiener_kernel_u16(
                m0_lo, m1_lo, m2_lo, m3_lo, m4_lo, m5_lo, four, five, sixteen, pixel_max,
            );

            // Process next 8 pixels
            let m0_hi = _mm256_unpackhi_epi16(m0_words, _mm256_setzero_si256());
            let m1_hi = _mm256_unpackhi_epi16(m1_words, _mm256_setzero_si256());
            let m2_hi = _mm256_unpackhi_epi16(m2_words, _mm256_setzero_si256());
            let m3_hi = _mm256_unpackhi_epi16(m3_words, _mm256_setzero_si256());
            let m4_hi = _mm256_unpackhi_epi16(m4_words, _mm256_setzero_si256());
            let m5_hi = _mm256_unpackhi_epi16(m5_words, _mm256_setzero_si256());

            let result_hi = apply_wiener_kernel_u16(
                m0_hi, m1_hi, m2_hi, m3_hi, m4_hi, m5_hi, four, five, sixteen, pixel_max,
            );

            let result = _mm256_packus_epi32(result_lo, result_hi);
            _mm256_storeu_si256((dest.add(offset + i)) as *mut __m256i, result);

            i += 16;
        }

        while i < width.get() {
            let m0 = *src.add(offset + i - pitch.get() * 2) as i32;
            let m1 = *src.add(offset + i - pitch.get()) as i32;
            let mut m2 = *src.add(offset + i) as i32;
            let m3 = *src.add(offset + i + pitch.get()) as i32;
            let m4 = *src.add(offset + i + pitch.get() * 2) as i32;
            let m5 = *src.add(offset + i + pitch.get() * 3) as i32;

            m2 = (m2 + m3) * 4;
            m2 -= m1 + m4;
            m2 *= 5;
            let result = (m0 + m5 + m2 + 16) >> 5;

            *dest.add(offset + i) = result.max(0).min((1 << bits_per_sample.get()) - 1) as u16;
            i += 1;
        }

        offset += pitch.get();
    }

    // Handle last few rows with bilinear interpolation
    for _j in (height.get() - 4).max(2)..(height.get() - 1) {
        let mut i = 0;
        while i + 16 <= width.get() {
            let a_words = _mm256_loadu_si256((src.add(offset + i)) as *const __m256i);
            let b_words = _mm256_loadu_si256((src.add(offset + i + pitch.get())) as *const __m256i);

            let a_lo = _mm256_unpacklo_epi16(a_words, _mm256_setzero_si256());
            let b_lo = _mm256_unpacklo_epi16(b_words, _mm256_setzero_si256());
            let a_hi = _mm256_unpackhi_epi16(a_words, _mm256_setzero_si256());
            let b_hi = _mm256_unpackhi_epi16(b_words, _mm256_setzero_si256());

            let sum_lo = _mm256_add_epi32(_mm256_add_epi32(a_lo, b_lo), _mm256_set1_epi32(1));
            let sum_hi = _mm256_add_epi32(_mm256_add_epi32(a_hi, b_hi), _mm256_set1_epi32(1));
            let avg_lo = _mm256_srli_epi32(sum_lo, 1);
            let avg_hi = _mm256_srli_epi32(sum_hi, 1);

            let result = _mm256_packus_epi32(avg_lo, avg_hi);
            _mm256_storeu_si256((dest.add(offset + i)) as *mut __m256i, result);

            i += 16;
        }

        while i < width.get() {
            let a = *src.add(offset + i) as u32;
            let b = *src.add(offset + i + pitch.get()) as u32;
            *dest.add(offset + i) = ((a + b + 1) / 2) as u16;
            i += 1;
        }

        offset += pitch.get();
    }

    // Copy last row
    if height.get() > 0 {
        std::ptr::copy_nonoverlapping(src.add(offset), dest.add(offset), width.get());
    }
}

// Helper function to apply Wiener kernel for u8 (working with i16 intermediates)
#[target_feature(enable = "avx2")]
unsafe fn apply_wiener_kernel_u8(
    m0: __m256i,
    m1: __m256i,
    m2: __m256i,
    m3: __m256i,
    m4: __m256i,
    m5: __m256i,
    four: __m256i,
    five: __m256i,
    sixteen: __m256i,
    pixel_max: __m256i,
) -> __m256i {
    // m2 = (m2 + m3) * 4
    let sum23 = _mm256_add_epi16(m2, m3);
    let mut temp = _mm256_mullo_epi16(sum23, four);

    // m2 -= m1 + m4
    let sum14 = _mm256_add_epi16(m1, m4);
    temp = _mm256_sub_epi16(temp, sum14);

    // m2 *= 5
    temp = _mm256_mullo_epi16(temp, five);

    // m0 += m5 + m2 + 16
    let sum05 = _mm256_add_epi16(m0, m5);
    let sum = _mm256_add_epi16(_mm256_add_epi16(sum05, temp), sixteen);

    // >>= 5
    let result = _mm256_srai_epi16(sum, 5);

    // Clamp to [0, pixel_max]
    let zero = _mm256_setzero_si256();

    _mm256_max_epi16(zero, _mm256_min_epi16(result, pixel_max))
}

// Helper function to apply Wiener kernel for u16 (working with i32 intermediates)
#[target_feature(enable = "avx2")]
unsafe fn apply_wiener_kernel_u16(
    m0: __m256i,
    m1: __m256i,
    m2: __m256i,
    m3: __m256i,
    m4: __m256i,
    m5: __m256i,
    four: __m256i,
    five: __m256i,
    sixteen: __m256i,
    pixel_max: __m256i,
) -> __m256i {
    // m2 = (m2 + m3) * 4
    let sum23 = _mm256_add_epi32(m2, m3);
    let mut temp = _mm256_mullo_epi32(sum23, four);

    // m2 -= m1 + m4
    let sum14 = _mm256_add_epi32(m1, m4);
    temp = _mm256_sub_epi32(temp, sum14);

    // m2 *= 5
    temp = _mm256_mullo_epi32(temp, five);

    // m0 += m5 + m2 + 16
    let sum05 = _mm256_add_epi32(m0, m5);
    let sum = _mm256_add_epi32(_mm256_add_epi32(sum05, temp), sixteen);

    // >>= 5
    let result = _mm256_srai_epi32(sum, 5);

    // Clamp to [0, pixel_max]
    let zero = _mm256_setzero_si256();

    _mm256_max_epi32(zero, _mm256_min_epi32(result, pixel_max))
}
