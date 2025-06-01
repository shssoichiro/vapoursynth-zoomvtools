use std::{
    cmp::{max, min},
    num::{NonZeroU8, NonZeroUsize},
};

use crate::util::Pixel;

/// Performs horizontal Wiener filtering for sub-pixel motion estimation refinement.
///
/// This function applies a Wiener filter horizontally to create high-quality sub-pixel
/// samples between existing pixels. The Wiener filter uses a 6-tap kernel with optimized
/// coefficients that provide excellent interpolation quality by minimizing reconstruction
/// error while preserving image details.
///
/// Edge pixels use simple averaging due to insufficient neighbors for the full kernel.
/// The Wiener filter is particularly effective for maintaining sharpness during
/// sub-pixel interpolation in motion estimation applications.
///
/// # Parameters
/// - `src`: Source image buffer
/// - `dest`: Destination buffer for interpolated results
/// - `pitch`: Number of pixels per row in both buffers
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
/// - `bits_per_sample`: Bit depth of the pixel format for clamping
pub fn refine_horizontal_wiener<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    let pixel_max = (1i32 << bits_per_sample.get()) - 1;
    let mut offset = 0;

    for _j in 0..height.get() {
        let a: u32 = src[offset].into();
        let b: u32 = src[offset + 1].into();
        let c: u32 = src[offset + 2].into();
        dest[offset] = T::from_or_max((a + b + 1) / 2);
        dest[offset + 1] = T::from_or_max((b + c + 1) / 2);

        for i in 2..(width.get() - 4) {
            let mut m0: i32 = src[offset + i - 2].into();
            let m1: i32 = src[offset + i - 1].into();
            let mut m2: i32 = src[offset + i].into();
            let m3: i32 = src[offset + i + 1].into();
            let m4: i32 = src[offset + i + 2].into();
            let m5: i32 = src[offset + i + 3].into();

            m2 = (m2 + m3) * 4;

            m2 -= m1 + m4;
            m2 *= 5;

            m0 += m5 + m2 + 16;
            m0 >>= 5;

            dest[offset + i] = T::from_or_max(max(0, min(m0, pixel_max)) as u32);
        }

        for i in (width.get() - 4)..(width.get() - 1) {
            let a: u32 = src[offset + i].into();
            let b: u32 = src[offset + i + 1].into();
            dest[offset + i] = T::from_or_max((a + b + 1) / 2);
        }

        dest[offset + width.get() - 1] = src[offset + width.get() - 1];
        offset += pitch.get();
    }
}

/// Performs vertical Wiener filtering for sub-pixel motion estimation refinement.
///
/// This function applies a Wiener filter vertically to create high-quality sub-pixel
/// samples between existing pixels. The Wiener filter uses a 6-tap kernel with optimized
/// coefficients that provide excellent interpolation quality by minimizing reconstruction
/// error while preserving image details.
///
/// Edge rows use simple averaging due to insufficient neighbors for the full kernel,
/// and the last row is copied directly from the source. The Wiener filter is
/// particularly effective for maintaining sharpness during sub-pixel interpolation.
///
/// # Parameters
/// - `src`: Source image buffer
/// - `dest`: Destination buffer for interpolated results
/// - `pitch`: Number of pixels per row in both buffers
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
/// - `bits_per_sample`: Bit depth of the pixel format for clamping
pub fn refine_vertical_wiener<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    let pixel_max = (1i32 << bits_per_sample.get()) - 1;
    let mut offset = 0;

    for _j in 0..2 {
        for i in 0..width.get() {
            let a: u32 = src[offset + i].into();
            let b: u32 = src[offset + i + pitch.get()].into();
            dest[offset + i] = T::from_or_max((a + b + 1) / 2);
        }
        offset += pitch.get();
    }

    for _j in 2..(height.get() - 4) {
        for i in 0..width.get() {
            let mut m0: i32 = src[offset + i - pitch.get() * 2].into();
            let m1: i32 = src[offset + i - pitch.get()].into();
            let mut m2: i32 = src[offset + i].into();
            let m3: i32 = src[offset + i + pitch.get()].into();
            let m4: i32 = src[offset + i + pitch.get() * 2].into();
            let m5: i32 = src[offset + i + pitch.get() * 3].into();

            m2 = (m2 + m3) * 4;

            m2 -= m1 + m4;
            m2 *= 5;

            m0 += m5 + m2 + 16;
            m0 >>= 5;

            dest[offset + i] = T::from_or_max(max(0, min(m0, pixel_max)) as u32);
        }
        offset += pitch.get();
    }

    for _j in (height.get() - 4)..(height.get() - 1) {
        for i in 0..width.get() {
            let a: u32 = src[offset + i].into();
            let b: u32 = src[offset + i + pitch.get()].into();
            dest[offset + i] = T::from_or_max((a + b + 1) / 2);
        }

        offset += pitch.get();
    }

    // last row
    dest[offset..offset + width.get()].copy_from_slice(&src[offset..offset + width.get()]);
}
