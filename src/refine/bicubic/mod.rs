use std::num::{NonZeroU8, NonZeroUsize};

use crate::util::Pixel;

pub fn refine_horizontal_bicubic<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    todo!()
}

pub fn refine_vertical_bicubic<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    todo!()
}
