use std::{
    cmp::{max, min},
    num::{NonZeroU8, NonZeroUsize},
};

use crate::util::Pixel;

/// Performs horizontal bicubic interpolation for sub-pixel motion estimation refinement.
///
/// This function applies bicubic interpolation horizontally to create sub-pixel samples
/// between existing pixels. Bicubic interpolation uses a 4-tap kernel that considers
/// 4 horizontal neighbors, providing smooth and high-quality interpolation suitable
/// for motion estimation with sub-pixel accuracy.
///
/// Edge pixels use simple averaging due to insufficient neighbors for the full kernel.
///
/// # Parameters
/// - `src`: Source image buffer
/// - `dest`: Destination buffer for interpolated results
/// - `pitch`: Number of pixels per row in both buffers
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
/// - `bits_per_sample`: Bit depth of the pixel format for clamping
pub fn refine_horizontal_bicubic<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    debug_assert!(
        bits_per_sample.get() as usize > (size_of::<T>() - 1) * 8
            && (bits_per_sample.get() as usize <= size_of::<T>() * 8)
    );

    let pixel_max = (1u32 << bits_per_sample.get()) - 1;
    let mut offset = 0;

    for _j in 0..height.get() {
        let src_row = &src[offset..][..width.get()];
        let dest_row = &mut dest[offset..][..width.get()];

        let a: u32 = src_row[0].to_u32().expect("fits in u32");
        let b: u32 = src_row[1].to_u32().expect("fits in u32");
        dest_row[0] = T::from_u32_or_max_value((a + b + 1) / 2);
        for i in 1..(width.get() - 3) {
            let a: i32 = src_row[i - 1].to_i32().expect("fits in i32");
            let b: i32 = src_row[i].to_i32().expect("fits in i32");
            let c: i32 = src_row[i + 1].to_i32().expect("fits in i32");
            let d: i32 = src_row[i + 2].to_i32().expect("fits in i32");
            dest_row[i] = T::from_u32_or_max_value(min(
                pixel_max,
                max(0, (-(a + d) + (b + c) * 9 + 8) >> 4) as u32,
            ));
        }

        for i in (width.get() - 3)..(width.get() - 1) {
            let a: u32 = src_row[i].to_u32().expect("fits in u32");
            let b: u32 = src_row[i + 1].to_u32().expect("fits in u32");
            dest_row[i] = T::from_u32_or_max_value((a + b + 1) / 2);
        }

        dest_row[width.get() - 1] = src_row[width.get() - 1];
        offset += pitch.get();
    }
}

/// Performs vertical bicubic interpolation for sub-pixel motion estimation refinement.
///
/// This function applies bicubic interpolation vertically to create sub-pixel samples
/// between existing pixels. Bicubic interpolation uses a 4-tap kernel that considers
/// 4 vertical neighbors, providing smooth and high-quality interpolation suitable
/// for motion estimation with sub-pixel accuracy.
///
/// Edge rows use simple averaging due to insufficient neighbors for the full kernel,
/// and the last row is copied directly from the source.
///
/// # Parameters
/// - `src`: Source image buffer
/// - `dest`: Destination buffer for interpolated results
/// - `pitch`: Number of pixels per row in both buffers
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
/// - `bits_per_sample`: Bit depth of the pixel format for clamping
pub fn refine_vertical_bicubic<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    debug_assert!(
        bits_per_sample.get() as usize > (size_of::<T>() - 1) * 8
            && (bits_per_sample.get() as usize <= size_of::<T>() * 8)
    );

    let pixel_max = (1u32 << bits_per_sample.get()) - 1;
    let mut offset = 0;

    // first row
    for i in 0..width.get() {
        let a: u32 = src[offset + i].to_u32().expect("fits in u32");
        let b: u32 = src[offset + i + pitch.get()].to_u32().expect("fits in u32");
        dest[offset + i] = T::from_u32_or_max_value((a + b + 1) / 2);
    }
    offset += pitch.get();

    for _j in 1..(height.get() - 3) {
        for i in 0..width.get() {
            let a: i32 = src[offset + i - pitch.get()].to_i32().expect("fits in i32");
            let b: i32 = src[offset + i].to_i32().expect("fits in i32");
            let c: i32 = src[offset + i + pitch.get()].to_i32().expect("fits in i32");
            let d: i32 = src[offset + i + pitch.get() * 2]
                .to_i32()
                .expect("fits in i32");
            dest[offset + i] = T::from_u32_or_max_value(min(
                pixel_max,
                max(0, (-(a + d) + (b + c) * 9 + 8) >> 4) as u32,
            ));
        }
        offset += pitch.get();
    }

    for _j in (height.get() - 3)..(height.get() - 1) {
        for i in 0..width.get() {
            let a: u32 = src[offset + i].to_u32().expect("fits in u32");
            let b: u32 = src[offset + i + pitch.get()].to_u32().expect("fits in u32");
            dest[offset + i] = T::from_u32_or_max_value((a + b + 1) / 2);
        }

        offset += pitch.get();
    }

    // last row
    dest[offset..(width.get() + offset)].copy_from_slice(&src[offset..(width.get() + offset)]);
}
