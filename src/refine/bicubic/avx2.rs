#![allow(clippy::undocumented_unsafe_blocks)]

use std::num::{NonZeroU8, NonZeroUsize};

use crate::util::Pixel;

/// Performs horizontal bicubic interpolation for sub-pixel motion estimation refinement.
///
/// This function applies bicubic interpolation horizontally to create sub-pixel samples
/// between existing pixels. Bicubic interpolation uses a 4-tap kernel that considers
/// 4 horizontal neighbors, providing smooth and high-quality interpolation suitable
/// for motion estimation with sub-pixel accuracy.
///
/// Edge pixels use simple averaging due to insufficient neighbors for the full kernel.
///
/// # Parameters
/// - `src`: Source image buffer
/// - `dest`: Destination buffer for interpolated results
/// - `pitch`: Number of pixels per row in both buffers
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
/// - `bits_per_sample`: Bit depth of the pixel format for clamping
#[target_feature(enable = "avx2")]
pub unsafe fn refine_horizontal_bicubic<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    debug_assert!(
        bits_per_sample.get() as usize > (size_of::<T>() - 1) * 8
            && (bits_per_sample.get() as usize <= size_of::<T>() * 8)
    );

    match size_of::<T>() {
        1 => unsafe {
            refine_horizontal_bicubic_u8(
                src.as_ptr() as *const u8,
                dest.as_mut_ptr() as *mut u8,
                pitch,
                width,
                height,
                bits_per_sample,
            )
        },
        2 => unsafe {
            refine_horizontal_bicubic_u16(
                src.as_ptr() as *const u16,
                dest.as_mut_ptr() as *mut u16,
                pitch,
                width,
                height,
                bits_per_sample,
            )
        },
        _ => unreachable!(),
    }
}

/// Performs vertical bicubic interpolation for sub-pixel motion estimation refinement.
///
/// This function applies bicubic interpolation vertically to create sub-pixel samples
/// between existing pixels. Bicubic interpolation uses a 4-tap kernel that considers
/// 4 vertical neighbors, providing smooth and high-quality interpolation suitable
/// for motion estimation with sub-pixel accuracy.
///
/// Edge rows use simple averaging due to insufficient neighbors for the full kernel,
/// and the last row is copied directly from the source.
///
/// # Parameters
/// - `src`: Source image buffer
/// - `dest`: Destination buffer for interpolated results
/// - `pitch`: Number of pixels per row in both buffers
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
/// - `bits_per_sample`: Bit depth of the pixel format for clamping
#[target_feature(enable = "avx2")]
pub unsafe fn refine_vertical_bicubic<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    debug_assert!(
        bits_per_sample.get() as usize > (size_of::<T>() - 1) * 8
            && (bits_per_sample.get() as usize <= size_of::<T>() * 8)
    );

    match size_of::<T>() {
        1 => unsafe {
            refine_vertical_bicubic_u8(
                src.as_ptr() as *const u8,
                dest.as_mut_ptr() as *mut u8,
                pitch,
                width,
                height,
                bits_per_sample,
            )
        },
        2 => unsafe {
            refine_vertical_bicubic_u16(
                src.as_ptr() as *const u16,
                dest.as_mut_ptr() as *mut u16,
                pitch,
                width,
                height,
                bits_per_sample,
            )
        },
        _ => unreachable!(),
    }
}

#[target_feature(enable = "avx2")]
unsafe fn refine_horizontal_bicubic_u8(
    src: *const u8,
    dest: *mut u8,
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    let pixel_max = (1u16 << bits_per_sample.get()) - 1;
    let width_val = width.get();
    let pitch_val = pitch.get();

    for j in 0..height.get() {
        let row_offset = j * pitch_val;
        let src_row = src.add(row_offset);
        let dest_row = dest.add(row_offset);

        // First pixel: linear interpolation
        let a = *src_row.add(0) as u16;
        let b = *src_row.add(1) as u16;
        *dest_row.add(0) = ((a + b + 1) / 2) as u8;

        // Handle middle pixels individually - SIMD implementation would be more complex
        // and needs careful boundary checking
        for i in 1..(width_val - 3) {
            let a = *src_row.add(i - 1) as i16;
            let b = *src_row.add(i) as i16;
            let c = *src_row.add(i + 1) as i16;
            let d = *src_row.add(i + 2) as i16;
            let result = (-(a + d) + (b + c) * 9 + 8) >> 4;
            *dest_row.add(i) = std::cmp::min(pixel_max, std::cmp::max(0, result) as u16) as u8;
        }

        // Second-to-last pixels: linear interpolation
        for i in (width_val - 3)..(width_val - 1) {
            let a = *src_row.add(i) as u16;
            let b = *src_row.add(i + 1) as u16;
            *dest_row.add(i) = ((a + b + 1) / 2) as u8;
        }

        // Last pixel: copy
        *dest_row.add(width_val - 1) = *src_row.add(width_val - 1);
    }
}

#[target_feature(enable = "avx2")]
unsafe fn refine_horizontal_bicubic_u16(
    src: *const u16,
    dest: *mut u16,
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    let pixel_max = (1u32 << bits_per_sample.get()) - 1;
    let width_val = width.get();
    let pitch_val = pitch.get();

    for j in 0..height.get() {
        let row_offset = j * pitch_val;
        let src_row = src.add(row_offset);
        let dest_row = dest.add(row_offset);

        // First pixel: linear interpolation
        let a = *src_row.add(0) as u32;
        let b = *src_row.add(1) as u32;
        *dest_row.add(0) = ((a + b + 1) / 2) as u16;

        // Handle middle pixels individually
        for i in 1..(width_val - 3) {
            let a = *src_row.add(i - 1) as i32;
            let b = *src_row.add(i) as i32;
            let c = *src_row.add(i + 1) as i32;
            let d = *src_row.add(i + 2) as i32;
            let result = (-(a + d) + (b + c) * 9 + 8) >> 4;
            *dest_row.add(i) = std::cmp::min(pixel_max, std::cmp::max(0, result) as u32) as u16;
        }

        // Second-to-last pixels: linear interpolation
        for i in (width_val - 3)..(width_val - 1) {
            let a = *src_row.add(i) as u32;
            let b = *src_row.add(i + 1) as u32;
            *dest_row.add(i) = ((a + b + 1) / 2) as u16;
        }

        // Last pixel: copy
        *dest_row.add(width_val - 1) = *src_row.add(width_val - 1);
    }
}

#[target_feature(enable = "avx2")]
unsafe fn refine_vertical_bicubic_u8(
    src: *const u8,
    dest: *mut u8,
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    let pixel_max = (1u16 << bits_per_sample.get()) - 1;
    let width_val = width.get();
    let pitch_val = pitch.get();
    let height_val = height.get();

    // First row: linear interpolation
    for i in 0..width_val {
        let a = *src.add(i) as u16;
        let b = *src.add(i + pitch_val) as u16;
        *dest.add(i) = ((a + b + 1) / 2) as u8;
    }

    // Middle rows: bicubic interpolation
    for j in 1..(height_val - 3) {
        let offset = j * pitch_val;

        for i in 0..width_val {
            let a = *src.add(offset + i - pitch_val) as i16;
            let b = *src.add(offset + i) as i16;
            let c = *src.add(offset + i + pitch_val) as i16;
            let d = *src.add(offset + i + pitch_val * 2) as i16;
            let result = (-(a + d) + (b + c) * 9 + 8) >> 4;
            *dest.add(offset + i) = std::cmp::min(pixel_max, std::cmp::max(0, result) as u16) as u8;
        }
    }

    // Second-to-last rows: linear interpolation
    for j in (height_val - 3)..(height_val - 1) {
        let offset = j * pitch_val;

        for i in 0..width_val {
            let a = *src.add(offset + i) as u16;
            let b = *src.add(offset + i + pitch_val) as u16;
            *dest.add(offset + i) = ((a + b + 1) / 2) as u8;
        }
    }

    // Last row: copy
    let last_offset = (height_val - 1) * pitch_val;
    std::ptr::copy_nonoverlapping(src.add(last_offset), dest.add(last_offset), width_val);
}

#[target_feature(enable = "avx2")]
unsafe fn refine_vertical_bicubic_u16(
    src: *const u16,
    dest: *mut u16,
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    let pixel_max = (1u32 << bits_per_sample.get()) - 1;
    let width_val = width.get();
    let pitch_val = pitch.get();
    let height_val = height.get();

    // First row: linear interpolation
    for i in 0..width_val {
        let a = *src.add(i) as u32;
        let b = *src.add(i + pitch_val) as u32;
        *dest.add(i) = ((a + b + 1) / 2) as u16;
    }

    // Middle rows: bicubic interpolation
    for j in 1..(height_val - 3) {
        let offset = j * pitch_val;

        for i in 0..width_val {
            let a = *src.add(offset + i - pitch_val) as i32;
            let b = *src.add(offset + i) as i32;
            let c = *src.add(offset + i + pitch_val) as i32;
            let d = *src.add(offset + i + pitch_val * 2) as i32;
            let result = (-(a + d) + (b + c) * 9 + 8) >> 4;
            *dest.add(offset + i) =
                std::cmp::min(pixel_max, std::cmp::max(0, result) as u32) as u16;
        }
    }

    // Second-to-last rows: linear interpolation
    for j in (height_val - 3)..(height_val - 1) {
        let offset = j * pitch_val;

        for i in 0..width_val {
            let a = *src.add(offset + i) as u32;
            let b = *src.add(offset + i + pitch_val) as u32;
            *dest.add(offset + i) = ((a + b + 1) / 2) as u16;
        }
    }

    // Last row: copy
    let last_offset = (height_val - 1) * pitch_val;
    std::ptr::copy_nonoverlapping(src.add(last_offset), dest.add(last_offset), width_val);
}
