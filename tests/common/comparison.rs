use anyhow::{Context, Result, bail};
use std::mem::size_of;
use vapoursynth::frame::FrameRef;

#[derive(Debug, Clone)]
pub struct ComparisonConfig {
    pub pixel_tolerance: f64,      // Absolute difference tolerance per pixel
    pub mean_tolerance: f64,       // Mean difference tolerance
    pub max_different_pixels: f64, // Percentage of pixels allowed to differ
}

impl Default for ComparisonConfig {
    fn default() -> Self {
        Self {
            pixel_tolerance: 0.0, // Exact match by default
            mean_tolerance: 0.0,
            max_different_pixels: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct PixelDifference {
    pub plane: usize,
    pub different_pixels: usize,
    pub total_pixels: usize,
    pub max_diff: i32,
    pub mean_diff: f64,
    pub differences: Vec<PixelDiff>, // Sample of differences for debugging
}

#[derive(Debug)]
pub struct PixelDiff {
    pub x: usize,
    pub y: usize,
    pub expected: u16,
    pub actual: u16,
    pub diff: i32,
}

/// Compare frames pixel by pixel
/// Generic over T which can be u8 or u16
pub fn compare_frames<T>(
    c_frame: &FrameRef,
    r_frame: &FrameRef,
    config: &ComparisonConfig,
) -> Result<Vec<PixelDifference>>
where
    T: Copy + Into<u16> + vapoursynth::component::Component,
{
    let format = c_frame.format();
    if format != r_frame.format() {
        bail!("Frame formats don't match");
    }

    if c_frame.width(0) != r_frame.width(0) || c_frame.height(0) != r_frame.height(0) {
        bail!(
            "Frame dimensions don't match: C={}x{}, Rust={}x{}",
            c_frame.width(0),
            c_frame.height(0),
            r_frame.width(0),
            r_frame.height(0)
        );
    }

    let mut results = Vec::new();

    for plane in 0..format.plane_count() {
        let width = c_frame.width(plane);
        let height = c_frame.height(plane);
        let c_stride = c_frame.stride(plane);
        let r_stride = r_frame.stride(plane);

        let bytes_per_sample = size_of::<T>();

        let mut different_pixels = 0;
        let mut max_diff = 0i32;
        let mut sum_diff = 0i64;
        let mut differences = Vec::new();
        let total_pixels = width * height;

        // Access data row by row to handle padding
        for y in 0..height {
            let c_row = c_frame.plane_row::<T>(plane, y);
            let r_row = r_frame.plane_row::<T>(plane, y);

            for x in 0..width {
                let c_val: u16 = c_row[x].into();
                let r_val: u16 = r_row[x].into();

                let diff = (c_val as i32 - r_val as i32).abs();

                if diff as f64 > config.pixel_tolerance {
                    different_pixels += 1;
                    if differences.len() < 10 {
                        // Keep first 10 for debugging
                        differences.push(PixelDiff {
                            x,
                            y,
                            expected: c_val,
                            actual: r_val,
                            diff,
                        });
                    }
                }

                max_diff = max_diff.max(diff);
                sum_diff += diff as i64;
            }
        }

        let mean_diff = sum_diff as f64 / total_pixels as f64;

        results.push(PixelDifference {
            plane,
            different_pixels,
            total_pixels,
            max_diff,
            mean_diff,
            differences,
        });
    }

    Ok(results)
}

/// Assert frames match within tolerance, fail test otherwise
pub fn assert_frames_match<T>(
    c_frame: &FrameRef,
    r_frame: &FrameRef,
    config: &ComparisonConfig,
    context: &str,
) -> Result<()>
where
    T: Copy + Into<u16> + vapoursynth::component::Component,
{
    let diffs = compare_frames::<T>(c_frame, r_frame, config)?;

    for diff in &diffs {
        let pct = (diff.different_pixels as f64 / diff.total_pixels as f64) * 100.0;

        if pct > config.max_different_pixels || diff.mean_diff > config.mean_tolerance {
            bail!(
                "{}\nPlane {}: {}/{} pixels differ ({:.2}%), max_diff={}, mean_diff={:.4}\n\
                 First differences: {:#?}",
                context,
                diff.plane,
                diff.different_pixels,
                diff.total_pixels,
                pct,
                diff.max_diff,
                diff.mean_diff,
                diff.differences
            );
        }
    }

    Ok(())
}

/// Compare frame properties (int and binary data)
pub fn compare_frame_properties(
    c_frame: &FrameRef,
    r_frame: &FrameRef,
    ignore_keys: &[&str],
) -> Result<()> {
    let c_props = c_frame.props();
    let r_props = r_frame.props();

    // Get all keys from both
    let c_keys: Vec<_> = c_props.keys().collect();
    let r_keys: Vec<_> = r_props.keys().collect();

    for key in &c_keys {
        if ignore_keys.contains(&key.as_ref()) {
            continue;
        }

        if !r_keys.contains(key) {
            bail!(
                "Property '{}' present in C frame but not in Rust frame",
                key
            );
        }

        // Try to compare as int first
        if let Ok(c_val) = c_props.get_int(key) {
            let r_val = r_props
                .get_int(key)
                .with_context(|| format!("Property '{}' type mismatch", key))?;
            if c_val != r_val {
                bail!("Property '{}': C={}, Rust={}", key, c_val, r_val);
            }
        } else if let Ok(c_data) = c_props.get_data(key) {
            // Try as binary data
            let r_data = r_props.get_data(key)?;
            if c_data != r_data {
                bail!(
                    "Property '{}' data mismatch (lengths: C={}, Rust={})",
                    key,
                    c_data.len(),
                    r_data.len()
                );
            }
        }
    }

    // Check for properties in Rust that aren't in C
    for key in &r_keys {
        if ignore_keys.contains(&key.as_ref()) {
            continue;
        }
        if !c_keys.contains(key) {
            bail!(
                "Property '{}' present in Rust frame but not in C frame",
                key
            );
        }
    }

    Ok(())
}

/// Compare motion vector data
pub fn compare_motion_vectors(
    c_vectors: &[u8],
    r_vectors: &[u8],
    sad_tolerance: i64,
) -> Result<()> {
    // MotionVector is repr(C) with x, y (isize), sad (i64)
    let isize_size = size_of::<isize>();
    let i64_size = size_of::<i64>();
    let mv_size = isize_size * 2 + i64_size;

    if c_vectors.len() != r_vectors.len() {
        bail!(
            "Vector data length mismatch: C={}, Rust={}",
            c_vectors.len(),
            r_vectors.len()
        );
    }

    if c_vectors.len() % mv_size != 0 {
        bail!(
            "Vector data length {} is not a multiple of motion vector size {}",
            c_vectors.len(),
            mv_size
        );
    }

    let num_vectors = c_vectors.len() / mv_size;
    let mut mismatches = Vec::new();

    for i in 0..num_vectors {
        let offset = i * mv_size;
        let c_mv = &c_vectors[offset..offset + mv_size];
        let r_mv = &r_vectors[offset..offset + mv_size];

        // Parse as MotionVector
        let c_x = isize::from_ne_bytes(c_mv[0..isize_size].try_into().unwrap());
        let c_y = isize::from_ne_bytes(c_mv[isize_size..isize_size * 2].try_into().unwrap());
        let c_sad = i64::from_ne_bytes(c_mv[isize_size * 2..].try_into().unwrap());

        let r_x = isize::from_ne_bytes(r_mv[0..isize_size].try_into().unwrap());
        let r_y = isize::from_ne_bytes(r_mv[isize_size..isize_size * 2].try_into().unwrap());
        let r_sad = i64::from_ne_bytes(r_mv[isize_size * 2..].try_into().unwrap());

        if c_x != r_x || c_y != r_y || (c_sad - r_sad).abs() > sad_tolerance {
            mismatches.push(format!(
                "MV[{}]: C=({},{},{}), Rust=({},{},{})",
                i, c_x, c_y, c_sad, r_x, r_y, r_sad
            ));

            if mismatches.len() >= 10 {
                break; // Limit output
            }
        }
    }

    if !mismatches.is_empty() {
        bail!("Motion vector mismatches:\n{}", mismatches.join("\n"));
    }

    Ok(())
}
