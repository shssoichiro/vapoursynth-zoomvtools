use std::num::NonZeroUsize;

use crate::util::Pixel;

#[cfg(test)]
mod tests;

/// Downscales an image by 2x using quadratic interpolation.
///
/// This function reduces both the width and height of the source image by half
/// using a two-pass quadratic filtering approach. First, vertical filtering is
/// applied to reduce the height, then horizontal filtering is applied in-place
/// to reduce the width. Quadratic interpolation provides a balance between
/// computational efficiency and image quality, using a 6-tap kernel with
/// quadratic weighting functions.
///
/// The quadratic filter uses different weights than cubic interpolation,
/// optimized for smooth gradients while maintaining sharpness in details.
///
/// # Parameters
/// - `dest`: Destination buffer to store the downscaled image
/// - `src`: Source image buffer to downscale
/// - `dest_pitch`: Number of pixels per row in the destination buffer
/// - `src_pitch`: Number of pixels per row in the source buffer
/// - `dest_width`: Width of the destination image (half of source width)
/// - `dest_height`: Height of the destination image (half of source height)
pub fn reduce_quadratic<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    reduce_quadratic_vertical(
        dest,
        src,
        dest_pitch,
        src_pitch,
        // SAFETY: non-zero constant
        dest_width.saturating_mul(unsafe { NonZeroUsize::new_unchecked(2) }),
        dest_height,
    );
    reduce_quadratic_horizontal_inplace(dest, dest_pitch, dest_width, dest_height);
}

/// Applies vertical quadratic filtering to reduce image height by 2x.
///
/// This function performs the first pass of quadratic downscaling by filtering
/// vertically using a 6-tap filter kernel. Edge lines use simple averaging,
/// while middle lines use the full quadratic filter that considers 6 vertical
/// neighbors with quadratic-weighted coefficients.
fn reduce_quadratic_vertical<T: Pixel>(
    mut dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    // Special case for first line
    for x in 0..dest_width.get() {
        let a: u32 = src[x].into();
        let b: u32 = src[x + src_pitch.get()].into();
        dest[x] = T::from_or_max((a + b).div_ceil(2));
    }
    dest = &mut dest[dest_pitch.get()..];

    // Middle lines
    for y in 1..(dest_height.get() - 1) {
        let src_row_offset = y * 2 * src_pitch.get();
        for x in 0..dest_width.get() {
            let mut m0: u32 = src[src_row_offset + x - src_pitch.get() * 2].into();
            let mut m1: u32 = src[src_row_offset + x - src_pitch.get()].into();
            let mut m2: u32 = src[src_row_offset + x].into();
            let m3: u32 = src[src_row_offset + x + src_pitch.get()].into();
            let m4: u32 = src[src_row_offset + x + src_pitch.get() * 2].into();
            let m5: u32 = src[src_row_offset + x + src_pitch.get() * 3].into();

            m2 = (m2 + m3) * 22;
            m1 = (m1 + m4) * 9;
            m0 += m5 + m2 + m1 + 32;
            m0 >>= 6;

            dest[x] = T::from_or_max(m0);
        }
        dest = &mut dest[dest_pitch.get()..];
    }

    // Special case for last line
    if dest_height.get() > 1 {
        let src_row_offset = (dest_height.get() - 1) * 2 * src_pitch.get();
        for x in 0..dest_width.get() {
            let a: u32 = src[src_row_offset + x].into();
            let b: u32 = src[src_row_offset + x + src_pitch.get()].into();
            dest[x] = T::from_or_max((a + b).div_ceil(2));
        }
    }
}

/// Applies horizontal quadratic filtering in-place to reduce image width by 2x.
///
/// This function performs the second pass of quadratic downscaling by filtering
/// horizontally on the already vertically-filtered data. It modifies the buffer
/// in-place, using the same 6-tap quadratic filter kernel horizontally.
/// Edge columns use simple averaging, while middle columns use the full filter.
fn reduce_quadratic_horizontal_inplace<T: Pixel>(
    mut dest: &mut [T],
    dest_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    for _y in 0..dest_height.get() {
        // Special case start of line
        let a: u32 = dest[0].into();
        let b: u32 = dest[1].into();
        let src0 = (a + b).div_ceil(2);

        // Middle of line
        for x in 1..(dest_width.get() - 1) {
            let mut m0: u32 = dest[x * 2 - 2].into();
            let mut m1: u32 = dest[x * 2 - 1].into();
            let mut m2: u32 = dest[x * 2].into();
            let m3: u32 = dest[x * 2 + 1].into();
            let m4: u32 = dest[x * 2 + 2].into();
            let m5: u32 = dest[x * 2 + 3].into();

            m2 = (m2 + m3) * 22;
            m1 = (m1 + m4) * 9;
            m0 += m5 + m2 + m1 + 32;
            m0 >>= 6;

            dest[x] = T::from_or_max(m0);
        }

        dest[0] = T::from_or_max(src0);

        // Special case end of line
        if dest_width.get() > 1 {
            let x = dest_width.get() - 1;
            let a: u32 = dest[x * 2].into();
            let b: u32 = dest[x * 2 + 1].into();
            dest[x] = T::from_or_max((a + b).div_ceil(2));
        }

        dest = &mut dest[dest_pitch.get()..];
    }
}
