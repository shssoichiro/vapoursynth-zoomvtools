use std::num::NonZeroUsize;

use crate::util::Pixel;

#[cfg(test)]
mod tests;

// Downscale the height and width of `src` by 2 and write the output into `dest`
pub fn reduce_quadratic<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    reduce_quadratic_vertical(
        dest,
        src,
        dest_pitch,
        src_pitch,
        // SAFETY: non-zero constant
        dest_width.saturating_mul(unsafe { NonZeroUsize::new_unchecked(2) }),
        dest_height,
    );
    reduce_quadratic_horizontal_inplace(dest, dest_pitch, dest_width, dest_height);
}

fn reduce_quadratic_vertical<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    todo!()
}

fn reduce_quadratic_horizontal_inplace<T: Pixel>(
    dest: &mut [T],
    dest_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    todo!()
}
