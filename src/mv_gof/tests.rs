#![allow(clippy::unwrap_used, reason = "allow in test files")]
#![allow(clippy::undocumented_unsafe_blocks, reason = "allow in test files")]

use vapoursynth::format::PresetFormat;

use super::*;
use crate::tests::create_test_env;

#[test]
fn mvgof_struct_fields() {
    // Test that the struct can be created and basic field access works
    // This test doesn't require complex VapourSynth setup
    let env = create_test_env(64, 48, PresetFormat::YUV420P8, 1).unwrap();
    let (node, _) = env.get_output(0).unwrap();
    let video_info = node.info();
    let format = video_info.format;

    let level_count = 2;
    let width = NonZeroUsize::new(64).unwrap();
    let height = NonZeroUsize::new(48).unwrap();
    let pel = Subpel::Full;
    let hpad = 8;
    let vpad = 8;
    let yuv_mode = MVPlaneSet::YPLANE; // Use single plane to simplify
    let x_ratio_uv = NonZeroU8::new(1).unwrap();
    let y_ratio_uv = NonZeroU8::new(1).unwrap();
    let bits_per_sample = NonZeroU8::new(8).unwrap();
    let pitch = [
        NonZeroUsize::new(80).unwrap(),
        NonZeroUsize::new(80).unwrap(),
        NonZeroUsize::new(80).unwrap(),
    ];

    let result = MVGroupOfFrames::new(
        level_count,
        width,
        height,
        pel,
        hpad,
        vpad,
        yuv_mode,
        x_ratio_uv,
        y_ratio_uv,
        bits_per_sample,
        &pitch,
        format,
    );

    assert!(result.is_ok(), "Should create MVGroupOfFrames successfully");
    let gof = result.unwrap();

    // Test basic structure properties
    assert_eq!(gof.level_count, level_count);
    assert_eq!(gof.frames.len(), level_count);
    assert_eq!(gof.pel, pel);
    assert_eq!(gof.x_ratio_uv, x_ratio_uv);
    assert_eq!(gof.y_ratio_uv, y_ratio_uv);

    // Test that width/height arrays are populated correctly
    assert_eq!(gof.width[0], width);
    assert_eq!(gof.height[0], height);

    // Test that hpad/vpad arrays are populated correctly
    assert_eq!(gof.hpad[0], hpad);
    assert_eq!(gof.vpad[0], vpad);

    // Test that all frames were created
    assert!(!gof.frames.is_empty());
    for frame in gof.frames.iter() {
        assert!(!frame.planes.is_empty(), "Each frame should have planes");
    }
}

#[test]
fn mvgof_different_level_counts() {
    let env = create_test_env(64, 48, PresetFormat::YUV420P8, 1).unwrap();
    let (node, _) = env.get_output(0).unwrap();
    let video_info = node.info();
    let format = video_info.format;

    for level_count in [1, 2, 3, 5] {
        let width = NonZeroUsize::new(64).unwrap();
        let height = NonZeroUsize::new(48).unwrap();
        let pel = Subpel::Full;
        let hpad = 8;
        let vpad = 8;
        let yuv_mode = MVPlaneSet::YPLANE;
        let x_ratio_uv = NonZeroU8::new(1).unwrap();
        let y_ratio_uv = NonZeroU8::new(1).unwrap();
        let bits_per_sample = NonZeroU8::new(8).unwrap();
        let pitch = [
            NonZeroUsize::new(80).unwrap(),
            NonZeroUsize::new(80).unwrap(),
            NonZeroUsize::new(80).unwrap(),
        ];

        let result = MVGroupOfFrames::new(
            level_count,
            width,
            height,
            pel,
            hpad,
            vpad,
            yuv_mode,
            x_ratio_uv,
            y_ratio_uv,
            bits_per_sample,
            &pitch,
            format,
        );

        assert!(result.is_ok(), "Should create with {} levels", level_count);
        let gof = result.unwrap();
        assert_eq!(gof.frames.len(), level_count);
        assert_eq!(gof.level_count, level_count);
    }
}

#[test]
fn mvgof_debug_and_clone() {
    let env = create_test_env(32, 32, PresetFormat::YUV420P8, 1).unwrap();
    let (node, _) = env.get_output(0).unwrap();
    let video_info = node.info();
    let format = video_info.format;

    let level_count = 2;
    let width = NonZeroUsize::new(32).unwrap();
    let height = NonZeroUsize::new(32).unwrap();
    let pel = Subpel::Full;
    let hpad = 4;
    let vpad = 4;
    let yuv_mode = MVPlaneSet::YPLANE;
    let x_ratio_uv = NonZeroU8::new(1).unwrap();
    let y_ratio_uv = NonZeroU8::new(1).unwrap();
    let bits_per_sample = NonZeroU8::new(8).unwrap();
    let pitch = [
        NonZeroUsize::new(40).unwrap(),
        NonZeroUsize::new(40).unwrap(),
        NonZeroUsize::new(40).unwrap(),
    ];

    let gof = MVGroupOfFrames::new(
        level_count,
        width,
        height,
        pel,
        hpad,
        vpad,
        yuv_mode,
        x_ratio_uv,
        y_ratio_uv,
        bits_per_sample,
        &pitch,
        format,
    )
    .unwrap();

    // Test Debug implementation
    let debug_str = format!("{:?}", gof);
    assert!(debug_str.contains("MVGroupOfFrames"));
    assert!(debug_str.contains("level_count"));

    // Test Clone implementation
    let cloned_gof = gof.clone();
    assert_eq!(gof.level_count, cloned_gof.level_count);
    assert_eq!(gof.frames.len(), cloned_gof.frames.len());
    assert_eq!(gof.width, cloned_gof.width);
    assert_eq!(gof.height, cloned_gof.height);
    assert_eq!(gof.pel, cloned_gof.pel);
}

// Note: More comprehensive tests would require complex VapourSynth Frame object creation
// which is better suited for integration tests. These unit tests focus on the basic
// constructor behavior and struct invariants that can be tested without complex mocking.
