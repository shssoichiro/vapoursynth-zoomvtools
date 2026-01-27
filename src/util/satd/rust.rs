use std::{
    mem::size_of,
    num::NonZeroUsize,
    ops::{Add, AddAssign, BitAnd, BitXor, Mul, Shl, Shr, Sub},
};

use num_traits::{One, PrimInt, WrappingAdd};

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
        // perf: branch is elided via generics at compile time
        return match size_of::<T>() {
            1 => satd_4x4::<T, u16, u32>(src, src_pitch, ref_, ref_pitch),
            2 => satd_4x4::<T, u32, u64>(src, src_pitch, ref_, ref_pitch),
            _ => unreachable!(),
        };
    }

    let partition_width = 8;
    let partition_height = 4;

    let mut sum = 0;
    for y in (0..HEIGHT).step_by(partition_height) {
        for x in (0..WIDTH).step_by(partition_width) {
            // perf: branch is elided via generics at compile time
            sum += match size_of::<T>() {
                1 => satd_8x4::<T, u16, u32>(
                    &src[y * src_pitch.get() + x..],
                    src_pitch,
                    &ref_[y * ref_pitch.get() + x..],
                    ref_pitch,
                ),
                2 => satd_8x4::<T, u32, u64>(
                    &src[y * src_pitch.get() + x..],
                    src_pitch,
                    &ref_[y * ref_pitch.get() + x..],
                    ref_pitch,
                ),
                _ => unreachable!(),
            };
        }
    }
    sum
}

#[must_use]
fn satd_4x4<
    T: Pixel,
    SUM1: PrimInt + Default,
    SUM2: PrimInt + Default + One + AddAssign<SUM2> + WrappingAdd + FromDiff + Into<u64>,
>(
    src: &[T],
    src_pitch: NonZeroUsize,
    ref_: &[T],
    ref_pitch: NonZeroUsize,
) -> u64 {
    let bits_per_sum = size_of::<SUM1>() * 8;
    let mut tmp: [[SUM2; 2]; 4] = Default::default();
    let mut a: [SUM2; 4] = Default::default();
    let mut b: [SUM2; 2] = Default::default();
    let mut sum: SUM2 = Default::default();

    for i in 0..4 {
        let src_offset = i * src_pitch.get();
        let ref_offset = i * ref_pitch.get();

        let src_row = &src[src_offset..src_offset + 4];
        let ref_row = &ref_[ref_offset..ref_offset + 4];

        let diff = |idx: usize| -> SUM2 {
            let s: i32 = src_row[idx].to_i32().expect("fits in i32");
            let r: i32 = ref_row[idx].to_i32().expect("fits in i32");
            SUM2::from_diff(s - r)
        };

        a[0] = diff(0);
        a[1] = diff(1);
        b[0] = (a[0] + a[1]) + ((a[0] - a[1]) << bits_per_sum);
        a[2] = diff(2);
        a[3] = diff(3);
        b[1] = (a[2] + a[3]) + ((a[2] - a[3]) << bits_per_sum);
        tmp[i][0] = b[0] + b[1];
        tmp[i][1] = b[0] - b[1];
    }

    let one = SUM2::one();
    let mask = (one << bits_per_sum) - one;
    for i in 0..2 {
        let [ref mut d0, ref mut d1, ref mut d2, ref mut d3] = a;
        hadamard4(d0, d1, d2, d3, tmp[0][i], tmp[1][i], tmp[2][i], tmp[3][i]);
        a[0] = abs2::<SUM1, SUM2>(a[0])
            + abs2::<SUM1, SUM2>(a[1])
            + abs2::<SUM1, SUM2>(a[2])
            + abs2::<SUM1, SUM2>(a[3]);
        sum += (a[0] & mask) + (a[0] >> bits_per_sum);
    }

    let result = sum >> 1;
    result.into()
}

#[must_use]
fn satd_8x4<
    T: Pixel,
    SUM1: PrimInt,
    SUM2: PrimInt
        + One
        + Default
        + BitAnd<Output = SUM2>
        + BitXor<Output = SUM2>
        + Mul<Output = SUM2>
        + Sub<Output = SUM2>
        + Add<Output = SUM2>
        + Shl<usize, Output = SUM2>
        + Shr<usize, Output = SUM2>
        + AddAssign<SUM2>
        + WrappingAdd
        + Into<u64>
        + FromDiff,
>(
    src: &[T],
    src_pitch: NonZeroUsize,
    ref_: &[T],
    ref_pitch: NonZeroUsize,
) -> u64 {
    let bits_per_sum = size_of::<SUM1>() * 8;
    let mut tmp: [[SUM2; 4]; 4] = Default::default();
    let mut a: [SUM2; 4] = Default::default();
    let mut sum: SUM2 = Default::default();

    for i in 0..4 {
        let src_offset = i * src_pitch.get();
        let ref_offset = i * ref_pitch.get();

        let src_row = &src[src_offset..src_offset + 8];
        let ref_row = &ref_[ref_offset..ref_offset + 8];

        let diff = |idx: usize| -> SUM2 {
            let s: i32 = src_row[idx].to_i32().expect("fits in i32");
            let r: i32 = ref_row[idx].to_i32().expect("fits in i32");
            SUM2::from_diff(s - r)
        };

        a[0] = diff(0).wrapping_add(&(diff(4) << bits_per_sum));
        a[1] = diff(1).wrapping_add(&(diff(5) << bits_per_sum));
        a[2] = diff(2).wrapping_add(&(diff(6) << bits_per_sum));
        a[3] = diff(3).wrapping_add(&(diff(7) << bits_per_sum));
        let [ref mut d0, ref mut d1, ref mut d2, ref mut d3] = tmp[i];
        hadamard4(d0, d1, d2, d3, a[0], a[1], a[2], a[3]);
    }

    for i in 0..4 {
        let [ref mut d0, ref mut d1, ref mut d2, ref mut d3] = a;
        hadamard4(d0, d1, d2, d3, tmp[0][i], tmp[1][i], tmp[2][i], tmp[3][i]);
        sum += abs2::<SUM1, SUM2>(a[0])
            + abs2::<SUM1, SUM2>(a[1])
            + abs2::<SUM1, SUM2>(a[2])
            + abs2::<SUM1, SUM2>(a[3]);
    }

    let one = SUM2::one();
    let mask = (one << bits_per_sum) - one;
    let result = ((sum & mask) + (sum >> bits_per_sum)) >> 1;
    result.into()
}

/// in: a pseudo-simd number of the form x+(y<<bits_per_sum)
/// return: abs(x)+(abs(y)<<bits_per_sum)
#[must_use]
fn abs2<SUM: PrimInt, SUM2>(a: SUM2) -> SUM2
where
    SUM2: PrimInt
        + One
        + BitAnd<Output = SUM2>
        + BitXor<Output = SUM2>
        + Mul<Output = SUM2>
        + Sub<Output = SUM2>
        + Add<Output = SUM2>
        + Shl<usize, Output = SUM2>
        + Shr<usize, Output = SUM2>
        + WrappingAdd,
{
    let bits_per_sum = size_of::<SUM>() * 8;
    let one = SUM2::one();
    let ones = (one << bits_per_sum) - one;
    let mask = (one << bits_per_sum) + one;
    let s = ((a >> (bits_per_sum - 1)) & mask) * ones;
    a.wrapping_add(&s) ^ s
}

trait FromDiff {
    fn from_diff(diff: i32) -> Self;
}

impl FromDiff for u32 {
    fn from_diff(diff: i32) -> Self {
        diff as u32
    }
}

impl FromDiff for u64 {
    fn from_diff(diff: i32) -> Self {
        diff as u64
    }
}

fn hadamard4<SUM: Copy + Add<SUM, Output = SUM> + Sub<SUM, Output = SUM>>(
    dest0: &mut SUM,
    dest1: &mut SUM,
    dest2: &mut SUM,
    dest3: &mut SUM,
    src0: SUM,
    src1: SUM,
    src2: SUM,
    src3: SUM,
) {
    let temp0: SUM = src0 + src1;
    let temp1: SUM = src0 - src1;
    let temp2: SUM = src2 + src3;
    let temp3: SUM = src2 - src3;
    *dest0 = temp0 + temp2;
    *dest2 = temp0 - temp2;
    *dest1 = temp1 + temp3;
    *dest3 = temp1 - temp3;
}
