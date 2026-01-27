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
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    let pixel_max = (1i32 << bits_per_sample.get()) - 1;
    let mut offset = 0;

    for _j in 0..height.get() {
        let src_row = &src[offset..][..width.get()];
        let dest_row = &mut dest[offset..][..width.get()];

        // Handle first two pixels with bilinear interpolation (if width >= 2)
        if width.get() >= 2 {
            let a: u32 = src_row[0].to_u32().expect("fits in u32");
            let b: u32 = src_row[1].to_u32().expect("fits in u32");
            dest_row[0] = T::from_u32_or_max_value((a + b + 1) / 2);

            if width.get() >= 3 {
                let c: u32 = src_row[2].to_u32().expect("fits in u32");
                dest_row[1] = T::from_u32_or_max_value((b + c + 1) / 2);
            }
        }

        // Process middle pixels with Wiener filter
        let wiener_start = 2;
        let wiener_end = if width.get() >= 4 {
            width.get() - 4
        } else {
            wiener_start
        };

        for i in wiener_start..wiener_end {
            let mut m0: i32 = src_row[i - 2].to_i32().expect("fits in i32");
            let m1: i32 = src_row[i - 1].to_i32().expect("fits in i32");
            let mut m2: i32 = src_row[i].to_i32().expect("fits in i32");
            let m3: i32 = src_row[i + 1].to_i32().expect("fits in i32");
            let m4: i32 = src_row[i + 2].to_i32().expect("fits in i32");
            let m5: i32 = src_row[i + 3].to_i32().expect("fits in i32");

            m2 = (m2 + m3) * 4;

            m2 -= m1 + m4;
            m2 *= 5;

            m0 += m5 + m2 + 16;
            m0 >>= 5;

            dest_row[i] = T::from_u32_or_max_value(max(0, min(m0, pixel_max)) as u32);
        }

        // Handle last few pixels with bilinear interpolation
        for i in wiener_end..(width.get() - 1).min(width.get()) {
            let a: u32 = src_row[i].to_u32().expect("fits in u32");
            let b: u32 = src_row[i + 1].to_u32().expect("fits in u32");
            dest_row[i] = T::from_u32_or_max_value((a + b + 1) / 2);
        }

        // Copy last pixel
        if width.get() > 0 {
            dest_row[width.get() - 1] = src_row[width.get() - 1];
        }
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
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    let pixel_max = (1i32 << bits_per_sample.get()) - 1;
    let mut offset = 0;

    for _j in 0..2 {
        for i in 0..width.get() {
            let a: u32 = src[offset + i].to_u32().expect("fits in u32");
            let b: u32 = src[offset + i + pitch.get()].to_u32().expect("fits in u32");
            dest[offset + i] = T::from_u32_or_max_value((a + b + 1) / 2);
        }
        offset += pitch.get();
    }

    for _j in 2..(height.get() - 4) {
        for i in 0..width.get() {
            let mut m0: i32 = src[offset + i - pitch.get() * 2]
                .to_i32()
                .expect("fits in i32");
            let m1: i32 = src[offset + i - pitch.get()].to_i32().expect("fits in i32");
            let mut m2: i32 = src[offset + i].to_i32().expect("fits in i32");
            let m3: i32 = src[offset + i + pitch.get()].to_i32().expect("fits in i32");
            let m4: i32 = src[offset + i + pitch.get() * 2]
                .to_i32()
                .expect("fits in i32");
            let m5: i32 = src[offset + i + pitch.get() * 3]
                .to_i32()
                .expect("fits in i32");

            m2 = (m2 + m3) * 4;

            m2 -= m1 + m4;
            m2 *= 5;

            m0 += m5 + m2 + 16;
            m0 >>= 5;

            dest[offset + i] = T::from_u32_or_max_value(max(0, min(m0, pixel_max)) as u32);
        }
        offset += pitch.get();
    }

    for _j in (height.get() - 4)..(height.get() - 1) {
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
