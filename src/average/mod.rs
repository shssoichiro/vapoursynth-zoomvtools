use std::num::NonZeroUsize;

use crate::util::Pixel;

pub fn average2<T: Pixel>(
    src1: &[T],
    src2: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    todo!()
}
