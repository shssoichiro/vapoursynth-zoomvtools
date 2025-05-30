use std::num::NonZeroUsize;

use super::average2;

#[test]
fn test_average2_u8_basic() {
    let src1 = [0u8, 2, 4, 6];
    let src2 = [1u8, 3, 5, 7];
    let mut dest = [0u8; 4];

    average2(
        &src1,
        &src2,
        &mut dest,
        NonZeroUsize::new(2).unwrap(), // pitch
        NonZeroUsize::new(2).unwrap(), // width
        NonZeroUsize::new(2).unwrap(), // height
    );

    // Expected: ceiling division of (0+1)/2=1, (2+3)/2=3, (4+5)/2=5, (6+7)/2=7
    assert_eq!(dest, [1, 3, 5, 7]);
}

#[test]
fn test_average2_u8_ceiling_division() {
    let src1 = [0u8, 1, 2, 3];
    let src2 = [1u8, 2, 3, 4];
    let mut dest = [0u8; 4];

    average2(
        &src1,
        &src2,
        &mut dest,
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(2).unwrap(),
    );

    // Expected: (0+1).div_ceil(2)=1, (1+2).div_ceil(2)=2, (2+3).div_ceil(2)=3,
    // (3+4).div_ceil(2)=4
    assert_eq!(dest, [1, 2, 3, 4]);
}

#[test]
fn test_average2_u8_max_values() {
    let src1 = [254u8, 255, 254, 255];
    let src2 = [255u8, 254, 255, 254];
    let mut dest = [0u8; 4];

    average2(
        &src1,
        &src2,
        &mut dest,
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(2).unwrap(),
    );

    // Expected: (254+255).div_ceil(2)=255, (255+254).div_ceil(2)=255, etc.
    assert_eq!(dest, [255, 255, 255, 255]);
}

#[test]
fn test_average2_u8_zero_values() {
    let src1 = [0u8, 0, 1, 1];
    let src2 = [0u8, 1, 0, 1];
    let mut dest = [0u8; 4];

    average2(
        &src1,
        &src2,
        &mut dest,
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(2).unwrap(),
    );

    // Expected: (0+0).div_ceil(2)=0, (0+1).div_ceil(2)=1, (1+0).div_ceil(2)=1,
    // (1+1).div_ceil(2)=1
    assert_eq!(dest, [0, 1, 1, 1]);
}

#[test]
fn test_average2_u16_basic() {
    let src1 = [0u16, 2, 4, 6];
    let src2 = [1u16, 3, 5, 7];
    let mut dest = [0u16; 4];

    average2(
        &src1,
        &src2,
        &mut dest,
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(2).unwrap(),
    );

    // Expected: ceiling division results
    assert_eq!(dest, [1, 3, 5, 7]);
}

#[test]
fn test_average2_u16_high_values() {
    let src1 = [1000u16, 2000, 3000, 4000];
    let src2 = [1001u16, 2001, 3001, 4001];
    let mut dest = [0u16; 4];

    average2(
        &src1,
        &src2,
        &mut dest,
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(2).unwrap(),
    );

    // Expected: (1000+1001).div_ceil(2)=1001, (2000+2001).div_ceil(2)=2001, etc.
    assert_eq!(dest, [1001, 2001, 3001, 4001]);
}

#[test]
fn test_average2_u16_max_values() {
    let src1 = [65534u16, 65535, 65534, 65535];
    let src2 = [65535u16, 65534, 65535, 65534];
    let mut dest = [0u16; 4];

    average2(
        &src1,
        &src2,
        &mut dest,
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(2).unwrap(),
    );

    // Expected: all values should be 65535 (max u16)
    assert_eq!(dest, [65535, 65535, 65535, 65535]);
}

#[test]
fn test_average2_u8_single_pixel() {
    let src1 = [100u8];
    let src2 = [200u8];
    let mut dest = [0u8; 1];

    average2(
        &src1,
        &src2,
        &mut dest,
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(1).unwrap(),
    );

    // Expected: (100+200).div_ceil(2) = 150
    assert_eq!(dest, [150]);
}

#[test]
fn test_average2_u8_different_pitch() {
    // Test with pitch > width to simulate actual image data with padding
    let src1 = [10u8, 20, 0, 0, 30, 40, 0, 0]; // 2x2 image with pitch=4
    let src2 = [11u8, 21, 0, 0, 31, 41, 0, 0]; // 2x2 image with pitch=4
    let mut dest = [0u8; 8];

    average2(
        &src1,
        &src2,
        &mut dest,
        NonZeroUsize::new(4).unwrap(), // pitch = 4
        NonZeroUsize::new(2).unwrap(), // width = 2
        NonZeroUsize::new(2).unwrap(), // height = 2
    );

    // Expected: only positions [0,1,4,5] should be modified
    // (10+11).div_ceil(2)=11, (20+21).div_ceil(2)=21, (30+31).div_ceil(2)=31,
    // (40+41).div_ceil(2)=41
    assert_eq!(dest[0], 11);
    assert_eq!(dest[1], 21);
    assert_eq!(dest[2], 0); // unchanged (padding)
    assert_eq!(dest[3], 0); // unchanged (padding)
    assert_eq!(dest[4], 31);
    assert_eq!(dest[5], 41);
    assert_eq!(dest[6], 0); // unchanged (padding)
    assert_eq!(dest[7], 0); // unchanged (padding)
}

#[test]
fn test_average2_u16_different_pitch() {
    // Test with pitch > width
    let src1 = [1000u16, 2000, 0, 0, 3000, 4000, 0, 0];
    let src2 = [1001u16, 2001, 0, 0, 3001, 4001, 0, 0];
    let mut dest = [0u16; 8];

    average2(
        &src1,
        &src2,
        &mut dest,
        NonZeroUsize::new(4).unwrap(), // pitch = 4
        NonZeroUsize::new(2).unwrap(), // width = 2
        NonZeroUsize::new(2).unwrap(), // height = 2
    );

    // Expected: only positions [0,1,4,5] should be modified
    assert_eq!(dest[0], 1001);
    assert_eq!(dest[1], 2001);
    assert_eq!(dest[2], 0); // unchanged
    assert_eq!(dest[3], 0); // unchanged
    assert_eq!(dest[4], 3001);
    assert_eq!(dest[5], 4001);
    assert_eq!(dest[6], 0); // unchanged
    assert_eq!(dest[7], 0); // unchanged
}

#[test]
fn test_average2_u8_rectangular() {
    // Test with non-square dimensions: 3x1 rectangle
    let src1 = [10u8, 20, 30];
    let src2 = [11u8, 21, 31];
    let mut dest = [0u8; 3];

    average2(
        &src1,
        &src2,
        &mut dest,
        NonZeroUsize::new(3).unwrap(), // pitch = width = 3
        NonZeroUsize::new(3).unwrap(), // width = 3
        NonZeroUsize::new(1).unwrap(), // height = 1
    );

    assert_eq!(dest, [11, 21, 31]);
}

#[test]
fn test_average2_u8_odd_addition_results() {
    // Test cases where addition results in odd numbers to verify ceiling division
    let src1 = [1u8, 3, 5, 7];
    let src2 = [2u8, 4, 6, 8];
    let mut dest = [0u8; 4];

    average2(
        &src1,
        &src2,
        &mut dest,
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(2).unwrap(),
    );

    // Expected: (1+2).div_ceil(2)=2, (3+4).div_ceil(2)=4, (5+6).div_ceil(2)=6,
    // (7+8).div_ceil(2)=8
    assert_eq!(dest, [2, 4, 6, 8]);
}

#[test]
fn test_average2_u16_odd_addition_results() {
    // Test cases where addition results in odd numbers
    let src1 = [101u16, 103, 105, 107];
    let src2 = [102u16, 104, 106, 108];
    let mut dest = [0u16; 4];

    average2(
        &src1,
        &src2,
        &mut dest,
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(2).unwrap(),
    );

    // Expected: (101+102).div_ceil(2)=102, (103+104).div_ceil(2)=104, etc.
    assert_eq!(dest, [102, 104, 106, 108]);
}
