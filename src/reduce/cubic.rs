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
    reduce_cubic_vertical(
        dest,
        src,
        dest_pitch,
        src_pitch,
        // SAFETY: non-zero constant
        dest_width.saturating_mul(unsafe { NonZeroUsize::new_unchecked(2) }),
        dest_height,
    );
    reduce_cubic_horizontal_inplace(dest, dest_pitch, dest_width, dest_height);
}

/// Applies vertical cubic filtering to reduce image height by 2x.
///
/// This function performs the first pass of cubic downscaling by filtering
/// vertically using a 6-tap filter kernel. Edge lines use simple averaging,
/// while middle lines use the full cubic filter that considers 6 vertical
/// neighbors with optimized weights for high-quality downscaling.
fn reduce_cubic_vertical<T: Pixel>(
    mut dest: &mut [T],
    src: &[T],
    dest_pitch: NonZeroUsize,
    src_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    // Special case for first line
    for x in 0..dest_width.get() {
        let a: u32 = src[x].into();
        let b: u32 = src[x + src_pitch.get()].into();
        dest[x] = T::from_or_max((a + b + 1) / 2);
    }
    dest = &mut dest[dest_pitch.get()..];

    // Middle lines
    for y in 1..(dest_height.get() - 1) {
        let src_row_offset = y * 2 * src_pitch.get();
        for x in 0..dest_width.get() {
            let mut m0: u32 = src[src_row_offset + x - src_pitch.get() * 2].into();
            let mut m1: u32 = src[src_row_offset + x - src_pitch.get()].into();
            let mut m2: u32 = src[src_row_offset + x].into();
            let m3: u32 = src[src_row_offset + x + src_pitch.get()].into();
            let m4: u32 = src[src_row_offset + x + src_pitch.get() * 2].into();
            let m5: u32 = src[src_row_offset + x + src_pitch.get() * 3].into();

            m2 = (m2 + m3) * 10;
            m1 = (m1 + m4) * 5;
            m0 += m5 + m2 + m1 + 16;
            m0 >>= 5;

            dest[x] = T::from_or_max(m0);
        }
        dest = &mut dest[dest_pitch.get()..];
    }

    // Special case for last line
    if dest_height.get() > 1 {
        let src_row_offset = (dest_height.get() - 1) * 2 * src_pitch.get();
        for x in 0..dest_width.get() {
            let a: u32 = src[src_row_offset + x].into();
            let b: u32 = src[src_row_offset + x + src_pitch.get()].into();
            dest[x] = T::from_or_max((a + b + 1) / 2);
        }
    }
}

/// Applies horizontal cubic filtering in-place to reduce image width by 2x.
///
/// This function performs the second pass of cubic downscaling by filtering
/// horizontally on the already vertically-filtered data. It modifies the buffer
/// in-place, using the same 6-tap cubic filter kernel horizontally.
/// Edge columns use simple averaging, while middle columns use the full filter.
fn reduce_cubic_horizontal_inplace<T: Pixel>(
    mut dest: &mut [T],
    dest_pitch: NonZeroUsize,
    dest_width: NonZeroUsize,
    dest_height: NonZeroUsize,
) {
    for _y in 0..dest_height.get() {
        // Special case start of line
        let a: u32 = dest[0].into();
        let b: u32 = dest[1].into();
        let src0 = (a + b + 1) / 2;

        // Middle of line
        for x in 1..(dest_width.get() - 1) {
            let mut m0: u32 = dest[x * 2 - 2].into();
            let mut m1: u32 = dest[x * 2 - 1].into();
            let mut m2: u32 = dest[x * 2].into();
            let m3: u32 = dest[x * 2 + 1].into();
            let m4: u32 = dest[x * 2 + 2].into();
            let m5: u32 = dest[x * 2 + 3].into();

            m2 = (m2 + m3) * 10;
            m1 = (m1 + m4) * 5;
            m0 += m5 + m2 + m1 + 16;
            m0 >>= 5;

            dest[x] = T::from_or_max(m0);
        }

        dest[0] = T::from_or_max(src0);

        // Special case end of line
        if dest_width.get() > 1 {
            let x = dest_width.get() - 1;
            let a: u32 = dest[x * 2].into();
            let b: u32 = dest[x * 2 + 1].into();
            dest[x] = T::from_or_max((a + b + 1) / 2);
        }

        dest = &mut dest[dest_pitch.get()..];
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use super::reduce_cubic;

    #[test]
    fn test_reduce_cubic_u8_2x2() {
        // Test basic 2x2 -> 1x1 reduction
        let src = vec![
            10u8, 20, // first row
            30, 40, // second row
        ];
        // Destination buffer needs to accommodate intermediate width of 2
        // (dest_width*2)
        let mut dest = vec![0u8; 2];
        let src_pitch = NonZeroUsize::new(2).unwrap();
        let dest_pitch = NonZeroUsize::new(2).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(1).unwrap();
        let dest_height = NonZeroUsize::new(1).unwrap();

        reduce_cubic(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Cubic filter is separable:
        // 1. Vertical: For single line (height=1), uses simple averaging: (10 +
        //    30).div_ceil(2) = 20, (20 + 40).div_ceil(2) = 30 So intermediate = [20,
        //    30]
        // 2. Horizontal: For single pixel (width=1), uses simple averaging: (20 +
        //    30).div_ceil(2) = 25
        assert_eq!(dest[0], 25);
    }

    #[test]
    fn test_reduce_cubic_u8_4x2() {
        // Test 4x2 -> 2x1 reduction
        let src = vec![
            10u8, 20, 30, 40, // first row
            50, 60, 70, 80, // second row
        ];
        // Destination buffer needs to accommodate intermediate width of 4
        // (dest_width*2)
        let mut dest = vec![0u8; 4];
        let src_pitch = NonZeroUsize::new(4).unwrap();
        let dest_pitch = NonZeroUsize::new(4).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(2).unwrap();
        let dest_height = NonZeroUsize::new(1).unwrap();

        reduce_cubic(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Cubic filter is separable:
        // 1. Vertical: For single height, uses simple averaging: [0]: (10 +
        //    50).div_ceil(2) = 30, [1]: (20 + 60).div_ceil(2) = 40 [2]: (30 +
        //    70).div_ceil(2) = 50, [3]: (40 + 80).div_ceil(2) = 60 So intermediate =
        //    [30, 40, 50, 60]
        // 2. Horizontal: For width=2, edge cases use simple averaging: dest[0]: (30 +
        //    40).div_ceil(2) = 35 (start of line) dest[1]: (50 + 60).div_ceil(2) = 55
        //    (end of line)
        assert_eq!(dest[0], 35);
        assert_eq!(dest[1], 55);
    }

    #[test]
    fn test_reduce_cubic_u8_4x4() {
        // Test 4x4 -> 2x2 reduction
        let src = vec![
            10u8, 20, 30, 40, // first row
            50, 60, 70, 80, // second row
            90, 100, 110, 120, // third row
            130, 140, 150, 160, // fourth row
        ];
        // Destination buffer needs to accommodate intermediate width of 4
        // (dest_width*2) and height of 2
        let mut dest = vec![0u8; 8]; // 4 width * 2 height
        let src_pitch = NonZeroUsize::new(4).unwrap();
        let dest_pitch = NonZeroUsize::new(4).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(2).unwrap();
        let dest_height = NonZeroUsize::new(2).unwrap();

        reduce_cubic(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Cubic filter is separable with edge case handling:
        // 1. Vertical: Row 0 uses simple averaging, Row 1 uses edge case averaging Row
        //    0: [0]: (10 + 50).div_ceil(2) = 30, [1]: (20 + 60).div_ceil(2) = 40 [2]:
        //    (30 + 70).div_ceil(2) = 50, [3]: (40 + 80).div_ceil(2) = 60 Row 1: [0]:
        //    (90 + 130).div_ceil(2) = 110, [1]: (100 + 140).div_ceil(2) = 120 [2]: (110
        //    + 150).div_ceil(2) = 130, [3]: (120 + 160).div_ceil(2) = 140
        // 2. Horizontal: Uses edge case averaging for width=2 Row 0: dest[0]: (30 +
        //    40).div_ceil(2) = 35, dest[1]: (50 + 60).div_ceil(2) = 55 Row 1: dest[4]:
        //    (110 + 120).div_ceil(2) = 115, dest[5]: (130 + 140).div_ceil(2) = 135
        assert_eq!(dest[0], 35); // Top-left
        assert_eq!(dest[1], 55); // Top-right
        assert_eq!(dest[4], 115); // Bottom-left
        assert_eq!(dest[5], 135); // Bottom-right
    }

    #[test]
    fn test_reduce_cubic_u8_6x4() {
        // Test 6x4 -> 3x2 reduction with more moderate filtering
        let src = vec![
            10u8, 20, 30, 40, 50, 60, // first row
            70, 80, 90, 100, 110, 120, // second row
            130, 140, 150, 160, 170, 180, // third row
            190, 200, 210, 220, 230, 240, // fourth row
        ];
        // Destination buffer needs to accommodate intermediate width of 6
        // (dest_width*2) and height of 2
        let mut dest = vec![0u8; 12]; // 6 width * 2 height
        let src_pitch = NonZeroUsize::new(6).unwrap();
        let dest_pitch = NonZeroUsize::new(6).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(3).unwrap();
        let dest_height = NonZeroUsize::new(2).unwrap();

        reduce_cubic(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Cubic filter should produce reasonable smoothed values
        // We verify that values are reasonable and follow expected trends
        assert!(dest[0] > 10 && dest[0] < 200); // Should be smoothed values
        assert!(dest[1] > 10 && dest[1] < 200);
        assert!(dest[2] > 10 && dest[2] < 200);
        assert!(dest[6] > 10 && dest[6] < 240); // Second row
        assert!(dest[7] > 10 && dest[7] < 240);
        assert!(dest[8] > 10 && dest[8] < 240);
    }

    #[test]
    fn test_reduce_cubic_gradient() {
        // Test with a simple gradient pattern 4x4 -> 2x2
        let src = vec![
            0u8, 25, 50, 75, // first row
            50, 75, 100, 125, // second row
            100, 125, 150, 175, // third row
            150, 175, 200, 225, // fourth row
        ];
        // Destination buffer needs to accommodate intermediate width of 4
        // (dest_width*2) and height of 2
        let mut dest = vec![0u8; 8]; // 4 width * 2 height
        let src_pitch = NonZeroUsize::new(4).unwrap();
        let dest_pitch = NonZeroUsize::new(4).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(2).unwrap();
        let dest_height = NonZeroUsize::new(2).unwrap();

        reduce_cubic(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Verify gradient property is preserved
        assert!(dest[0] < dest[1]); // Left to right increase
        assert!(dest[0] < dest[4]); // Top to bottom increase
        assert!(dest[1] < dest[5]); // Diagonal increase
        assert!(dest[4] < dest[5]); // Left to right in bottom row

        // Values should be reasonable
        assert!(dest[0] > 0 && dest[0] < 100);
        assert!(dest[5] > dest[0]); // Bottom-right should be largest
    }

    #[test]
    fn test_reduce_cubic_u16_basic() {
        // Test with u16 values
        let src = vec![
            1000u16, 2000, // first row
            3000, 4000, // second row
        ];
        // Destination buffer needs to accommodate intermediate width of 2
        // (dest_width*2)
        let mut dest = vec![0u16; 2];
        let src_pitch = NonZeroUsize::new(2).unwrap();
        let dest_pitch = NonZeroUsize::new(2).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(1).unwrap();
        let dest_height = NonZeroUsize::new(1).unwrap();

        reduce_cubic(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Cubic filter with edge case handling:
        // Vertical: (1000 + 3000).div_ceil(2) = 2000, (2000 + 4000).div_ceil(2) = 3000
        // Horizontal: (2000 + 3000).div_ceil(2) = 2500
        assert_eq!(dest[0], 2500);
    }

    #[test]
    fn test_reduce_cubic_u16_large_values() {
        // Test with larger u16 values near the upper range
        let src = vec![
            60000u16, 61000, // first row
            62000, 63000, // second row
        ];
        // Destination buffer needs to accommodate intermediate width of 2
        // (dest_width*2)
        let mut dest = vec![0u16; 2];
        let src_pitch = NonZeroUsize::new(2).unwrap();
        let dest_pitch = NonZeroUsize::new(2).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(1).unwrap();
        let dest_height = NonZeroUsize::new(1).unwrap();

        reduce_cubic(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Should handle large values without overflow
        // Vertical: (60000 + 62000).div_ceil(2) = 61000, (61000 + 63000).div_ceil(2) =
        // 62000 Horizontal: (61000 + 62000).div_ceil(2) = 61500
        assert_eq!(dest[0], 61500);
    }

    #[test]
    fn test_reduce_cubic_u16_4x4() {
        // Test with u16 values in a 4x4 configuration
        let src = vec![
            1000u16, 2000, 3000, 4000, // first row
            5000, 6000, 7000, 8000, // second row
            9000, 10000, 11000, 12000, // third row
            13000, 14000, 15000, 16000, // fourth row
        ];
        // Destination buffer needs to accommodate intermediate width of 4
        // (dest_width*2) and height of 2
        let mut dest = vec![0u16; 8]; // 4 width * 2 height
        let src_pitch = NonZeroUsize::new(4).unwrap();
        let dest_pitch = NonZeroUsize::new(4).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(2).unwrap();
        let dest_height = NonZeroUsize::new(2).unwrap();

        reduce_cubic(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Verify reasonable values and ordering
        assert!(dest[0] > 1000 && dest[0] < 8000); // Top-left
        assert!(dest[1] > 2000 && dest[1] < 10000); // Top-right
        assert!(dest[4] > 5000 && dest[4] < 14000); // Bottom-left
        assert!(dest[5] > 6000 && dest[5] < 16000); // Bottom-right

        // Values should increase from left to right and top to bottom
        assert!(dest[0] < dest[1]);
        assert!(dest[0] < dest[4]);
        assert!(dest[1] < dest[5]);
        assert!(dest[4] < dest[5]);
    }

    #[test]
    fn test_reduce_cubic_with_padding() {
        // Test with source pitch > width (includes padding)
        let src = vec![
            10u8, 20, 255, 255, // first row (last 2 are padding)
            30, 40, 255, 255, // second row (last 2 are padding)
        ];
        // Destination buffer needs to accommodate intermediate width of 2
        // (dest_width*2)
        let mut dest = vec![0u8; 4]; // 2 intermediate width, plus padding for dest_pitch=4
        let src_pitch = NonZeroUsize::new(4).unwrap();
        let dest_pitch = NonZeroUsize::new(4).unwrap(); // Must accommodate intermediate width + padding
        let dest_width = NonZeroUsize::new(1).unwrap();
        let dest_height = NonZeroUsize::new(1).unwrap();

        reduce_cubic(
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
    }

    #[test]
    fn test_reduce_cubic_uniform_values() {
        // Test with uniform values to ensure filter preserves them
        let src = vec![
            100u8, 100, 100, 100, // first row
            100, 100, 100, 100, // second row
            100, 100, 100, 100, // third row
            100, 100, 100, 100, // fourth row
        ];
        // Destination buffer needs to accommodate intermediate width of 4
        // (dest_width*2) and height of 2
        let mut dest = vec![0u8; 8]; // 4 width * 2 height
        let src_pitch = NonZeroUsize::new(4).unwrap();
        let dest_pitch = NonZeroUsize::new(4).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(2).unwrap();
        let dest_height = NonZeroUsize::new(2).unwrap();

        reduce_cubic(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Uniform input should produce uniform output
        assert_eq!(dest[0], 100);
        assert_eq!(dest[1], 100);
        assert_eq!(dest[4], 100);
        assert_eq!(dest[5], 100);
    }

    #[test]
    fn test_reduce_cubic_edge_case_single_pixel() {
        // Test edge case with 2x2 -> 1x1 reduction (minimal case)
        let src = vec![
            50u8, 60, // first row
            70, 80, // second row
        ];
        // Destination buffer needs to accommodate intermediate width of 2
        // (dest_width*2)
        let mut dest = vec![0u8; 2];
        let src_pitch = NonZeroUsize::new(2).unwrap();
        let dest_pitch = NonZeroUsize::new(2).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(1).unwrap();
        let dest_height = NonZeroUsize::new(1).unwrap();

        reduce_cubic(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // With minimal case, should use simple averaging
        // Vertical: (50 + 70).div_ceil(2) = 60, (60 + 80).div_ceil(2) = 70
        // Horizontal: (60 + 70).div_ceil(2) = 65
        assert_eq!(dest[0], 65);
    }

    #[test]
    fn test_reduce_cubic_max_values() {
        // Test with maximum values to ensure no overflow
        let src = vec![
            255u8, 255, // first row
            255, 255, // second row
        ];
        // Destination buffer needs to accommodate intermediate width of 2
        // (dest_width*2)
        let mut dest = vec![0u8; 2];
        let src_pitch = NonZeroUsize::new(2).unwrap();
        let dest_pitch = NonZeroUsize::new(2).unwrap(); // Must accommodate intermediate width
        let dest_width = NonZeroUsize::new(1).unwrap();
        let dest_height = NonZeroUsize::new(1).unwrap();

        reduce_cubic(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // Maximum values should be preserved
        assert_eq!(dest[0], 255);
    }

    #[test]
    fn test_reduce_cubic_large_height() {
        // Test 4x6 -> 2x3 reduction to exercise the middle lines loop
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

        reduce_cubic(
            &mut dest,
            &src,
            dest_pitch,
            src_pitch,
            dest_width,
            dest_height,
        );

        // This test primarily ensures the middle lines loop doesn't crash
        // The exact values are less important than ensuring no index out of bounds
        assert_ne!(dest[0], 0); // Should have been modified
        assert_ne!(dest[4], 0); // Second row should have been modified
        assert_ne!(dest[8], 0); // Third row should have been modified
    }
}
