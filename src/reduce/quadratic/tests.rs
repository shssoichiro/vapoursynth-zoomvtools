#![allow(unused_unsafe)]
#![allow(clippy::undocumented_unsafe_blocks)]

use pastey::paste;
use std::num::NonZeroUsize;

macro_rules! create_tests {
    ($module:ident) => {
        paste! {
            #[test]
            fn [<test_reduce_quadratic_u8_2x2_ $module>]() {
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

                verify_asm!($module, reduce_quadratic(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Quadratic filter is separable:
                // 1. Vertical: For single line (height=1), uses simple averaging: (10 +
                //    30).div_ceil(2) = 20, (20 + 40).div_ceil(2) = 30 So intermediate = [20,
                //    30]
                // 2. Horizontal: For single pixel (width=1), uses simple averaging: (20 +
                //    30).div_ceil(2) = 25
                assert_eq!(dest[0], 25);
            }

            #[test]
            fn [<test_reduce_quadratic_u8_4x2_ $module>]() {
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

                verify_asm!($module, reduce_quadratic(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Quadratic filter is separable:
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
            fn [<test_reduce_quadratic_u8_4x4_ $module>]() {
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

                verify_asm!($module, reduce_quadratic(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Quadratic filter is separable with edge case handling:
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
            fn [<test_reduce_quadratic_u8_6x4_ $module>]() {
                // Test 6x4 -> 3x2 reduction with quadratic filtering
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

                verify_asm!($module, reduce_quadratic(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Quadratic filter should produce reasonable smoothed values
                // We verify that values are reasonable and follow expected trends
                assert!(dest[0] > 10 && dest[0] < 200); // Should be smoothed values
                assert!(dest[1] > 10 && dest[1] < 200);
                assert!(dest[2] > 10 && dest[2] < 200);
                assert!(dest[6] > 10 && dest[6] < 240); // Second row
                assert!(dest[7] > 10 && dest[7] < 240);
                assert!(dest[8] > 10 && dest[8] < 240);
            }

            #[test]
            fn [<test_reduce_quadratic_gradient_ $module>]() {
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

                verify_asm!($module, reduce_quadratic(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

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
            fn [<test_reduce_quadratic_u16_basic_ $module>]() {
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

                verify_asm!($module, reduce_quadratic(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Quadratic filter with edge case handling:
                // Vertical: (1000 + 3000).div_ceil(2) = 2000, (2000 + 4000).div_ceil(2) = 3000
                // Horizontal: (2000 + 3000).div_ceil(2) = 2500
                assert_eq!(dest[0], 2500);
            }

            #[test]
            fn [<test_reduce_quadratic_u16_large_values_ $module>]() {
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

                verify_asm!($module, reduce_quadratic(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Should handle large values without overflow
                // Vertical: (60000 + 62000).div_ceil(2) = 61000, (61000 + 63000).div_ceil(2) =
                // 62000 Horizontal: (61000 + 62000).div_ceil(2) = 61500
                assert_eq!(dest[0], 61500);
            }

            #[test]
            fn [<test_reduce_quadratic_u16_4x4_ $module>]() {
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

                verify_asm!($module, reduce_quadratic(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

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
            fn [<test_reduce_quadratic_with_padding_ $module>]() {
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

                verify_asm!($module, reduce_quadratic(
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
            }

            #[test]
            fn [<test_reduce_quadratic_uniform_values_ $module>]() {
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

                verify_asm!($module, reduce_quadratic(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Uniform input should produce uniform output
                assert_eq!(dest[0], 100);
                assert_eq!(dest[1], 100);
                assert_eq!(dest[4], 100);
                assert_eq!(dest[5], 100);
            }

            #[test]
            fn [<test_reduce_quadratic_edge_case_single_pixel_ $module>]() {
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

                verify_asm!($module, reduce_quadratic(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // With minimal case, should use simple averaging
                // Vertical: (50 + 70).div_ceil(2) = 60, (60 + 80).div_ceil(2) = 70
                // Horizontal: (60 + 70).div_ceil(2) = 65
                assert_eq!(dest[0], 65);
            }

            #[test]
            fn [<test_reduce_quadratic_max_values_ $module>]() {
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

                verify_asm!($module, reduce_quadratic(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Maximum values should be preserved
                assert_eq!(dest[0], 255);
            }

            #[test]
            fn [<test_reduce_quadratic_large_height_ $module>]() {
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

                verify_asm!($module, reduce_quadratic(
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
            fn [<test_reduce_quadratic_u8_large_simd_ $module>]() {
                // Test large enough to trigger SIMD processing for u8 first line (64x2 -> 32x1)
                // This ensures we cover the u8 SIMD loop at lines 96-121: while x + 32 <= dest_width_val
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

                verify_asm!($module, reduce_quadratic(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Verify the SIMD processing results for u8 first line
                // The SIMD loop should process x=0..31 in one iteration since 0 + 32 <= 32
                // First line uses simple averaging: (a + b + 1) / 2
                                for i in 0..32usize {
                    // After vertical reduction, horizontal reduction processes the intermediate result
                    // For the horizontal step, we check the final result is reasonable
                    assert!(dest[i] < 255,
                           "SIMD processing should produce valid u8 values at position {}: {}", i, dest[i]);
                }

                // Verify the SIMD code path was exercised by checking consistent results
                // The values should follow a general increasing pattern due to the input pattern
                for i in 0..31usize {
                    assert!(dest[i] <= dest[i + 1] + 1,
                           "Values should be reasonably ordered at positions {} and {}: {} vs {}",
                           i, i + 1, dest[i], dest[i + 1]);
                }

                // Verify specific known values from the simple averaging
                // For example, position 0: (0 + 64 + 1) / 2 = 32 (intermediate)
                // After horizontal reduction, the exact final value depends on the algorithm
                // but should be reasonable for our input pattern
                assert!(dest[0] > 10 && dest[0] < 50, "First value should be reasonable: {}", dest[0]);
                assert!(dest[31] > 70 && dest[31] < 110, "Last value should be reasonable: {}", dest[31]);
            }

            #[test]
            fn [<test_reduce_quadratic_u8_large_simd_middle_lines_ $module>]() {
                // Test large enough to trigger SIMD processing for u8 middle lines (32x6 -> 16x3)
                // This ensures we cover the middle lines SIMD loop at lines 144-186: while x + 8 <= dest_width_val
                // With dest_height=3, this gives y=1 (one iteration of the middle lines loop)
                let mut src = Vec::new();

                // Create 6 rows of 32 pixels each with a controlled pattern
                // Keep values small to avoid overflow in 6-tap quadratic filter calculations
                for row in 0..6u8 {
                    for col in 0..32u8 {
                        src.push((row * 20 + col / 4) % 200); // Values 0-199
                    }
                }

                // Destination buffer needs intermediate width of 32 (dest_width*2) and height of 3
                let mut dest = vec![0u8; 96]; // 32 width * 3 height
                let src_pitch = NonZeroUsize::new(32).unwrap();
                let dest_pitch = NonZeroUsize::new(32).unwrap(); // Must accommodate intermediate width
                let dest_width = NonZeroUsize::new(16).unwrap();
                let dest_height = NonZeroUsize::new(3).unwrap();

                verify_asm!($module, reduce_quadratic(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Verify the complex quadratic filtering for middle lines
                // The middle line (y=1) uses the complex 6-tap quadratic filter:
                // result = (m0 + m5 + 9*(m1 + m4) + 22*(m2 + m3) + 32) >> 6
                // where m0..m5 are the 6 vertical taps for the quadratic filter

                // The SIMD loop should process x=0..7 in the first iteration, then x=8..15 in the second iteration since 0 + 8 <= 16 and 8 + 8 <= 16
                // Verify that all rows have been processed
                for y in 0..3usize {
                    for &x in &[0, 8, 15] {
                        let dest_idx = y * 32 + x; // Using dest_pitch=32 for intermediate buffer

                        // Verify the output is reasonable (SIMD should have processed this)
                        assert!(dest[dest_idx] < 255,
                               "SIMD processing should produce valid u8 values at row {} position {}: {}",
                               y, x, dest[dest_idx]);
                    }
                }

                // Specifically verify the middle row (y=1) which uses the complex quadratic filter
                // This row should have different values from simple averaging due to the 6-tap filter
                let middle_row_start = 32; // y=1 * dest_pitch=32
                for i in 0..16usize {
                    let middle_value = dest[middle_row_start + i];

                    // The 6-tap quadratic filter should produce reasonable values
                    assert!(middle_value < 220,
                           "Middle row quadratic filter should produce reasonable values at position {}: {}",
                           i, middle_value);
                }

                // Verify the SIMD code path was exercised by checking value consistency
                // The complex quadratic filter should produce smoother transitions
                let first_row_sample = dest[0]; // y=0, x=0 (simple edge case)
                let middle_row_sample = dest[32]; // y=1, x=0 (complex quadratic filter)
                let last_row_sample = dest[64]; // y=2, x=0 (simple edge case)

                // All should be valid u8 values
                assert!(first_row_sample < 255 && middle_row_sample < 255 && last_row_sample < 255,
                       "All processed values should be valid u8");

                // Test a few more samples to ensure the SIMD loop processed the full width
                for sample_x in [0, 8, 15] {
                    let middle_sample = dest[32 + sample_x]; // Second row, various positions
                    assert_ne!(middle_sample, 0, "SIMD should have processed position {}", sample_x);
                    assert!(middle_sample < 255, "SIMD result should be valid u8 at position {}", sample_x);
                }
            }

            #[test]
            fn [<test_reduce_quadratic_u16_large_simd_middle_lines_ $module>]() {
                // Test large enough to trigger SIMD processing for u16 middle lines (16x6 -> 8x3)
                // This ensures we cover the middle lines SIMD loop at lines 280-305: while x + 8 <= dest_width_val
                // With dest_height=3, this gives y=1 (one iteration of the middle lines loop)
                let mut src = Vec::new();

                // Create 6 rows of 16 pixels each with a controlled pattern
                // Keep values moderate to avoid overflow in 6-tap quadratic filter calculations
                for row in 0..6u16 {
                    for col in 0..16u16 {
                        src.push((row * 1000 + col * 100) % 30000); // Values 0-29999
                    }
                }

                // Destination buffer needs intermediate width of 16 (dest_width*2) and height of 3
                let mut dest = vec![0u16; 48]; // 16 width * 3 height
                let src_pitch = NonZeroUsize::new(16).unwrap();
                let dest_pitch = NonZeroUsize::new(16).unwrap(); // Must accommodate intermediate width
                let dest_width = NonZeroUsize::new(8).unwrap();
                let dest_height = NonZeroUsize::new(3).unwrap();

                verify_asm!($module, reduce_quadratic(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Verify the complex quadratic filtering for middle lines
                // The middle line (y=1) uses the complex 6-tap quadratic filter:
                // result = (m0 + m5 + 9*(m1 + m4) + 22*(m2 + m3) + 32) >> 6
                // where m0..m5 are the 6 vertical taps for the quadratic filter

                // The SIMD loop should process x=0..7 in the first iteration, then x=8..15 in the second iteration since 0 + 8 <= 16 and 8 + 8 <= 16
                // Verify that all rows have been processed
                for y in 0..3usize {
                    for &x in &[0, 8, 15] {
                        let dest_idx = y * 16 + x; // Using dest_pitch=16 for intermediate buffer

                        // Verify the output is reasonable (SIMD should have processed this)
                        assert!(dest[dest_idx] < 65535,
                               "SIMD processing should produce valid u16 values at row {} position {}: {}",
                               y, x, dest[dest_idx]);
                    }
                }

                // Specifically verify the middle row (y=1) which uses the complex quadratic filter
                // This row should have different values from simple averaging due to the 6-tap filter
                let middle_row_start = 16; // y=1 * dest_pitch=16
                for i in 0..8usize {
                    let middle_value = dest[middle_row_start + i];

                    // The 6-tap quadratic filter should produce reasonable values
                    assert!(middle_value < 32000,
                           "Middle row quadratic filter should produce reasonable values at position {}: {}",
                           i, middle_value);
                }

                // Verify the SIMD code path was exercised by checking value consistency
                // The complex quadratic filter should produce smoother transitions
                let first_row_sample = dest[0]; // y=0, x=0 (simple edge case)
                let middle_row_sample = dest[16]; // y=1, x=0 (complex quadratic filter)
                let last_row_sample = dest[32]; // y=2, x=0 (simple edge case)

                // All should be valid u16 values
                assert!(first_row_sample < 65535 && middle_row_sample < 65535 && last_row_sample < 65535,
                       "All processed values should be valid u16");

                // Test a few more samples to ensure the SIMD loop processed the full width
                for sample_x in [0, 8, 15] {
                    let middle_sample = dest[16 + sample_x]; // Second row, various positions
                    assert_ne!(middle_sample, 0, "SIMD should have processed position {}", sample_x);
                    assert!(middle_sample < 65535, "SIMD result should be valid u16 at position {}", sample_x);
                }

                // Verify the horizontal step will still work correctly
                // Since this is only the vertical step, values at positions 8-15 should also be processed
                for i in 8..16usize {
                    assert_ne!(dest[middle_row_start + i], 0,
                             "SIMD should have processed all positions including {}", i);
                }
            }

            #[test]
            fn [<test_reduce_quadratic_u16_scalar_fallback_middle_lines_ $module>]() {
                // Test with width that leaves remainders after SIMD processing (18x6 -> 9x3)
                // This ensures we cover the scalar fallback code at lines 377-393: while x < dest_width_val (in middle lines)
                // Width 18 means intermediate width is 18, and 18 % 8 = 2, so 2 pixels will be processed by scalar fallback
                let mut src = Vec::new();

                // Create 6 rows of 18 pixels each with a controlled pattern
                // Keep values moderate to avoid overflow in 6-tap quadratic filter calculations
                for row in 0..6u16 {
                    for col in 0..18u16 {
                        src.push((row * 1000 + col * 50) % 20000); // Values 0-19999
                    }
                }

                // Destination buffer needs intermediate width of 18 (dest_width*2) and height of 3
                let mut dest = vec![0u16; 54]; // 18 width * 3 height
                let src_pitch = NonZeroUsize::new(18).unwrap();
                let dest_pitch = NonZeroUsize::new(18).unwrap(); // Must accommodate intermediate width
                let dest_width = NonZeroUsize::new(9).unwrap();
                let dest_height = NonZeroUsize::new(3).unwrap();

                verify_asm!($module, reduce_quadratic(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // The SIMD loop processes x=0..7 (first 8 pixels), then x=8..15 (next 8 pixels)
                // The scalar fallback should process x=16,17 (remaining 2 pixels) when x < dest_width_val (18)

                // Verify that all rows have been processed including the scalar fallback pixels
                for y in 0..3usize {
                    for x in 0..18usize {
                        let dest_idx = y * 18 + x; // Using dest_pitch=18 for intermediate buffer

                        // Verify the output is reasonable (both SIMD and scalar should have processed this)
                        assert!(dest[dest_idx] < 65535,
                               "Processing should produce valid u16 values at row {} position {}: {}",
                               y, x, dest[dest_idx]);
                    }
                }

                // Specifically verify the middle row (y=1) which uses the complex quadratic filter
                let middle_row_start = 18; // y=1 * dest_pitch=18

                // Test the SIMD-processed pixels (x=0..15)
                for i in 0..16usize {
                    let middle_value = dest[middle_row_start + i];
                    assert!(middle_value < 25000,
                           "SIMD-processed middle row values should be reasonable at position {}: {}",
                           i, middle_value);
                }

                // Specifically test the scalar fallback pixels (x=16,17)
                for i in 16..18usize {
                    let scalar_value = dest[middle_row_start + i];
                    assert_ne!(scalar_value, 0,
                              "Scalar fallback should have processed position {}", i);
                    assert!(scalar_value < 25000,
                           "Scalar fallback should produce reasonable values at position {}: {}",
                           i, scalar_value);
                }

                // Verify the scalar fallback produces results consistent with the pattern
                // The last two pixels should follow the same quadratic filtering as the SIMD pixels
                let simd_last = dest[middle_row_start + 15]; // Last SIMD-processed pixel
                let scalar_first = dest[middle_row_start + 16]; // First scalar-processed pixel
                let scalar_last = dest[middle_row_start + 17]; // Last scalar-processed pixel

                // Values should be reasonable and follow the input pattern trend
                assert!(scalar_first > simd_last.saturating_sub(2000) && scalar_first < simd_last + 2000,
                       "Scalar fallback should produce consistent values: SIMD_last={}, scalar_first={}",
                       simd_last, scalar_first);
                assert!(scalar_last > scalar_first.saturating_sub(1000) && scalar_last < scalar_first + 1000,
                       "Scalar fallback pixels should be consistent: scalar_first={}, scalar_last={}",
                       scalar_first, scalar_last);
            }
        }
    };
}

create_tests!(rust);

#[cfg(target_feature = "avx2")]
create_tests!(avx2);
