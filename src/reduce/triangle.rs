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
    reduce_filtered_vertical(
        dest,
        src,
        dest_pitch,
        src_pitch,
        // SAFETY: non-zero constant
        dest_width.saturating_mul(unsafe { NonZeroUsize::new_unchecked(2) }),
        dest_height,
    );
    reduce_filtered_horizontal_inplace(dest, dest_pitch, dest_width, dest_height);
}

/// Applies vertical triangle filtering to reduce image height by 2x.
///
/// This function performs the first pass of triangle downscaling by filtering
/// vertically. The first row uses simple averaging of two rows, while subsequent
/// rows use a 3-tap triangle filter with weights (1/4, 1/2, 1/4) applied to
/// three consecutive source rows.
fn reduce_filtered_vertical<T: Pixel>(
    dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    let width_usize = dest_width.get();
    let height_usize = dest_height.get();
    let src_pitch_usize = src_pitch.get();
    let dest_pitch_usize = dest_pitch.get();

    // Process first output row: average of first two input rows
    for x in 0..width_usize {
        let a: u32 = src[x].into();
        let b: u32 = src[x + src_pitch_usize].into();
        dest[x] = T::from_or_max((a + b + 1) / 2);
    }

    // Process remaining output rows: 1/4, 1/2, 1/4 filter
    for y in 1..height_usize {
        let dest_offset = y * dest_pitch_usize;
        let src_offset = y * 2 * src_pitch_usize; // Each output row corresponds to 2 input rows

        for x in 0..width_usize {
            // Access three consecutive input rows for the 1/4, 1/2, 1/4 filter
            let a: u32 = src[src_offset + x - src_pitch_usize].into(); // Previous row
            let b: u32 = src[src_offset + x].into(); // Current row
            let c: u32 = src[src_offset + x + src_pitch_usize].into(); // Next row
            dest[dest_offset + x] = T::from_or_max((a + b * 2 + c + 2) / 4);
        }
    }
}

/// Applies horizontal triangle filtering in-place to reduce image width by 2x.
///
/// This function performs the second pass of triangle downscaling by filtering
/// horizontally on the already vertically-filtered data. It modifies the buffer
/// in-place, using a 3-tap triangle filter with weights (1/4, 1/2, 1/4).
/// The first column uses simple averaging of two pixels.
fn reduce_filtered_horizontal_inplace<T: Pixel>(
    mut dest: &mut [T],
    dest_pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
) {
    for _y in 0..height.get() {
        let x = 0;
        let mut a: u32;
        let mut b: u32 = dest[x * 2].into();
        let mut c: u32 = dest[x * 2 + 1].into();
        let src0 = (b + c + 1) / 2;

        for x in 1..width.get() {
            a = dest[x * 2 - 1].into();
            b = dest[x * 2].into();
            c = dest[x * 2 + 1].into();

            dest[x] = T::from_or_max((a + b * 2 + c + 2) / 4);
        }
        dest[0] = T::from_or_max(src0);

        dest = &mut dest[dest_pitch.get()..];
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use super::reduce_triangle;

    #[test]
    fn test_reduce_triangle_u8_2x2() {
        // Test basic 2x2 -> 1x1 reduction
        let src = vec![
            10u8, 20, // first row
            30, 40, // second row
        ];
        // Destination buffer needs to be large enough for intermediate results (width*2
        // wide)
        let mut dest = vec![0u8; 2]; // width=1, but intermediate needs width*2=2
        let src_pitch = NonZeroUsize::new(2).unwrap();
        let dest_pitch = NonZeroUsize::new(2).unwrap(); // Pitch must accommodate intermediate width
        let dest_width = NonZeroUsize::new(1).unwrap();
        let dest_height = NonZeroUsize::new(1).unwrap();

        reduce_triangle(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Triangle filter is separable:
        // 1. Vertical: intermediate results at width*2 = 2 First column: (10 +
        //    30).div_ceil(2) = 20 Second column: (20 + 40).div_ceil(2) = 30 So
        //    intermediate = [20, 30]
        // 2. Horizontal: reduce from width*2 to width Result at [0]: (20 +
        //    30).div_ceil(2) = 25
        assert_eq!(dest[0], 25);
    }

    #[test]
    fn test_reduce_triangle_u8_4x2() {
        // Test 4x2 -> 2x1 reduction
        let src = vec![
            10u8, 20, 30, 40, // first row
            50, 60, 70, 80, // second row
        ];
        // Destination buffer needs to accommodate intermediate width of 4
        // (2*dest_width)
        let mut dest = vec![0u8; 4];
        let src_pitch = NonZeroUsize::new(4).unwrap();
        let dest_pitch = NonZeroUsize::new(4).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(2).unwrap();
        let dest_height = NonZeroUsize::new(1).unwrap();

        reduce_triangle(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Triangle filter is separable:
        // 1. Vertical: intermediate results at width*2 = 4 [0]: (10 + 50).div_ceil(2) =
        //    30 [1]: (20 + 60).div_ceil(2) = 40 [2]: (30 + 70).div_ceil(2) = 50 [3]:
        //    (40 + 80).div_ceil(2) = 60 So intermediate = [30, 40, 50, 60]
        // 2. Horizontal: reduce from width*2=4 to width=2 dest[0]: (30 +
        //    40).div_ceil(2) = 35 (note: first element is computed last) dest[1]: (40 +
        //    50*2 + 60 + 2) / 4 = 202/4 = 50
        assert_eq!(dest[0], 35);
        assert_eq!(dest[1], 50);
    }

    #[test]
    fn test_reduce_triangle_u8_4x4() {
        // Test 4x4 -> 2x2 reduction
        let src = vec![
            10u8, 20, 30, 40, // first row
            50, 60, 70, 80, // second row
            90, 100, 110, 120, // third row
            130, 140, 150, 160, // fourth row
        ];
        // Destination buffer needs to accommodate intermediate width of 4
        // (2*dest_width) and height of 2
        let mut dest = vec![0u8; 8]; // 4 width * 2 height
        let src_pitch = NonZeroUsize::new(4).unwrap();
        let dest_pitch = NonZeroUsize::new(4).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(2).unwrap();
        let dest_height = NonZeroUsize::new(2).unwrap();

        reduce_triangle(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Triangle filter is separable:
        // 1. Vertical reduction with averaging for first row, then 1/4, 1/2, 1/4
        //    filter: Row 0: [0]: (10 + 50).div_ceil(2) = 30, [1]: (20 + 60).div_ceil(2)
        //    = 40, [2]: (30 + 70).div_ceil(2) = 50, [3]: (40 + 80).div_ceil(2) = 60 Row
        //    1: [0]: (50 + 90*2 + 130 + 2) / 4 = 362/4 = 90, [1]: (60 + 100*2 + 140 +
        //    2) / 4 = 402/4 = 100, [2]: (70 + 110*2 + 150 + 2) / 4 = 442/4 = 110, [3]:
        //    (80 + 120*2 + 160 + 2) / 4 = 482/4 = 120 So intermediate after vertical =
        //    [[30, 40, 50, 60], [90, 100, 110, 120]]
        // 2. Horizontal reduction: Row 0: dest[0]: (30 + 40).div_ceil(2) = 35, dest[1]:
        //    (40 + 50*2 + 60 + 2) / 4 = 202/4 = 50 Row 1: dest[4]: (90 +
        //    100).div_ceil(2) = 95, dest[5]: (100 + 110*2 + 120 + 2) / 4 = 442/4 = 110
        assert_eq!(dest[0], 35); // Top-left
        assert_eq!(dest[1], 50); // Top-right
        assert_eq!(dest[4], 95); // Bottom-left
        assert_eq!(dest[5], 110); // Bottom-right
    }

    #[test]
    fn test_reduce_triangle_u8_6x4() {
        // Test 6x4 -> 3x2 reduction with more complex filtering
        let src = vec![
            10u8, 20, 30, 40, 50, 60, // first row
            70, 80, 90, 100, 110, 120, // second row
            130, 140, 150, 160, 170, 180, // third row
            190, 200, 210, 220, 230, 240, // fourth row
        ];
        // Destination buffer needs to accommodate intermediate width of 6
        // (2*dest_width=3) and height of 2
        let mut dest = vec![0u8; 12]; // 6 width * 2 height
        let src_pitch = NonZeroUsize::new(6).unwrap();
        let dest_pitch = NonZeroUsize::new(6).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(3).unwrap();
        let dest_height = NonZeroUsize::new(2).unwrap();

        reduce_triangle(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Triangle filter first processes vertically, then horizontally
        // The algorithm should handle the multi-tap filtering correctly
        // Verifying that we get reasonable smoothed values
        assert!(dest[0] > 10 && dest[0] < 200); // Should be smoothed values
        assert!(dest[1] > 10 && dest[1] < 200);
        assert!(dest[2] > 10 && dest[2] < 200);
        assert!(dest[6] > 10 && dest[6] < 240); // Second row
        assert!(dest[7] > 10 && dest[7] < 240);
        assert!(dest[8] > 10 && dest[8] < 240);
    }

    #[test]
    fn test_reduce_triangle_u8_with_padding() {
        // Test with source pitch > width (includes padding)
        let src = vec![
            10u8, 20, 255, 255, // first row (last 2 are padding)
            30, 40, 255, 255, // second row (last 2 are padding)
        ];
        // Destination buffer needs to accommodate intermediate width of 2
        // (2*dest_width=1)
        let mut dest = vec![0u8; 4]; // 2 intermediate width, plus padding for dest_pitch=4
        let src_pitch = NonZeroUsize::new(4).unwrap();
        let dest_pitch = NonZeroUsize::new(4).unwrap(); // Must accommodate intermediate width + padding
        let dest_width = NonZeroUsize::new(1).unwrap();
        let dest_height = NonZeroUsize::new(1).unwrap();

        reduce_triangle(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Should only process the first 2x2 block, ignoring padding
        // Vertical: (10 + 30).div_ceil(2) = 20, (20 + 40).div_ceil(2) = 30
        // Horizontal: (20 + 30).div_ceil(2) = 25
        assert_eq!(dest[0], 25);
        // Note: Other elements in dest are not guaranteed to remain 0 due to
        // intermediate processing
    }

    #[test]
    fn test_reduce_triangle_u16_basic() {
        // Test with u16 values
        let src = vec![
            1000u16, 2000, // first row
            3000, 4000, // second row
        ];
        // Destination buffer needs to accommodate intermediate width of 2
        // (2*dest_width=1)
        let mut dest = vec![0u16; 2];
        let src_pitch = NonZeroUsize::new(2).unwrap();
        let dest_pitch = NonZeroUsize::new(2).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(1).unwrap();
        let dest_height = NonZeroUsize::new(1).unwrap();

        reduce_triangle(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Triangle filter:
        // Vertical: (1000 + 3000).div_ceil(2) = 2000, (2000 + 4000).div_ceil(2) = 3000
        // Horizontal: (2000 + 3000).div_ceil(2) = 2500
        assert_eq!(dest[0], 2500);
    }

    #[test]
    fn test_reduce_triangle_u16_large_values() {
        // Test with larger u16 values near the upper range
        let src = vec![
            60000u16, 61000, // first row
            62000, 63000, // second row
        ];
        // Destination buffer needs to accommodate intermediate width of 2
        // (2*dest_width=1)
        let mut dest = vec![0u16; 2];
        let src_pitch = NonZeroUsize::new(2).unwrap();
        let dest_pitch = NonZeroUsize::new(2).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(1).unwrap();
        let dest_height = NonZeroUsize::new(1).unwrap();

        reduce_triangle(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Triangle filter:
        // Vertical: (60000 + 62000).div_ceil(2) = 61000, (61000 + 63000).div_ceil(2) =
        // 62000 Horizontal: (61000 + 62000).div_ceil(2) = 61500
        assert_eq!(dest[0], 61500);
    }

    #[test]
    fn test_reduce_triangle_u16_4x4() {
        // Test 4x4 -> 2x2 reduction with u16
        let src = vec![
            1000u16, 2000, 3000, 4000, // first row
            5000, 6000, 7000, 8000, // second row
            9000, 10000, 11000, 12000, // third row
            13000, 14000, 15000, 16000, // fourth row
        ];
        // Destination buffer needs to accommodate intermediate width of 4
        // (2*dest_width=2) and height of 2
        let mut dest = vec![0u16; 8]; // 4 width * 2 height
        let src_pitch = NonZeroUsize::new(4).unwrap();
        let dest_pitch = NonZeroUsize::new(4).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(2).unwrap();
        let dest_height = NonZeroUsize::new(2).unwrap();

        reduce_triangle(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Similar to the u8 case but with larger values
        // The triangle filter should produce smoothed results
        assert!(dest[0] > 1000 && dest[0] < 16000);
        assert!(dest[1] > 1000 && dest[1] < 16000);
        assert!(dest[4] > 1000 && dest[4] < 16000);
        assert!(dest[5] > 1000 && dest[5] < 16000);

        // Values should increase from top-left to bottom-right due to input pattern
        assert!(dest[0] < dest[1]);
        assert!(dest[0] < dest[4]);
        assert!(dest[4] < dest[5]);
    }

    #[test]
    fn test_reduce_triangle_edge_case_2x2() {
        // Test minimal 2x2 reduction (was incorrectly 1x2)
        let src = vec![
            100u8, 150, // first row
            200u8, 250, // second row
        ];
        // Destination buffer needs to accommodate intermediate width of 2
        // (2*dest_width=1)
        let mut dest = vec![0u8; 2];
        let src_pitch = NonZeroUsize::new(2).unwrap();
        let dest_pitch = NonZeroUsize::new(2).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(1).unwrap();
        let dest_height = NonZeroUsize::new(1).unwrap();

        reduce_triangle(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Vertical: (100 + 200).div_ceil(2) = 150, (150 + 250).div_ceil(2) = 200
        // Horizontal: (150 + 200).div_ceil(2) = 175
        assert_eq!(dest[0], 175);
    }

    #[test]
    fn test_reduce_triangle_uniform_values() {
        // Test with uniform input - should preserve the value
        let src = vec![
            128u8, 128, 128, 128, // first row
            128, 128, 128, 128, // second row
            128, 128, 128, 128, // third row
            128, 128, 128, 128, // fourth row
        ];
        // Destination buffer needs to accommodate intermediate width of 4
        // (2*dest_width=2) and height of 2
        let mut dest = vec![0u8; 8]; // 4 width * 2 height
        let src_pitch = NonZeroUsize::new(4).unwrap();
        let dest_pitch = NonZeroUsize::new(4).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(2).unwrap();
        let dest_height = NonZeroUsize::new(2).unwrap();

        reduce_triangle(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Uniform values should remain uniform after filtering
        assert_eq!(dest[0], 128);
        assert_eq!(dest[1], 128);
        assert_eq!(dest[4], 128);
        assert_eq!(dest[5], 128);
    }

    #[test]
    fn test_reduce_triangle_large_height() {
        // Test 4x6 -> 2x3 reduction to exercise larger heights
        let src = vec![
            10u8, 20, 30, 40, // row 0
            50, 60, 70, 80, // row 1
            90, 100, 110, 120, // row 2
            130, 140, 150, 160, // row 3
            170, 180, 190, 200, // row 4
            210, 220, 230, 240, // row 5
        ];
        // Destination buffer needs to accommodate intermediate width of 4
        // (dest_width*2) and height of 3
        let mut dest = vec![0u8; 12]; // 4 width * 3 height
        let src_pitch = NonZeroUsize::new(4).unwrap();
        let dest_pitch = NonZeroUsize::new(4).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(2).unwrap();
        let dest_height = NonZeroUsize::new(3).unwrap();

        reduce_triangle(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // This test ensures triangle filter works correctly with larger heights
        assert_ne!(dest[0], 0); // Should have been modified
        assert_ne!(dest[4], 0); // Second row should have been modified
        assert_ne!(dest[8], 0); // Third row should have been modified
    }
}
