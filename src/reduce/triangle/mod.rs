use std::num::NonZeroUsize;

use crate::util::Pixel;

#[cfg(test)]
mod tests;

// separable Filtered with 1/4, 1/2, 1/4 filter for smoothing and anti-aliasing.
// assume we have enough horizontal dimension for intermediate results.
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

// Filtered with 1/4, 1/2, 1/4 filter for smoothing and anti-aliasing.
// height is dest height which is half of source height.
fn reduce_filtered_vertical<T: Pixel>(
    mut dest: &mut [T],
    mut src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    for x in 0..width.get() {
        let a: u32 = src[x].into();
        let b: u32 = src[x + src_pitch.get()].into();
        dest[x] = T::from_or_max((a + b).div_ceil(2));
    }
    dest = &mut dest[dest_pitch.get()..];
    src = &src[src_pitch.get() * 2..];

    for _y in 1..height.get() {
        for x in 0..width.get() {
            let a: u32 = src[x - src_pitch.get()].into();
            let b: u32 = src[x].into();
            let c: u32 = src[x + src_pitch.get()].into();
            dest[x] = T::from_or_max((a + b * 2 + c + 2) / 4);
        }

        dest = &mut dest[dest_pitch.get()..];
        src = &src[src_pitch.get() * 2..];
    }
}

// Filtered with 1/4, 1/2, 1/4 filter for smoothing and anti-aliasing.
// width is dest width which is half of source width.
fn reduce_filtered_horizontal_inplace<T: Pixel>(
    mut dest: &mut [T],
    dest_pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    for _y in 0..height.get() {
        let x = 0;
        let mut a: u32;
        let mut b: u32 = dest[x * 2].into();
        let mut c: u32 = dest[x * 2 + 1].into();
        let src0 = (b + c).div_ceil(2);

        for x in 1..width.get() {
            a = dest[x * 2 - 1].into();
            b = dest[x * 2].into();
            c = dest[x * 2 + 1].into();

            dest[x] = T::from_or_max((a + b * 2 + c + 2) / 4);
        }
        dest[0] = T::try_from(src0).unwrap_or_else(|_| {
            // If conversion fails (shouldn't happen with our inputs), fallback to u8::MAX
            // conversion
            T::max_value()
        });

        dest = &mut dest[dest_pitch.get()..];
    }
}
