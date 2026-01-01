#[cfg(feature = "e2e")]
#[macro_use]
mod common;

#[cfg(feature = "e2e")]
use anyhow::{Context, Result};
#[cfg(feature = "e2e")]
use common::*;
#[cfg(feature = "e2e")]
use vapoursynth::prelude::Environment;

#[test]
#[cfg(feature = "e2e")]
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

        // Compare vectors with tolerance
        compare_motion_vectors(c_vectors, r_vectors, 100)?;

        // Compare MVAnalysisData
        let c_analysis = c_props.get_data("MVTools_MVAnalysisData")?;
        let r_analysis = r_props.get_data("MVTools_MVAnalysisData")?;

        assert_eq!(
            c_analysis.len(),
            r_analysis.len(),
            "MVAnalysisData size mismatch at frame {}",
            n
        );
        assert_eq!(
            c_analysis, r_analysis,
            "MVAnalysisData content mismatch at frame {}",
            n
        );
    }

    Ok(())
}

#[test]
#[cfg(feature = "e2e")]
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

        // Higher tolerance for different search types
        compare_motion_vectors(c_vectors, r_vectors, 200)
            .with_context(|| format!("Search type {} ({})", search, name))?;
    }

    Ok(())
}

#[test]
#[cfg(feature = "e2e")]
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

        compare_motion_vectors(c_vectors, r_vectors, 100)?;
    }

    Ok(())
}

#[test]
#[cfg(feature = "e2e")]
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

        compare_motion_vectors(c_vectors, r_vectors, 100)
            .with_context(|| format!("Block size {}", blksize))?;

        // Verify analysis data matches
        let c_analysis = c_props.get_data("MVTools_MVAnalysisData")?;
        let r_analysis = r_props.get_data("MVTools_MVAnalysisData")?;

        assert_eq!(
            c_analysis, r_analysis,
            "MVAnalysisData mismatch for block size {}",
            blksize
        );
    }

    Ok(())
}

#[test]
#[cfg(feature = "e2e")]
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

        compare_motion_vectors(c_vectors, r_vectors, 100)?;

        // Verify analysis data
        let c_analysis = c_props.get_data("MVTools_MVAnalysisData")?;
        let r_analysis = r_props.get_data("MVTools_MVAnalysisData")?;

        assert_eq!(
            c_analysis, r_analysis,
            "MVAnalysisData mismatch at frame {}",
            n
        );
    }

    Ok(())
}

#[test]
#[cfg(feature = "e2e")]
#[ignore] // Performance test - run explicitly
fn test_analyse_performance() -> Result<()> {
    require_mvtools!();

    let clip_config = TestClipConfig {
        width: 1920,
        height: 1080,
        format: "vs.YUV420P8",
        length: 100,
        content_type: ClipContentType::Noise { seed: 42 },
    };

    let super_params = FilterParams::default();
    let analyse_params = FilterParams::default();

    let script = generate_comparison_script(&clip_config, &super_params, Some(&analyse_params));

    let env = Environment::from_script(&script)?;
    let (c_node, _) = env.get_output(0)?;
    let (r_node, _) = env.get_output(1)?;

    let c_perf = measure_filter_performance(&c_node, 30, "C MVTools", "Analyse")?;
    let r_perf = measure_filter_performance(&r_node, 30, "Rust zoomv", "Analyse")?;

    println!("{}", compare_performance(&c_perf, &r_perf));

    // Just informational, don't fail on performance
    Ok(())
}
