use std::num::NonZeroUsize;

use vapoursynth::prelude::Component;

pub fn vs_bitblt<T: Component>(
    dest: &[T],
    dest_stride: NonZeroUsize,
    src: &[T],
    src_stride: NonZeroUsize,
    row_size: NonZeroUsize,
    height: NonZeroUsize,
) {
    todo!()
}
