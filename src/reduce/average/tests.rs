#![allow(unused_unsafe)]
#![allow(clippy::undocumented_unsafe_blocks)]

use pastey::paste;
use std::num::NonZeroUsize;

macro_rules! create_tests {
    ($module:ident) => {
        paste! {
            #[test]
            fn [<test_reduce_average_u8_2x2_ $module>]() {
                // Test basic 2x2 -> 1x1 reduction
                let src = vec![
                    10u8, 20, // first row
                    30, 40, // second row
                ];
                let mut dest = vec![0u8; 1];
                let src_pitch = NonZeroUsize::new(2).unwrap();
                let dest_pitch = NonZeroUsize::new(1).unwrap();
                let dest_width = NonZeroUsize::new(1).unwrap();
                let dest_height = NonZeroUsize::new(1).unwrap();

                verify_asm!($module, reduce_average(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Expected: (10 + 20 + 30 + 40 + 2) / 4 = 102 / 4 = 25
                assert_eq!(dest[0], 25);
            }

            #[test]
            fn [<test_reduce_average_u8_4x2_ $module>]() {
                // Test 4x2 -> 2x1 reduction
                let src = vec![
                    10u8, 20, 30, 40, // first row
                    50, 60, 70, 80, // second row
                ];
                let mut dest = vec![0u8; 2];
                let src_pitch = NonZeroUsize::new(4).unwrap();
                let dest_pitch = NonZeroUsize::new(2).unwrap();
                let dest_width = NonZeroUsize::new(2).unwrap();
                let dest_height = NonZeroUsize::new(1).unwrap();

                verify_asm!($module, reduce_average(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // First 2x2 block: (10 + 20 + 50 + 60 + 2) / 4 = 142 / 4 = 35
                // Second 2x2 block: (30 + 40 + 70 + 80 + 2) / 4 = 222 / 4 = 55
                assert_eq!(dest[0], 35);
                assert_eq!(dest[1], 55);
            }

            #[test]
            fn [<test_reduce_average_u8_4x4_ $module>]() {
                // Test 4x4 -> 2x2 reduction
                let src = vec![
                    10u8, 20, 30, 40, // first row
                    50, 60, 70, 80, // second row
                    90, 100, 110, 120, // third row
                    130, 140, 150, 160, // fourth row
                ];
                let mut dest = vec![0u8; 4];
                let src_pitch = NonZeroUsize::new(4).unwrap();
                let dest_pitch = NonZeroUsize::new(2).unwrap();
                let dest_width = NonZeroUsize::new(2).unwrap();
                let dest_height = NonZeroUsize::new(2).unwrap();

                verify_asm!($module, reduce_average(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Top-left 2x2 block: (10 + 20 + 50 + 60 + 2) / 4 = 142 / 4 = 35
                // Top-right 2x2 block: (30 + 40 + 70 + 80 + 2) / 4 = 222 / 4 = 55
                // Bottom-left 2x2 block: (90 + 100 + 130 + 140 + 2) / 4 = 462 / 4 = 115
                // Bottom-right 2x2 block: (110 + 120 + 150 + 160 + 2) / 4 = 542 / 4 = 135
                assert_eq!(dest[0], 35); // Top-left
                assert_eq!(dest[1], 55); // Top-right
                assert_eq!(dest[2], 115); // Bottom-left
                assert_eq!(dest[3], 135); // Bottom-right
            }

            #[test]
            fn [<test_reduce_average_u8_with_padding_ $module>]() {
                // Test with source pitch > width (includes padding)
                let src = vec![
                    10u8, 20, 255, 255, // first row (last 2 are padding)
                    30, 40, 255, 255, // second row (last 2 are padding)
                ];
                let mut dest = vec![0u8; 2]; // padding in dest too
                let src_pitch = NonZeroUsize::new(4).unwrap();
                let dest_pitch = NonZeroUsize::new(2).unwrap();
                let dest_width = NonZeroUsize::new(1).unwrap();
                let dest_height = NonZeroUsize::new(1).unwrap();

                verify_asm!($module, reduce_average(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Should only process the first 2x2 block, ignoring padding
                // (10 + 20 + 30 + 40 + 2) / 4 = 102 / 4 = 25
                assert_eq!(dest[0], 25);
                assert_eq!(dest[1], 0); // padding should remain unchanged
            }

            #[test]
            fn [<test_reduce_average_u16_basic_ $module>]() {
                // Test with u16 values
                let src = vec![
                    1000u16, 2000, // first row
                    3000, 4000, // second row
                ];
                let mut dest = vec![0u16; 1];
                let src_pitch = NonZeroUsize::new(2).unwrap();
                let dest_pitch = NonZeroUsize::new(1).unwrap();
                let dest_width = NonZeroUsize::new(1).unwrap();
                let dest_height = NonZeroUsize::new(1).unwrap();

                verify_asm!($module, reduce_average(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Expected: (1000 + 2000 + 3000 + 4000 + 2) / 4 = 10002 / 4 = 2500
                assert_eq!(dest[0], 2500);
            }

            #[test]
            fn [<test_reduce_average_u16_large_values_ $module>]() {
                // Test with larger u16 values near the upper range
                let src = vec![
                    60000u16, 61000, // first row
                    62000, 63000, // second row
                ];
                let mut dest = vec![0u16; 1];
                let src_pitch = NonZeroUsize::new(2).unwrap();
                let dest_pitch = NonZeroUsize::new(1).unwrap();
                let dest_width = NonZeroUsize::new(1).unwrap();
                let dest_height = NonZeroUsize::new(1).unwrap();

                verify_asm!($module, reduce_average(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Expected: (60000 + 61000 + 62000 + 63000 + 2) / 4 = 246002 / 4 = 61500
                assert_eq!(dest[0], 61500);
            }

            #[test]
            fn [<test_reduce_average_u16_4x4_ $module>]() {
                // Test u16 with 4x4 -> 2x2 reduction
                let src = vec![
                    100u16, 200, 300, 400, // first row
                    500, 600, 700, 800, // second row
                    900, 1000, 1100, 1200, // third row
                    1300, 1400, 1500, 1600, // fourth row
                ];
                let mut dest = vec![0u16; 4];
                let src_pitch = NonZeroUsize::new(4).unwrap();
                let dest_pitch = NonZeroUsize::new(2).unwrap();
                let dest_width = NonZeroUsize::new(2).unwrap();
                let dest_height = NonZeroUsize::new(2).unwrap();

                verify_asm!($module, reduce_average(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Top-left 2x2 block: (100 + 200 + 500 + 600 + 2) / 4 = 1402 / 4 = 350
                // Top-right 2x2 block: (300 + 400 + 700 + 800 + 2) / 4 = 2202 / 4 = 550
                // Bottom-left 2x2 block: (900 + 1000 + 1300 + 1400 + 2) / 4 = 4602 / 4 = 1150
                // Bottom-right 2x2 block: (1100 + 1200 + 1500 + 1600 + 2) / 4 = 5402 / 4 = 1350
                assert_eq!(dest[0], 350); // Top-left
                assert_eq!(dest[1], 550); // Top-right
                assert_eq!(dest[2], 1150); // Bottom-left
                assert_eq!(dest[3], 1350); // Bottom-right
            }

            #[test]
            fn [<test_reduce_average_u8_edge_cases_ $module>]() {
                // Test edge cases with small values
                let src = vec![
                    0u8, 1, // first row
                    2, 3, // second row
                ];
                let mut dest = vec![0u8; 1];
                let src_pitch = NonZeroUsize::new(2).unwrap();
                let dest_pitch = NonZeroUsize::new(1).unwrap();
                let dest_width = NonZeroUsize::new(1).unwrap();
                let dest_height = NonZeroUsize::new(1).unwrap();

                verify_asm!($module, reduce_average(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Expected: (0 + 1 + 2 + 3 + 2) / 4 = 8 / 4 = 2
                assert_eq!(dest[0], 2);
            }

            #[test]
            fn [<test_reduce_average_u8_max_values_ $module>]() {
                // Test with maximum u8 values
                let src = vec![
                    255u8, 255, // first row
                    255, 255, // second row
                ];
                let mut dest = vec![0u8; 1];
                let src_pitch = NonZeroUsize::new(2).unwrap();
                let dest_pitch = NonZeroUsize::new(1).unwrap();
                let dest_width = NonZeroUsize::new(1).unwrap();
                let dest_height = NonZeroUsize::new(1).unwrap();

                verify_asm!($module, reduce_average(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Expected: (255 + 255 + 255 + 255 + 2) / 4 = 1022 / 4 = 255
                assert_eq!(dest[0], 255);
            }

            #[test]
            fn [<test_reduce_average_u16_edge_cases_ $module>]() {
                // Test edge cases with small u16 values
                let src = vec![
                    0u16, 1, // first row
                    2, 3, // second row
                ];
                let mut dest = vec![0u16; 1];
                let src_pitch = NonZeroUsize::new(2).unwrap();
                let dest_pitch = NonZeroUsize::new(1).unwrap();
                let dest_width = NonZeroUsize::new(1).unwrap();
                let dest_height = NonZeroUsize::new(1).unwrap();

                verify_asm!($module, reduce_average(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Expected: (0 + 1 + 2 + 3 + 2) / 4 = 8 / 4 = 2
                assert_eq!(dest[0], 2);
            }

            #[test]
            fn [<test_reduce_average_rounding_behavior_ $module>]() {
                // Test rounding behavior with the +2 bias
                // The function adds T::from(2u8) before dividing by 4, which provides rounding
                let src = vec![
                    1u8, 1, // first row
                    1, 1, // second row
                ];
                let mut dest = vec![0u8; 1];
                let src_pitch = NonZeroUsize::new(2).unwrap();
                let dest_pitch = NonZeroUsize::new(1).unwrap();
                let dest_width = NonZeroUsize::new(1).unwrap();
                let dest_height = NonZeroUsize::new(1).unwrap();

                verify_asm!($module, reduce_average(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Expected: (1 + 1 + 1 + 1 + 2) / 4 = 6 / 4 = 1
                assert_eq!(dest[0], 1);

                // Test another rounding case
                let src2 = vec![
                    1u8, 1, // first row
                    1, 2, // second row
                ];
                let mut dest2 = vec![0u8; 1];

                verify_asm!($module, reduce_average(
                    &mut dest2,
                    &src2,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Expected: (1 + 1 + 1 + 2 + 2) / 4 = 7 / 4 = 1
                assert_eq!(dest2[0], 1);
            }

            #[test]
            fn [<test_reduce_average_u8_large_simd_ $module>]() {
                // Test large enough to trigger SIMD processing (32x2 -> 16x1)
                // This ensures we cover the SIMD loop in AVX2 implementation
                let mut src = Vec::new();

                // First row: 32 pixels with values 0-31
                for i in 0..32u8 {
                    src.push(i);
                }

                // Second row: 32 pixels with values 32-63
                for i in 32..64u8 {
                    src.push(i);
                }

                let mut dest = vec![0u8; 16];
                let src_pitch = NonZeroUsize::new(32).unwrap();
                let dest_pitch = NonZeroUsize::new(16).unwrap();
                let dest_width = NonZeroUsize::new(16).unwrap();
                let dest_height = NonZeroUsize::new(1).unwrap();

                verify_asm!($module, reduce_average(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Verify each 2x2 block average
                for i in 0..16usize {
                    let x = i * 2;
                    let top_left = src[x] as u16;
                    let top_right = src[x + 1] as u16;
                    let bottom_left = src[32 + x] as u16;
                    let bottom_right = src[32 + x + 1] as u16;

                    let expected = ((top_left + top_right + bottom_left + bottom_right + 2) / 4) as u8;
                    assert_eq!(dest[i], expected, "Mismatch at position {}", i);
                }
            }

            #[test]
            fn [<test_reduce_average_u16_large_simd_ $module>]() {
                // Test large enough to trigger SIMD processing for u16 (16x2 -> 8x1)
                // This ensures we cover the u16 SIMD loop in AVX2 implementation
                let mut src = Vec::new();

                // First row: 16 pixels with values 0-15 scaled to u16 range
                for i in 0..16u16 {
                    src.push(i * 1000);
                }

                // Second row: 16 pixels with values 16-31 scaled to u16 range
                for i in 16..32u16 {
                    src.push(i * 1000);
                }

                let mut dest = vec![0u16; 8];
                let src_pitch = NonZeroUsize::new(16).unwrap();
                let dest_pitch = NonZeroUsize::new(8).unwrap();
                let dest_width = NonZeroUsize::new(8).unwrap();
                let dest_height = NonZeroUsize::new(1).unwrap();

                verify_asm!($module, reduce_average(
                    &mut dest,
                    &src,
                    dest_pitch,
                    src_pitch,
                    dest_width,
                    dest_height,
                ));

                // Verify each 2x2 block average
                for i in 0..8usize {
                    let x = i * 2;
                    let top_left = src[x] as u32;
                    let top_right = src[x + 1] as u32;
                    let bottom_left = src[16 + x] as u32;
                    let bottom_right = src[16 + x + 1] as u32;

                    let expected = ((top_left + top_right + bottom_left + bottom_right + 2) / 4) as u16;
                    assert_eq!(dest[i], expected, "Mismatch at position {}", i);
                }
            }
        }
    };
}

create_tests!(rust);

#[cfg(target_feature = "avx2")]
create_tests!(avx2);
