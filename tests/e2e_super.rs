#![cfg(feature = "e2e")]

#[macro_use]
mod common;

use anyhow::Result;
use parameterized::parameterized;
use vapoursynth::prelude::Environment;

use crate::common::{
    comparison::{ComparisonConfig, assert_frames_match, compare_frame_properties},
    script_gen::{ClipContentType, FilterParams, TestClipConfig, generate_comparison_script},
};

#[parameterized(
    format = {
        "vs.YUV420P8", "vs.YUV420P10", "vs.YUV420P16"
    }
)]
fn test_super_default_params(format: &str) -> Result<()> {
    require_mvtools!();

    let clip_config = TestClipConfig {
        width: 640,
        height: 480,
        format,
        length: 10,
        content_type: ClipContentType::Gradient,
    };

    let super_params = FilterParams::default();

    let script = generate_comparison_script(&clip_config, &super_params, None);

    let env = Environment::from_script(&script)?;
    let (c_node, _) = env.get_output(0)?; // C MVTools
    let (r_node, _) = env.get_output(1)?; // Rust zoomv

    // Compare all frames
    let config = ComparisonConfig::default();
    for n in 0..clip_config.length {
        let c_frame = c_node.get_frame(n)?;
        let r_frame = r_node.get_frame(n)?;

        if format.ends_with("P8") {
            assert_frames_match::<u8>(&c_frame, &r_frame, &config, &format!("Super frame {}", n))?;
        } else {
            assert_frames_match::<u16>(&c_frame, &r_frame, &config, &format!("Super frame {}", n))?;
        }

        // Compare Super properties on first frame
        if n == 0 {
            compare_frame_properties(&c_frame, &r_frame, &[])?;
        }
    }

    Ok(())
}

#[parameterized(
    format = {
        "vs.YUV420P8", "vs.YUV420P10", "vs.YUV420P16"
    }
)]
fn test_super_custom_params(format: &str) -> Result<()> {
    require_mvtools!();

    let clip_config = TestClipConfig {
        width: 320,
        height: 240,
        format,
        length: 5,
        content_type: ClipContentType::Checkerboard,
    };

    let super_params = FilterParams {
        hpad: Some(8),
        vpad: Some(8),
        pel: Some(2),
        levels: Some(3),
        chroma: Some(1),
        ..Default::default()
    };

    let script = generate_comparison_script(&clip_config, &super_params, None);

    let env = Environment::from_script(&script)?;
    let (c_node, _) = env.get_output(0)?;
    let (r_node, _) = env.get_output(1)?;

    let config = ComparisonConfig::default();
    for n in 0..clip_config.length {
        let c_frame = c_node.get_frame(n)?;
        let r_frame = r_node.get_frame(n)?;

        if format.ends_with("P8") {
            assert_frames_match::<u8>(&c_frame, &r_frame, &config, &format!("Super frame {}", n))?;
        } else {
            assert_frames_match::<u16>(&c_frame, &r_frame, &config, &format!("Super frame {}", n))?;
        }

        // Compare properties on first frame
        if n == 0 {
            compare_frame_properties(&c_frame, &r_frame, &[])?;
        }
    }

    Ok(())
}

#[test]
fn test_super_different_formats() -> Result<()> {
    require_mvtools!();

    // Note: GRAY formats excluded for now due to plane access issues
    let formats = [
        ("vs.YUV420P8", false),
        ("vs.YUV420P16", true),
        ("vs.YUV422P8", false),
        ("vs.YUV422P16", true),
        ("vs.YUV444P8", false),
        ("vs.YUV444P16", true),
    ];

    for (format, is_16bit) in &formats {
        let clip_config = TestClipConfig {
            width: 64,
            height: 48,
            format,
            length: 3,
            content_type: ClipContentType::Blank,
        };

        let super_params = FilterParams::default();
        let script = generate_comparison_script(&clip_config, &super_params, None);

        let env = Environment::from_script(&script)?;
        let (c_node, _) = env.get_output(0)?;
        let (r_node, _) = env.get_output(1)?;

        let c_frame = c_node.get_frame(0)?;
        let r_frame = r_node.get_frame(0)?;

        let config = ComparisonConfig::default();

        if *is_16bit {
            assert_frames_match::<u16>(
                &c_frame,
                &r_frame,
                &config,
                &format!("Super format {}", format),
            )?;
        } else {
            assert_frames_match::<u8>(
                &c_frame,
                &r_frame,
                &config,
                &format!("Super format {}", format),
            )?;
        }

        // Verify properties match
        compare_frame_properties(&c_frame, &r_frame, &[])?;
    }

    Ok(())
}

#[parameterized(
    format = {
        "vs.YUV420P8", "vs.YUV420P10", "vs.YUV420P16"
    }
)]
fn test_super_with_pelclip(format: &str) -> Result<()> {
    require_mvtools!();

    let clip_config = TestClipConfig {
        width: 320,
        height: 240,
        format,
        length: 5,
        content_type: ClipContentType::Blank,
    };

    // Create a script that provides a pelclip
    let script = format!(
        r#"
import vapoursynth as vs
core = vs.core

# Generate base clip
clip = core.std.BlankClip(width={}, height={}, format={}, length={})
# Create a double-resolution pelclip
pelclip = core.std.BlankClip(width={}, height={}, format={}, length={})

# Apply C MVTools with pelclip
c_super = core.mv.Super(clip, pel=2, pelclip=pelclip)

# Apply Rust ZoomVTools with pelclip
r_super = core.zoomv.Super(clip, pel=2, pelclip=pelclip)

c_super.set_output(0)
r_super.set_output(1)
"#,
        clip_config.width,
        clip_config.height,
        clip_config.format,
        clip_config.length,
        clip_config.width * 2,
        clip_config.height * 2,
        clip_config.format,
        clip_config.length,
    );

    let env = Environment::from_script(&script)?;
    let (c_node, _) = env.get_output(0)?;
    let (r_node, _) = env.get_output(1)?;

    let config = ComparisonConfig::default();
    for n in 0..clip_config.length {
        let c_frame = c_node.get_frame(n)?;
        let r_frame = r_node.get_frame(n)?;

        if format.ends_with("P8") {
            assert_frames_match::<u8>(&c_frame, &r_frame, &config, &format!("Super frame {}", n))?;
        } else {
            assert_frames_match::<u16>(&c_frame, &r_frame, &config, &format!("Super frame {}", n))?;
        }
    }

    Ok(())
}
