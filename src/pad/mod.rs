#[cfg(test)]
mod tests;

use std::num::NonZeroUsize;

use crate::util::Pixel;

/// Pads a reference frame by extending edge pixels to fill the padding areas.
///
/// This function takes a frame and extends its borders by replicating edge pixels
/// to create padding around the original image. This is commonly used in video
/// processing for motion estimation and filtering operations where algorithms
/// need to access pixels beyond the frame boundaries.
///
/// The padding is applied in all directions:
/// - Corners are filled with the nearest corner pixel value
/// - Top/bottom edges are filled by extending the first/last row
/// - Left/right edges are filled by extending the first/last column
///
/// # Parameters
/// - `offset`: Starting byte offset in the destination buffer where the padded frame begins
/// - `ref_pitch`: Number of pixels per row in the destination buffer (including padding)
/// - `hpad`: Horizontal padding amount (pixels to add on left and right sides)
/// - `vpad`: Vertical padding amount (pixels to add on top and bottom)
/// - `width`: Width of the original frame in pixels (excluding padding)
/// - `height`: Height of the original frame in pixels (excluding padding)
/// - `dest`: Destination buffer containing the frame data to be padded
pub fn pad_reference_frame<T: Pixel>(
    offset: usize,
    ref_pitch: NonZeroUsize,
    hpad: usize,
    vpad: usize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    dest: &mut [T],
) {
    let pfoff = offset + vpad * ref_pitch.get() + hpad;

    // Up-Left
    pad_corner(offset, dest[pfoff], hpad, vpad, ref_pitch, dest);
    // Up-Right
    pad_corner(
        offset + hpad + width.get(),
        dest[pfoff + width.get() - 1],
        hpad,
        vpad,
        ref_pitch,
        dest,
    );
    // Down-Left
    pad_corner(
        offset + (vpad + height.get()) * ref_pitch.get(),
        dest[pfoff + (height.get() - 1) * ref_pitch.get()],
        hpad,
        vpad,
        ref_pitch,
        dest,
    );
    // Down-Right
    pad_corner(
        offset + hpad + width.get() + (vpad + height.get()) * ref_pitch.get(),
        dest[pfoff + (height.get() - 1) * ref_pitch.get() + width.get() - 1],
        hpad,
        vpad,
        ref_pitch,
        dest,
    );

    // Up
    for i in 0..width.get() {
        let value = dest[pfoff + i];
        let mut poff = offset + hpad + i;
        for _j in 0..vpad {
            dest[poff] = value;
            poff += ref_pitch.get();
        }
    }

    // Left
    for i in 0..height.get() {
        let value = dest[pfoff + i * ref_pitch.get()];
        let poff = offset + (vpad + i) * ref_pitch.get();
        dest[poff..poff + hpad].fill(value);
    }

    // Right
    for i in 0..height.get() {
        let value = dest[pfoff + i * ref_pitch.get() + width.get() - 1];
        let poff = offset + (vpad + i) * ref_pitch.get() + width.get() + hpad;
        dest[poff..poff + hpad].fill(value);
    }

    // Down
    for i in 0..width.get() {
        let value = dest[pfoff + i + (height.get() - 1) * ref_pitch.get()];
        let mut poff = offset + hpad + i + (height.get() + vpad) * ref_pitch.get();
        for _j in 0..vpad {
            dest[poff] = value;
            poff += ref_pitch.get();
        }
    }
}

fn pad_corner<T: Pixel>(
    mut offset: usize,
    val: T,
    hpad: usize,
    vpad: usize,
    ref_pitch: NonZeroUsize,
    dest: &mut [T],
) {
    for _i in 0..vpad {
        dest[offset..offset + hpad].fill(val);
        offset += ref_pitch.get();
    }
}
