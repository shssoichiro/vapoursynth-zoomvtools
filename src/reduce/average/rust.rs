use std::num::NonZeroUsize;

use crate::util::Pixel;

pub(super) fn reduce_average<T: Pixel>(
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
                let a: u32 = (*src.add(x * 2)).to_u32().expect("fits in u32");
                let b: u32 = (*src.add(x * 2 + 1)).to_u32().expect("fits in u32");
                let c: u32 = (*src.add(x * 2 + src_pitch.get()))
                    .to_u32()
                    .expect("fits in u32");
                let d: u32 = (*src.add(x * 2 + src_pitch.get() + 1))
                    .to_u32()
                    .expect("fits in u32");

                // Calculate average with proper rounding
                *dest.add(x) = T::from_u32_or_max_value((a + b + c + d + 2) / 4);
            }
            dest = dest.add(dest_pitch.get());
            src = src.add(src_pitch.get() * 2);
        }
    }
}
