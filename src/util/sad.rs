mod rust;

#[cfg(test)]
mod tests;

use std::num::NonZeroUsize;

use crate::util::Pixel;

#[must_use]
pub fn get_sad<T: Pixel>(
    width: NonZeroUsize,
    height: NonZeroUsize,
    src: &[T],
    src_pitch: NonZeroUsize,
    ref_: &[T],
    ref_pitch: NonZeroUsize,
) -> u64 {
    rust::get_sad(width, height, src, src_pitch, ref_, ref_pitch)
}
