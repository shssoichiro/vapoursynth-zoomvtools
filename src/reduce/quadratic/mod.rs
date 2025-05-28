use std::num::NonZeroUsize;

use crate::util::Pixel;

#[cfg(test)]
mod tests;

// Downscale the height and width of `src` by 2 and write the output into `dest`
pub fn reduce_quadratic<T: Pixel>(
    _dest: &mut [T],
    _src: &[T],
    _dest_pitch: NonZeroUsize,
    _src_pitch: NonZeroUsize,
    _dest_width: NonZeroUsize,
    _dest_height: NonZeroUsize,
) {
    todo!()
}
