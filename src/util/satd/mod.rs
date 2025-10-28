use std::num::NonZeroUsize;

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
    todo!()
}
