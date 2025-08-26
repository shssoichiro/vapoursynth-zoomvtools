use std::num::NonZeroUsize;

use crate::util::Pixel;

/// Downscales an image by 2x using cubic interpolation.
///
/// This function reduces both the width and height of the source image by half
/// using a two-pass cubic filtering approach. First, vertical filtering is
/// applied to reduce the height, then horizontal filtering is applied in-place
/// to reduce the width. Cubic interpolation provides higher quality than bilinear
/// by using a wider kernel that considers more neighboring pixels for smoother results.
///
/// The cubic filter uses a 6-tap kernel with specific weights optimized for
/// downscaling while preserving image details and reducing artifacts.
///
/// # Parameters
/// - `dest`: Destination buffer to store the downscaled image
/// - `src`: Source image buffer to downscale
/// - `dest_pitch`: Number of pixels per row in the destination buffer
/// - `src_pitch`: Number of pixels per row in the source buffer
/// - `dest_width`: Width of the destination image (half of source width)
/// - `dest_height`: Height of the destination image (half of source height)
pub fn reduce_cubic<T: Pixel>(
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
        reduce_cubic_vertical(
            dest,
            src,
            dest_pitch,
            src_pitch,
            dest_width.saturating_mul(NonZeroUsize::new_unchecked(2)),
            dest_height,
        );
        reduce_cubic_horizontal_inplace(dest, dest_pitch, dest_width, dest_height);
    }
}

/// Applies vertical cubic filtering to reduce image height by 2x.
///
/// This function performs the first pass of cubic downscaling by filtering
/// vertically using a 6-tap filter kernel. Edge lines use simple averaging,
/// while middle lines use the full cubic filter that considers 6 vertical
/// neighbors with optimized weights for high-quality downscaling.
unsafe fn reduce_cubic_vertical<T: Pixel>(
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
        let a: u32 = (*src.add(x)).into();
        let b: u32 = (*src.add(x + src_pitch.get())).into();
        *dest.add(x) = T::from_u32_or_max_value((a + b + 1) / 2);
    }
    dest = dest.add(dest_pitch.get());

    // Middle lines
    for y in 1..(dest_height.get() - 1) {
        let src_row_offset = y * 2 * src_pitch.get();
        for x in 0..dest_width.get() {
            let mut m0: u32 = (*src.add(src_row_offset + x - src_pitch.get() * 2)).into();
            let mut m1: u32 = (*src.add(src_row_offset + x - src_pitch.get())).into();
            let mut m2: u32 = (*src.add(src_row_offset + x)).into();
            let m3: u32 = (*src.add(src_row_offset + x + src_pitch.get())).into();
            let m4: u32 = (*src.add(src_row_offset + x + src_pitch.get() * 2)).into();
            let m5: u32 = (*src.add(src_row_offset + x + src_pitch.get() * 3)).into();

            m2 = (m2 + m3) * 10;
            m1 = (m1 + m4) * 5;
            m0 += m5 + m2 + m1 + 16;
            m0 >>= 5;

            *dest.add(x) = T::from_u32_or_max_value(m0);
        }
        dest = dest.add(dest_pitch.get());
    }

    // Special case for last line
    if dest_height.get() > 1 {
        let src_row_offset = (dest_height.get() - 1) * 2 * src_pitch.get();
        for x in 0..dest_width.get() {
            let a: u32 = (*src.add(src_row_offset + x)).into();
            let b: u32 = (*src.add(src_row_offset + x + src_pitch.get())).into();
            *dest.add(x) = T::from_u32_or_max_value((a + b + 1) / 2);
        }
    }
}

/// Applies horizontal cubic filtering in-place to reduce image width by 2x.
///
/// This function performs the second pass of cubic downscaling by filtering
/// horizontally on the already vertically-filtered data. It modifies the buffer
/// in-place, using the same 6-tap cubic filter kernel horizontally.
/// Edge columns use simple averaging, while middle columns use the full filter.
unsafe fn reduce_cubic_horizontal_inplace<T: Pixel>(
    dest: &mut [T],
    dest_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let mut dest = dest.as_mut_ptr();

    for _y in 0..dest_height.get() {
        // Special case start of line
        let a: u32 = (*dest).into();
        let b: u32 = (*dest.add(1)).into();
        let src0 = (a + b + 1) / 2;

        // Middle of line
        for x in 1..(dest_width.get() - 1) {
            let mut m0: u32 = (*dest.add(x * 2 - 2)).into();
            let mut m1: u32 = (*dest.add(x * 2 - 1)).into();
            let mut m2: u32 = (*dest.add(x * 2)).into();
            let m3: u32 = (*dest.add(x * 2 + 1)).into();
            let m4: u32 = (*dest.add(x * 2 + 2)).into();
            let m5: u32 = (*dest.add(x * 2 + 3)).into();

            m2 = (m2 + m3) * 10;
            m1 = (m1 + m4) * 5;
            m0 += m5 + m2 + m1 + 16;
            m0 >>= 5;

            *dest.add(x) = T::from_u32_or_max_value(m0);
        }

        *dest = T::from_u32_or_max_value(src0);

        // Special case end of line
        if dest_width.get() > 1 {
            let x = dest_width.get() - 1;
            let a: u32 = (*dest.add(x * 2)).into();
            let b: u32 = (*dest.add(x * 2 + 1)).into();
            *dest.add(x) = T::from_u32_or_max_value((a + b + 1) / 2);
        }

        dest = dest.add(dest_pitch.get());
    }
}
