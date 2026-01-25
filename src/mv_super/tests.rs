#![allow(clippy::unwrap_used, reason = "allow in test files")]
#![allow(clippy::undocumented_unsafe_blocks, reason = "allow in test files")]

use quickcheck::TestResult;
use quickcheck_macros::quickcheck;
use vapoursynth::format::PresetFormat;

use super::*;
use crate::{
    params::{ReduceFilter, Subpel, SubpelMethod},
    tests::create_test_env,
};

#[test]
fn new_with_default_args() {
    let env = create_test_env(640, 480, PresetFormat::YUV420P8, 10).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    let super_instance = Super::new(node, None, None, None, None, None, None, None, None).unwrap();

    assert_eq!(super_instance.hpad, 16);
    assert_eq!(super_instance.vpad, 16);
    assert_eq!(super_instance.pel, Subpel::Half);
    assert_eq!(super_instance.levels, 8);
    assert!(super_instance.chroma);
    assert_eq!(super_instance.sharp, SubpelMethod::Wiener);
    assert_eq!(super_instance.rfilter, ReduceFilter::Bilinear);
}

#[quickcheck]
fn new_with_specified_args(
    hpad: usize,
    vpad: usize,
    pel: u8,
    levels: usize,
    chroma: bool,
    sharp: u8,
    rfilter: u8,
) -> TestResult {
    if ![1, 2, 4].contains(&pel)
        || !(0..3).contains(&sharp)
        || !(0..5).contains(&rfilter)
        || hpad > 1024
        || vpad > 1024
        || levels > 64
    {
        return TestResult::discard();
    }

    let env = create_test_env(640, 480, PresetFormat::YUV420P8, 10).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    let super_instance = Super::new(
        node,
        Some(hpad as i64),
        Some(vpad as i64),
        Some(pel as i64),
        Some(levels as i64),
        Some(chroma as i64),
        Some(sharp as i64),
        Some(rfilter as i64),
        None,
    )
    .unwrap();

    TestResult::from_bool(
        super_instance.hpad == hpad
            && super_instance.vpad == vpad
            && super_instance.pel == Subpel::try_from(pel as i64).unwrap()
            && super_instance.levels <= levels
            && super_instance.chroma == chroma
            && super_instance.sharp == SubpelMethod::try_from(sharp as i64).unwrap()
            && super_instance.rfilter == ReduceFilter::try_from(rfilter as i64).unwrap(),
    )
}

#[test]
fn super_dimension_calculations() {
    let env = create_test_env(64, 48, PresetFormat::YUV420P8, 5).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    let super_instance = Super::new(
        node,
        Some(8), // hpad
        Some(8), // vpad
        Some(2), // pel
        Some(3), // levels
        Some(1), // chroma
        Some(1), // sharp
        Some(1), // rfilter
        None,
    )
    .unwrap();

    // Verify super dimensions include padding
    assert_eq!(super_instance.super_width.get(), 64 + 2 * 8); // width + 2 * hpad
    assert!(super_instance.super_height.get() > 48); // height + padding + extra space for levels

    // Verify original dimensions are preserved
    assert_eq!(super_instance.width.get(), 64);
    assert_eq!(super_instance.height.get(), 48);
}

#[test]
fn levels_calculation() {
    let env = create_test_env(128, 96, PresetFormat::YUV420P8, 5).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    // Test auto level calculation (levels = 0)
    let super_instance = Super::new(
        node.clone(),
        Some(16), // hpad
        Some(16), // vpad
        Some(2),  // pel
        Some(0),  // levels (auto)
        Some(1),  // chroma
        Some(1),  // sharp
        Some(1),  // rfilter
        None,
    )
    .unwrap();

    // Auto levels should be calculated based on frame size
    assert!(super_instance.levels > 0);
    assert!(super_instance.levels <= 10); // reasonable upper bound

    // Test manual level setting
    let super_instance_manual = Super::new(
        node,
        Some(16), // hpad
        Some(16), // vpad
        Some(2),  // pel
        Some(5),  // levels (manual)
        Some(1),  // chroma
        Some(1),  // sharp
        Some(1),  // rfilter
        None,
    )
    .unwrap();

    assert!(super_instance_manual.levels <= 5);
}

#[test]
fn different_pel_values() {
    for pel in [1, 2, 4] {
        let env = create_test_env(64, 48, PresetFormat::YUV420P8, 5).unwrap();
        let (node, _) = env.get_output(0).unwrap();

        let super_instance = Super::new(
            node,
            Some(8),   // hpad
            Some(8),   // vpad
            Some(pel), // pel
            Some(3),   // levels
            Some(1),   // chroma
            Some(1),   // sharp
            Some(1),   // rfilter
            None,
        )
        .unwrap();

        assert_eq!(super_instance.pel, Subpel::try_from(pel).unwrap());
    }
}

#[test]
fn different_formats() {
    let formats = [
        PresetFormat::YUV420P8,
        PresetFormat::YUV420P16,
        PresetFormat::YUV422P8,
        PresetFormat::YUV422P16,
        PresetFormat::YUV444P8,
        PresetFormat::YUV444P16,
        PresetFormat::Gray8,
        PresetFormat::Gray16,
    ];

    for format in formats {
        let env = create_test_env(64, 48, format, 5).unwrap();
        let (node, _) = env.get_output(0).unwrap();

        let super_instance = Super::new(
            node,
            Some(8), // hpad
            Some(8), // vpad
            Some(2), // pel
            Some(2), // levels
            Some(1), // chroma
            Some(1), // sharp
            Some(1), // rfilter
            None,
        )
        .unwrap();

        // Verify format is preserved
        let expected_bytes = match format {
            PresetFormat::YUV420P16
            | PresetFormat::YUV422P16
            | PresetFormat::YUV444P16
            | PresetFormat::Gray16 => 2,
            _ => 1,
        };
        assert_eq!(super_instance.format.bytes_per_sample(), expected_bytes);
    }
}

#[test]
fn gray_format_chroma_handling() {
    let env = create_test_env(64, 48, PresetFormat::Gray8, 5).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    let super_instance = Super::new(
        node,
        Some(16), // hpad
        Some(16), // vpad
        Some(1),  // pel
        Some(2),  // levels
        Some(1),  // chroma (should be ignored for Gray format)
        Some(0),  // sharp
        Some(0),  // rfilter
        None,
    )
    .unwrap();

    // Verify chroma is disabled for Gray format
    assert!(!super_instance.chroma);

    // Verify format is preserved
    assert_eq!(super_instance.format.bytes_per_sample(), 1);
}

#[test]
fn error_handling_invalid_format() {
    // This test verifies that Super::new properly handles invalid inputs
    // Note: We can't easily test with invalid formats using create_test_env,
    // but we can verify the validation logic is in place by testing edge cases

    let env = create_test_env(64, 48, PresetFormat::YUV420P8, 5).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    // Test invalid pel value
    let result = Super::new(
        node.clone(),
        Some(8), // hpad
        Some(8), // vpad
        Some(3), // pel (invalid - not 1, 2, or 4)
        Some(3), // levels
        Some(1), // chroma
        Some(1), // sharp
        Some(1), // rfilter
        None,
    );
    assert!(result.is_err());

    // Test invalid sharp value
    let result = Super::new(
        node.clone(),
        Some(8), // hpad
        Some(8), // vpad
        Some(2), // pel
        Some(3), // levels
        Some(1), // chroma
        Some(5), // sharp (invalid - outside 0-2 range)
        Some(1), // rfilter
        None,
    );
    assert!(result.is_err());

    // Test invalid rfilter value
    let result = Super::new(
        node,
        Some(8),  // hpad
        Some(8),  // vpad
        Some(2),  // pel
        Some(3),  // levels
        Some(1),  // chroma
        Some(1),  // sharp
        Some(10), // rfilter (invalid - outside 0-4 range)
        None,
    );
    assert!(result.is_err());
}

#[test]
fn format_handling_8bit_vs_16bit() {
    // Test 8-bit format
    let env_8bit = create_test_env(64, 48, PresetFormat::YUV420P8, 5).unwrap();
    let (node_8bit, _) = env_8bit.get_output(0).unwrap();

    let super_instance_8bit = Super::new(
        node_8bit,
        Some(8), // hpad
        Some(8), // vpad
        Some(2), // pel
        Some(3), // levels
        Some(1), // chroma
        Some(1), // sharp
        Some(1), // rfilter
        None,
    )
    .unwrap();

    // Test 16-bit format
    let env_16bit = create_test_env(64, 48, PresetFormat::YUV420P16, 5).unwrap();
    let (node_16bit, _) = env_16bit.get_output(0).unwrap();

    let super_instance_16bit = Super::new(
        node_16bit,
        Some(8), // hpad
        Some(8), // vpad
        Some(2), // pel
        Some(3), // levels
        Some(1), // chroma
        Some(1), // sharp
        Some(1), // rfilter
        None,
    )
    .unwrap();

    // Verify format differences
    assert_eq!(super_instance_8bit.format.bytes_per_sample(), 1);
    assert_eq!(super_instance_16bit.format.bytes_per_sample(), 2);

    // Verify both have same dimensions
    assert_eq!(super_instance_8bit.width, super_instance_16bit.width);
    assert_eq!(super_instance_8bit.height, super_instance_16bit.height);
    assert_eq!(
        super_instance_8bit.super_width,
        super_instance_16bit.super_width
    );
    assert_eq!(
        super_instance_8bit.super_height,
        super_instance_16bit.super_height
    );
}

#[test]
fn subsampling_handling() {
    let formats_and_ratios = [
        (PresetFormat::YUV420P8, (2, 2)), // 4:2:0 subsampling
        (PresetFormat::YUV422P8, (2, 1)), // 4:2:2 subsampling
        (PresetFormat::YUV444P8, (1, 1)), // 4:4:4 no subsampling
        (PresetFormat::Gray8, (1, 1)),    // Gray has no chroma
    ];

    for (format, (expected_x_ratio, expected_y_ratio)) in formats_and_ratios {
        let env = create_test_env(64, 48, format, 5).unwrap();
        let (node, _) = env.get_output(0).unwrap();

        let super_instance = Super::new(
            node,
            Some(8), // hpad
            Some(8), // vpad
            Some(2), // pel
            Some(2), // levels
            Some(1), // chroma
            Some(1), // sharp
            Some(1), // rfilter
            None,
        )
        .unwrap();

        // Verify subsampling ratios are calculated correctly
        assert_eq!(super_instance.x_ratio_uv.get(), expected_x_ratio);
        assert_eq!(super_instance.y_ratio_uv.get(), expected_y_ratio);
    }
}

#[test]
fn padding_values() {
    let env = create_test_env(64, 48, PresetFormat::YUV420P8, 5).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    // Test different padding values
    let padding_values = [0, 8, 16, 32];

    for pad in padding_values {
        let super_instance = Super::new(
            node.clone(),
            Some(pad as i64), // hpad
            Some(pad as i64), // vpad
            Some(2),          // pel
            Some(2),          // levels
            Some(1),          // chroma
            Some(1),          // sharp
            Some(1),          // rfilter
            None,
        )
        .unwrap();

        assert_eq!(super_instance.hpad, pad);
        assert_eq!(super_instance.vpad, pad);

        // Verify padded dimensions
        assert_eq!(super_instance.super_width.get(), 64 + 2 * pad);
    }
}
