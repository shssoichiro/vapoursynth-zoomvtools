use std::num::NonZeroUsize;

use crate::util::Pixel;

#[cfg(test)]
mod tests;

pub type ReduceFn<T> = fn(&mut [T], &[T], NonZeroUsize, NonZeroUsize, NonZeroUsize, NonZeroUsize);

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
            let sum = a + b + c + d + 2;
            let average = sum / 4;

            // Convert back to original type
            dest[x] = T::try_from(average).unwrap_or_else(|_| {
                // If conversion fails (shouldn't happen with our inputs), fallback to u8::MAX
                // conversion
                T::from(u8::MAX)
            });
        }
        dest = &mut dest[dest_pitch.get()..];
        src = &src[src_pitch.get() * 2..];
    }
}

pub fn reduce_triangle<T: Pixel>(
    _dest: &mut [T],
    _src: &[T],
    _dest_pitch: NonZeroUsize,
    _src_pitch: NonZeroUsize,
    _dest_width: NonZeroUsize,
    _dest_height: NonZeroUsize,
) {
    todo!()
}

pub fn reduce_bilinear<T: Pixel>(
    _dest: &mut [T],
    _src: &[T],
    _dest_pitch: NonZeroUsize,
    _src_pitch: NonZeroUsize,
    _dest_width: NonZeroUsize,
    _dest_height: NonZeroUsize,
) {
    todo!()
}

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

pub fn reduce_cubic<T: Pixel>(
    _dest: &mut [T],
    _src: &[T],
    _dest_pitch: NonZeroUsize,
    _src_pitch: NonZeroUsize,
    _dest_width: NonZeroUsize,
    _dest_height: NonZeroUsize,
) {
    todo!()
}
