use std::num::{NonZeroU8, NonZeroUsize};

use crate::util::Pixel;

pub(super) fn refine_horizontal_bilinear<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    _bits_per_sample: NonZeroU8,
) {
    let mut offset = 0;
    for _j in 0..height.get() {
        let src_row = &src[offset..][..width.get()];
        let dest_row = &mut dest[offset..][..width.get()];

        for i in 0..width.get() - 1 {
            let a: u32 = src_row[i].to_u32().expect("fits in u32");
            let b: u32 = src_row[i + 1].to_u32().expect("fits in u32");
            dest_row[i] = T::from_u32_or_max_value((a + b + 1) / 2);
        }
        // last column
        dest_row[width.get() - 1] = src_row[width.get() - 1];

        offset += pitch.get();
    }
}

pub(super) fn refine_vertical_bilinear<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    _bits_per_sample: NonZeroU8,
) {
    let mut offset = 0;
    for _j in 0..height.get() - 1 {
        for i in 0..width.get() {
            let a: u32 = src[offset + i].to_u32().expect("fits in u32");
            let b: u32 = src[offset + i + pitch.get()].to_u32().expect("fits in u32");
            dest[offset + i] = T::from_u32_or_max_value((a + b + 1) / 2);
        }
        offset += pitch.get();
    }

    // last row
    dest[offset..offset + width.get()].copy_from_slice(&src[offset..offset + width.get()]);
}

pub(super) fn refine_diagonal_bilinear<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    _bits_per_sample: NonZeroU8,
) {
    let mut offset = 0;

    for _j in 0..height.get() {
        for i in 0..width.get() {
            let a: u32 = src[offset + i].to_u32().expect("fits in u32");
            let b: u32 = src[offset + i + 1].to_u32().expect("fits in u32");
            let c: u32 = src[offset + i + pitch.get()].to_u32().expect("fits in u32");
            let d: u32 = src[offset + i + pitch.get() + 1]
                .to_u32()
                .expect("fits in u32");

            dest[offset + i] = T::from_u32_or_max_value((a + b + c + d + 2) / 4);
        }
        // last column
        let a: u32 = src[offset + width.get() - 1].to_u32().expect("fits in u32");
        let b: u32 = src[offset + width.get() - 1 + pitch.get()]
            .to_u32()
            .expect("fits in u32");
        dest[offset + width.get() - 1] = T::from_u32_or_max_value((a + b + 1) / 2);

        offset += pitch.get();
    }

    // last row
    for i in 0..width.get() - 1 {
        let a: u32 = src[offset + i].to_u32().expect("fits in u32");
        let b: u32 = src[offset + i + 1].to_u32().expect("fits in u32");
        dest[offset + i] = T::from_u32_or_max_value((a + b + 1) / 2);
    }
    // last pixel
    dest[offset + width.get() - 1] = src[offset + width.get() - 1];
}
