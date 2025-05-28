use std::num::NonZeroUsize;

use super::reduce_bilinear;

#[test]
fn test_reduce_bilinear_u8_2x2() {
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

    reduce_bilinear(
        &mut dest,
        &src,
        dest_pitch,
        src_pitch,
        dest_width,
        dest_height,
    );

    // Bilinear filter is separable:
    // 1. Vertical: (10 + 30).div_ceil(2) = 20, (20 + 40).div_ceil(2) = 30 So
    //    intermediate = [20, 30]
    // 2. Horizontal: (20 + 30).div_ceil(2) = 25
    assert_eq!(dest[0], 25);
}

#[test]
fn test_reduce_bilinear_u8_4x2() {
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

    reduce_bilinear(
        &mut dest,
        &src,
        dest_pitch,
        src_pitch,
        dest_width,
        dest_height,
    );

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
fn test_reduce_bilinear_u8_4x4() {
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

    reduce_bilinear(
        &mut dest,
        &src,
        dest_pitch,
        src_pitch,
        dest_width,
        dest_height,
    );

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
fn test_reduce_bilinear_u8_6x4() {
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

    reduce_bilinear(
        &mut dest,
        &src,
        dest_pitch,
        src_pitch,
        dest_width,
        dest_height,
    );

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
fn test_reduce_bilinear_u16_basic() {
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

    reduce_bilinear(
        &mut dest,
        &src,
        dest_pitch,
        src_pitch,
        dest_width,
        dest_height,
    );

    // Bilinear filter:
    // Vertical: (1000 + 3000).div_ceil(2) = 2000, (2000 + 4000).div_ceil(2) = 3000
    // Horizontal: (2000 + 3000).div_ceil(2) = 2500
    assert_eq!(dest[0], 2500);
}

#[test]
fn test_reduce_bilinear_u16_large_values() {
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

    reduce_bilinear(
        &mut dest,
        &src,
        dest_pitch,
        src_pitch,
        dest_width,
        dest_height,
    );

    // Bilinear filter:
    // Vertical: (60000 + 62000).div_ceil(2) = 61000, (61000 + 63000).div_ceil(2) =
    // 62000 Horizontal: (61000 + 62000).div_ceil(2) = 61500
    assert_eq!(dest[0], 61500);
}

#[test]
fn test_reduce_bilinear_u16_4x4() {
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

    reduce_bilinear(
        &mut dest,
        &src,
        dest_pitch,
        src_pitch,
        dest_width,
        dest_height,
    );

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
fn test_reduce_bilinear_uniform_values() {
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

    reduce_bilinear(
        &mut dest,
        &src,
        dest_pitch,
        src_pitch,
        dest_width,
        dest_height,
    );

    // All outputs should be 128 since input is uniform
    assert_eq!(dest[0], 128);
    assert_eq!(dest[1], 128);
    assert_eq!(dest[4], 128);
    assert_eq!(dest[5], 128);
}

#[test]
fn test_reduce_bilinear_edge_case_2x2() {
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

    reduce_bilinear(
        &mut dest,
        &src,
        dest_pitch,
        src_pitch,
        dest_width,
        dest_height,
    );

    // Bilinear filter:
    // Vertical: (0 + 255).div_ceil(2) = 128, (255 + 0).div_ceil(2) = 128
    // Horizontal: (128 + 128).div_ceil(2) = 128
    assert_eq!(dest[0], 128);
}

#[test]
fn test_reduce_bilinear_with_padding() {
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

    reduce_bilinear(
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
    // Note: dest[1] might be modified due to intermediate processing
}

#[test]
fn test_reduce_bilinear_max_values() {
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

    reduce_bilinear(
        &mut dest,
        &src,
        dest_pitch,
        src_pitch,
        dest_width,
        dest_height,
    );

    // All 255 values should result in 255
    assert_eq!(dest[0], 255);
}

#[test]
fn test_reduce_bilinear_gradient() {
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

    reduce_bilinear(
        &mut dest,
        &src,
        dest_pitch,
        src_pitch,
        dest_width,
        dest_height,
    );

    // Values should form a smooth gradient
    // Check that values increase roughly from top-left to bottom-right
    assert!(dest[0] < dest[1]); // Top row: left < right
    assert!(dest[0] < dest[4]); // Left column: top < bottom
    assert!(dest[1] < dest[5]); // Right column: top < bottom
    assert!(dest[4] < dest[5]); // Bottom row: left < right

    // All values should be reasonable - no need to check u8 range as it's
    // guaranteed by type
}
