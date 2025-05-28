use std::num::NonZeroUsize;

use crate::util::Pixel;

#[cfg(test)]
mod tests;

// Downscale the height and width of `src` by 2 and write the output into `dest`
pub fn reduce_average<T: Pixel>(
    mut dest: &mut [T],
    mut src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    for _y in 0..dest_height.get() {
        for x in 0..dest_width.get() {
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
