use std::{
    mem::transmute,
    num::NonZeroUsize,
    ops::{Add, Sub},
};

use crate::util::Pixel;

#[must_use]
pub fn get_satd<T: Pixel>(
    width: NonZeroUsize,
    height: NonZeroUsize,
    src: &[T],
    src_pitch: NonZeroUsize,
    ref_: &[T],
    ref_pitch: NonZeroUsize,
) -> u64 {
    match (width.get(), height.get()) {
        (4, 4) => get_satd_impl::<T, 4, 4>(src, src_pitch, ref_, ref_pitch),
        (8, 4) => get_satd_impl::<T, 8, 4>(src, src_pitch, ref_, ref_pitch),
        (8, 8) => get_satd_impl::<T, 8, 8>(src, src_pitch, ref_, ref_pitch),
        (16, 8) => get_satd_impl::<T, 16, 8>(src, src_pitch, ref_, ref_pitch),
        (16, 16) => get_satd_impl::<T, 16, 16>(src, src_pitch, ref_, ref_pitch),
        (32, 16) => get_satd_impl::<T, 32, 16>(src, src_pitch, ref_, ref_pitch),
        (32, 32) => get_satd_impl::<T, 32, 32>(src, src_pitch, ref_, ref_pitch),
        (64, 32) => get_satd_impl::<T, 64, 32>(src, src_pitch, ref_, ref_pitch),
        (64, 64) => get_satd_impl::<T, 64, 64>(src, src_pitch, ref_, ref_pitch),
        (128, 64) => get_satd_impl::<T, 128, 64>(src, src_pitch, ref_, ref_pitch),
        (128, 128) => get_satd_impl::<T, 128, 128>(src, src_pitch, ref_, ref_pitch),
        _ => unimplemented!("Invalid block size for SAD"),
    }
}

#[must_use]
fn get_satd_impl<T: Pixel, const WIDTH: usize, const HEIGHT: usize>(
    src: &[T],
    src_pitch: NonZeroUsize,
    ref_: &[T],
    ref_pitch: NonZeroUsize,
) -> u64 {
    if WIDTH == 4 && HEIGHT == 4 {
        return if size_of::<T>() == 2 {
            // SAFETY: We checked the size of T
            unsafe { satd_4x4_16b(transmute(src), src_pitch, transmute(ref_), ref_pitch) }
        } else {
            unsafe { satd_4x4_8b(transmute(src), src_pitch, transmute(ref_), ref_pitch) }
        };
    }

    let partition_width = 8;
    let partition_height = 4;

    let mut sum = 0;
    for y in (0..HEIGHT).step_by(partition_height) {
        for x in (0..WIDTH).step_by(partition_width) {
            sum += if size_of::<T>() == 2 {
                // SAFETY: We checked the size of T
                unsafe {
                    satd_8x4_16b(
                        transmute(&src[y * src_pitch.get() + x..]),
                        src_pitch,
                        transmute(&ref_[y * ref_pitch.get() + x..]),
                        ref_pitch,
                    )
                }
            } else {
                unsafe {
                    satd_8x4_8b(
                        transmute(&src[y * src_pitch.get() + x..]),
                        src_pitch,
                        transmute(&ref_[y * ref_pitch.get() + x..]),
                        ref_pitch,
                    )
                }
            };
        }
    }
    sum
}

#[must_use]
fn satd_4x4_8b(src: &[u8], src_pitch: NonZeroUsize, ref_: &[u8], ref_pitch: NonZeroUsize) -> u64 {
    todo!()
}

#[must_use]
fn satd_4x4_16b(
    src: &[u16],
    src_pitch: NonZeroUsize,
    ref_: &[u16],
    ref_pitch: NonZeroUsize,
) -> u64 {
    todo!()
}

#[must_use]
fn satd_8x4_8b(src: &[u8], src_pitch: NonZeroUsize, ref_: &[u8], ref_pitch: NonZeroUsize) -> u64 {
    const BITS_PER_SUM: usize = 16;
    let mut tmp = [[0u32; 4]; 4];
    let mut a = (0u32, 0u32, 0u32, 0u32);
    let mut sum = 0u32;

    for i in 0..4 {
        let src_offset = i * src_pitch.get();
        let ref_offset = i * ref_pitch.get();

        let src_row = &src[src_offset..src_offset + 8];
        let ref_row = &ref_[ref_offset..ref_offset + 8];

        let diff = |idx: usize| -> i32 { i32::from(src_row[idx]) - i32::from(ref_row[idx]) };

        a.0 = (diff(0) as u32).wrapping_add((diff(4) as u32) << BITS_PER_SUM);
        a.1 = (diff(1) as u32).wrapping_add((diff(5) as u32) << BITS_PER_SUM);
        a.2 = (diff(2) as u32).wrapping_add((diff(6) as u32) << BITS_PER_SUM);
        a.3 = (diff(3) as u32).wrapping_add((diff(7) as u32) << BITS_PER_SUM);
        let tmp_row = &mut tmp[i];
        let [ref mut d0, ref mut d1, ref mut d2, ref mut d3] = *tmp_row;
        hadamard4(d0, d1, d2, d3, a.0, a.1, a.2, a.3);
    }

    for i in 0..4 {
        hadamard4(
            &mut a.0, &mut a.1, &mut a.2, &mut a.3, tmp[0][i], tmp[1][i], tmp[2][i], tmp[3][i],
        );
        sum += abs2_8b(a.0) + abs2_8b(a.1) + abs2_8b(a.2) + abs2_8b(a.3);
    }

    (((sum as u16 as u32) + (sum >> BITS_PER_SUM)) >> 1) as u64
}

#[must_use]
fn satd_8x4_16b(
    src: &[u16],
    src_pitch: NonZeroUsize,
    ref_: &[u16],
    ref_pitch: NonZeroUsize,
) -> u64 {
    const BITS_PER_SUM: usize = 32;
    let mut tmp = [[0u64; 4]; 4];
    let mut a = (0u64, 0u64, 0u64, 0u64);
    let mut sum = 0u64;

    for i in 0..4 {
        let src_offset = i * src_pitch.get();
        let ref_offset = i * ref_pitch.get();

        let src_row = &src[src_offset..src_offset + 8];
        let ref_row = &ref_[ref_offset..ref_offset + 8];

        let diff = |idx: usize| -> i64 { i64::from(src_row[idx]) - i64::from(ref_row[idx]) };

        a.0 = (diff(0) as u64).wrapping_add((diff(4) as u64) << BITS_PER_SUM);
        a.1 = (diff(1) as u64).wrapping_add((diff(5) as u64) << BITS_PER_SUM);
        a.2 = (diff(2) as u64).wrapping_add((diff(6) as u64) << BITS_PER_SUM);
        a.3 = (diff(3) as u64).wrapping_add((diff(7) as u64) << BITS_PER_SUM);
        let tmp_row = &mut tmp[i];
        let [ref mut d0, ref mut d1, ref mut d2, ref mut d3] = *tmp_row;
        hadamard4(d0, d1, d2, d3, a.0, a.1, a.2, a.3);
    }

    for i in 0..4 {
        hadamard4(
            &mut a.0, &mut a.1, &mut a.2, &mut a.3, tmp[0][i], tmp[1][i], tmp[2][i], tmp[3][i],
        );
        sum += abs2_16b(a.0) + abs2_16b(a.1) + abs2_16b(a.2) + abs2_16b(a.3);
    }

    (((sum as u32 as u64) + (sum >> BITS_PER_SUM)) >> 1) as u64
}

/// in: a pseudo-simd number of the form x+(y<<16)
/// return: abs(x)+(abs(y)<<16)
#[must_use]
#[inline(always)]
fn abs2_8b(a: u32) -> u32 {
    const BITS_PER_SUM: usize = 16;

    let s: u32 = ((a >> (BITS_PER_SUM - 1)) & ((1u32 << BITS_PER_SUM) + 1)) * (-1i16 as u16 as u32);
    return a.wrapping_add(s) ^ s;
}

/// in: a pseudo-simd number of the form x+(y<<16)
/// return: abs(x)+(abs(y)<<16)
#[must_use]
#[inline(always)]
fn abs2_16b(a: u64) -> u64 {
    const BITS_PER_SUM: usize = 32;

    let s: u64 = ((a >> (BITS_PER_SUM - 1)) & ((1u64 << BITS_PER_SUM) + 1)) * (-1i32 as u32 as u64);
    return a.wrapping_add(s) ^ s;
}

#[inline(always)]
fn hadamard4<T: Copy + Add<T, Output = T> + Sub<T, Output = T>>(
    dest0: &mut T,
    dest1: &mut T,
    dest2: &mut T,
    dest3: &mut T,
    src0: T,
    src1: T,
    src2: T,
    src3: T,
) {
    let temp0: T = src0 + src1;
    let temp1: T = src0 - src1;
    let temp2: T = src2 + src3;
    let temp3: T = src2 - src3;
    *dest0 = temp0 + temp2;
    *dest2 = temp0 - temp2;
    *dest1 = temp1 + temp3;
    *dest3 = temp1 - temp3;
}
