use std::num::NonZeroUsize;

use crate::util::Pixel;

#[must_use]
pub(super) fn luma_sum<T: Pixel>(
    width: NonZeroUsize,
    height: NonZeroUsize,
    src: &[T],
    src_pitch: NonZeroUsize,
) -> u64 {
    match (width.get(), height.get()) {
        (4, 4) => luma_sum_impl::<T, 4, 4>(src, src_pitch),
        (8, 4) => luma_sum_impl::<T, 8, 4>(src, src_pitch),
        (8, 8) => luma_sum_impl::<T, 8, 8>(src, src_pitch),
        (16, 2) => luma_sum_impl::<T, 16, 2>(src, src_pitch),
        (16, 8) => luma_sum_impl::<T, 16, 8>(src, src_pitch),
        (16, 16) => luma_sum_impl::<T, 16, 16>(src, src_pitch),
        (32, 16) => luma_sum_impl::<T, 32, 16>(src, src_pitch),
        (32, 32) => luma_sum_impl::<T, 32, 32>(src, src_pitch),
        (64, 32) => luma_sum_impl::<T, 64, 32>(src, src_pitch),
        (64, 64) => luma_sum_impl::<T, 64, 64>(src, src_pitch),
        (128, 64) => luma_sum_impl::<T, 128, 64>(src, src_pitch),
        (128, 128) => luma_sum_impl::<T, 128, 128>(src, src_pitch),
        _ => unreachable!("unsupported block size"),
    }
}

#[must_use]
fn luma_sum_impl<T: Pixel, const WIDTH: usize, const HEIGHT: usize>(
    src: &[T],
    src_pitch: NonZeroUsize,
) -> u64 {
    let mut luma_sum = 0u64;
    for j in 0..HEIGHT {
        let src_row = &src[j * src_pitch.get()..][..WIDTH];
        for &pix in src_row {
            let pixel_value: u64 = pix.to_u64().expect("fits in u64");
            luma_sum += pixel_value;
        }
    }
    luma_sum
}
