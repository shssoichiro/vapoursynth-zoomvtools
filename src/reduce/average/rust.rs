use std::num::NonZeroUsize;

use crate::util::Pixel;

/// Downscales an image by 2x using simple averaging of 2x2 pixel blocks.
///
/// This function reduces both the width and height of the source image by half
/// by averaging each 2x2 block of pixels into a single output pixel. The averaging
/// uses proper rounding by adding 2 before dividing by 4, ensuring accurate
/// color representation in the downscaled result.
///
/// # Parameters
/// - `dest`: Destination buffer to store the downscaled image
/// - `src`: Source image buffer to downscale
/// - `dest_pitch`: Number of pixels per row in the destination buffer
/// - `src_pitch`: Number of pixels per row in the source buffer
/// - `dest_width`: Width of the destination image (half of source width)
/// - `dest_height`: Height of the destination image (half of source height)
pub fn reduce_average<T: Pixel>(
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
        let mut src = src.as_ptr();
        let mut dest = dest.as_mut_ptr();
        for _y in 0..dest_height.get() {
            for x in 0..dest_width.get() {
                // Convert to u32 for intermediate calculation to prevent overflow
                let a: u32 = (*src.add(x * 2)).into();
                let b: u32 = (*src.add(x * 2 + 1)).into();
                let c: u32 = (*src.add(x * 2 + src_pitch.get())).into();
                let d: u32 = (*src.add(x * 2 + src_pitch.get() + 1)).into();

                // Calculate average with proper rounding
                *dest.add(x) = T::from_u32_or_max_value((a + b + c + d + 2) / 4);
            }
            dest = dest.add(dest_pitch.get());
            src = src.add(src_pitch.get() * 2);
        }
    }
}
