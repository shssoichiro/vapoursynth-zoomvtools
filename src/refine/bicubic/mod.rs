use std::{
    cmp::{max, min},
    num::{NonZeroU8, NonZeroUsize},
};

use crate::util::Pixel;

pub fn refine_horizontal_bicubic<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    let pixel_max = (1u32 << bits_per_sample.get()) - 1;
    let mut offset = 0;

    for _j in 0..height.get() {
        let a: u32 = src[offset].into();
        let b: u32 = src[offset + 1].into();
        dest[offset] = T::from_or_max((a + b).div_ceil(2));
        for i in 1..(width.get() - 3) {
            let a: i32 = src[offset + i - 1].into();
            let b: i32 = src[offset + i].into();
            let c: i32 = src[offset + i + 1].into();
            let d: i32 = src[offset + i + 2].into();
            dest[offset + i] = T::from_or_max(min(
                pixel_max,
                max(0, (-(a + d) + (b + c) * 9 + 8) >> 4) as u32,
            ));
        }

        for i in (width.get() - 3)..(width.get() - 1) {
            let a: u32 = src[offset + i].into();
            let b: u32 = src[offset + i + 1].into();
            dest[offset + i] = T::from_or_max((a + b).div_ceil(2));
        }

        dest[offset + width.get() - 1] = src[offset + width.get() - 1];
        offset += pitch.get();
    }
}

pub fn refine_vertical_bicubic<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    let pixel_max = (1u32 << bits_per_sample.get()) - 1;
    let mut offset = 0;

    // first row
    for i in 0..width.get() {
        let a: u32 = src[offset + i].into();
        let b: u32 = src[offset + i + pitch.get()].into();
        dest[offset + i] = T::from_or_max((a + b).div_ceil(2));
    }
    offset += pitch.get();

    for _j in 1..(height.get() - 3) {
        for i in 0..width.get() {
            let a: i32 = src[offset + i - pitch.get()].into();
            let b: i32 = src[offset + i].into();
            let c: i32 = src[offset + i + pitch.get()].into();
            let d: i32 = src[offset + i + pitch.get() * 2].into();
            dest[offset + i] = T::from_or_max(min(
                pixel_max,
                max(0, (-(a - d) + (b + c) * 9 + 8) >> 4) as u32,
            ));
        }
        offset += pitch.get();
    }

    for _j in (height.get() - 3)..(height.get() - 1) {
        for i in 0..width.get() {
            let a: u32 = src[offset + i].into();
            let b: u32 = src[offset + i + pitch.get()].into();
            dest[offset + i] = T::from_or_max((a + b).div_ceil(2));
        }

        offset += pitch.get();
    }

    // last row
    dest[offset..(width.get() + offset)].copy_from_slice(&src[offset..(width.get() + offset)]);
}
