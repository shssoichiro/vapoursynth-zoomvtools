use std::num::NonZeroUsize;

use crate::util::Pixel;

#[cfg(test)]
mod tests;

/// Downscales an image by 2x using bilinear interpolation.
///
/// This function reduces both the width and height of the source image by half
/// using a two-pass bilinear filtering approach. First, vertical filtering is
/// applied to reduce the height, then horizontal filtering is applied in-place
/// to reduce the width. This produces higher quality results than simple averaging
/// by using weighted interpolation that considers neighboring pixels.
///
/// # Parameters
/// - `dest`: Destination buffer to store the downscaled image
/// - `src`: Source image buffer to downscale
/// - `dest_pitch`: Number of pixels per row in the destination buffer
/// - `src_pitch`: Number of pixels per row in the source buffer
/// - `dest_width`: Width of the destination image (half of source width)
/// - `dest_height`: Height of the destination image (half of source height)
pub fn reduce_bilinear<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    reduce_bilinear_vertical(
        dest,
        src,
        dest_pitch,
        src_pitch,
        // SAFETY: non-zero constant
        dest_width.saturating_mul(unsafe { NonZeroUsize::new_unchecked(2) }),
        dest_height,
    );
    reduce_bilinear_horizontal_inplace(dest, dest_pitch, dest_width, dest_height);
}

/// Applies vertical bilinear filtering to reduce image height by 2x.
///
/// This function performs the first pass of bilinear downscaling by filtering
/// vertically. It uses different weights for edge pixels versus middle pixels
/// to maintain image quality. Edge lines use simple averaging, while middle
/// lines use a weighted filter that considers 4 vertical neighbors.
fn reduce_bilinear_vertical<T: Pixel>(
    mut dest: &mut [T],
    mut src: &[T],
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
    src = &src[src_pitch.get() * 2..];

    // Middle lines
    for _y in 1..(dest_height.get() - 1) {
        for x in 0..dest_width.get() {
            let a: u32 = src[x - src_pitch.get()].into();
            let b: u32 = src[x].into();
            let c: u32 = src[x + src_pitch.get()].into();
            let d: u32 = src[x + src_pitch.get() * 2].into();
            dest[x] = T::from_or_max((a + (b + c) * 3 + d + 4) / 8);
        }
        dest = &mut dest[dest_pitch.get()..];
        src = &src[src_pitch.get() * 2..];
    }

    // Special case for last line
    if dest_height.get() > 1 {
        for x in 0..dest_width.get() {
            let a: u32 = src[x].into();
            let b: u32 = src[x + src_pitch.get()].into();
            dest[x] = T::from_or_max((a + b).div_ceil(2));
        }
    }
}

/// Applies horizontal bilinear filtering in-place to reduce image width by 2x.
///
/// This function performs the second pass of bilinear downscaling by filtering
/// horizontally on the already vertically-filtered data. It modifies the buffer
/// in-place, using different weights for edge pixels versus middle pixels.
/// Edge columns use simple averaging, while middle columns use a weighted filter.
fn reduce_bilinear_horizontal_inplace<T: Pixel>(
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
            let a: u32 = dest[x * 2 - 1].into();
            let b: u32 = dest[x * 2].into();
            let c: u32 = dest[x * 2 + 1].into();
            let d: u32 = dest[x * 2 + 2].into();
            dest[x] = T::from_or_max((a + (b + c) * 3 + d + 4) / 8);
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
