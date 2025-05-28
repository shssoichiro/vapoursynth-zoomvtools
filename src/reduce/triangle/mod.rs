use std::num::NonZeroUsize;

use crate::util::Pixel;

#[cfg(test)]
mod tests;

// Downscale the height and width of `src` by 2 and write the output into `dest`
pub fn reduce_triangle<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    reduce_filtered_vertical(
        dest,
        src,
        dest_pitch,
        src_pitch,
        // SAFETY: non-zero constant
        dest_width.saturating_mul(unsafe { NonZeroUsize::new_unchecked(2) }),
        dest_height,
    );
    reduce_filtered_horizontal_inplace(dest, dest_pitch, dest_width, dest_height);
}

fn reduce_filtered_vertical<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let width_usize = dest_width.get();
    let height_usize = dest_height.get();
    let src_pitch_usize = src_pitch.get();
    let dest_pitch_usize = dest_pitch.get();

    // Process first output row: average of first two input rows
    for x in 0..width_usize {
        let a: u32 = src[x].into();
        let b: u32 = src[x + src_pitch_usize].into();
        dest[x] = T::from_or_max((a + b).div_ceil(2));
    }

    // Process remaining output rows: 1/4, 1/2, 1/4 filter
    for y in 1..height_usize {
        let dest_offset = y * dest_pitch_usize;
        let src_offset = y * 2 * src_pitch_usize; // Each output row corresponds to 2 input rows

        for x in 0..width_usize {
            // Access three consecutive input rows for the 1/4, 1/2, 1/4 filter
            let a: u32 = src[src_offset + x - src_pitch_usize].into(); // Previous row
            let b: u32 = src[src_offset + x].into(); // Current row
            let c: u32 = src[src_offset + x + src_pitch_usize].into(); // Next row
            dest[dest_offset + x] = T::from_or_max((a + b * 2 + c + 2) / 4);
        }
    }
}

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
        dest[0] = T::from_or_max(src0);

        dest = &mut dest[dest_pitch.get()..];
    }
}
