use std::num::NonZeroUsize;

use crate::util::Pixel;

#[cfg(test)]
mod tests;

pub fn reduce_average<T: Pixel>(
    mut dest: &mut [T],
    mut src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    for _y in 0..height.get() {
        for x in 0..width.get() {
            // Convert to u32 for intermediate calculation to prevent overflow
            let a: u32 = src[x * 2].into();
            let b: u32 = src[x * 2 + 1].into();
            let c: u32 = src[x * 2 + src_pitch.get()].into();
            let d: u32 = src[x * 2 + src_pitch.get() + 1].into();

            // Calculate average with proper rounding: (a + b + c + d + 2) / 4
            dest[x] = T::from_or_max((a + b + c + d + 2) / 4);
        }
        dest = &mut dest[dest_pitch.get()..];
        src = &src[src_pitch.get() * 2..];
    }
}
