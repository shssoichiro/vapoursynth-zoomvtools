use std::num::NonZeroUsize;

use crate::util::Pixel;

/// Downscales an image by 2x using triangle (linear) filtering.
///
/// This function reduces both the width and height of the source image by half
/// using a two-pass triangle filtering approach. First, vertical filtering is
/// applied to reduce the height, then horizontal filtering is applied in-place
/// to reduce the width. Triangle filtering uses a simple linear weighting
/// scheme (1/4, 1/2, 1/4) that provides good anti-aliasing while being
/// computationally efficient.
///
/// The triangle filter is particularly effective at reducing aliasing artifacts
/// when downscaling images with fine details or high-frequency content.
///
/// # Parameters
/// - `dest`: Destination buffer to store the downscaled image
/// - `src`: Source image buffer to downscale
/// - `dest_pitch`: Number of pixels per row in the destination buffer
/// - `src_pitch`: Number of pixels per row in the source buffer
/// - `dest_width`: Width of the destination image (half of source width)
/// - `dest_height`: Height of the destination image (half of source height)
pub fn reduce_triangle<T: Pixel>(
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
        reduce_triangle_vertical(
            dest,
            src,
            dest_pitch,
            src_pitch,
            dest_width.saturating_mul(NonZeroUsize::new_unchecked(2)),
            dest_height,
        );
        reduce_triangle_horizontal_inplace(dest, dest_pitch, dest_width, dest_height);
    }
}

/// Applies vertical triangle filtering to reduce image height by 2x.
///
/// This function performs the first pass of triangle downscaling by filtering
/// vertically. The first row uses simple averaging of two rows, while subsequent
/// rows use a 3-tap triangle filter with weights (1/4, 1/2, 1/4) applied to
/// three consecutive source rows.
unsafe fn reduce_triangle_vertical<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let dest = dest.as_mut_ptr();
    let src = src.as_ptr();

    let width_usize = dest_width.get();
    let height_usize = dest_height.get();
    let src_pitch_usize = src_pitch.get();
    let dest_pitch_usize = dest_pitch.get();

    // Process first output row: average of first two input rows
    for x in 0..width_usize {
        let a: u32 = (*src.add(x)).into();
        let b: u32 = (*src.add(x + src_pitch_usize)).into();
        *dest.add(x) = T::from_u32_or_max_value((a + b + 1) / 2);
    }

    // Process remaining output rows: 1/4, 1/2, 1/4 filter
    for y in 1..height_usize {
        let dest_offset = y * dest_pitch_usize;
        let src_offset = y * 2 * src_pitch_usize; // Each output row corresponds to 2 input rows

        for x in 0..width_usize {
            // Access three consecutive input rows for the 1/4, 1/2, 1/4 filter
            let a: u32 = (*src.add(src_offset + x - src_pitch_usize)).into(); // Previous row
            let b: u32 = (*src.add(src_offset + x)).into(); // Current row
            let c: u32 = (*src.add(src_offset + x + src_pitch_usize)).into(); // Next row
            *dest.add(dest_offset + x) = T::from_u32_or_max_value((a + b * 2 + c + 2) / 4);
        }
    }
}

/// Applies horizontal triangle filtering in-place to reduce image width by 2x.
///
/// This function performs the second pass of triangle downscaling by filtering
/// horizontally on the already vertically-filtered data. It modifies the buffer
/// in-place, using a 3-tap triangle filter with weights (1/4, 1/2, 1/4).
/// The first column uses simple averaging of two pixels.
unsafe fn reduce_triangle_horizontal_inplace<T: Pixel>(
    dest: &mut [T],
    dest_pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    let mut dest = dest.as_mut_ptr();

    for _y in 0..height.get() {
        let x = 0;
        let mut a: u32;
        let mut b: u32 = (*dest.add(x * 2)).into();
        let mut c: u32 = (*dest.add(x * 2 + 1)).into();
        let src0 = (b + c + 1) / 2;

        for x in 1..width.get() {
            a = (*dest.add(x * 2 - 1)).into();
            b = (*dest.add(x * 2)).into();
            c = (*dest.add(x * 2 + 1)).into();

            *dest.add(x) = T::from_u32_or_max_value((a + b * 2 + c + 2) / 4);
        }
        *dest = T::from_u32_or_max_value(src0);

        dest = dest.add(dest_pitch.get());
    }
}
