use std::num::NonZeroUsize;

use crate::util::Pixel;

#[cfg(test)]
mod tests;

pub fn reduce_cubic<T: Pixel>(
    _dest: &mut [T],
    _src: &[T],
    _dest_pitch: NonZeroUsize,
    _src_pitch: NonZeroUsize,
    _width: NonZeroUsize,
    _height: NonZeroUsize,
) {
    todo!()
}
