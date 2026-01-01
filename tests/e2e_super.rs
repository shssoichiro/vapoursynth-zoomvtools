#[cfg(feature = "e2e")]
#[macro_use]
mod common;

#[cfg(feature = "e2e")]
use anyhow::Result;
#[cfg(feature = "e2e")]
use common::*;
#[cfg(feature = "e2e")]
use vapoursynth::prelude::Environment;

#[test]
#[cfg(feature = "e2e")]
fn test_super_8bit_yuv420_default_params() -> Result<()> {
    require_mvtools!();

    let clip_config = TestClipConfig {
        width: 640,
        height: 480,
        format: "vs.YUV420P8",
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

        assert_frames_match::<u8>(&c_frame, &r_frame, &config, &format!("Super frame {}", n))?;

        // Compare Super properties on first frame
        if n == 0 {
            compare_frame_properties(&c_frame, &r_frame, &[])?;
        }
    }

    Ok(())
}

#[test]
#[cfg(feature = "e2e")]
fn test_super_16bit_yuv420_custom_params() -> Result<()> {
    require_mvtools!();

    let clip_config = TestClipConfig {
        width: 320,
        height: 240,
        format: "vs.YUV420P16",
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

        assert_frames_match::<u16>(
            &c_frame,
            &r_frame,
            &config,
            &format!("Super 16-bit frame {}", n),
        )?;

        // Compare properties on first frame
        if n == 0 {
            compare_frame_properties(&c_frame, &r_frame, &[])?;
        }
    }

    Ok(())
}

#[test]
#[cfg(feature = "e2e")]
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

#[test]
#[cfg(feature = "e2e")]
#[ignore] // Performance test - run explicitly
fn test_super_performance() -> Result<()> {
    require_mvtools!();

    let clip_config = TestClipConfig {
        width: 1920,
        height: 1080,
        format: "vs.YUV420P8",
        length: 100,
        content_type: ClipContentType::Noise { seed: 42 },
    };

    let super_params = FilterParams::default();
    let script = generate_comparison_script(&clip_config, &super_params, None);

    let env = Environment::from_script(&script)?;
    let (c_node, _) = env.get_output(0)?;
    let (r_node, _) = env.get_output(1)?;

    let c_perf = measure_filter_performance(&c_node, 50, "C MVTools", "Super")?;
    let r_perf = measure_filter_performance(&r_node, 50, "Rust zoomv", "Super")?;

    println!("{}", compare_performance(&c_perf, &r_perf));

    // Just informational, don't fail on performance
    Ok(())
}

#[test]
#[cfg(feature = "e2e")]
fn test_super_with_pelclip() -> Result<()> {
    require_mvtools!();

    let clip_config = TestClipConfig {
        width: 320,
        height: 240,
        format: "vs.YUV420P8",
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

        assert_frames_match::<u8>(
            &c_frame,
            &r_frame,
            &config,
            &format!("Super with pelclip frame {}", n),
        )?;
    }

    Ok(())
}
