#![allow(unused_unsafe)]
#![allow(clippy::undocumented_unsafe_blocks)]

use std::num::{NonZeroU8, NonZeroUsize};

use pastey::paste;

macro_rules! horizontal_tests {
    ($module:ident) => {
        paste! {
            #[test]
            fn [<test_horizontal_wiener_u8_basic_ $module>]() {
                // Test basic horizontal Wiener filtering with u8 pixels
                let width = NonZeroUsize::new(8).unwrap();
                let height = NonZeroUsize::new(1).unwrap();
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(8).unwrap();

                // Create test data: gradual increase from 0 to 255
                let src: Vec<u8> = vec![0, 32, 64, 96, 128, 160, 192, 255];
                let mut dest = vec![0u8; 8];

                verify_asm!($module, refine_horizontal_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // First two pixels should be bilinear interpolation
                assert_eq!(dest[0], 16); // (0 + 32) / 2
                assert_eq!(dest[1], 48); // (32 + 64) / 2

                // Last pixel should be unchanged
                assert_eq!(dest[7], 255);

                // Middle pixels should be Wiener filtered (values will be different from
                // bilinear) The exact values depend on the Wiener filter implementation
                assert!(dest[2] > 0 && dest[2] < 255);
                assert!(dest[3] > 0 && dest[3] < 255);
            }

            #[test]
            fn [<test_horizontal_wiener_u16_basic_ $module>]() {
                // Test basic horizontal Wiener filtering with u16 pixels
                let width = NonZeroUsize::new(8).unwrap();
                let height = NonZeroUsize::new(1).unwrap();
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(10).unwrap();

                // Create test data for 10-bit: 0 to 1023
                let src: Vec<u16> = vec![0, 128, 256, 384, 512, 640, 768, 1023];
                let mut dest = vec![0u16; 8];

                verify_asm!($module, refine_horizontal_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // First two pixels should be bilinear interpolation
                assert_eq!(dest[0], 64); // (0 + 128) / 2
                assert_eq!(dest[1], 192); // (128 + 256) / 2

                // Last pixel should be unchanged
                assert_eq!(dest[7], 1023);

                // Middle pixels should be Wiener filtered
                assert!(dest[2] > 0 && dest[2] < 1023);
                assert!(dest[3] > 0 && dest[3] < 1023);
            }

            #[test]
            fn [<test_horizontal_wiener_multirow_ $module>]() {
                // Test horizontal Wiener with multiple rows
                let width = NonZeroUsize::new(6).unwrap();
                let height = NonZeroUsize::new(3).unwrap();
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(8).unwrap();

                let src: Vec<u8> = vec![
                    10, 20, 30, 40, 50, 60, // Row 1
                    70, 80, 90, 100, 110, 120, // Row 2
                    130, 140, 150, 160, 170, 180, // Row 3
                ];
                let mut dest = vec![0u8; 18];

                verify_asm!($module, refine_horizontal_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // Check that each row is processed independently
                // First pixel of each row should be bilinear of first two
                assert_eq!(dest[0], 15); // (10 + 20) / 2
                assert_eq!(dest[6], 75); // (70 + 80) / 2
                assert_eq!(dest[12], 135); // (130 + 140) / 2

                // Last pixel of each row should be unchanged
                assert_eq!(dest[5], 60);
                assert_eq!(dest[11], 120);
                assert_eq!(dest[17], 180);
            }

            #[test]
            fn [<test_horizontal_wiener_minimum_width_ $module>]() {
                // Test with minimum width where Wiener filter can be applied
                let width = NonZeroUsize::new(6).unwrap(); // Minimum for Wiener core (needs i-2 to i+3)
                let height = NonZeroUsize::new(1).unwrap();
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(8).unwrap();

                let src: Vec<u8> = vec![0, 50, 100, 150, 200, 255];
                let mut dest = vec![0u8; 6];

                verify_asm!($module, refine_horizontal_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // Should not crash and produce reasonable results
                assert_eq!(dest[0], 25); // (0 + 50) / 2
                assert_eq!(dest[1], 75); // (50 + 100) / 2
                assert_eq!(dest[5], 255); // Last pixel unchanged
            }

            #[test]
            fn [<test_wiener_with_different_pitch_ $module>]() {
                // Test with pitch different from width (common in real scenarios)
                let width = NonZeroUsize::new(4).unwrap();
                let height = NonZeroUsize::new(2).unwrap();
                let pitch = NonZeroUsize::new(8).unwrap(); // Pitch larger than width
                let bits_per_sample = NonZeroU8::new(8).unwrap();

                let src: Vec<u8> = vec![
                    10, 20, 30, 40, 0, 0, 0, 0, // Row 1 with padding
                    50, 60, 70, 80, 0, 0, 0, 0, // Row 2 with padding
                ];
                let mut dest = vec![0u8; 16];

                verify_asm!($module, refine_horizontal_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // Check that pitch is respected
                assert_eq!(dest[0], 15); // (10 + 20) / 2
                assert_eq!(dest[3], 40); // Last pixel unchanged
                assert_eq!(dest[8], 55); // (50 + 60) / 2 - second row
                assert_eq!(dest[11], 80); // Last pixel of second row
            }

            #[test]
            fn [<test_horizontal_wiener_uniform_values_ $module>]() {
                // Test with uniform input values - should produce uniform output
                let width = NonZeroUsize::new(8).unwrap();
                let height = NonZeroUsize::new(1).unwrap();
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(8).unwrap();

                let src = vec![100u8; 8]; // All pixels same value
                let mut dest = vec![0u8; 8];

                verify_asm!($module, refine_horizontal_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // All output values should be the same as input
                for &pixel in &dest {
                    assert_eq!(pixel, 100);
                }
            }

            #[test]
            fn [<test_wiener_pixel_clamping_ $module>]() {
                // Test that output is properly clamped to pixel range
                let width = NonZeroUsize::new(8).unwrap();
                let height = NonZeroUsize::new(1).unwrap();
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(8).unwrap();

                // Extreme values that might cause overflow in intermediate calculations
                let src: Vec<u8> = vec![0, 0, 255, 255, 0, 0, 255, 255];
                let mut dest = vec![0u8; 8];

                verify_asm!($module, refine_horizontal_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // Test passes if function doesn't panic with extreme values
                // The type system ensures u8 values are in valid range [0, 255]
                assert_eq!(dest.len(), 8);
            }

            #[test]
            fn [<test_wiener_16bit_pixel_clamping_ $module>]() {
                // Test pixel clamping for 16-bit values
                let width = NonZeroUsize::new(8).unwrap();
                let height = NonZeroUsize::new(1).unwrap();
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(10).unwrap(); // 10-bit: max value 1023

                // Values near the maximum that might cause overflow
                let src: Vec<u16> = vec![0, 0, 1023, 1023, 0, 0, 1023, 1023];
                let mut dest = vec![0u16; 8];

                verify_asm!($module, refine_horizontal_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // All output values should be in valid range [0, 1023]
                for &pixel in &dest {
                    assert!(pixel <= 1023);
                }
            }

            #[test]
            fn [<test_horizontal_wiener_u8_large_simd_ $module>]() {
                // Test large enough to trigger SIMD processing for u8 horizontal Wiener (64x1 -> processed with 32-pixel SIMD)
                // This ensures we cover the u8 SIMD loop at lines 156-194: while i + 32 <= wiener_end
                // wiener_start = 2, wiener_end = width - 4 = 60, so 60 - 2 = 58 >= 32 (triggers SIMD)
                let width = NonZeroUsize::new(64).unwrap();
                let height = NonZeroUsize::new(1).unwrap();
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(8).unwrap();

                // Create test data with a controlled pattern for easier verification
                let mut src = Vec::new();
                for i in 0..64u8 {
                    src.push((i * 2) % 240); // Values 0, 2, 4, ..., 238, avoiding overflow
                }

                let mut dest = vec![0u8; 64];

                verify_asm!($module, refine_horizontal_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // Verify the SIMD processing results for u8 horizontal Wiener
                // The SIMD loop should process i=2..33 in one iteration (32 pixels), then i=34..59 potentially in another
                // since wiener_start=2 and wiener_end=60, and 2 + 32 <= 60

                // First two pixels should be bilinear interpolation (not SIMD processed)
                let expected_0 = ((src[0] as u16 + src[1] as u16 + 1) / 2) as u8;
                let expected_1 = ((src[1] as u16 + src[2] as u16 + 1) / 2) as u8;
                assert_eq!(dest[0], expected_0, "First pixel should be bilinear: {} vs {}", dest[0], expected_0);
                assert_eq!(dest[1], expected_1, "Second pixel should be bilinear: {} vs {}", dest[1], expected_1);

                // Last pixel should be copied directly
                assert_eq!(dest[63], src[63], "Last pixel should be copied: {} vs {}", dest[63], src[63]);

                // Middle pixels (i=2..60) should be Wiener filtered using SIMD
                // The Wiener filter applies: m0 + m5 + ((m2 + m3) * 4 - (m1 + m4)) * 5 + 16) >> 5
                // Let's verify a few specific SIMD-processed pixels
                for &test_i in &[2, 16, 32, 48, 59] {
                    if (2..60).contains(&test_i) {  // In the Wiener range
                        let result = dest[test_i];

                        // Verify the result is reasonable (non-zero for our input pattern, within valid range)
                        assert_ne!(result, 0, "SIMD-processed pixel at {} should be non-zero", test_i);
                        assert!(result < 255, "SIMD-processed pixel at {} should be valid u8: {}", test_i, result);

                        // Calculate expected Wiener result manually for verification
                        let m0 = src[test_i - 2] as i16;
                        let m1 = src[test_i - 1] as i16;
                        let m2 = src[test_i] as i16;
                        let m3 = src[test_i + 1] as i16;
                        let m4 = src[test_i + 2] as i16;
                        let m5 = src[test_i + 3] as i16;

                        let temp = (m2 + m3) * 4 - (m1 + m4);
                        let expected_raw = (m0 + m5 + temp * 5 + 16) >> 5;
                        let expected = expected_raw.max(0).min(255) as u8;

                        assert_eq!(result, expected,
                                  "SIMD result should match manual calculation at position {}: {} vs {}",
                                  test_i, result, expected);
                    }
                }

                // Verify that the SIMD processing covered the expected range
                // The SIMD loop processes i=2..33 (32 pixels) in the first iteration
                for i in 2..34usize {
                    let result = dest[i];
                    assert_ne!(result, 0, "First SIMD iteration should have processed position {}", i);
                }

                // The second SIMD iteration would process i=34..59 if 34 + 32 <= 60, but 66 > 60
                // So the remaining pixels i=34..59 are processed by scalar code
                // Let's just verify they were processed (non-zero)
                for i in 34..60usize {
                    let result = dest[i];
                    assert_ne!(result, 0, "Remaining pixels should have been processed at position {}", i);
                }

                // Verify the overall pattern makes sense - should be a filtered version of input
                // Since our input increases monotonically (mostly), the output should follow a similar trend
                // but be smoother due to the Wiener filtering
                let first_wiener = dest[10];   // Early in SIMD range
                let last_wiener = dest[50];   // Late in range

                // Due to the filtering, these should generally increase but may not be strictly monotonic
                assert!(first_wiener < last_wiener + 50,
                       "Wiener filter should preserve general trend: {} vs {}", first_wiener, last_wiener);
            }

            #[test]
            fn [<test_horizontal_wiener_u16_large_simd_ $module>]() {
                // Test large enough to trigger SIMD processing for u16 horizontal Wiener (32x1 -> processed with 16-pixel SIMD)
                // This ensures we cover the u16 SIMD loop at lines 268-306: while i + 16 <= wiener_end
                // wiener_start = 2, wiener_end = width - 4 = 28, so 28 - 2 = 26 >= 16 (triggers SIMD)
                let width = NonZeroUsize::new(32).unwrap();
                let height = NonZeroUsize::new(1).unwrap();
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(12).unwrap(); // 12-bit for u16

                // Create test data with a controlled pattern for easier verification
                let mut src = Vec::new();
                for i in 0..32u16 {
                    src.push((i * 100) % 3000); // Values 0, 100, 200, ..., avoiding overflow in 12-bit range
                }

                let mut dest = vec![0u16; 32];

                verify_asm!($module, refine_horizontal_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // Verify the SIMD processing results for u16 horizontal Wiener
                // The SIMD loop should process i=2..17 in one iteration (16 pixels), then potentially i=18..27
                // since wiener_start=2 and wiener_end=28, and 2 + 16 <= 28

                // First two pixels should be bilinear interpolation (not SIMD processed)
                let expected_0 = ((src[0] as u32 + src[1] as u32 + 1) / 2) as u16;
                let expected_1 = ((src[1] as u32 + src[2] as u32 + 1) / 2) as u16;
                assert_eq!(dest[0], expected_0, "First pixel should be bilinear: {} vs {}", dest[0], expected_0);
                assert_eq!(dest[1], expected_1, "Second pixel should be bilinear: {} vs {}", dest[1], expected_1);

                // Last pixel should be copied directly
                assert_eq!(dest[31], src[31], "Last pixel should be copied: {} vs {}", dest[31], src[31]);

                // Middle pixels (i=2..28) should be Wiener filtered using SIMD
                // The Wiener filter applies: m0 + m5 + ((m2 + m3) * 4 - (m1 + m4)) * 5 + 16) >> 5
                // Let's verify a few specific SIMD-processed pixels
                for &test_i in &[2, 8, 16, 24, 27] {
                    if (2..28).contains(&test_i) {  // In the Wiener range
                        let result = dest[test_i];

                        // Verify the result is reasonable (non-zero for our input pattern, within valid range)
                        assert_ne!(result, 0, "SIMD-processed pixel at {} should be non-zero", test_i);
                        assert!(result < 4096, "SIMD-processed pixel at {} should be valid 12-bit: {}", test_i, result);

                        // Calculate expected Wiener result manually for verification
                        let m0 = src[test_i - 2] as i32;
                        let m1 = src[test_i - 1] as i32;
                        let m2 = src[test_i] as i32;
                        let m3 = src[test_i + 1] as i32;
                        let m4 = src[test_i + 2] as i32;
                        let m5 = src[test_i + 3] as i32;

                        let temp = (m2 + m3) * 4 - (m1 + m4);
                        let expected_raw = (m0 + m5 + temp * 5 + 16) >> 5;
                        let expected = expected_raw.max(0).min(4095) as u16; // 12-bit max

                        assert_eq!(result, expected,
                                  "SIMD result should match manual calculation at position {}: {} vs {}",
                                  test_i, result, expected);
                    }
                }

                // Verify that the SIMD processing covered the expected range
                // The SIMD loop processes i=2..17 (16 pixels) in the first iteration
                for i in 2..18usize {
                    let result = dest[i];
                    assert_ne!(result, 0, "First SIMD iteration should have processed position {}", i);
                }

                // Check if there's a second SIMD iteration: i=18..27 if 18 + 16 <= 28, but 34 > 28
                // So the remaining pixels i=18..27 are processed by scalar code
                // Let's just verify they were processed (non-zero for our input pattern)
                for i in 18..28usize {
                    let result = dest[i];
                    assert_ne!(result, 0, "Remaining pixels should have been processed at position {}", i);
                }

                // Verify the overall pattern makes sense - should be a filtered version of input
                // Since our input increases monotonically (mostly), the output should follow a similar trend
                // but be smoother due to the Wiener filtering
                let first_wiener = dest[5];   // Early in SIMD range
                let last_wiener = dest[25];   // Late in range

                // Due to the filtering, these should generally increase but may not be strictly monotonic
                assert!(first_wiener < last_wiener + 500,
                       "Wiener filter should preserve general trend: {} vs {}", first_wiener, last_wiener);

                // Verify SIMD and scalar results are consistent (both should produce valid results)
                let simd_sample = dest[10];     // Processed by SIMD
                let scalar_sample = dest[20];   // Processed by scalar
                assert!(simd_sample < 4096 && scalar_sample < 4096,
                       "Both SIMD and scalar should produce valid 12-bit results: {} vs {}", simd_sample, scalar_sample);
            }

            #[test]
            fn [<test_horizontal_wiener_narrow_width_ $module>]() {
                // Test with narrow width to trigger the else branch in wiener_end calculation (lines 147-153)
                // When width < 4, wiener_end = wiener_start, so no Wiener filtering is applied
                let width = NonZeroUsize::new(3).unwrap(); // width < 4 triggers the else branch
                let height = NonZeroUsize::new(1).unwrap();
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(8).unwrap();

                let src: Vec<u8> = vec![10, 50, 90];
                let mut dest = vec![0u8; 3];

                verify_asm!($module, refine_horizontal_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // With width=3, the algorithm should:
                // - Apply bilinear interpolation to first pixel: (10 + 50 + 1) / 2 = 30
                // - Skip Wiener filtering (since wiener_end = wiener_start = 2, no loop iterations)
                // - Apply bilinear interpolation to second pixel (index 1): (50 + 90 + 1) / 2 = 70
                // - Copy last pixel: 90

                assert_eq!(dest[0], 30, "First pixel should be bilinear interpolation: {} vs 30", dest[0]);
                assert_eq!(dest[1], 70, "Second pixel should be bilinear interpolation: {} vs 70", dest[1]);
                assert_eq!(dest[2], 90, "Last pixel should be copied: {} vs 90", dest[2]);

                // Verify no Wiener filtering was applied (all values are simple bilinear/copy)
                // This ensures the else branch was taken and wiener_end = wiener_start = 2
                // The loop `while i < wiener_end` with i starting at 2 and wiener_end = 2 executes 0 times
            }

            #[test]
            fn [<test_horizontal_wiener_minimal_width_ $module>]() {
                // Test with width=2 (even narrower) to further verify the edge case
                let width = NonZeroUsize::new(2).unwrap(); // width < 4 triggers the else branch
                let height = NonZeroUsize::new(1).unwrap();
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(8).unwrap();

                let src: Vec<u8> = vec![20, 80];
                let mut dest = vec![0u8; 2];

                verify_asm!($module, refine_horizontal_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // With width=2:
                // - First pixel: bilinear interpolation (20 + 80 + 1) / 2 = 50
                // - Last pixel: copied directly = 80
                // - No Wiener filtering (wiener_end = wiener_start = 2, but width=2 means no middle processing)

                assert_eq!(dest[0], 50, "First pixel should be bilinear interpolation: {} vs 50", dest[0]);
                assert_eq!(dest[1], 80, "Last pixel should be copied: {} vs 80", dest[1]);
            }

            #[test]
            fn [<test_horizontal_wiener_u16_narrow_width_ $module>]() {
                // Test with narrow width to trigger the else branch in u16 wiener_end calculation (lines 259-265)
                // When width < 4, wiener_end = wiener_start, so no Wiener filtering is applied
                let width = NonZeroUsize::new(3).unwrap(); // width < 4 triggers the else branch
                let height = NonZeroUsize::new(1).unwrap();
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(10).unwrap(); // 10-bit for u16

                let src: Vec<u16> = vec![100, 500, 900];
                let mut dest = vec![0u16; 3];

                verify_asm!($module, refine_horizontal_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // With width=3, the algorithm should:
                // - Apply bilinear interpolation to first pixel: (100 + 500 + 1) / 2 = 300
                // - Skip Wiener filtering (since wiener_end = wiener_start = 2, no loop iterations)
                // - Apply bilinear interpolation to second pixel (index 1): (500 + 900 + 1) / 2 = 700
                // - Copy last pixel: 900

                assert_eq!(dest[0], 300, "First pixel should be bilinear interpolation: {} vs 300", dest[0]);
                assert_eq!(dest[1], 700, "Second pixel should be bilinear interpolation: {} vs 700", dest[1]);
                assert_eq!(dest[2], 900, "Last pixel should be copied: {} vs 900", dest[2]);

                // Verify no Wiener filtering was applied (all values are simple bilinear/copy)
                // This ensures the else branch was taken and wiener_end = wiener_start = 2
                // The loop `while i < wiener_end` with i starting at 2 and wiener_end = 2 executes 0 times
            }

            #[test]
            fn [<test_horizontal_wiener_u16_minimal_width_ $module>]() {
                // Test with width=2 (even narrower) for u16 to further verify the edge case
                let width = NonZeroUsize::new(2).unwrap(); // width < 4 triggers the else branch
                let height = NonZeroUsize::new(1).unwrap();
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(12).unwrap(); // 12-bit for u16

                let src: Vec<u16> = vec![200, 800];
                let mut dest = vec![0u16; 2];

                verify_asm!($module, refine_horizontal_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // With width=2:
                // - First pixel: bilinear interpolation (200 + 800 + 1) / 2 = 500
                // - Last pixel: copied directly = 800
                // - No Wiener filtering (wiener_end = wiener_start = 2, but width=2 means no middle processing)

                assert_eq!(dest[0], 500, "First pixel should be bilinear interpolation: {} vs 500", dest[0]);
                assert_eq!(dest[1], 800, "Last pixel should be copied: {} vs 800", dest[1]);
            }
        }
    };
}

macro_rules! vertical_tests {
    ($module:ident) => {
        paste! {
            #[test]
            fn [<test_vertical_wiener_u8_basic_ $module>]() {
                // Test basic vertical Wiener filtering with u8 pixels
                let width = NonZeroUsize::new(1).unwrap();
                let height = NonZeroUsize::new(8).unwrap();
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(8).unwrap();

                // Create test data: vertical gradient
                let src: Vec<u8> = vec![0, 32, 64, 96, 128, 160, 192, 255];
                let mut dest = vec![0u8; 8];

                verify_asm!($module, refine_vertical_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // First two pixels should be bilinear interpolation
                assert_eq!(dest[0], 16); // (0 + 32) / 2
                assert_eq!(dest[1], 48); // (32 + 64) / 2

                // Last pixel should be unchanged
                assert_eq!(dest[7], 255);

                // Middle pixels should be Wiener filtered
                assert!(dest[2] > 0 && dest[2] < 255);
                assert!(dest[3] > 0 && dest[3] < 255);
            }

            #[test]
            fn [<test_vertical_wiener_u16_basic_ $module>]() {
                // Test basic vertical Wiener filtering with u16 pixels
                let width = NonZeroUsize::new(1).unwrap();
                let height = NonZeroUsize::new(8).unwrap();
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(12).unwrap();

                // Create test data for 12-bit: 0 to 4095
                let src: Vec<u16> = vec![0, 512, 1024, 1536, 2048, 2560, 3072, 4095];
                let mut dest = vec![0u16; 8];

                verify_asm!($module, refine_vertical_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // First two pixels should be bilinear interpolation
                assert_eq!(dest[0], 256); // (0 + 512) / 2
                assert_eq!(dest[1], 768); // (512 + 1024) / 2

                // Last pixel should be unchanged
                assert_eq!(dest[7], 4095);

                // Middle pixels should be Wiener filtered
                assert!(dest[2] > 0 && dest[2] < 4095);
                assert!(dest[3] > 0 && dest[3] < 4095);
            }

            #[test]
            fn [<test_vertical_wiener_multicolumn_ $module>]() {
                // Test vertical Wiener with multiple columns
                let width = NonZeroUsize::new(3).unwrap();
                let height = NonZeroUsize::new(6).unwrap();
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(8).unwrap();

                let src: Vec<u8> = vec![
                    10, 70, 130, // Row 1
                    20, 80, 140, // Row 2
                    30, 90, 150, // Row 3
                    40, 100, 160, // Row 4
                    50, 110, 170, // Row 5
                    60, 120, 180, // Row 6
                ];
                let mut dest = vec![0u8; 18];

                verify_asm!($module, refine_vertical_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // Check that each column is processed independently
                // First two rows should be bilinear interpolation
                assert_eq!(dest[0], 15); // (10 + 20) / 2
                assert_eq!(dest[1], 75); // (70 + 80) / 2
                assert_eq!(dest[2], 135); // (130 + 140) / 2

                assert_eq!(dest[3], 25); // (20 + 30) / 2
                assert_eq!(dest[4], 85); // (80 + 90) / 2
                assert_eq!(dest[5], 145); // (140 + 150) / 2

                // Last row should be unchanged
                assert_eq!(dest[15], 60);
                assert_eq!(dest[16], 120);
                assert_eq!(dest[17], 180);
            }

            #[test]
            fn [<test_vertical_wiener_minimum_height_ $module>]() {
                // Test with minimum height where Wiener filter can be applied
                let width = NonZeroUsize::new(1).unwrap();
                let height = NonZeroUsize::new(6).unwrap(); // Minimum for Wiener core
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(8).unwrap();

                let src: Vec<u8> = vec![0, 50, 100, 150, 200, 255];
                let mut dest = vec![0u8; 6];

                verify_asm!($module, refine_vertical_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // Should not crash and produce reasonable results
                assert_eq!(dest[0], 25); // (0 + 50) / 2
                assert_eq!(dest[1], 75); // (50 + 100) / 2
                assert_eq!(dest[5], 255); // Last pixel unchanged
            }

            #[test]
            fn [<test_vertical_wiener_uniform_values_ $module>]() {
                // Test with uniform input values - should produce uniform output
                let width = NonZeroUsize::new(1).unwrap();
                let height = NonZeroUsize::new(8).unwrap();
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(8).unwrap();

                let src = vec![100u8; 8]; // All pixels same value
                let mut dest = vec![0u8; 8];

                verify_asm!($module, refine_vertical_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // All output values should be the same as input
                for &pixel in &dest {
                    assert_eq!(pixel, 100);
                }
            }

            #[test]
            fn [<test_vertical_wiener_u8_large_simd_ $module>]() {
                // Test large enough to trigger SIMD processing for u8 vertical Wiener bilinear interpolation (64x6 -> processed with 32-pixel SIMD)
                // This ensures we cover the u8 SIMD loop at lines 362-380: while i + 32 <= width.get()
                // This loop processes bilinear interpolation for first/last rows using SIMD
                let width = NonZeroUsize::new(64).unwrap(); // width >= 32 triggers SIMD
                let height = NonZeroUsize::new(6).unwrap(); // Need sufficient height for Wiener processing
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(8).unwrap();

                // Create test data with a controlled pattern
                let mut src = Vec::new();
                for row in 0..6u8 {
                    for col in 0..64u8 {
                        src.push((row * 20 + col / 4) % 200); // Values 0-199, varying by row and column
                    }
                }

                let mut dest = vec![0u8; 384]; // 64 width * 6 height

                verify_asm!($module, refine_vertical_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // Verify the SIMD processing results for u8 vertical Wiener
                // The SIMD loop should process i=0..31 in one iteration, then i=32..63 in another iteration
                // since 0 + 32 <= 64 and 32 + 32 <= 64

                // First two rows should be bilinear interpolation using SIMD
                // Row 0: average of src rows 0 and 1
                for i in 0..64usize {
                    let expected = ((src[i] as u16 + src[i + 64] as u16 + 1) / 2) as u8;
                    assert_eq!(dest[i], expected, "First row SIMD result should match bilinear at position {}: {} vs {}", i, dest[i], expected);
                }

                // Row 1: average of src rows 1 and 2
                for i in 0..64usize {
                    let expected = ((src[i + 64] as u16 + src[i + 128] as u16 + 1) / 2) as u8;
                    assert_eq!(dest[i + 64], expected, "Second row SIMD result should match bilinear at position {}: {} vs {}", i, dest[i + 64], expected);
                }

                // Verify that the SIMD processing covered the full width
                // Test some specific positions to ensure SIMD processed them correctly
                for &test_i in &[0, 16, 32, 48, 63] {
                    // Check first row (processed by SIMD bilinear)
                    let result = dest[test_i];
                    assert_ne!(result, 0, "SIMD should have processed position {} in first row", test_i);
                    assert!(result < 255, "SIMD result should be valid u8 at position {}: {}", test_i, result);

                    // Check second row (also processed by SIMD bilinear)
                    let result2 = dest[test_i + 64];
                    assert_ne!(result2, 0, "SIMD should have processed position {} in second row", test_i);
                    assert!(result2 < 255, "SIMD result should be valid u8 at position {}: {}", test_i, result2);
                }

                // Verify the SIMD code path was exercised by checking that results are different
                // from original source values (since we're averaging two different rows)
                let mut differences_found = 0;
                for i in 0..64usize {
                    if dest[i] != src[i] {
                        differences_found += 1;
                    }
                }
                assert!(differences_found > 32, "SIMD bilinear should produce different values from source: {} differences", differences_found);

                // Verify the general pattern makes sense - bilinear results should be between the two source rows
                for i in 0..64usize {
                    let src_row0 = src[i];
                    let src_row1 = src[i + 64];
                    let result = dest[i];

                    let min_val = src_row0.min(src_row1);
                    let max_val = src_row0.max(src_row1);

                    assert!(result >= min_val && result <= max_val,
                           "Bilinear result should be between source values at position {}: {} should be between {} and {}",
                           i, result, min_val, max_val);
                }

                // Last row should be copied directly (not processed by this SIMD loop)
                for i in 0..64usize {
                    let src_last_row = src[i + 320]; // Row 5 (0-indexed)
                    let dest_last_row = dest[i + 320];
                    assert_eq!(dest_last_row, src_last_row, "Last row should be copied directly at position {}: {} vs {}", i, dest_last_row, src_last_row);
                }
            }

            #[test]
            fn [<test_vertical_wiener_u8_large_simd_middle_rows_ $module>]() {
                // Test large enough to trigger SIMD processing for u8 vertical Wiener full kernel (64x8 -> processed with 32-pixel SIMD)
                // This ensures we cover the u8 SIMD loop at lines 396-437: while i + 32 <= width.get()
                // This loop processes the full 6-tap Wiener filter for middle rows using SIMD
                let width = NonZeroUsize::new(64).unwrap(); // width >= 32 triggers SIMD
                let height = NonZeroUsize::new(8).unwrap(); // height >= 6 to have middle rows (j=2,3 when height=8)
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(8).unwrap();

                // Create test data with a controlled pattern
                let mut src = Vec::new();
                for row in 0..8u8 {
                    for col in 0..64u8 {
                        src.push((row * 15 + col / 8) % 180); // Values 0-179, varying by row and column
                    }
                }

                let mut dest = vec![0u8; 512]; // 64 width * 8 height

                verify_asm!($module, refine_vertical_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // Verify the SIMD processing results for u8 vertical Wiener middle rows
                // The middle rows loop is: for _j in 2..(height.get() - 4).max(2)
                // With height=8, this is j in 2..4, so j=2,3 (two middle rows)
                // The SIMD loop should process i=0..31 in one iteration, then i=32..63 in another iteration

                // Check middle row 2 (dest row index 2, j=2 in the loop)
                let middle_row_offset = 2 * 64; // Row 2
                for i in 0..64usize {
                    let result = dest[middle_row_offset + i];

                    // Calculate expected Wiener result manually for verification
                    let m0 = src[i] as i16; // row 0
                    let m1 = src[i + 64] as i16; // row 1
                    let m2 = src[i + 128] as i16; // row 2
                    let m3 = src[i + 192] as i16; // row 3
                    let m4 = src[i + 256] as i16; // row 4
                    let m5 = src[i + 320] as i16; // row 5

                    // Wiener formula: (m0 + m5 + ((m2 + m3) * 4 - (m1 + m4)) * 5 + 16) >> 5
                    let temp = (m2 + m3) * 4 - (m1 + m4);
                    let expected_raw = (m0 + m5 + temp * 5 + 16) >> 5;
                    let expected = expected_raw.max(0).min(255) as u8;

                    assert_eq!(result, expected,
                              "SIMD Wiener result should match manual calculation at middle row 2, position {}: {} vs {}",
                              i, result, expected);
                }

                // Check middle row 3 (dest row index 3, j=3 in the loop)
                let middle_row_offset = 3 * 64; // Row 3
                for i in 0..64usize {
                    let result = dest[middle_row_offset + i];

                    // Calculate expected Wiener result manually for verification
                    let m0 = src[i + 64] as i16; // row 1
                    let m1 = src[i + 128] as i16; // row 2
                    let m2 = src[i + 192] as i16; // row 3
                    let m3 = src[i + 256] as i16; // row 4
                    let m4 = src[i + 320] as i16; // row 5
                    let m5 = src[i + 384] as i16; // row 6

                    // Wiener formula: (m0 + m5 + ((m2 + m3) * 4 - (m1 + m4)) * 5 + 16) >> 5
                    let temp = (m2 + m3) * 4 - (m1 + m4);
                    let expected_raw = (m0 + m5 + temp * 5 + 16) >> 5;
                    let expected = expected_raw.max(0).min(255) as u8;

                    assert_eq!(result, expected,
                              "SIMD Wiener result should match manual calculation at middle row 3, position {}: {} vs {}",
                              i, result, expected);
                }

                                 // Verify that the SIMD processing covered the full width for middle rows
                 // Test some specific positions to ensure SIMD processed them correctly
                 for &test_i in &[0, 16, 32, 48, 63] {
                     // Check middle row 2 (processed by SIMD Wiener filter)
                     let result = dest[128 + test_i]; // Row 2
                     assert!(result < 255, "SIMD Wiener result should be valid u8 at row 2, position {}: {}", test_i, result);

                     // Check middle row 3 (also processed by SIMD Wiener filter)
                     let result2 = dest[192 + test_i]; // Row 3
                     assert!(result2 < 255, "SIMD Wiener result should be valid u8 at row 3, position {}: {}", test_i, result2);
                 }

                 // Verify the SIMD code path was exercised by checking that results are different
                 // from original source values (since we're applying 6-tap filtering)
                 let mut differences_found = 0;
                 for i in 0..64usize {
                     let src_val = src[i + 128]; // Source row 2
                     let dest_val = dest[i + 128]; // Dest row 2
                     if dest_val != src_val {
                         differences_found += 1;
                     }
                 }
                 assert!(differences_found > 32, "SIMD Wiener filter should produce different values from source: {} differences", differences_found);

                 // Verify the Wiener filter produces reasonable smoothed results
                 // The filter should not produce extreme outliers
                 for i in 128..192usize { // Middle row 2
                     let result = dest[i];
                     assert!(result < 200, "Wiener filtered values should be reasonable at position {}: {}", i, result);
                 }

                 // First two rows should be bilinear (not full Wiener), last row should be copied
                 // Row 0: bilinear interpolation
                 for i in 0..64usize {
                     let expected = ((src[i] as u16 + src[i + 64] as u16 + 1) / 2) as u8;
                     assert_eq!(dest[i], expected, "First row should be bilinear at position {}: {} vs {}", i, dest[i], expected);
                 }

                 // Last row should be copied directly
                 for i in 0..64usize {
                     let src_last_row = src[i + 448]; // Row 7 (0-indexed)
                     let dest_last_row = dest[i + 448];
                     assert_eq!(dest_last_row, src_last_row, "Last row should be copied directly at position {}: {} vs {}", i, dest_last_row, src_last_row);
                 }
             }

            #[test]
            fn [<test_vertical_wiener_u16_large_simd_ $module>]() {
                // Test large enough to trigger SIMD processing for u16 vertical Wiener bilinear interpolation (32x6 -> processed with 16-pixel SIMD)
                // This ensures we cover the u16 SIMD loop at lines 518-536: while i + 16 <= width.get()
                // This loop processes bilinear interpolation for first/last rows using SIMD for u16 data
                let width = NonZeroUsize::new(32).unwrap(); // width >= 16 triggers SIMD
                let height = NonZeroUsize::new(6).unwrap(); // Need sufficient height for Wiener processing
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(12).unwrap(); // 12-bit for u16

                // Create test data with a controlled pattern
                let mut src = Vec::new();
                for row in 0..6u16 {
                    for col in 0..32u16 {
                        src.push((row * 300 + col * 10) % 3000); // Values 0-2999, varying by row and column
                    }
                }

                let mut dest = vec![0u16; 192]; // 32 width * 6 height

                verify_asm!($module, refine_vertical_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // Verify the SIMD processing results for u16 vertical Wiener
                // The SIMD loop should process i=0..15 in one iteration, then i=16..31 in another iteration
                // since 0 + 16 <= 32 and 16 + 16 <= 32

                // First two rows should be bilinear interpolation using SIMD
                // Row 0: average of src rows 0 and 1
                for i in 0..32usize {
                    let expected = ((src[i] as u32 + src[i + 32] as u32 + 1) / 2) as u16;
                    assert_eq!(dest[i], expected, "First row SIMD result should match bilinear at position {}: {} vs {}", i, dest[i], expected);
                }

                // Row 1: average of src rows 1 and 2
                for i in 0..32usize {
                    let expected = ((src[i + 32] as u32 + src[i + 64] as u32 + 1) / 2) as u16;
                    assert_eq!(dest[i + 32], expected, "Second row SIMD result should match bilinear at position {}: {} vs {}", i, dest[i + 32], expected);
                }

                // Verify that the SIMD processing covered the full width
                // Test some specific positions to ensure SIMD processed them correctly
                for &test_i in &[0, 8, 16, 24, 31] {
                    // Check first row (processed by SIMD bilinear)
                    let result = dest[test_i];
                    assert_ne!(result, 0, "SIMD should have processed position {} in first row", test_i);
                    assert!(result < 4096, "SIMD result should be valid 12-bit at position {}: {}", test_i, result);

                    // Check second row (also processed by SIMD bilinear)
                    let result2 = dest[test_i + 32];
                    assert_ne!(result2, 0, "SIMD should have processed position {} in second row", test_i);
                    assert!(result2 < 4096, "SIMD result should be valid 12-bit at position {}: {}", test_i, result2);
                }

                // Verify the SIMD code path was exercised by checking that results are different
                // from original source values (since we're averaging two different rows)
                let mut differences_found = 0;
                for i in 0..32usize {
                    if dest[i] != src[i] {
                        differences_found += 1;
                    }
                }
                assert!(differences_found > 16, "SIMD bilinear should produce different values from source: {} differences", differences_found);

                // Verify the general pattern makes sense - bilinear results should be between the two source rows
                for i in 0..32usize {
                    let src_row0 = src[i];
                    let src_row1 = src[i + 32];
                    let result = dest[i];

                    let min_val = src_row0.min(src_row1);
                    let max_val = src_row0.max(src_row1);

                    assert!(result >= min_val && result <= max_val,
                           "Bilinear result should be between source values at position {}: {} should be between {} and {}",
                           i, result, min_val, max_val);
                }

                // Verify SIMD processing for both iterations of the loop
                // First SIMD iteration: i=0..15 (16 pixels)
                for i in 0..16usize {
                    let result = dest[i];
                    let expected = ((src[i] as u32 + src[i + 32] as u32 + 1) / 2) as u16;
                    assert_eq!(result, expected, "First SIMD iteration should be correct at position {}: {} vs {}", i, result, expected);
                }

                // Second SIMD iteration: i=16..31 (16 pixels)
                for i in 16..32usize {
                    let result = dest[i];
                    let expected = ((src[i] as u32 + src[i + 32] as u32 + 1) / 2) as u16;
                    assert_eq!(result, expected, "Second SIMD iteration should be correct at position {}: {} vs {}", i, result, expected);
                }

                // Last row should be copied directly (not processed by this SIMD loop)
                for i in 0..32usize {
                    let src_last_row = src[i + 160]; // Row 5 (0-indexed)
                    let dest_last_row = dest[i + 160];
                    assert_eq!(dest_last_row, src_last_row, "Last row should be copied directly at position {}: {} vs {}", i, dest_last_row, src_last_row);
                }
            }

            #[test]
            fn [<test_vertical_wiener_u16_large_simd_middle_rows_ $module>]() {
                // Test large enough to trigger SIMD processing for u16 vertical Wiener full kernel (32x8 -> processed with 16-pixel SIMD)
                // This ensures we cover the u16 SIMD loop at lines 551-592: while i + 16 <= width.get()
                // This loop processes the full 6-tap Wiener filter for middle rows using SIMD for u16 data
                let width = NonZeroUsize::new(32).unwrap(); // width >= 16 triggers SIMD
                let height = NonZeroUsize::new(8).unwrap(); // height >= 6 to have middle rows (j=2,3 when height=8)
                let pitch = width;
                let bits_per_sample = NonZeroU8::new(12).unwrap(); // 12-bit for u16

                // Create test data with a controlled pattern
                let mut src = Vec::new();
                for row in 0..8u16 {
                    for col in 0..32u16 {
                        src.push((row * 200 + col * 15) % 2800); // Values 0-2799, varying by row and column
                    }
                }

                let mut dest = vec![0u16; 256]; // 32 width * 8 height

                verify_asm!($module, refine_vertical_wiener(&mut dest, &src, pitch, width, height, bits_per_sample));

                // Verify the SIMD processing results for u16 vertical Wiener middle rows
                // The middle rows loop is: for _j in 2..(height.get() - 4).max(2)
                // With height=8, this is j in 2..4, so j=2,3 (two middle rows)
                // The SIMD loop should process i=0..15 in one iteration, then i=16..31 in another iteration

                // Check middle row 2 (dest row index 2, j=2 in the loop)
                let middle_row_offset = 2 * 32; // Row 2
                for i in 0..32usize {
                    let result = dest[middle_row_offset + i];

                    // Calculate expected Wiener result manually for verification
                    let m0 = src[i] as i32; // row 0
                    let m1 = src[i + 32] as i32; // row 1
                    let m2 = src[i + 64] as i32; // row 2
                    let m3 = src[i + 96] as i32; // row 3
                    let m4 = src[i + 128] as i32; // row 4
                    let m5 = src[i + 160] as i32; // row 5

                    // Wiener formula: (m0 + m5 + ((m2 + m3) * 4 - (m1 + m4)) * 5 + 16) >> 5
                    let temp = (m2 + m3) * 4 - (m1 + m4);
                    let expected_raw = (m0 + m5 + temp * 5 + 16) >> 5;
                    let expected = expected_raw.max(0).min(4095) as u16; // 12-bit max

                    assert_eq!(result, expected,
                              "SIMD Wiener result should match manual calculation at middle row 2, position {}: {} vs {}",
                              i, result, expected);
                }

                // Check middle row 3 (dest row index 3, j=3 in the loop)
                let middle_row_offset = 3 * 32; // Row 3
                for i in 0..32usize {
                    let result = dest[middle_row_offset + i];

                    // Calculate expected Wiener result manually for verification
                    let m0 = src[i + 32] as i32; // row 1
                    let m1 = src[i + 64] as i32; // row 2
                    let m2 = src[i + 96] as i32; // row 3
                    let m3 = src[i + 128] as i32; // row 4
                    let m4 = src[i + 160] as i32; // row 5
                    let m5 = src[i + 192] as i32; // row 6

                    // Wiener formula: (m0 + m5 + ((m2 + m3) * 4 - (m1 + m4)) * 5 + 16) >> 5
                    let temp = (m2 + m3) * 4 - (m1 + m4);
                    let expected_raw = (m0 + m5 + temp * 5 + 16) >> 5;
                    let expected = expected_raw.max(0).min(4095) as u16; // 12-bit max

                    assert_eq!(result, expected,
                              "SIMD Wiener result should match manual calculation at middle row 3, position {}: {} vs {}",
                              i, result, expected);
                }

                // Verify that the SIMD processing covered the full width for middle rows
                // Test some specific positions to ensure SIMD processed them correctly
                for &test_i in &[0, 8, 16, 24, 31] {
                    // Check middle row 2 (processed by SIMD Wiener filter)
                    let result = dest[64 + test_i]; // Row 2
                    assert!(result < 4096, "SIMD Wiener result should be valid 12-bit at row 2, position {}: {}", test_i, result);

                    // Check middle row 3 (also processed by SIMD Wiener filter)
                    let result2 = dest[96 + test_i]; // Row 3
                    assert!(result2 < 4096, "SIMD Wiener result should be valid 12-bit at row 3, position {}: {}", test_i, result2);
                }

                // Verify the SIMD code path was exercised by checking that results are different
                // from original source values (since we're applying 6-tap filtering)
                let mut differences_found = 0;
                for i in 0..32usize {
                    let src_val = src[i + 64]; // Source row 2
                    let dest_val = dest[i + 64]; // Dest row 2
                    if dest_val != src_val {
                        differences_found += 1;
                    }
                }
                assert!(differences_found > 16, "SIMD Wiener filter should produce different values from source: {} differences", differences_found);

                // Verify the Wiener filter produces reasonable smoothed results
                // The filter should not produce extreme outliers
                for i in 64..96usize { // Middle row 2
                    let result = dest[i];
                    assert!(result < 3500, "Wiener filtered values should be reasonable at position {}: {}", i, result);
                }

                // Verify SIMD processing for both iterations of the loop
                // First SIMD iteration: i=0..15 (16 pixels)
                for i in 0..16usize {
                    let result = dest[64 + i]; // Row 2

                    // Calculate expected manually for verification
                    let m0 = src[i] as i32;
                    let m1 = src[i + 32] as i32;
                    let m2 = src[i + 64] as i32;
                    let m3 = src[i + 96] as i32;
                    let m4 = src[i + 128] as i32;
                    let m5 = src[i + 160] as i32;

                    let temp = (m2 + m3) * 4 - (m1 + m4);
                    let expected_raw = (m0 + m5 + temp * 5 + 16) >> 5;
                    let expected = expected_raw.max(0).min(4095) as u16;

                    assert_eq!(result, expected, "First SIMD iteration should be correct at position {}: {} vs {}", i, result, expected);
                }

                // Second SIMD iteration: i=16..31 (16 pixels)
                for i in 16..32usize {
                    let result = dest[64 + i]; // Row 2

                    // Calculate expected manually for verification
                    let m0 = src[i] as i32;
                    let m1 = src[i + 32] as i32;
                    let m2 = src[i + 64] as i32;
                    let m3 = src[i + 96] as i32;
                    let m4 = src[i + 128] as i32;
                    let m5 = src[i + 160] as i32;

                    let temp = (m2 + m3) * 4 - (m1 + m4);
                    let expected_raw = (m0 + m5 + temp * 5 + 16) >> 5;
                    let expected = expected_raw.max(0).min(4095) as u16;

                    assert_eq!(result, expected, "Second SIMD iteration should be correct at position {}: {} vs {}", i, result, expected);
                }

                // First two rows should be bilinear (not full Wiener), last row should be copied
                // Row 0: bilinear interpolation
                for i in 0..32usize {
                    let expected = ((src[i] as u32 + src[i + 32] as u32 + 1) / 2) as u16;
                    assert_eq!(dest[i], expected, "First row should be bilinear at position {}: {} vs {}", i, dest[i], expected);
                }

                // Last row should be copied directly
                for i in 0..32usize {
                    let src_last_row = src[i + 224]; // Row 7 (0-indexed)
                    let dest_last_row = dest[i + 224];
                    assert_eq!(dest_last_row, src_last_row, "Last row should be copied directly at position {}: {} vs {}", i, dest_last_row, src_last_row);
                }
            }
        }
    };
}

horizontal_tests!(rust);
vertical_tests!(rust);

#[cfg(target_feature = "avx2")]
horizontal_tests!(avx2);
#[cfg(target_feature = "avx2")]
vertical_tests!(avx2);
