#![allow(clippy::unwrap_used, reason = "allow in test files")]
#![allow(clippy::undocumented_unsafe_blocks, reason = "allow in test files")]

use anyhow::Result;
use vapoursynth::{
    format::{FormatID, PresetFormat},
    prelude::Environment,
};

use super::Analyse;
use crate::params::{
    DctMode,
    DivideMode,
    MVPlaneSet,
    MotionFlags,
    PenaltyScaling,
    SearchType,
    Subpel,
};

fn create_test_env(
    width: usize,
    height: usize,
    format: PresetFormat,
    frames: usize,
    super_height: usize,
    super_hpad: usize,
    super_levels: usize,
    super_modeyuv: usize,
    super_pel: usize,
    super_vpad: usize,
) -> Result<Environment> {
    let format = i32::from(FormatID::from(format));
    let script = format!(
        r#"
import vapoursynth as vs
core = vs.core
clip = core.std.BlankClip(width={width}, height={height}, format={format}, length={frames})
clip = core.std.SetFrameProps(
    clip,
    Super_height={super_height},
    Super_hpad={super_hpad},
    Super_levels={super_levels},
    Super_modeyuv={super_modeyuv},
    Super_pel={super_pel},
    Super_vpad={super_vpad}
)
clip.set_output()
"#,
    );

    let env = Environment::from_script(&script)?;
    Ok(env)
}

#[test]
fn analyse_new_defaults() {
    // These are the expected results from a 640x480 input to Super.
    let env = create_test_env(672, 2750, PresetFormat::YUV420P8, 10, 480, 16, 8, 7, 2, 16).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    let analyse = Analyse::new(
        node, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    )
    .expect("Failed to create Analyse struct");

    // Verify default parameter values
    assert_eq!(analyse.levels, 0, "Default levels should be 0 (auto)");
    assert_eq!(
        analyse.search_type,
        SearchType::Hex2,
        "Default search type should be Hex2"
    );
    assert_eq!(
        analyse.search_type_coarse,
        SearchType::Exhaustive,
        "Default coarse search should be Exhaustive"
    );
    assert_eq!(analyse.search_param, 2, "Default search param should be 2");
    assert_eq!(
        analyse.pel_search, 2,
        "Default pel_search should equal super_pel (2)"
    );
    assert!(
        analyse.chroma,
        "Default chroma should be true for YUV format"
    );
    assert!(analyse.truemotion, "Default truemotion should be true");
    assert_eq!(
        analyse.penalty_level,
        PenaltyScaling::Linear,
        "Default penalty_level should be Linear for truemotion=true"
    );
    assert!(
        analyse.global,
        "Default global should match truemotion (true)"
    );
    assert_eq!(
        analyse.penalty_new, 50,
        "Default penalty_new should be 50 for truemotion=true"
    );
    assert_eq!(
        analyse.penalty_zero, 50,
        "Default penalty_zero should equal penalty_new"
    );
    assert_eq!(
        analyse.penalty_global, 0,
        "Default penalty_global should be 0"
    );
    assert_eq!(
        analyse.dct_mode,
        DctMode::Spatial,
        "Default dctmode should be Spatial"
    );
    assert_eq!(
        analyse.divide_extra,
        DivideMode::None,
        "Default divide should be None"
    );
    assert_eq!(analyse.bad_range, 24, "Default bad_range should be 24");
    assert!(analyse.meander, "Default meander should be true");
    assert!(!analyse.try_many, "Default try_many should be false");
    assert!(!analyse.fields, "Default fields should be false");
    assert_eq!(analyse.tff, None, "Default tff should be None");

    // Verify calculated values for 8x8 blocks, truemotion=true, 8-bit format
    // lambda = (1000 * 8 * 8 / 64) = 1000, scaled for 8-bit = 1000
    assert_eq!(
        analyse.lambda, 1000,
        "Lambda should be calculated correctly for 8x8 blocks"
    );
    // lambda_sad = 1200 for truemotion, scaled for 8-bit = 1200, block-scaled = 1200
    assert_eq!(
        analyse.lambda_sad, 1200,
        "Lambda_sad should be calculated correctly"
    );
    // bad_sad = 10000, scaled for 8-bit = 10000, block-scaled = 10000
    assert_eq!(
        analyse.bad_sad, 10000,
        "Bad_sad should be calculated correctly"
    );

    // Verify format and internal properties
    assert_eq!(
        analyse.format.bits_per_sample(),
        8,
        "Format should be 8-bit"
    );
    assert_eq!(
        analyse.yuv_mode,
        MVPlaneSet::YUVPLANES,
        "YUV mode should include all planes for chroma=true"
    );
    assert_eq!(
        analyse.super_hpad, 16,
        "Super hpad should match frame property"
    );
    assert_eq!(
        analyse.super_vpad, 16,
        "Super vpad should match frame property"
    );
    assert_eq!(
        analyse.super_pel,
        Subpel::Half,
        "Super pel should be Half (2)"
    );
    assert_eq!(
        analyse.super_mode_yuv,
        MVPlaneSet::from_bits(7).unwrap(),
        "Super mode YUV should match frame property"
    );
    assert_eq!(
        analyse.super_levels, 8,
        "Super levels should match frame property"
    );

    // Verify analysis data
    assert_eq!(
        analyse.analysis_data.blk_size_x.get(),
        8,
        "Block size X should be 8"
    );
    assert_eq!(
        analyse.analysis_data.blk_size_y.get(),
        8,
        "Block size Y should be 8"
    );
    assert_eq!(
        analyse.analysis_data.pel,
        Subpel::Half,
        "Analysis data pel should match super pel"
    );
    assert_eq!(
        analyse.analysis_data.delta_frame, 1,
        "Default delta frame should be 1"
    );
    assert!(
        !analyse.analysis_data.is_backward,
        "Default is_backward should be false"
    );
    assert_eq!(
        analyse.analysis_data.width.get(),
        640,
        "Analysis width should be super_width"
    );
    assert_eq!(
        analyse.analysis_data.height.get(),
        480,
        "Analysis height should be super_height"
    );
    assert_eq!(
        analyse.analysis_data.overlap_x, 0,
        "Default overlap X should be 0"
    );
    assert_eq!(
        analyse.analysis_data.overlap_y, 0,
        "Default overlap Y should be 0"
    );
    assert_eq!(
        analyse.analysis_data.bits_per_sample.get(),
        8,
        "Bits per sample should be 8"
    );
    assert_eq!(
        analyse.analysis_data.h_padding, 16,
        "H padding should match super hpad"
    );
    assert_eq!(
        analyse.analysis_data.v_padding, 16,
        "V padding should match super vpad"
    );

    // Verify motion flags
    let expected_flags = MotionFlags::USE_CHROMA_MOTION;
    assert_eq!(
        analyse.analysis_data.motion_flags, expected_flags,
        "Motion flags should include chroma"
    );

    // Verify no divided data for default case
    assert!(
        analyse.analysis_data_divided.is_none(),
        "Analysis data divided should be None for divide=None"
    );
}

#[test]
fn analyse_new_custom_block_sizes() {
    let env = create_test_env(672, 2750, PresetFormat::YUV420P8, 10, 480, 16, 8, 7, 2, 16).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    // Test different valid block sizes
    let valid_block_sizes = [
        (4, 4),
        (8, 4),
        (8, 8),
        (16, 2),
        (16, 8),
        (16, 16),
        (32, 16),
        (32, 32),
        (64, 32),
        (64, 64),
        (128, 64),
        (128, 128),
    ];

    for (blk_x, blk_y) in valid_block_sizes {
        let analyse = Analyse::new(
            node.clone(),
            Some(blk_x),
            Some(blk_y),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap_or_else(|_| panic!("Should accept valid block size {}x{}", blk_x, blk_y));

        assert_eq!(analyse.analysis_data.blk_size_x.get(), blk_x as usize);
        assert_eq!(analyse.analysis_data.blk_size_y.get(), blk_y as usize);
    }
}

#[test]
fn analyse_new_invalid_block_sizes() {
    let env = create_test_env(672, 2750, PresetFormat::YUV420P8, 10, 480, 16, 8, 7, 2, 16).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    // Test invalid block sizes
    let invalid_block_sizes = [(3, 3), (5, 5), (7, 7), (9, 9), (15, 15), (12, 12), (16, 3)];

    for (blk_x, blk_y) in invalid_block_sizes {
        let result = Analyse::new(
            node.clone(),
            Some(blk_x),
            Some(blk_y),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert!(
            result.is_err(),
            "Should reject invalid block size {}x{}",
            blk_x,
            blk_y
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("block size must be")
        );
    }
}

#[test]
fn analyse_new_penalty_validation() {
    let env = create_test_env(672, 2750, PresetFormat::YUV420P8, 10, 480, 16, 8, 7, 2, 16).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    // Test valid penalty values (0-256 inclusive)
    for penalty in [0, 128, 256] {
        let result = Analyse::new(
            node.clone(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(penalty),
            Some(penalty),
            Some(penalty),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(
            result.is_ok(),
            "Should accept valid penalty value {}",
            penalty
        );
    }

    // Test invalid penalty values (>256)
    for penalty in [257, 300, 1000] {
        // Test pnew
        let result = Analyse::new(
            node.clone(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(penalty),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(
            result.is_err(),
            "Should reject invalid pnew value {}",
            penalty
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("pnew must be between 0 and 256")
        );

        // Test pzero
        let result = Analyse::new(
            node.clone(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(penalty),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(
            result.is_err(),
            "Should reject invalid pzero value {}",
            penalty
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("pzero must be between 0 and 256")
        );

        // Test pglobal
        let result = Analyse::new(
            node.clone(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(penalty),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(
            result.is_err(),
            "Should reject invalid pglobal value {}",
            penalty
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("pglobal must be between 0 and 256")
        );
    }
}

#[test]
fn analyse_new_overlap_validation() {
    let env = create_test_env(672, 2750, PresetFormat::YUV420P8, 10, 480, 16, 8, 7, 2, 16).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    // Test valid overlaps (up to half of block size)
    let result = Analyse::new(
        node.clone(),
        Some(8),
        Some(8),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(4),
        Some(4),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert!(
        result.is_ok(),
        "Should accept overlap up to half of block size"
    );

    // Test invalid overlaps (more than half of block size)
    let result = Analyse::new(
        node.clone(),
        Some(8),
        Some(8),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(5),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert!(
        result.is_err(),
        "Should reject overlap greater than half of block size"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("overlap must be at most half")
    );
}

#[test]
fn analyse_new_divide_validation() {
    let env = create_test_env(672, 2750, PresetFormat::YUV420P8, 10, 480, 16, 8, 7, 2, 16).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    // Test divide with valid block size (8x8)
    let result = Analyse::new(
        node.clone(),
        Some(8),
        Some(8),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(1),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert!(result.is_ok(), "Should accept divide with 8x8 blocks");

    // Test divide with invalid block size (4x4)
    let result = Analyse::new(
        node.clone(),
        Some(4),
        Some(4),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(1),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert!(
        result.is_err(),
        "Should reject divide with blocks smaller than 8x8"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("blksize and blksizev must be at least 8 when divide=True")
    );
}

#[test]
fn analyse_new_dct_satd_validation() {
    let env = create_test_env(672, 2750, PresetFormat::YUV420P8, 10, 480, 16, 8, 7, 2, 16).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    // Test SATD DCT modes with 16x2 blocks (should fail)
    for dct_mode in [5, 6, 7, 8, 9, 10] {
        let result = Analyse::new(
            node.clone(),
            Some(16),
            Some(2),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(dct_mode),
        );
        assert!(
            result.is_err(),
            "Should reject SATD DCT mode {} with 16x2 blocks",
            dct_mode
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("dct 5-10 cannot work with 16x2 blocks")
        );
    }

    // Test SATD DCT modes with compatible blocks (should succeed)
    let result = Analyse::new(
        node.clone(),
        Some(8),
        Some(8),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(5),
    );
    assert!(
        result.is_ok(),
        "Should accept SATD DCT mode with compatible blocks"
    );
}

#[test]
fn analyse_new_truemotion_false() {
    let env = create_test_env(672, 2750, PresetFormat::YUV420P8, 10, 480, 16, 8, 7, 2, 16).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    let analyse = Analyse::new(
        node,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(0),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();

    assert!(!analyse.truemotion, "Truemotion should be false");
    assert_eq!(
        analyse.penalty_new, 0,
        "Penalty_new should be 0 for truemotion=false"
    );
    assert_eq!(
        analyse.penalty_level,
        PenaltyScaling::None,
        "Penalty_level should be None for truemotion=false"
    );
    assert!(
        !analyse.global,
        "Global should be false for truemotion=false"
    );
    assert_eq!(analyse.lambda, 0, "Lambda should be 0 for truemotion=false");
    assert_eq!(
        analyse.lambda_sad, 400,
        "Lambda_sad should be 400 for truemotion=false"
    );
}

#[test]
fn analyse_new_backward_motion() {
    let env = create_test_env(672, 2750, PresetFormat::YUV420P8, 10, 480, 16, 8, 7, 2, 16).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    let analyse = Analyse::new(
        node,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(1),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();

    assert!(
        analyse.analysis_data.is_backward,
        "Is_backward should be true"
    );
    let expected_flags = MotionFlags::IS_BACKWARD | MotionFlags::USE_CHROMA_MOTION;
    assert_eq!(
        analyse.analysis_data.motion_flags, expected_flags,
        "Motion flags should include backward"
    );
}

#[test]
fn analyse_new_chroma_disabled() {
    let env = create_test_env(672, 2750, PresetFormat::YUV420P8, 10, 480, 16, 8, 7, 2, 16).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    let analyse = Analyse::new(
        node,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(0),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();

    assert!(!analyse.chroma, "Chroma should be false");
    assert_eq!(
        analyse.yuv_mode,
        MVPlaneSet::YPLANE,
        "YUV mode should be Y-plane only"
    );
    let expected_flags = MotionFlags::empty(); // No chroma flag
    assert_eq!(
        analyse.analysis_data.motion_flags, expected_flags,
        "Motion flags should not include chroma"
    );
}

#[test]
fn analyse_new_grayscale_format() {
    let env = create_test_env(672, 2750, PresetFormat::Gray8, 10, 480, 16, 8, 1, 2, 16).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    let analyse = Analyse::new(
        node, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    )
    .unwrap();

    assert!(
        !analyse.chroma,
        "Chroma should be automatically disabled for grayscale"
    );
    assert_eq!(
        analyse.yuv_mode,
        MVPlaneSet::YPLANE,
        "YUV mode should be Y-plane only for grayscale"
    );
}

#[test]
fn analyse_new_custom_delta() {
    let env = create_test_env(672, 2750, PresetFormat::YUV420P8, 10, 480, 16, 8, 7, 2, 16).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    // Test custom delta frame
    let analyse = Analyse::new(
        node.clone(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(3),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        analyse.analysis_data.delta_frame, 3,
        "Delta frame should be 3"
    );

    // Test negative delta (static mode) - should fail if pointing past clip end
    let result = Analyse::new(
        node,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(-15),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert!(
        result.is_err(),
        "Should reject delta pointing past clip end"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("delta points to frame past the input clip's end")
    );
}

#[test]
fn analyse_new_divide_creates_extra_data() {
    let env = create_test_env(672, 2750, PresetFormat::YUV420P8, 10, 480, 16, 8, 7, 2, 16).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    let analyse = Analyse::new(
        node,
        Some(16),
        Some(16),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(1),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();

    assert!(
        analyse.analysis_data_divided.is_some(),
        "Should create divided analysis data"
    );
    let divided_data = analyse.analysis_data_divided.unwrap();

    // Divided data should have doubled block counts and halved block sizes
    assert_eq!(
        divided_data.blk_x.get(),
        analyse.analysis_data.blk_x.get() * 2
    );
    assert_eq!(
        divided_data.blk_y.get(),
        analyse.analysis_data.blk_y.get() * 2
    );
    assert_eq!(
        divided_data.blk_size_x.get(),
        analyse.analysis_data.blk_size_x.get() / 2
    );
    assert_eq!(
        divided_data.blk_size_y.get(),
        analyse.analysis_data.blk_size_y.get() / 2
    );
    assert_eq!(divided_data.overlap_x, analyse.analysis_data.overlap_x / 2);
    assert_eq!(divided_data.overlap_y, analyse.analysis_data.overlap_y / 2);
    assert_eq!(
        divided_data.level_count,
        analyse.analysis_data.level_count + 1
    );
}

#[test]
fn analyse_new_search_param_adjustment() {
    let env = create_test_env(672, 2750, PresetFormat::YUV420P8, 10, 480, 16, 8, 7, 2, 16).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    // Test Nstep search with negative param (should be adjusted to 0)
    let analyse = Analyse::new(
        node.clone(),
        None,
        None,
        None,
        Some(1),
        Some(-5),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        analyse.search_param, 0,
        "Negative search param should be adjusted to 0 for Nstep"
    );

    // Test non-Nstep search with param < 1 (should be adjusted to 1)
    let analyse = Analyse::new(
        node,
        None,
        None,
        None,
        Some(4),
        Some(0),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        analyse.search_param, 1,
        "Zero search param should be adjusted to 1 for non-Nstep"
    );
}

#[test]
fn analyse_new_pel_search_auto() {
    let env = create_test_env(672, 2750, PresetFormat::YUV420P8, 10, 480, 16, 8, 7, 2, 16).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    // Test pelsearch=0 (auto) should use super_pel value
    let analyse = Analyse::new(
        node.clone(),
        None,
        None,
        None,
        None,
        None,
        Some(0),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        analyse.pel_search, 2,
        "Pel_search should auto-set to super_pel value"
    );

    // Test explicit pelsearch value
    let analyse = Analyse::new(
        node,
        None,
        None,
        None,
        None,
        None,
        Some(4),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        analyse.pel_search, 4,
        "Pel_search should use explicit value"
    );
}

#[test]
fn analyse_new_insufficient_super_levels() {
    // Create a super clip with only 2 levels but request analysis that needs more
    let env = create_test_env(672, 2750, PresetFormat::YUV420P8, 10, 480, 16, 2, 7, 2, 16).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    // Request 5 levels when super only has 2
    let result = Analyse::new(
        node,
        None,
        None,
        Some(5),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert!(
        result.is_err(),
        "Should reject when requested levels exceed super levels"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("super clip has 2 levels. Analyse needs")
    );
}

#[test]
fn analyse_new_colour_data_mismatch() {
    // Create a super clip that only has Y plane data but request chroma analysis
    let env = create_test_env(672, 2750, PresetFormat::YUV420P8, 10, 480, 16, 8, 1, 2, 16).unwrap();
    let (node, _) = env.get_output(0).unwrap();

    let result = Analyse::new(
        node,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(1),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert!(
        result.is_err(),
        "Should reject when super clip lacks needed colour data"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("super clip does not contain needed colour data")
    );
}
