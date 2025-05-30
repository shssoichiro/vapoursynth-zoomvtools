#[cfg(test)]
mod tests;

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
    let mut offset = 0;
    for _j in 0..height.get() {
        for i in 0..width.get() {
            let a: u32 = src1[offset + i].into();
            let b: u32 = src2[offset + i].into();
            dest[offset + i] = T::from_or_max((a + b).div_ceil(2));
        }
        offset += pitch.get();
    }
}
