use std::num::NonZeroUsize;

use crate::util::Pixel;

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
    // For performance reasons, check the array bounds once at the start of the loop.
    assert!(src.len() >= src_pitch.get() * dest_height.get() * 2);
    assert!(dest.len() >= dest_pitch.get() * dest_height.get());

    // SAFETY: Validated bounds above
    unsafe {
        reduce_bilinear_vertical(
            dest,
            src,
            dest_pitch,
            src_pitch,
            dest_width.saturating_mul(NonZeroUsize::new_unchecked(2)),
            dest_height,
        );
        reduce_bilinear_horizontal_inplace(dest, dest_pitch, dest_width, dest_height);
    }
}

/// Applies vertical bilinear filtering to reduce image height by 2x.
///
/// This function performs the first pass of bilinear downscaling by filtering
/// vertically. It uses different weights for edge pixels versus middle pixels
/// to maintain image quality. Edge lines use simple averaging, while middle
/// lines use a weighted filter that considers 4 vertical neighbors.
unsafe fn reduce_bilinear_vertical<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let mut dest = dest.as_mut_ptr();
    let src = src.as_ptr();

    // Special case for first line
    for x in 0..dest_width.get() {
        let a: u32 = (*src.add(x)).to_u32().expect("fits in u32");
        let b: u32 = (*src.add(x + src_pitch.get()))
            .to_u32()
            .expect("fits in u32");
        *dest.add(x) = T::from_u32_or_max_value((a + b + 1) / 2);
    }
    dest = dest.add(dest_pitch.get());

    // Middle lines
    for y in 1..(dest_height.get() - 1) {
        let src_row = src.add(y * 2 * src_pitch.get());
        for x in 0..dest_width.get() {
            let a: u32 = (*src_row.offset(x as isize - src_pitch.get() as isize))
                .to_u32()
                .expect("fits in u32");
            let b: u32 = (*src_row.add(x)).to_u32().expect("fits in u32");
            let c: u32 = (*src_row.add(x + src_pitch.get()))
                .to_u32()
                .expect("fits in u32");
            let d: u32 = (*src_row.add(x + src_pitch.get() * 2))
                .to_u32()
                .expect("fits in u32");
            *dest.add(x) = T::from_u32_or_max_value((a + (b + c) * 3 + d + 4) / 8);
        }
        dest = dest.add(dest_pitch.get());
    }

    // Special case for last line
    if dest_height.get() > 1 {
        let src_row = src.add((dest_height.get() - 1) * 2 * src_pitch.get());
        for x in 0..dest_width.get() {
            let a: u32 = (*src_row.add(x)).to_u32().expect("fits in u32");
            let b: u32 = (*src_row.add(x + src_pitch.get()))
                .to_u32()
                .expect("fits in u32");
            *dest.add(x) = T::from_u32_or_max_value((a + b + 1) / 2);
        }
    }
}

/// Applies horizontal bilinear filtering in-place to reduce image width by 2x.
///
/// This function performs the second pass of bilinear downscaling by filtering
/// horizontally on the already vertically-filtered data. It modifies the buffer
/// in-place, using different weights for edge pixels versus middle pixels.
/// Edge columns use simple averaging, while middle columns use a weighted filter.
unsafe fn reduce_bilinear_horizontal_inplace<T: Pixel>(
    dest: &mut [T],
    dest_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let mut dest = dest.as_mut_ptr();

    for _y in 0..dest_height.get() {
        // Special case start of line
        let a: u32 = (*dest).to_u32().expect("fits in u32");
        let b: u32 = (*dest.add(1)).to_u32().expect("fits in u32");
        let src0 = (a + b + 1) / 2;

        // Middle of line
        for x in 1..(dest_width.get() - 1) {
            let dest_row = dest.add(x * 2);
            let a: u32 = (*dest_row.sub(1)).to_u32().expect("fits in u32");
            let b: u32 = (*dest_row).to_u32().expect("fits in u32");
            let c: u32 = (*dest_row.add(1)).to_u32().expect("fits in u32");
            let d: u32 = (*dest_row.add(2)).to_u32().expect("fits in u32");
            *dest.add(x) = T::from_u32_or_max_value((a + (b + c) * 3 + d + 4) / 8);
        }

        *dest = T::from_u32_or_max_value(src0);

        // Special case end of line
        if dest_width.get() > 1 {
            let x = dest_width.get() - 1;
            let dest_row = dest.add(x * 2);
            let a: u32 = (*dest_row).to_u32().expect("fits in u32");
            let b: u32 = (*dest_row.add(1)).to_u32().expect("fits in u32");
            *dest.add(x) = T::from_u32_or_max_value((a + b + 1) / 2);
        }

        dest = dest.add(dest_pitch.get());
    }
}
