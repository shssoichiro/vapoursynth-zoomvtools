use std::num::NonZeroUsize;

use crate::util::Pixel;

#[must_use]
pub fn select_sad_fn<T: Pixel>(
    width: NonZeroUsize,
    height: NonZeroUsize,
) -> impl Fn(&[T], NonZeroUsize, &[T], NonZeroUsize) -> u64 {
    match (width.get(), height.get()) {
        (2, 2) => get_sad::<T, 2, 2>,
        (2, 4) => get_sad::<T, 2, 4>,
        (4, 2) => get_sad::<T, 4, 2>,
        (4, 4) => get_sad::<T, 4, 4>,
        (4, 8) => get_sad::<T, 4, 8>,
        (8, 1) => get_sad::<T, 8, 1>,
        (8, 2) => get_sad::<T, 8, 2>,
        (8, 4) => get_sad::<T, 8, 4>,
        (8, 8) => get_sad::<T, 8, 8>,
        (8, 16) => get_sad::<T, 8, 16>,
        (16, 1) => get_sad::<T, 16, 1>,
        (16, 2) => get_sad::<T, 16, 2>,
        (16, 4) => get_sad::<T, 16, 4>,
        (16, 8) => get_sad::<T, 16, 8>,
        (16, 16) => get_sad::<T, 16, 16>,
        (16, 32) => get_sad::<T, 16, 32>,
        (32, 8) => get_sad::<T, 32, 8>,
        (32, 16) => get_sad::<T, 32, 16>,
        (32, 32) => get_sad::<T, 32, 32>,
        (32, 64) => get_sad::<T, 32, 64>,
        (64, 16) => get_sad::<T, 64, 16>,
        (64, 32) => get_sad::<T, 64, 32>,
        (64, 64) => get_sad::<T, 64, 64>,
        (64, 128) => get_sad::<T, 64, 128>,
        (128, 32) => get_sad::<T, 128, 32>,
        (128, 64) => get_sad::<T, 128, 64>,
        (128, 128) => get_sad::<T, 128, 128>,
        _ => unimplemented!("Invalid block size for SAD"),
    }
}

#[must_use]
pub fn get_sad<T: Pixel, const WIDTH: usize, const HEIGHT: usize>(
    src: &[T],
    src_pitch: NonZeroUsize,
    ref_: &[T],
    ref_pitch: NonZeroUsize,
) -> u64 {
    let mut sum = 0;
    for y in 0..HEIGHT {
        let src_row = &src[y * src_pitch.get()..][..WIDTH];
        let ref_row = &ref_[y * ref_pitch.get()..][..WIDTH];
        for x in 0..WIDTH {
            let val1: i64 = src_row[x].into();
            let val2: i64 = ref_row[x].into();
            sum += (val1 + val2).unsigned_abs();
        }
    }
    sum
}
