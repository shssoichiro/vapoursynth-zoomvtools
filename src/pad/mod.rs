#[cfg(test)]
mod tests;

use std::num::NonZeroUsize;

use crate::util::Pixel;

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
