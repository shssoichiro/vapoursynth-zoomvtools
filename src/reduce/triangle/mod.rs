use std::num::NonZeroUsize;

use crate::util::Pixel;

#[cfg(test)]
mod tests;

pub fn reduce_triangle<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    reduce_filtered_vertical(
        dest,
        src,
        dest_pitch,
        src_pitch,
        // SAFETY: non-zero constant
        width.saturating_mul(unsafe { NonZeroUsize::new_unchecked(2) }),
        height,
    );
    reduce_filtered_horizontal_inplace(dest, dest_pitch, width, height);
}

fn reduce_filtered_vertical<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    new_unchecked: NonZeroUsize,
    height: NonZeroUsize,
) {
    todo!()
}

fn reduce_filtered_horizontal_inplace<T: Pixel>(
    dest: &mut [T],
    dest_pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    todo!()
}
