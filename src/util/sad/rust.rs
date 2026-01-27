use std::num::NonZeroUsize;

use crate::util::Pixel;

#[must_use]
pub(super) fn get_sad<T: Pixel>(
    width: NonZeroUsize,
    height: NonZeroUsize,
    src: &[T],
    src_pitch: NonZeroUsize,
    ref_: &[T],
    ref_pitch: NonZeroUsize,
) -> u64 {
    match (width.get(), height.get()) {
        (2, 2) => get_sad_impl::<T, 2, 2>(src, src_pitch, ref_, ref_pitch),
        (2, 4) => get_sad_impl::<T, 2, 4>(src, src_pitch, ref_, ref_pitch),
        (4, 2) => get_sad_impl::<T, 4, 2>(src, src_pitch, ref_, ref_pitch),
        (4, 4) => get_sad_impl::<T, 4, 4>(src, src_pitch, ref_, ref_pitch),
        (4, 8) => get_sad_impl::<T, 4, 8>(src, src_pitch, ref_, ref_pitch),
        (8, 1) => get_sad_impl::<T, 8, 1>(src, src_pitch, ref_, ref_pitch),
        (8, 2) => get_sad_impl::<T, 8, 2>(src, src_pitch, ref_, ref_pitch),
        (8, 4) => get_sad_impl::<T, 8, 4>(src, src_pitch, ref_, ref_pitch),
        (8, 8) => get_sad_impl::<T, 8, 8>(src, src_pitch, ref_, ref_pitch),
        (8, 16) => get_sad_impl::<T, 8, 16>(src, src_pitch, ref_, ref_pitch),
        (16, 1) => get_sad_impl::<T, 16, 1>(src, src_pitch, ref_, ref_pitch),
        (16, 2) => get_sad_impl::<T, 16, 2>(src, src_pitch, ref_, ref_pitch),
        (16, 4) => get_sad_impl::<T, 16, 4>(src, src_pitch, ref_, ref_pitch),
        (16, 8) => get_sad_impl::<T, 16, 8>(src, src_pitch, ref_, ref_pitch),
        (16, 16) => get_sad_impl::<T, 16, 16>(src, src_pitch, ref_, ref_pitch),
        (16, 32) => get_sad_impl::<T, 16, 32>(src, src_pitch, ref_, ref_pitch),
        (32, 8) => get_sad_impl::<T, 32, 8>(src, src_pitch, ref_, ref_pitch),
        (32, 16) => get_sad_impl::<T, 32, 16>(src, src_pitch, ref_, ref_pitch),
        (32, 32) => get_sad_impl::<T, 32, 32>(src, src_pitch, ref_, ref_pitch),
        (32, 64) => get_sad_impl::<T, 32, 64>(src, src_pitch, ref_, ref_pitch),
        (64, 16) => get_sad_impl::<T, 64, 16>(src, src_pitch, ref_, ref_pitch),
        (64, 32) => get_sad_impl::<T, 64, 32>(src, src_pitch, ref_, ref_pitch),
        (64, 64) => get_sad_impl::<T, 64, 64>(src, src_pitch, ref_, ref_pitch),
        (64, 128) => get_sad_impl::<T, 64, 128>(src, src_pitch, ref_, ref_pitch),
        (128, 32) => get_sad_impl::<T, 128, 32>(src, src_pitch, ref_, ref_pitch),
        (128, 64) => get_sad_impl::<T, 128, 64>(src, src_pitch, ref_, ref_pitch),
        (128, 128) => get_sad_impl::<T, 128, 128>(src, src_pitch, ref_, ref_pitch),
        _ => unimplemented!("Invalid block size for SAD"),
    }
}

#[must_use]
fn get_sad_impl<T: Pixel, const WIDTH: usize, const HEIGHT: usize>(
    src: &[T],
    src_pitch: NonZeroUsize,
    ref_: &[T],
    ref_pitch: NonZeroUsize,
) -> u64 {
    let mut sum = 0;
    for y in 0..HEIGHT {
        let src_row = &src[y * src_pitch.get()..][..WIDTH];
        let ref_row = &ref_[y * ref_pitch.get()..][..WIDTH];
        sum += src_row.iter().zip(ref_row.iter()).fold(0, |acc, (s, r)| {
            if size_of::<T>() == 1 {
                // smaller types for 8-bit for faster code generation
                // branch elided at compile time
                let val1: i16 = s.to_i16().expect("fits in i16");
                let val2: i16 = r.to_i16().expect("fits in i16");
                acc + (val1 - val2).unsigned_abs() as u64
            } else {
                let val1: i32 = s.to_i32().expect("fits in i32");
                let val2: i32 = r.to_i32().expect("fits in i32");
                acc + (val1 - val2).unsigned_abs() as u64
            }
        });
    }
    sum
}
