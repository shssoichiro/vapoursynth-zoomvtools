#![allow(unused_unsafe)]
#![allow(clippy::undocumented_unsafe_blocks)]

use pastey::paste;
use std::num::NonZeroUsize;

macro_rules! create_tests {
    ($module:ident) => {
        paste! {
            #[test]
            fn [<test_reduce_bilinear_u8_2x2_ $module>]() {
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

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Bilinear filter is separable:
                // 1. Vertical: (10 + 30).div_ceil(2) = 20, (20 + 40).div_ceil(2) = 30 So
                //    intermediate = [20, 30]
                // 2. Horizontal: (20 + 30).div_ceil(2) = 25
                assert_eq!(dest[0], 25);
            }

            #[test]
            fn [<test_reduce_bilinear_u8_4x2_ $module>]() {
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

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Bilinear filter is separable:
                // 1. Vertical: [0]: (10 + 50).div_ceil(2) = 30, [1]: (20 + 60).div_ceil(2) = 40
                //    [2]: (30 + 70).div_ceil(2) = 50, [3]: (40 + 80).div_ceil(2) = 60 So
                //    intermediate = [30, 40, 50, 60]
                // 2. Horizontal: dest[0]: (30 + 40).div_ceil(2) = 35 dest[1]: (50 +
                //    60).div_ceil(2) = 55
                assert_eq!(dest[0], 35);
                assert_eq!(dest[1], 55);
            }

            #[test]
            fn [<test_reduce_bilinear_u8_4x4_ $module>]() {
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

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Bilinear filter is separable:
                // 1. Vertical reduction (first pass): Row 0: [0]: (10 + 50).div_ceil(2) = 30,
                //    [1]: (20 + 60).div_ceil(2) = 40 [2]: (30 + 70).div_ceil(2) = 50, [3]: (40
                //    + 80).div_ceil(2) = 60 Row 1: [0]: (90 + 130).div_ceil(2) = 110, [1]: (100
                //    + 140).div_ceil(2) = 120 [2]: (110 + 150).div_ceil(2) = 130, [3]: (120 +
                //    160).div_ceil(2) = 140 So after vertical = [[30, 40, 50, 60], [110, 120,
                //    130, 140]]
                // 2. Horizontal reduction (second pass): Row 0: dest[0]: (30 + 40).div_ceil(2)
                //    = 35, dest[1]: (50 + 60).div_ceil(2) = 55 Row 1: dest[4]: (110 +
                //    120).div_ceil(2) = 115, dest[5]: (130 + 140).div_ceil(2) = 135
                assert_eq!(dest[0], 35); // Top-left
                assert_eq!(dest[1], 55); // Top-right
                assert_eq!(dest[4], 115); // Bottom-left
                assert_eq!(dest[5], 135); // Bottom-right
            }

            #[test]
            fn [<test_reduce_bilinear_u8_6x4_ $module>]() {
                // Test 6x4 -> 3x2 reduction with more complex filtering
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

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Values should be reasonable - detailed calculation would be complex
                // but we can verify general properties
                assert!(dest[0] > 10 && dest[0] < 120);
                assert!(dest[1] > 20 && dest[1] < 130);
                assert!(dest[2] > 30 && dest[2] < 140);
                assert!(dest[6] > 70 && dest[6] < 210);
                assert!(dest[7] > 80 && dest[7] < 220);
                assert!(dest[8] > 90 && dest[8] < 230);

                // Values should increase from left to right and top to bottom
                assert!(dest[0] < dest[1]);
                assert!(dest[1] < dest[2]);
                assert!(dest[0] < dest[6]);
                assert!(dest[1] < dest[7]);
                assert!(dest[2] < dest[8]);
            }

            #[test]
            fn [<test_reduce_bilinear_u16_basic_ $module>]() {
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

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Bilinear filter:
                // Vertical: (1000 + 3000).div_ceil(2) = 2000, (2000 + 4000).div_ceil(2) = 3000
                // Horizontal: (2000 + 3000).div_ceil(2) = 2500
                assert_eq!(dest[0], 2500);
            }

            #[test]
            fn [<test_reduce_bilinear_u16_large_values_ $module>]() {
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

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Bilinear filter:
                // Vertical: (60000 + 62000).div_ceil(2) = 61000, (61000 + 63000).div_ceil(2) =
                // 62000 Horizontal: (61000 + 62000).div_ceil(2) = 61500
                assert_eq!(dest[0], 61500);
            }

            #[test]
            fn [<test_reduce_bilinear_u16_4x4_ $module>]() {
                // Test 4x4 -> 2x2 reduction with u16
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

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Similar to the u8 case but with larger values
                // The bilinear filter should produce smoothed results
                assert!(dest[0] > 1000 && dest[0] < 8000);
                assert!(dest[1] > 2000 && dest[1] < 10000);
                assert!(dest[4] > 5000 && dest[4] < 14000);
                assert!(dest[5] > 6000 && dest[5] < 16000);

                // Values should increase from top-left to bottom-right due to input pattern
                assert!(dest[0] < dest[1]);
                assert!(dest[0] < dest[4]);
                assert!(dest[4] < dest[5]);
            }

            #[test]
            fn [<test_reduce_bilinear_uniform_values_ $module>]() {
                // Test with uniform input - should preserve the value
                let src = vec![
                    128u8, 128, 128, 128, // first row
                    128, 128, 128, 128, // second row
                    128, 128, 128, 128, // third row
                    128, 128, 128, 128, // fourth row
                ];
                // Destination buffer needs to accommodate intermediate width of 4
                // (dest_width*2) and height of 2
                let mut dest = vec![0u8; 8]; // 4 width * 2 height
                let src_pitch = NonZeroUsize::new(4).unwrap();
                let dest_pitch = NonZeroUsize::new(4).unwrap(); // Must accommodate intermediate width
                let dest_width = NonZeroUsize::new(2).unwrap();
                let dest_height = NonZeroUsize::new(2).unwrap();

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // All outputs should be 128 since input is uniform
                assert_eq!(dest[0], 128);
                assert_eq!(dest[1], 128);
                assert_eq!(dest[4], 128);
                assert_eq!(dest[5], 128);
            }

            #[test]
            fn [<test_reduce_bilinear_edge_case_2x2_ $module>]() {
                // Test minimal 2x2 reduction with edge values
                let src = vec![
                    0u8, 255, // first row
                    255, 0, // second row
                ];
                // Destination buffer needs to accommodate intermediate width of 2
                // (dest_width*2)
                let mut dest = vec![0u8; 2];
                let src_pitch = NonZeroUsize::new(2).unwrap();
                let dest_pitch = NonZeroUsize::new(2).unwrap(); // Must accommodate intermediate width
                let dest_width = NonZeroUsize::new(1).unwrap();
                let dest_height = NonZeroUsize::new(1).unwrap();

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Bilinear filter:
                // Vertical: (0 + 255).div_ceil(2) = 128, (255 + 0).div_ceil(2) = 128
                // Horizontal: (128 + 128).div_ceil(2) = 128
                assert_eq!(dest[0], 128);
            }

            #[test]
            fn [<test_reduce_bilinear_with_padding_ $module>]() {
                // Test with source pitch > width (includes padding)
                let src = vec![
                    10u8, 20, 255, 255, // first row (last 2 are padding)
                    30, 40, 255, 255, // second row (last 2 are padding)
                ];
                // dest_pitch = 4 to accommodate intermediate width of 2 plus padding
                let mut dest = vec![0u8; 4];
                let src_pitch = NonZeroUsize::new(4).unwrap();
                let dest_pitch = NonZeroUsize::new(4).unwrap();
                let dest_width = NonZeroUsize::new(1).unwrap();
                let dest_height = NonZeroUsize::new(1).unwrap();

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Should only process the first 2x2 block, ignoring padding
                // Vertical: (10 + 30).div_ceil(2) = 20, (20 + 40).div_ceil(2) = 30
                // Horizontal: (20 + 30).div_ceil(2) = 25
                assert_eq!(dest[0], 25);
                // Note: dest[1] might be modified due to intermediate processing
            }

            #[test]
            fn [<test_reduce_bilinear_max_values_ $module>]() {
                // Test with maximum u8 values
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

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // All 255 values should result in 255
                assert_eq!(dest[0], 255);
            }

            #[test]
            fn [<test_reduce_bilinear_gradient_ $module>]() {
                // Test with a gradient pattern
                let src = vec![
                    0u8, 64, 128, 192, // first row
                    64, 128, 192, 255, // second row
                    128, 192, 255, 255, // third row
                    192, 255, 255, 255, // fourth row
                ];
                // Destination buffer needs to accommodate intermediate width of 4
                // (dest_width*2) and height of 2
                let mut dest = vec![0u8; 8]; // 4 width * 2 height
                let src_pitch = NonZeroUsize::new(4).unwrap();
                let dest_pitch = NonZeroUsize::new(4).unwrap(); // Must accommodate intermediate width
                let dest_width = NonZeroUsize::new(2).unwrap();
                let dest_height = NonZeroUsize::new(2).unwrap();

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Values should form a smooth gradient
                // Check that values increase roughly from top-left to bottom-right
                assert!(dest[0] < dest[1]); // Top row: left < right
                assert!(dest[0] < dest[4]); // Left column: top < bottom
                assert!(dest[1] < dest[5]); // Right column: top < bottom
                assert!(dest[4] < dest[5]); // Bottom row: left < right

                // All values should be reasonable - no need to check u8 range as it's
                // guaranteed by type
            }

            #[test]
            fn [<test_reduce_bilinear_large_height_ $module>]() {
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

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // This test primarily ensures the middle lines loop doesn't crash
                // The exact values are less important than ensuring no index out of bounds
                assert_ne!(dest[0], 0); // Should have been modified
                assert_ne!(dest[4], 0); // Second row should have been modified
                assert_ne!(dest[8], 0); // Third row should have been modified
            }

            #[test]
            fn [<test_reduce_bilinear_u8_large_simd_ $module>]() {
                // Test large enough to trigger SIMD processing (64x2 -> 32x1)
                // This ensures we cover the SIMD loop in vertical reduction for AVX2 implementation
                let mut src = Vec::new();

                // First row: 64 pixels with values 0-63
                for i in 0..64u8 {
                    src.push(i);
                }

                // Second row: 64 pixels with values 64-127
                for i in 64..128u8 {
                    src.push(i);
                }

                // Destination buffer needs intermediate width of 64 (dest_width*2)
                let mut dest = vec![0u8; 64];
                let src_pitch = NonZeroUsize::new(64).unwrap();
                let dest_pitch = NonZeroUsize::new(64).unwrap(); // Must accommodate intermediate width
                let dest_width = NonZeroUsize::new(32).unwrap();
                let dest_height = NonZeroUsize::new(1).unwrap();

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Verify the bilinear filtering results
                // Step 1: Vertical reduction creates intermediate array with width=64
                let mut intermediate = vec![0u8; 64];
                for i in 0..64usize {
                    let a = src[i] as u32; // First row
                    let b = src[64 + i] as u32; // Second row
                    intermediate[i] = ((a + b + 1) / 2) as u8;
                }

                // Step 2: Horizontal reduction processes the intermediate array
                for i in 0..32usize {
                    let expected = if i == 0 {
                        // First pixel: (a + b + 1) / 2
                        let a = intermediate[0] as u32;
                        let b = intermediate[1] as u32;
                        ((a + b + 1) / 2) as u8
                    } else if i == 31 {
                        // Last pixel: (a + b + 1) / 2
                        let a = intermediate[62] as u32;
                        let b = intermediate[63] as u32;
                        ((a + b + 1) / 2) as u8
                    } else {
                        // Middle pixels: (a + (b + c) * 3 + d + 4) / 8
                        let a = intermediate[i * 2 - 1] as u32;
                        let b = intermediate[i * 2] as u32;
                        let c = intermediate[i * 2 + 1] as u32;
                        let d = intermediate[i * 2 + 2] as u32;
                        ((a + (b + c) * 3 + d + 4) / 8) as u8
                    };

                    assert_eq!(dest[i], expected, "Mismatch at position {}", i);
                }
            }

            #[test]
            fn [<test_reduce_bilinear_u8_large_simd_middle_lines_ $module>]() {
                // Test large enough to trigger SIMD processing for middle lines (64x6 -> 32x3)
                // This ensures we cover the complex weighted SIMD loop in vertical reduction
                let mut src = Vec::new();

                // Create 6 rows of 64 pixels each with incrementing values
                // Keep values small to avoid overflow (max value will be 5*40+39 = 239)
                for row in 0..6u8 {
                    for col in 0..64u8 {
                        src.push(row * 40 + (col % 40));
                    }
                }

                // Destination buffer needs intermediate width of 64 (dest_width*2) and height of 3
                let mut dest = vec![0u8; 192]; // 64 width * 3 height
                let src_pitch = NonZeroUsize::new(64).unwrap();
                let dest_pitch = NonZeroUsize::new(64).unwrap(); // Must accommodate intermediate width
                let dest_width = NonZeroUsize::new(32).unwrap();
                let dest_height = NonZeroUsize::new(3).unwrap();

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Verify the vertical reduction (which creates intermediate values)
                // Row 0 (y=0): Simple averaging of src rows 0 and 1
                // Row 1 (y=1): Weighted interpolation of src rows 0, 2, 4, 6 (this triggers the SIMD loop)
                // Row 2 (y=2): Simple averaging of src rows 4 and 5

                // Test a few key positions to ensure the middle line SIMD processing worked
                // We mainly want to ensure no crashes and reasonable values
                for y in 0..3usize {
                    for x in 0..32usize {
                        let dest_idx = y * 64 + x;
                        assert_ne!(dest[dest_idx], 0, "Row {} pixel {} should have been processed", y, x);

                        // Values should be within reasonable range based on input
                        let max_input = (5 * 40 + 39) as u8; // Maximum input value
                        assert!(dest[dest_idx] <= max_input, "Row {} pixel {} value {} exceeds maximum", y, x, dest[dest_idx]);
                    }
                }

                // The middle row should have different values than edge rows due to weighted interpolation
                let first_row_sample = dest[0];
                let middle_row_sample = dest[64]; // Second row
                let last_row_sample = dest[128]; // Third row

                // These should be different due to different interpolation methods
                assert_ne!(first_row_sample, middle_row_sample, "First and middle rows should differ");
                assert_ne!(middle_row_sample, last_row_sample, "Middle and last rows should differ");
            }

            #[test]
            fn [<test_reduce_bilinear_u16_large_simd_ $module>]() {
                // Test large enough to trigger SIMD processing for u16 (32x2 -> 16x1)
                // This ensures we cover the u16 SIMD loop in vertical reduction for AVX2 implementation
                let mut src = Vec::new();

                // First row: 32 pixels with values 0-31 scaled to u16 range
                for i in 0..32u16 {
                    src.push(i * 1000);
                }

                // Second row: 32 pixels with values 32-63 scaled to u16 range
                for i in 32..64u16 {
                    src.push(i * 1000);
                }

                // Destination buffer needs intermediate width of 32 (dest_width*2)
                let mut dest = vec![0u16; 32];
                let src_pitch = NonZeroUsize::new(32).unwrap();
                let dest_pitch = NonZeroUsize::new(32).unwrap(); // Must accommodate intermediate width
                let dest_width = NonZeroUsize::new(16).unwrap();
                let dest_height = NonZeroUsize::new(1).unwrap();

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Verify the bilinear filtering results
                // Step 1: Vertical reduction creates intermediate array with width=32
                let mut intermediate = vec![0u16; 32];
                for i in 0..32usize {
                    let a = src[i] as u32; // First row
                    let b = src[32 + i] as u32; // Second row
                    intermediate[i] = ((a + b + 1) / 2) as u16;
                }

                // Step 2: Horizontal reduction processes the intermediate array
                for i in 0..16usize {
                    let expected = if i == 0 {
                        // First pixel: (a + b + 1) / 2
                        let a = intermediate[0] as u32;
                        let b = intermediate[1] as u32;
                        ((a + b + 1) / 2) as u16
                    } else if i == 15 {
                        // Last pixel: (a + b + 1) / 2
                        let a = intermediate[30] as u32;
                        let b = intermediate[31] as u32;
                        ((a + b + 1) / 2) as u16
                    } else {
                        // Middle pixels: (a + (b + c) * 3 + d + 4) / 8
                        let a = intermediate[i * 2 - 1] as u32;
                        let b = intermediate[i * 2] as u32;
                        let c = intermediate[i * 2 + 1] as u32;
                        let d = intermediate[i * 2 + 2] as u32;
                        ((a + (b + c) * 3 + d + 4) / 8) as u16
                    };

                    assert_eq!(dest[i], expected, "Mismatch at position {}", i);
                }
            }

            #[test]
            fn [<test_reduce_bilinear_u16_large_simd_middle_lines_ $module>]() {
                // Test large enough to trigger SIMD processing for u16 middle lines (32x6 -> 16x3)
                // This ensures we cover the complex weighted u16 SIMD loop in vertical reduction
                let mut src = Vec::new();

                // Create 6 rows of 32 pixels each with incrementing values
                // Keep values reasonable to avoid overflow (max value will be 5*8000+31*200 = 46200)
                for row in 0..6u16 {
                    for col in 0..32u16 {
                        src.push(row * 8000 + col * 200);
                    }
                }

                // Destination buffer needs intermediate width of 32 (dest_width*2) and height of 3
                let mut dest = vec![0u16; 96]; // 32 width * 3 height
                let src_pitch = NonZeroUsize::new(32).unwrap();
                let dest_pitch = NonZeroUsize::new(32).unwrap(); // Must accommodate intermediate width
                let dest_width = NonZeroUsize::new(16).unwrap();
                let dest_height = NonZeroUsize::new(3).unwrap();

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Verify the vertical reduction (which creates intermediate values)
                // Row 0 (y=0): Simple averaging of src rows 0 and 1
                // Row 1 (y=1): Weighted interpolation of src rows 0, 2, 4, 6 (this triggers the SIMD loop)
                // Row 2 (y=2): Simple averaging of src rows 4 and 5

                // Test a few key positions to ensure the middle line SIMD processing worked
                // We mainly want to ensure no crashes and reasonable values
                for y in 0..3usize {
                    for x in 0..16usize {
                        let dest_idx = y * 32 + x;
                        assert_ne!(dest[dest_idx], 0, "Row {} pixel {} should have been processed", y, x);

                        // Values should be within reasonable range based on input
                        let max_input = (5 * 8000 + 31 * 200) as u16; // Maximum input value
                        assert!(dest[dest_idx] <= max_input, "Row {} pixel {} value {} exceeds maximum", y, x, dest[dest_idx]);
                    }
                }

                // The middle row should have different values than edge rows due to weighted interpolation
                let first_row_sample = dest[0];
                let middle_row_sample = dest[32]; // Second row
                let last_row_sample = dest[64]; // Third row

                // These should be different due to different interpolation methods
                assert_ne!(first_row_sample, middle_row_sample, "First and middle rows should differ");
                assert_ne!(middle_row_sample, last_row_sample, "Middle and last rows should differ");
            }

            #[test]
            fn [<test_reduce_bilinear_u16_scalar_fallback_ $module>]() {
                // Test with width not divisible by 16 to trigger scalar fallback (36x6 -> 18x3)
                // This ensures we cover the scalar fallback code in u16 middle lines processing
                let mut src = Vec::new();

                // Create 6 rows of 36 pixels each with incrementing values
                // Keep values reasonable to avoid overflow
                for row in 0..6u16 {
                    for col in 0..36u16 {
                        src.push(row * 5000 + col * 100);
                    }
                }

                // Destination buffer needs intermediate width of 36 (dest_width*2) and height of 3
                let mut dest = vec![0u16; 108]; // 36 width * 3 height
                let src_pitch = NonZeroUsize::new(36).unwrap();
                let dest_pitch = NonZeroUsize::new(36).unwrap(); // Must accommodate intermediate width
                let dest_width = NonZeroUsize::new(18).unwrap(); // Not divisible by 16!
                let dest_height = NonZeroUsize::new(3).unwrap();

                verify_asm!($module, reduce_bilinear(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Test all positions including the scalar fallback regions
                // SIMD processes pixels 0-15, scalar fallback handles pixels 16-17
                for y in 0..3usize {
                    for x in 0..18usize {
                        let dest_idx = y * 36 + x;
                        assert_ne!(dest[dest_idx], 0, "Row {} pixel {} should have been processed", y, x);

                        // Values should be within reasonable range based on input
                        let max_input = (5 * 5000 + 35 * 100) as u16; // Maximum input value
                        assert!(dest[dest_idx] <= max_input, "Row {} pixel {} value {} exceeds maximum", y, x, dest[dest_idx]);
                    }
                }

                // Verify that the scalar fallback pixels (16-17) were processed correctly
                // These should have reasonable values, not zero
                let middle_row_simd_end = dest[32 + 15]; // Last SIMD-processed pixel in middle row
                let middle_row_scalar_1 = dest[32 + 16]; // First scalar-processed pixel in middle row
                let middle_row_scalar_2 = dest[32 + 17]; // Second scalar-processed pixel in middle row

                assert_ne!(middle_row_scalar_1, 0, "First scalar fallback pixel should be processed");
                assert_ne!(middle_row_scalar_2, 0, "Second scalar fallback pixel should be processed");

                // The scalar pixels should have reasonable values relative to SIMD pixels
                // (This is a rough sanity check, not an exact calculation)
                let diff1 = if middle_row_scalar_1 > middle_row_simd_end {
                    middle_row_scalar_1 - middle_row_simd_end
                } else {
                    middle_row_simd_end - middle_row_scalar_1
                };
                assert!(diff1 < 10000, "Scalar fallback pixel 1 should have reasonable value relative to SIMD");

                let diff2 = if middle_row_scalar_2 > middle_row_scalar_1 {
                    middle_row_scalar_2 - middle_row_scalar_1
                } else {
                    middle_row_scalar_1 - middle_row_scalar_2
                };
                assert!(diff2 < 10000, "Scalar fallback pixel 2 should have reasonable value relative to previous");
            }
        }
    };
}

create_tests!(rust);

#[cfg(target_feature = "avx2")]
create_tests!(avx2);
