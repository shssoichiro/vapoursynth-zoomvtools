#![cfg(feature = "e2e")]

#[macro_use]
mod common;

use anyhow::{Context, Result};
use vapoursynth::prelude::Environment;

use crate::common::script_gen::{
    ClipContentType,
    FilterParams,
    TestClipConfig,
    generate_comparison_script,
};

#[test]
fn test_analyse_default_params() -> Result<()> {
    require_mvtools!();

    let clip_config = TestClipConfig {
        width: 320,
        height: 240,
        format: "vs.YUV420P8",
        length: 20,
        content_type: ClipContentType::MovingBox {
            speed_x: 2,
            speed_y: 1,
        },
    };

    let super_params = FilterParams::default();
    let analyse_params = FilterParams::default();

    let script = generate_comparison_script(&clip_config, &super_params, Some(&analyse_params));

    let env = Environment::from_script(&script)?;
    let (c_node, _) = env.get_output(0)?;
    let (r_node, _) = env.get_output(1)?;

    for n in 0..clip_config.length {
        let c_frame = c_node.get_frame(n)?;
        let r_frame = r_node.get_frame(n)?;

        // Get motion vector data
        let c_props = c_frame.props();
        let r_props = r_frame.props();

        let c_vectors = c_props.get_data("MVTools_vectors")?;
        let r_vectors = r_props.get_data("MVTools_vectors")?;
        compare_vectors_data(c_vectors, r_vectors, n);

        // Compare MVAnalysisData
        let c_analysis = c_props.get_data("MVTools_MVAnalysisData")?;
        let r_analysis = r_props.get_data("MVTools_MVAnalysisData")?;
        compare_analysis_data(c_analysis, r_analysis, n);
    }

    Ok(())
}

#[test]
fn test_analyse_search_types() -> Result<()> {
    require_mvtools!();

    let search_types = [
        (0, "Onetime"),
        (1, "Nstep"),
        (3, "Exhaustive"),
        (4, "Hex2"),
        (5, "UMH"),
        (6, "Horizontal"),
        (7, "Vertical"),
    ];

    for (search, name) in &search_types {
        let clip_config = TestClipConfig {
            width: 128,
            height: 96,
            format: "vs.YUV420P8",
            length: 10,
            content_type: ClipContentType::MovingBox {
                speed_x: 1,
                speed_y: 0,
            },
        };

        let super_params = FilterParams::default();
        let analyse_params = FilterParams {
            search: Some(*search),
            ..Default::default()
        };

        let script = generate_comparison_script(&clip_config, &super_params, Some(&analyse_params));

        let env = Environment::from_script(&script)
            .with_context(|| format!("Failed with search type {}", name))?;
        let (c_node, _) = env.get_output(0)?;
        let (r_node, _) = env.get_output(1)?;

        let c_frame = c_node.get_frame(5)?;
        let r_frame = r_node.get_frame(5)?;

        let c_props = c_frame.props();
        let r_props = r_frame.props();
        let c_vectors = c_props.get_data("MVTools_vectors")?;
        let r_vectors = r_props.get_data("MVTools_vectors")?;
        compare_vectors_data(c_vectors, r_vectors, *search as usize);
    }

    Ok(())
}

#[test]
fn test_analyse_backward_motion() -> Result<()> {
    require_mvtools!();

    let clip_config = TestClipConfig {
        width: 256,
        height: 192,
        format: "vs.YUV420P8",
        length: 15,
        content_type: ClipContentType::MovingBox {
            speed_x: 3,
            speed_y: 2,
        },
    };

    let super_params = FilterParams::default();
    let analyse_params = FilterParams {
        isb: Some(1), // backward
        ..Default::default()
    };

    let script = generate_comparison_script(&clip_config, &super_params, Some(&analyse_params));

    let env = Environment::from_script(&script)?;
    let (c_node, _) = env.get_output(0)?;
    let (r_node, _) = env.get_output(1)?;

    for n in 5..10 {
        let c_frame = c_node.get_frame(n)?;
        let r_frame = r_node.get_frame(n)?;

        let c_props = c_frame.props();
        let r_props = r_frame.props();
        let c_vectors = c_props.get_data("MVTools_vectors")?;
        let r_vectors = r_props.get_data("MVTools_vectors")?;

        compare_vectors_data(c_vectors, r_vectors, n);
    }

    Ok(())
}

#[test]
fn test_analyse_different_block_sizes() -> Result<()> {
    require_mvtools!();

    let block_sizes = [4, 8, 16, 32];

    for blksize in &block_sizes {
        let clip_config = TestClipConfig {
            width: 256,
            height: 192,
            format: "vs.YUV420P8",
            length: 10,
            content_type: ClipContentType::MovingBox {
                speed_x: 1,
                speed_y: 1,
            },
        };

        let super_params = FilterParams::default();
        let analyse_params = FilterParams {
            blksize: Some(*blksize),
            ..Default::default()
        };

        let script = generate_comparison_script(&clip_config, &super_params, Some(&analyse_params));

        let env = Environment::from_script(&script)
            .with_context(|| format!("Failed with block size {}", blksize))?;
        let (c_node, _) = env.get_output(0)?;
        let (r_node, _) = env.get_output(1)?;

        let c_frame = c_node.get_frame(5)?;
        let r_frame = r_node.get_frame(5)?;

        let c_props = c_frame.props();
        let r_props = r_frame.props();
        let c_vectors = c_props.get_data("MVTools_vectors")?;
        let r_vectors = r_props.get_data("MVTools_vectors")?;
        compare_vectors_data(c_vectors, r_vectors, *blksize as usize);

        // Verify analysis data matches
        let c_analysis = c_props.get_data("MVTools_MVAnalysisData")?;
        let r_analysis = r_props.get_data("MVTools_MVAnalysisData")?;

        compare_analysis_data(c_analysis, r_analysis, *blksize as usize);
    }

    Ok(())
}

#[test]
fn test_analyse_dct_modes() -> Result<()> {
    require_mvtools!();

    let dct_modes = [
        (0, "Spatial"),
        (1, "Dct"),
        (2, "MixedSpatialDct"),
        (3, "AdaptiveSpatialMixed"),
        (4, "AdaptiveSpatialDct"),
        (5, "Satd"),
        (6, "MixedSatdDct"),
        (7, "AdaptiveSatdMixed"),
        (8, "AdaptiveSatdDct"),
        (9, "MixedSadEqSatdDct"),
        (10, "AdaptiveSatdLuma"),
    ];

    for (dct, name) in &dct_modes {
        let clip_config = TestClipConfig {
            width: 128,
            height: 96,
            format: "vs.YUV420P8",
            length: 10,
            content_type: ClipContentType::MovingBox {
                speed_x: 1,
                speed_y: 0,
            },
        };

        let super_params = FilterParams::default();
        let analyse_params = FilterParams {
            dct: Some(*dct),
            ..Default::default()
        };

        let script = generate_comparison_script(&clip_config, &super_params, Some(&analyse_params));

        let env = Environment::from_script(&script)
            .with_context(|| format!("Failed with search type {}", name))?;
        let (c_node, _) = env.get_output(0)?;
        let (r_node, _) = env.get_output(1)?;

        let c_frame = c_node.get_frame(5)?;
        let r_frame = r_node.get_frame(5)?;

        let c_props = c_frame.props();
        let r_props = r_frame.props();
        let c_vectors = c_props.get_data("MVTools_vectors")?;
        let r_vectors = r_props.get_data("MVTools_vectors")?;
        compare_vectors_data(c_vectors, r_vectors, *dct as usize);
    }

    Ok(())
}

#[test]
fn test_analyse_divide_extra_original() -> Result<()> {
    require_mvtools!();

    let clip_config = TestClipConfig {
        width: 320,
        height: 240,
        format: "vs.YUV420P8",
        length: 20,
        content_type: ClipContentType::MovingBox {
            speed_x: 2,
            speed_y: 1,
        },
    };

    let super_params = FilterParams::default();
    let analyse_params = FilterParams {
        divide: Some(1),
        ..FilterParams::default()
    };

    let script = generate_comparison_script(&clip_config, &super_params, Some(&analyse_params));

    let env = Environment::from_script(&script)?;
    let (c_node, _) = env.get_output(0)?;
    let (r_node, _) = env.get_output(1)?;

    for n in 0..clip_config.length {
        let c_frame = c_node.get_frame(n)?;
        let r_frame = r_node.get_frame(n)?;

        // Get motion vector data
        let c_props = c_frame.props();
        let r_props = r_frame.props();

        let c_vectors = c_props.get_data("MVTools_vectors")?;
        let r_vectors = r_props.get_data("MVTools_vectors")?;
        compare_vectors_data(c_vectors, r_vectors, n);

        // Compare MVAnalysisData
        let c_analysis = c_props.get_data("MVTools_MVAnalysisData")?;
        let r_analysis = r_props.get_data("MVTools_MVAnalysisData")?;
        compare_analysis_data(c_analysis, r_analysis, n);
    }

    Ok(())
}

#[test]
fn test_analyse_divide_extra_median() -> Result<()> {
    require_mvtools!();

    let clip_config = TestClipConfig {
        width: 320,
        height: 240,
        format: "vs.YUV420P8",
        length: 20,
        content_type: ClipContentType::MovingBox {
            speed_x: 2,
            speed_y: 1,
        },
    };

    let super_params = FilterParams::default();
    let analyse_params = FilterParams {
        divide: Some(2),
        ..FilterParams::default()
    };

    let script = generate_comparison_script(&clip_config, &super_params, Some(&analyse_params));

    let env = Environment::from_script(&script)?;
    let (c_node, _) = env.get_output(0)?;
    let (r_node, _) = env.get_output(1)?;

    for n in 0..clip_config.length {
        let c_frame = c_node.get_frame(n)?;
        let r_frame = r_node.get_frame(n)?;

        // Get motion vector data
        let c_props = c_frame.props();
        let r_props = r_frame.props();

        let c_vectors = c_props.get_data("MVTools_vectors")?;
        let r_vectors = r_props.get_data("MVTools_vectors")?;
        compare_vectors_data(c_vectors, r_vectors, n);

        // Compare MVAnalysisData
        let c_analysis = c_props.get_data("MVTools_MVAnalysisData")?;
        let r_analysis = r_props.get_data("MVTools_MVAnalysisData")?;
        compare_analysis_data(c_analysis, r_analysis, n);
    }

    Ok(())
}

#[test]
fn test_analyse_16bit() -> Result<()> {
    require_mvtools!();

    let clip_config = TestClipConfig {
        width: 320,
        height: 240,
        format: "vs.YUV420P16",
        length: 10,
        content_type: ClipContentType::MovingBox {
            speed_x: 2,
            speed_y: 1,
        },
    };

    let super_params = FilterParams::default();
    let analyse_params = FilterParams::default();

    let script = generate_comparison_script(&clip_config, &super_params, Some(&analyse_params));

    let env = Environment::from_script(&script)?;
    let (c_node, _) = env.get_output(0)?;
    let (r_node, _) = env.get_output(1)?;

    for n in 0..clip_config.length {
        let c_frame = c_node.get_frame(n)?;
        let r_frame = r_node.get_frame(n)?;

        let c_props = c_frame.props();
        let r_props = r_frame.props();
        let c_vectors = c_props.get_data("MVTools_vectors")?;
        let r_vectors = r_props.get_data("MVTools_vectors")?;
        compare_vectors_data(c_vectors, r_vectors, n);

        // Verify analysis data
        let c_analysis = c_props.get_data("MVTools_MVAnalysisData")?;
        let r_analysis = r_props.get_data("MVTools_MVAnalysisData")?;

        compare_analysis_data(c_analysis, r_analysis, n);
    }

    Ok(())
}

fn compare_analysis_data(c_analysis: &[u8], r_analysis: &[u8], test_no: usize) {
    // Expected size difference of 12, to account for removed fields
    assert_eq!(
        c_analysis.len(),
        r_analysis.len() + 12,
        "MVAnalysisData size mismatch, test {test_no}",
    );
    for ((_, c_field), (i, r_field)) in c_analysis
        .chunks_exact(4)
        .enumerate()
        .filter(|(i, _)| {
            // removed fields: magic number (0), version (1), cpu flags (8)
            ![0, 1, 8].contains(i)
        })
        .zip(r_analysis.chunks_exact(4).enumerate())
    {
        let mut c_field = [c_field[0], c_field[1], c_field[2], c_field[3]];
        if i == 6 {
            // ignore cpu flags field on motion flags field
            c_field[0] &= 0b11111110;
        };
        assert_eq!(
            c_field, r_field,
            "MVAnalysisData content mismatch on field {i}, test {test_no}",
        );
    }
}

fn compare_vectors_data(c_vectors: &[u8], r_vectors: &[u8], test_no: usize) {
    assert_eq!(
        c_vectors.len(),
        r_vectors.len(),
        "MVTools_vectors size mismatch, test {test_no}",
    );

    // Compare the size and validity headers
    let (c_header, c_vectors) = c_vectors.split_at(8);
    let (r_header, r_vectors) = r_vectors.split_at(8);
    assert_eq!(
        c_header, r_header,
        "MVTools_vectors headers mismatch, test {test_no}"
    );

    // Parse each motion vector and compare, this makes failures easier to read
    c_vectors
        .chunks_exact(16)
        .zip(r_vectors.chunks_exact(16))
        .enumerate()
        .for_each(|(i, (c_mv, r_mv))| {
            let (c_size, c_mv) = c_mv.split_at(4);
            let (c_x, c_mv) = c_mv.split_at(4);
            let (c_y, c_sad) = c_mv.split_at(4);

            let (r_size, r_mv) = r_mv.split_at(4);
            let (r_x, r_mv) = r_mv.split_at(4);
            let (r_y, r_sad) = r_mv.split_at(4);

            assert_eq!(
                c_size, r_size,
                "Size header mismatch on MV {i}, test {test_no}"
            );
            assert_eq!(c_x, r_x, "X value mismatch on MV {i}, test {test_no}");
            assert_eq!(c_y, r_y, "Y value mismatch on MV {i}, test {test_no}");
            assert_eq!(c_sad, r_sad, "SAD value mismatch on MV {i}, test {test_no}");
        });
}
