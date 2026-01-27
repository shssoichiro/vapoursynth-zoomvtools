#![allow(clippy::undocumented_unsafe_blocks)]
#![allow(unsafe_op_in_unsafe_fn)]

use std::{arch::x86_64::*, num::NonZeroUsize};

use crate::util::Pixel;

#[must_use]
#[target_feature(enable = "avx2")]
pub unsafe fn luma_sum<T: Pixel>(
    width: NonZeroUsize,
    height: NonZeroUsize,
    src: &[T],
    src_pitch: NonZeroUsize,
) -> u64 {
    match size_of::<T>() {
        1 => match (width.get(), height.get()) {
            (4, 4) => luma_sum_u8::<4, 4>(src.as_ptr().cast(), src_pitch),
            (8, 4) => luma_sum_u8::<8, 4>(src.as_ptr().cast(), src_pitch),
            (8, 8) => luma_sum_u8::<8, 8>(src.as_ptr().cast(), src_pitch),
            (16, 2) => luma_sum_u8::<16, 2>(src.as_ptr().cast(), src_pitch),
            (16, 8) => luma_sum_u8::<16, 8>(src.as_ptr().cast(), src_pitch),
            (16, 16) => luma_sum_u8::<16, 16>(src.as_ptr().cast(), src_pitch),
            (32, 16) => luma_sum_u8::<32, 16>(src.as_ptr().cast(), src_pitch),
            (32, 32) => luma_sum_u8::<32, 32>(src.as_ptr().cast(), src_pitch),
            (64, 32) => luma_sum_u8::<64, 32>(src.as_ptr().cast(), src_pitch),
            (64, 64) => luma_sum_u8::<64, 64>(src.as_ptr().cast(), src_pitch),
            (128, 64) => luma_sum_u8::<128, 64>(src.as_ptr().cast(), src_pitch),
            (128, 128) => luma_sum_u8::<128, 128>(src.as_ptr().cast(), src_pitch),
            _ => unreachable!("unsupported block size"),
        },
        2 => match (width.get(), height.get()) {
            (4, 4) => luma_sum_u16::<4, 4>(src.as_ptr().cast(), src_pitch),
            (8, 4) => luma_sum_u16::<8, 4>(src.as_ptr().cast(), src_pitch),
            (8, 8) => luma_sum_u16::<8, 8>(src.as_ptr().cast(), src_pitch),
            (16, 2) => luma_sum_u16::<16, 2>(src.as_ptr().cast(), src_pitch),
            (16, 8) => luma_sum_u16::<16, 8>(src.as_ptr().cast(), src_pitch),
            (16, 16) => luma_sum_u16::<16, 16>(src.as_ptr().cast(), src_pitch),
            (32, 16) => luma_sum_u16::<32, 16>(src.as_ptr().cast(), src_pitch),
            (32, 32) => luma_sum_u16::<32, 32>(src.as_ptr().cast(), src_pitch),
            (64, 32) => luma_sum_u16::<64, 32>(src.as_ptr().cast(), src_pitch),
            (64, 64) => luma_sum_u16::<64, 64>(src.as_ptr().cast(), src_pitch),
            (128, 64) => luma_sum_u16::<128, 64>(src.as_ptr().cast(), src_pitch),
            (128, 128) => luma_sum_u16::<128, 128>(src.as_ptr().cast(), src_pitch),
            _ => unreachable!("unsupported block size"),
        },
        _ => unreachable!(),
    }
}

#[must_use]
#[target_feature(enable = "avx2")]
pub unsafe fn luma_sum_u8<const WIDTH: usize, const HEIGHT: usize>(
    src: *const u8,
    src_pitch: NonZeroUsize,
) -> u64 {
    let src_pitch = src_pitch.get();

    let zero256 = _mm256_setzero_si256();
    let zero128 = _mm_setzero_si128();
    let mut acc256 = _mm256_setzero_si256();
    let mut acc128 = _mm_setzero_si128();

    for j in 0..HEIGHT {
        let row = src.add(j * src_pitch);
        let mut i = 0;

        while i + 32 <= WIDTH {
            let data = _mm256_loadu_si256(row.add(i) as *const __m256i);
            acc256 = _mm256_add_epi64(acc256, _mm256_sad_epu8(data, zero256));
            i += 32;
        }

        if i + 16 <= WIDTH {
            let data = _mm_loadu_si128(row.add(i) as *const __m128i);
            acc128 = _mm_add_epi64(acc128, _mm_sad_epu8(data, zero128));
            i += 16;
        }

        if i + 8 <= WIDTH {
            let data = _mm_loadl_epi64(row.add(i) as *const __m128i);
            acc128 = _mm_add_epi64(acc128, _mm_sad_epu8(data, zero128));
            i += 8;
        }

        if i + 4 <= WIDTH {
            let four_bytes = (row.add(i) as *const u32).read_unaligned();
            let data = _mm_cvtsi32_si128(four_bytes as i32);
            acc128 = _mm_add_epi64(acc128, _mm_sad_epu8(data, zero128));
        }
    }

    // Reduce acc256: add high and low 128-bit lanes
    let acc256_lo = _mm256_castsi256_si128(acc256);
    let acc256_hi = _mm256_extracti128_si256(acc256, 1);
    let combined = _mm_add_epi64(_mm_add_epi64(acc256_lo, acc256_hi), acc128);

    // Reduce 2x u64 lanes to scalar
    let high = _mm_unpackhi_epi64(combined, combined);
    let total = _mm_add_epi64(combined, high);
    _mm_cvtsi128_si64(total) as u64
}

#[must_use]
#[target_feature(enable = "avx2")]
pub unsafe fn luma_sum_u16<const WIDTH: usize, const HEIGHT: usize>(
    src: *const u16,
    src_pitch: NonZeroUsize,
) -> u64 {
    let src_pitch = src_pitch.get();

    let zero256 = _mm256_setzero_si256();
    let zero128 = _mm_setzero_si128();
    let mut acc256 = _mm256_setzero_si256();
    let mut acc128 = _mm_setzero_si128();

    for j in 0..HEIGHT {
        let row = src.add(j * src_pitch);
        let mut i = 0;

        while i + 16 <= WIDTH {
            let data = _mm256_loadu_si256(row.add(i) as *const __m256i);
            let lo = _mm256_unpacklo_epi16(data, zero256);
            let hi = _mm256_unpackhi_epi16(data, zero256);
            acc256 = _mm256_add_epi32(acc256, lo);
            acc256 = _mm256_add_epi32(acc256, hi);
            i += 16;
        }

        if i + 8 <= WIDTH {
            let data = _mm_loadu_si128(row.add(i) as *const __m128i);
            let lo = _mm_unpacklo_epi16(data, zero128);
            let hi = _mm_unpackhi_epi16(data, zero128);
            acc128 = _mm_add_epi32(acc128, lo);
            acc128 = _mm_add_epi32(acc128, hi);
            i += 8;
        }

        if i + 4 <= WIDTH {
            let data = _mm_loadl_epi64(row.add(i) as *const __m128i);
            let widened = _mm_unpacklo_epi16(data, zero128);
            acc128 = _mm_add_epi32(acc128, widened);
        }
    }

    // Reduce acc256 (8 × u32): add high and low 128-bit lanes, combine with acc128
    let acc256_lo = _mm256_castsi256_si128(acc256);
    let acc256_hi = _mm256_extracti128_si256(acc256, 1);
    let combined = _mm_add_epi32(_mm_add_epi32(acc256_lo, acc256_hi), acc128);

    // Horizontal sum of 4 × u32 to scalar
    let sum2 = _mm_add_epi32(combined, _mm_shuffle_epi32(combined, 0x4e));
    let sum4 = _mm_add_epi32(sum2, _mm_shuffle_epi32(sum2, 0xb1));
    _mm_cvtsi128_si32(sum4) as u32 as u64
}
