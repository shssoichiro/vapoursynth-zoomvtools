use super::*;
use crate::params::{ReduceFilter, Subpel, SubpelMethod};
use std::num::{NonZeroU8, NonZeroUsize};

// Helper function to create a test MVPlane
fn create_test_mvplane(
    width: usize,
    height: usize,
    pel: Subpel,
    hpad: usize,
    vpad: usize,
    bits_per_sample: u8,
    plane_offset: usize,
    pitch: usize,
) -> MVPlane {
    MVPlane::new(
        NonZeroUsize::new(width).unwrap(),
        NonZeroUsize::new(height).unwrap(),
        pel,
        hpad,
        vpad,
        NonZeroU8::new(bits_per_sample).unwrap(),
        plane_offset,
        NonZeroUsize::new(pitch).unwrap(),
    )
    .unwrap()
}

#[test]
fn test_mvplane_new_basic() {
    let plane = create_test_mvplane(64, 48, Subpel::Full, 8, 8, 8, 0, 80);

    assert_eq!(plane.width.get(), 64);
    assert_eq!(plane.height.get(), 48);
    assert_eq!(plane.padded_width.get(), 80); // 64 + 2*8
    assert_eq!(plane.padded_height.get(), 64); // 48 + 2*8
    assert_eq!(plane.hpad, 8);
    assert_eq!(plane.vpad, 8);
    assert_eq!(plane.pitch.get(), 80);
    assert_eq!(plane.pel, Subpel::Full);
    assert_eq!(plane.bits_per_sample.get(), 8);
    assert!(!plane.is_padded);
    assert!(!plane.is_refined);
    assert!(!plane.is_filled);

    // For Full subpel, should have 1 window offset
    assert_eq!(plane.subpel_window_offsets.len(), 1);
    assert_eq!(plane.subpel_window_offsets[0], 0);
}

#[test]
fn test_mvplane_new_half_pel() {
    let plane = create_test_mvplane(32, 24, Subpel::Half, 4, 4, 8, 100, 40);

    assert_eq!(plane.pel, Subpel::Half);
    assert_eq!(plane.hpad_pel, 8); // 4 * 2
    assert_eq!(plane.vpad_pel, 8); // 4 * 2

    // For Half subpel, should have 4 window offsets (2x2)
    assert_eq!(plane.subpel_window_offsets.len(), 4);
    assert_eq!(plane.subpel_window_offsets[0], 100); // plane_offset

    // Check offset calculations
    let padded_height = 32; // 24 + 2*4
    let expected_offset_1 = 100 + 40 * padded_height;
    assert_eq!(plane.subpel_window_offsets[1], expected_offset_1);
}

#[test]
fn test_mvplane_new_quarter_pel() {
    let plane = create_test_mvplane(16, 16, Subpel::Quarter, 2, 2, 16, 0, 24);

    assert_eq!(plane.pel, Subpel::Quarter);
    assert_eq!(plane.hpad_pel, 8); // 2 * 4
    assert_eq!(plane.vpad_pel, 8); // 2 * 4

    // For Quarter subpel, should have 16 window offsets (4x4)
    assert_eq!(plane.subpel_window_offsets.len(), 16);
}

#[test]
fn test_mvplane_new_with_plane_offset() {
    let plane_offset = 1000;
    let plane = create_test_mvplane(8, 8, Subpel::Half, 2, 2, 8, plane_offset, 12);

    // All offsets should include the plane_offset
    for offset in &plane.subpel_window_offsets {
        assert!(*offset >= plane_offset);
    }
    assert_eq!(plane.subpel_window_offsets[0], plane_offset);
}

#[test]
fn test_mvplane_fill_plane_basic() {
    let mut plane = create_test_mvplane(4, 4, Subpel::Full, 2, 2, 8, 0, 8);

    // Create source data
    let src_data = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let src_pitch = NonZeroUsize::new(4).unwrap();

    // Create destination buffer (larger to accommodate padding and plane structure)
    let total_size =
        plane.pitch.get() * (plane.height.get() + 2 * plane.vpad) + plane.offset_padding;
    let mut dest = vec![0u8; total_size];

    assert!(!plane.is_filled);
    plane.fill_plane(&src_data, src_pitch, &mut dest);
    assert!(plane.is_filled);

    // Verify data was copied correctly
    let offset = plane.subpel_window_offsets[0] + plane.offset_padding;
    for row in 0..4 {
        for col in 0..4 {
            let src_index = row * 4 + col;
            let dest_index = offset + row * plane.pitch.get() + col;
            assert_eq!(dest[dest_index], src_data[src_index]);
        }
    }
}

#[test]
fn test_mvplane_fill_plane_already_filled() {
    let mut plane = create_test_mvplane(2, 2, Subpel::Full, 1, 1, 8, 0, 4);

    let src_data = vec![1u8, 2, 3, 4];
    let src_pitch = NonZeroUsize::new(2).unwrap();
    let total_size =
        plane.pitch.get() * (plane.height.get() + 2 * plane.vpad) + plane.offset_padding;
    let mut dest = vec![0u8; total_size];

    // Fill once
    plane.fill_plane(&src_data, src_pitch, &mut dest);
    assert!(plane.is_filled);

    // Modify dest to verify it doesn't get overwritten
    let offset = plane.subpel_window_offsets[0] + plane.offset_padding;
    dest[offset] = 99;

    // Try to fill again - should be a no-op
    let src_data2 = vec![10u8, 20, 30, 40];
    plane.fill_plane(&src_data2, src_pitch, &mut dest);

    // Original modification should still be there
    assert_eq!(dest[offset], 99);
}

#[test]
fn test_mvplane_fill_plane_different_pitch() {
    let mut plane = create_test_mvplane(3, 2, Subpel::Full, 1, 1, 8, 0, 5);

    // Source has different pitch than plane
    let src_data = vec![1u8, 2, 3, 0, 4, 5, 6, 0]; // pitch=4, width=3
    let src_pitch = NonZeroUsize::new(4).unwrap();

    let total_size =
        plane.pitch.get() * (plane.height.get() + 2 * plane.vpad) + plane.offset_padding;
    let mut dest = vec![0u8; total_size];

    plane.fill_plane(&src_data, src_pitch, &mut dest);

    // Verify only the actual width was copied, not the padding
    let offset = plane.subpel_window_offsets[0] + plane.offset_padding;
    assert_eq!(dest[offset], 1);
    assert_eq!(dest[offset + 1], 2);
    assert_eq!(dest[offset + 2], 3);
    assert_eq!(dest[offset + plane.pitch.get()], 4);
    assert_eq!(dest[offset + plane.pitch.get() + 1], 5);
    assert_eq!(dest[offset + plane.pitch.get() + 2], 6);
}

#[test]
fn test_mvplane_refine_ext_full_pel() {
    let mut plane = create_test_mvplane(4, 4, Subpel::Full, 2, 2, 8, 0, 8);

    let src_2x = vec![0u8; 32]; // 8x4 for 2x upsampled
    let src_2x_pitch = NonZeroUsize::new(8).unwrap();
    let mut dest = vec![0u8; 128];

    assert!(!plane.is_refined);
    plane.refine_ext(&src_2x, src_2x_pitch, true, &mut dest);
    assert!(plane.is_refined);

    // For Full pel, no actual refinement should occur
}

#[test]
fn test_mvplane_refine_ext_half_pel() {
    let mut plane = create_test_mvplane(2, 2, Subpel::Half, 1, 1, 8, 0, 4);

    let src_2x = vec![0u8; 16]; // 4x4 for 2x upsampled
    let src_2x_pitch = NonZeroUsize::new(4).unwrap();
    let mut dest = vec![0u8; 64];

    assert!(!plane.is_refined);
    plane.refine_ext(&src_2x, src_2x_pitch, true, &mut dest);
    assert!(plane.is_refined);
}

#[test]
fn test_mvplane_refine_ext_quarter_pel() {
    let mut plane = create_test_mvplane(1, 1, Subpel::Quarter, 1, 1, 8, 0, 3);

    let src_2x = vec![0u8; 16]; // 4x4 for 4x upsampled
    let src_2x_pitch = NonZeroUsize::new(4).unwrap();
    // Need much larger buffer to accommodate all 16 subpel windows
    let buffer_size = plane.subpel_window_offsets.iter().max().unwrap_or(&0)
        + plane.pitch.get() * (plane.height.get() + 2 * plane.vpad) * 2;
    let mut dest = vec![0u8; buffer_size];

    assert!(!plane.is_refined);
    plane.refine_ext(&src_2x, src_2x_pitch, true, &mut dest);
    assert!(plane.is_refined);
}

#[test]
fn test_mvplane_refine_ext_already_refined() {
    let mut plane = create_test_mvplane(2, 2, Subpel::Half, 1, 1, 8, 0, 4);

    let src_2x = vec![0u8; 16];
    let src_2x_pitch = NonZeroUsize::new(4).unwrap();
    // Calculate proper buffer size
    let buffer_size = plane.subpel_window_offsets.iter().max().unwrap_or(&0)
        + plane.pitch.get() * (plane.height.get() + 2 * plane.vpad) * 2;
    let mut dest = vec![0u8; buffer_size];

    // Refine once
    plane.refine_ext(&src_2x, src_2x_pitch, true, &mut dest);
    assert!(plane.is_refined);

    // Try to refine again - should be a no-op
    plane.refine_ext(&src_2x, src_2x_pitch, true, &mut dest);
    assert!(plane.is_refined);
}

#[test]
fn test_mvplane_reduce_to_already_filled() {
    let plane = create_test_mvplane(8, 8, Subpel::Full, 2, 2, 8, 0, 12);
    let mut reduced_plane = create_test_mvplane(4, 4, Subpel::Full, 1, 1, 8, 0, 6);

    // Mark the source plane as filled
    reduced_plane.is_filled = true;

    let src = vec![0u8; 144]; // 12 * 12
    let mut dest = vec![0u8; 36]; // 6 * 6
    let dest_copy = dest.clone();

    plane.reduce_to(
        &mut reduced_plane,
        ReduceFilter::Average,
        &mut dest,
        &src,
        NonZeroUsize::new(6).unwrap(),
        NonZeroUsize::new(12).unwrap(),
        NonZeroUsize::new(4).unwrap(),
        NonZeroUsize::new(4).unwrap(),
    );

    // Should be a no-op since source is already filled
    assert!(reduced_plane.is_filled);
    assert!(dest == dest_copy);
}

#[test]
fn test_mvplane_reduce_to_basic() {
    let plane = create_test_mvplane(4, 4, Subpel::Full, 1, 1, 8, 0, 6);
    let mut reduced_plane = create_test_mvplane(2, 2, Subpel::Full, 1, 1, 8, 0, 4);

    // Create source data with known pattern
    let mut src = vec![0u8; 36]; // 6 * 6
    for i in 0..36 {
        src[i] = (i % 256) as u8;
    }
    let mut dest = vec![0u8; 16]; // 4 * 4

    assert!(!reduced_plane.is_filled);
    plane.reduce_to(
        &mut reduced_plane,
        ReduceFilter::Average,
        &mut dest,
        &src,
        NonZeroUsize::new(4).unwrap(),
        NonZeroUsize::new(6).unwrap(),
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(2).unwrap(),
    );

    assert!(reduced_plane.is_filled);
}

#[test]
fn test_mvplane_reduce_to_different_filters() {
    let plane = create_test_mvplane(4, 2, Subpel::Full, 0, 0, 8, 0, 4);

    let src = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
    let mut dest = vec![0u8; 4];

    for filter in [
        ReduceFilter::Average,
        ReduceFilter::Triangle,
        ReduceFilter::Bilinear,
        ReduceFilter::Quadratic,
        ReduceFilter::Cubic,
    ] {
        let mut reduced_plane = create_test_mvplane(2, 1, Subpel::Full, 0, 0, 8, 0, 2);

        plane.reduce_to(
            &mut reduced_plane,
            filter,
            &mut dest,
            &src,
            NonZeroUsize::new(2).unwrap(),
            NonZeroUsize::new(4).unwrap(),
            NonZeroUsize::new(2).unwrap(),
            NonZeroUsize::new(1).unwrap(),
        );

        assert!(reduced_plane.is_filled);
    }
}

#[test]
fn test_mvplane_pad_basic() {
    let mut plane = create_test_mvplane(2, 2, Subpel::Full, 1, 1, 8, 0, 4);

    // Create source data that includes the plane area and padding
    let total_size = plane.pitch.get() * (plane.height.get() + 2 * plane.vpad);
    let mut src = vec![0u8; total_size];

    // Fill the central area with known values
    let start_offset = plane.pitch.get() * plane.vpad + plane.hpad;
    src[start_offset] = 1;
    src[start_offset + 1] = 2;
    src[start_offset + plane.pitch.get()] = 3;
    src[start_offset + plane.pitch.get() + 1] = 4;

    assert!(!plane.is_padded);
    plane.pad(&mut src);
    assert!(plane.is_padded);

    // Verify padding was applied (edges should be replicated)
    // Top padding
    assert_eq!(src[plane.hpad], 1); // Top-left should match source top-left
    assert_eq!(src[plane.hpad + 1], 2); // Top-right should match source top-right

    // Left padding
    assert_eq!(src[start_offset - 1], 1); // Left of first pixel should match first pixel
}

#[test]
fn test_mvplane_pad_already_padded() {
    let mut plane = create_test_mvplane(2, 2, Subpel::Full, 1, 1, 8, 0, 4);

    let total_size = plane.pitch.get() * (plane.height.get() + 2 * plane.vpad);
    let mut src = vec![0u8; total_size];

    // Pad once
    plane.pad(&mut src);
    assert!(plane.is_padded);

    // Modify a padding area to verify it doesn't get overwritten
    src[0] = 99;

    // Try to pad again - should be a no-op
    plane.pad(&mut src);
    assert_eq!(src[0], 99);
}

#[test]
fn test_mvplane_refine_full_pel() {
    let mut plane = create_test_mvplane(4, 4, Subpel::Full, 2, 2, 8, 0, 8);

    let total_size = plane.pitch.get() * (plane.height.get() + 2 * plane.vpad);
    let mut plane_data = vec![0u8; total_size];

    assert!(!plane.is_refined);
    plane.refine(SubpelMethod::Bilinear, &mut plane_data);
    assert!(plane.is_refined);

    // For Full pel, no actual refinement should occur
}

#[test]
fn test_mvplane_refine_half_pel() {
    let mut plane = create_test_mvplane(4, 4, Subpel::Half, 2, 2, 8, 0, 8);

    // Need buffer large enough for all subpel windows - use much larger buffer
    let max_offset = plane.subpel_window_offsets.iter().max().unwrap_or(&0);
    let window_size = plane.pitch.get() * (plane.height.get() + 2 * plane.vpad);
    let total_size = max_offset + window_size + 1000; // Extra safety margin
    let mut plane_data = vec![1u8; total_size];

    assert!(!plane.is_refined);
    plane.refine(SubpelMethod::Bilinear, &mut plane_data);
    assert!(plane.is_refined);
}

#[test]
fn test_mvplane_refine_quarter_pel() {
    let mut plane = create_test_mvplane(4, 4, Subpel::Quarter, 2, 2, 8, 0, 8);

    // Need buffer large enough for all 16 subpel windows
    let max_offset = plane.subpel_window_offsets.iter().max().unwrap_or(&0);
    let window_size = plane.pitch.get() * (plane.height.get() + 2 * plane.vpad);
    let total_size = max_offset + window_size + 1000; // Extra safety margin
    let mut plane_data = vec![1u8; total_size];

    assert!(!plane.is_refined);
    plane.refine(SubpelMethod::Bicubic, &mut plane_data);
    assert!(plane.is_refined);
}

#[test]
fn test_mvplane_refine_different_subpel_methods() {
    for subpel_method in [
        SubpelMethod::Bilinear,
        SubpelMethod::Bicubic,
        SubpelMethod::Wiener,
    ] {
        let mut plane = create_test_mvplane(4, 4, Subpel::Half, 2, 2, 8, 0, 8);

        let max_offset = plane.subpel_window_offsets.iter().max().unwrap_or(&0);
        let window_size = plane.pitch.get() * (plane.height.get() + 2 * plane.vpad);
        let total_size = max_offset + window_size + 1000; // Extra safety margin
        let mut plane_data = vec![1u8; total_size];

        assert!(!plane.is_refined);
        plane.refine(subpel_method, &mut plane_data);
        assert!(plane.is_refined);
    }
}

#[test]
fn test_mvplane_refine_already_refined() {
    let mut plane = create_test_mvplane(4, 4, Subpel::Half, 2, 2, 8, 0, 8);

    let max_offset = plane.subpel_window_offsets.iter().max().unwrap_or(&0);
    let window_size = plane.pitch.get() * (plane.height.get() + 2 * plane.vpad);
    let total_size = max_offset + window_size + 1000; // Extra safety margin
    let mut plane_data = vec![1u8; total_size];

    // Refine once
    plane.refine(SubpelMethod::Bilinear, &mut plane_data);
    assert!(plane.is_refined);

    // Modify data to verify it doesn't get overwritten
    if plane.subpel_window_offsets.len() > 1 {
        plane_data[plane.subpel_window_offsets[1]] = 99;

        // Try to refine again - should be a no-op
        plane.refine(SubpelMethod::Bicubic, &mut plane_data);
        assert_eq!(plane_data[plane.subpel_window_offsets[1]], 99);
    }
}

#[test]
fn test_mvplane_constructor_valid_bits() {
    // Test with various valid bits per sample values
    for bits in [8, 10, 12, 16] {
        let result = MVPlane::new(
            NonZeroUsize::new(4).unwrap(),
            NonZeroUsize::new(4).unwrap(),
            Subpel::Full,
            2,
            2,
            NonZeroU8::new(bits).unwrap(),
            0,
            NonZeroUsize::new(8).unwrap(),
        );

        assert!(result.is_ok(), "Failed for {} bits per sample", bits);
    }
}

#[test]
fn test_mvplane_offset_calculations() {
    let plane = create_test_mvplane(8, 6, Subpel::Half, 4, 3, 8, 100, 16);

    // Verify offset_padding calculation
    // offset_padding = pitch * vpad + hpad * bytes_per_sample
    // = 16 * 3 + 4 * 1 = 48 + 4 = 52
    assert_eq!(plane.offset_padding, 52);

    // Verify subpel window offsets
    assert_eq!(plane.subpel_window_offsets[0], 100); // plane_offset

    // For half-pel (2x2), each window is separated by padded_height * pitch
    let padded_height = 6 + 2 * 3; // height + 2 * vpad = 12
    let window_spacing = padded_height * 16; // padded_height * pitch = 192

    assert_eq!(plane.subpel_window_offsets[1], 100 + window_spacing);
    assert_eq!(plane.subpel_window_offsets[2], 100 + 2 * window_spacing);
    assert_eq!(plane.subpel_window_offsets[3], 100 + 3 * window_spacing);
}

// Tests for the standalone functions
#[test]
fn test_plane_height_luma_level_0() {
    let src_height = NonZeroUsize::new(100).unwrap();
    let y_ratio_uv = NonZeroU8::new(2).unwrap();
    let vpad = 8;

    let result = plane_height_luma(src_height, 0, y_ratio_uv, vpad);

    // Level 0 should return original height
    assert_eq!(result.get(), 100);
}

#[test]
fn test_plane_height_luma_downscaling() {
    let src_height = NonZeroUsize::new(200).unwrap();
    let y_ratio_uv = NonZeroU8::new(2).unwrap();
    let vpad = 4; // vpad < y_ratio_uv

    // Level 1: height = ((200 / 2) / 2) * 2 = 50 * 2 = 100
    let result_level_1 = plane_height_luma(src_height, 1, y_ratio_uv, vpad);
    assert_eq!(result_level_1.get(), 100);

    // Level 2: height = ((100 / 2) / 2) * 2 = 25 * 2 = 50
    let result_level_2 = plane_height_luma(src_height, 2, y_ratio_uv, vpad);
    assert_eq!(result_level_2.get(), 50);
}

#[test]
fn test_plane_height_luma_with_large_vpad() {
    let src_height = NonZeroUsize::new(200).unwrap();
    let y_ratio_uv = NonZeroU8::new(2).unwrap();
    let vpad = 8; // vpad >= y_ratio_uv

    // Level 1: height = (200 / 2).div_ceil(2) * 2 = 100.div_ceil(2) * 2 = 50 * 2 = 100
    let result = plane_height_luma(src_height, 1, y_ratio_uv, vpad);
    assert_eq!(result.get(), 100);
}

#[test]
fn test_plane_height_luma_multiple_levels() {
    let src_height = NonZeroUsize::new(1600).unwrap();
    let y_ratio_uv = NonZeroU8::new(1).unwrap(); // 4:4:4 format
    let vpad = 0;

    // Level 1: height = ((1600 / 1) / 2) * 1 = 800
    let result_1 = plane_height_luma(src_height, 1, y_ratio_uv, vpad);
    assert_eq!(result_1.get(), 800);

    // Level 2: height = ((800 / 1) / 2) * 1 = 400
    let result_2 = plane_height_luma(src_height, 2, y_ratio_uv, vpad);
    assert_eq!(result_2.get(), 400);

    // Level 3: height = ((400 / 1) / 2) * 1 = 200
    let result_3 = plane_height_luma(src_height, 3, y_ratio_uv, vpad);
    assert_eq!(result_3.get(), 200);
}

#[test]
fn test_plane_height_luma_4_2_0_format() {
    let src_height = NonZeroUsize::new(480).unwrap();
    let y_ratio_uv = NonZeroU8::new(2).unwrap(); // 4:2:0 format
    let vpad = 0;

    // Level 1: height = ((480 / 2) / 2) * 2 = 120 * 2 = 240
    let result_1 = plane_height_luma(src_height, 1, y_ratio_uv, vpad);
    assert_eq!(result_1.get(), 240);

    // Level 2: height = ((240 / 2) / 2) * 2 = 60 * 2 = 120
    let result_2 = plane_height_luma(src_height, 2, y_ratio_uv, vpad);
    assert_eq!(result_2.get(), 120);
}

#[test]
fn test_plane_width_luma_level_0() {
    let src_width = NonZeroUsize::new(100).unwrap();
    let x_ratio_uv = NonZeroU8::new(2).unwrap();
    let hpad = 8;

    let result = plane_width_luma(src_width, 0, x_ratio_uv, hpad);

    // Level 0 should return original width
    assert_eq!(result.get(), 100);
}

#[test]
fn test_plane_width_luma_downscaling() {
    let src_width = NonZeroUsize::new(400).unwrap();
    let x_ratio_uv = NonZeroU8::new(2).unwrap();
    let hpad = 1; // hpad < x_ratio_uv

    // Level 1: width = ((400 / 2) / 2) * 2 = 100 * 2 = 200
    let result_level_1 = plane_width_luma(src_width, 1, x_ratio_uv, hpad);
    assert_eq!(result_level_1.get(), 200);

    // Level 2: width = ((200 / 2) / 2) * 2 = 50 * 2 = 100
    let result_level_2 = plane_width_luma(src_width, 2, x_ratio_uv, hpad);
    assert_eq!(result_level_2.get(), 100);
}

#[test]
fn test_plane_width_luma_with_large_hpad() {
    let src_width = NonZeroUsize::new(400).unwrap();
    let x_ratio_uv = NonZeroU8::new(2).unwrap();
    let hpad = 4; // hpad >= x_ratio_uv

    // Level 1: width = (400 / 2).div_ceil(2) * 2 = 200.div_ceil(2) * 2 = 100 * 2 = 200
    let result = plane_width_luma(src_width, 1, x_ratio_uv, hpad);
    assert_eq!(result.get(), 200);
}

#[test]
fn test_plane_width_luma_4_4_4_format() {
    let src_width = NonZeroUsize::new(1920).unwrap();
    let x_ratio_uv = NonZeroU8::new(1).unwrap(); // 4:4:4 format
    let hpad = 0;

    // Level 1: width = ((1920 / 1) / 2) * 1 = 960
    let result_1 = plane_width_luma(src_width, 1, x_ratio_uv, hpad);
    assert_eq!(result_1.get(), 960);

    // Level 2: width = ((960 / 1) / 2) * 1 = 480
    let result_2 = plane_width_luma(src_width, 2, x_ratio_uv, hpad);
    assert_eq!(result_2.get(), 480);
}

#[test]
fn test_plane_width_luma_4_2_0_format() {
    let src_width = NonZeroUsize::new(640).unwrap();
    let x_ratio_uv = NonZeroU8::new(2).unwrap(); // 4:2:0 format
    let hpad = 1;

    // Level 1: width = ((640 / 2) / 2) * 2 = 160 * 2 = 320
    let result_1 = plane_width_luma(src_width, 1, x_ratio_uv, hpad);
    assert_eq!(result_1.get(), 320);

    // Level 2: width = ((320 / 2) / 2) * 2 = 80 * 2 = 160
    let result_2 = plane_width_luma(src_width, 2, x_ratio_uv, hpad);
    assert_eq!(result_2.get(), 160);
}

#[test]
fn test_plane_super_offset_level_0() {
    let src_height = NonZeroUsize::new(100).unwrap();
    let plane_pitch = NonZeroUsize::new(120).unwrap();
    let y_ratio_uv = NonZeroU8::new(2).unwrap();
    let vpad = 8;

    // Level 0 should always return offset 0
    let offset_luma = plane_super_offset(
        false,
        src_height,
        0,
        Subpel::Full,
        vpad,
        plane_pitch,
        y_ratio_uv,
    );
    let offset_chroma = plane_super_offset(
        true,
        src_height,
        0,
        Subpel::Full,
        vpad,
        plane_pitch,
        y_ratio_uv,
    );

    assert_eq!(offset_luma, 0);
    assert_eq!(offset_chroma, 0);
}

#[test]
fn test_plane_super_offset_level_1_luma() {
    let src_height = NonZeroUsize::new(100).unwrap();
    let plane_pitch = NonZeroUsize::new(120).unwrap();
    let y_ratio_uv = NonZeroU8::new(2).unwrap();
    let vpad = 8;
    let pel = Subpel::Full;

    // Level 1: offset = pel * pel * plane_pitch * (src_height + vpad * 2)
    // offset = 1 * 1 * 120 * (100 + 8 * 2) = 120 * 116 = 13920
    let offset = plane_super_offset(false, src_height, 1, pel, vpad, plane_pitch, y_ratio_uv);
    assert_eq!(offset, 13920);
}

#[test]
fn test_plane_super_offset_level_1_chroma() {
    let src_height = NonZeroUsize::new(100).unwrap();
    let plane_pitch = NonZeroUsize::new(120).unwrap();
    let y_ratio_uv = NonZeroU8::new(2).unwrap();
    let vpad = 8;
    let pel = Subpel::Full;

    // Level 1: offset = pel * pel * plane_pitch * (src_height + vpad * 2)
    // offset = 1 * 1 * 120 * (100 + 8 * 2) = 120 * 116 = 13920
    let offset = plane_super_offset(true, src_height, 1, pel, vpad, plane_pitch, y_ratio_uv);
    assert_eq!(offset, 13920);
}

#[test]
fn test_plane_super_offset_half_pel() {
    let src_height = NonZeroUsize::new(100).unwrap();
    let plane_pitch = NonZeroUsize::new(120).unwrap();
    let y_ratio_uv = NonZeroU8::new(2).unwrap();
    let vpad = 8;
    let pel = Subpel::Half;

    // Level 1: offset = pel * pel * plane_pitch * (src_height + vpad * 2)
    // offset = 2 * 2 * 120 * (100 + 8 * 2) = 4 * 120 * 116 = 55680
    let offset = plane_super_offset(false, src_height, 1, pel, vpad, plane_pitch, y_ratio_uv);
    assert_eq!(offset, 55680);
}

#[test]
fn test_plane_super_offset_quarter_pel() {
    let src_height = NonZeroUsize::new(100).unwrap();
    let plane_pitch = NonZeroUsize::new(120).unwrap();
    let y_ratio_uv = NonZeroU8::new(2).unwrap();
    let vpad = 8;
    let pel = Subpel::Quarter;

    // Level 1: offset = pel * pel * plane_pitch * (src_height + vpad * 2)
    // offset = 4 * 4 * 120 * (100 + 8 * 2) = 16 * 120 * 116 = 222720
    let offset = plane_super_offset(false, src_height, 1, pel, vpad, plane_pitch, y_ratio_uv);
    assert_eq!(offset, 222720);
}

#[test]
fn test_plane_super_offset_multiple_levels() {
    let src_height = NonZeroUsize::new(200).unwrap();
    let plane_pitch = NonZeroUsize::new(240).unwrap();
    let y_ratio_uv = NonZeroU8::new(2).unwrap();
    let vpad = 4;
    let pel = Subpel::Full;

    // Level 2 offset calculation:
    // Base offset = 1 * 1 * 240 * (200 + 4 * 2) = 240 * 208 = 49920
    // Loop iteration 1: height = plane_height_luma(src_height, 1, y_ratio_uv, vpad) = 100
    // Additional offset = 240 * (100 + 4 * 2) = 240 * 108 = 25920
    // Total offset = 49920 + 25920 = 75840
    let offset = plane_super_offset(false, src_height, 2, pel, vpad, plane_pitch, y_ratio_uv);
    assert_eq!(offset, 75840);
}

#[test]
fn test_plane_super_offset_chroma_vs_luma() {
    let src_height = NonZeroUsize::new(200).unwrap();
    let plane_pitch = NonZeroUsize::new(240).unwrap();
    let y_ratio_uv = NonZeroU8::new(2).unwrap();
    let vpad = 4;
    let pel = Subpel::Full;

    let offset_luma = plane_super_offset(false, src_height, 2, pel, vpad, plane_pitch, y_ratio_uv);
    let offset_chroma = plane_super_offset(true, src_height, 2, pel, vpad, plane_pitch, y_ratio_uv);

    // Both should have the same offset for level > 0 since the chroma calculation
    // uses the same base offset and then calculates height differently but ends up
    // with the same result due to the division by y_ratio_uv_val
    assert_eq!(offset_luma, offset_chroma);
}

#[test]
fn test_plane_super_offset_realistic_video_dimensions() {
    // Test with realistic HD video dimensions
    let src_height = NonZeroUsize::new(1080).unwrap();
    let plane_pitch = NonZeroUsize::new(1920).unwrap();
    let y_ratio_uv = NonZeroU8::new(2).unwrap(); // 4:2:0
    let vpad = 16;
    let pel = Subpel::Quarter;

    // Level 1 offset should be calculable and reasonable
    let offset = plane_super_offset(false, src_height, 1, pel, vpad, plane_pitch, y_ratio_uv);

    // Verify it's the expected calculation:
    // offset = 4 * 4 * 1920 * (1080 + 16 * 2) = 16 * 1920 * 1112 = 34,160,640
    assert_eq!(offset, 34_160_640);
}

#[test]
fn test_plane_height_width_consistency() {
    // Test that height and width functions behave consistently
    let src_dim = NonZeroUsize::new(320).unwrap();
    let ratio_uv = NonZeroU8::new(2).unwrap();
    let pad = 8;

    let height = plane_height_luma(src_dim, 1, ratio_uv, pad);
    let width = plane_width_luma(src_dim, 1, ratio_uv, pad);

    // Both should give the same result for same input parameters
    assert_eq!(height, width);
}

#[test]
fn test_mvplane_refine_no_clones_performance() {
    // Test that the refine method works efficiently without clones on larger data
    let mut plane = create_test_mvplane(64, 64, Subpel::Quarter, 8, 8, 8, 0, 80);

    let max_offset = plane.subpel_window_offsets.iter().max().unwrap_or(&0);
    let window_size = plane.pitch.get() * (plane.height.get() + 2 * plane.vpad);
    let total_size = max_offset + window_size + 1000; // Extra safety margin
    let mut plane_data = vec![42u8; total_size];

    // Fill with a pattern to verify correctness
    for (i, val) in plane_data.iter_mut().enumerate() {
        *val = (i % 256) as u8;
    }

    assert!(!plane.is_refined);

    // This should work without creating any to_vec() clones
    plane.refine(SubpelMethod::Bilinear, &mut plane_data);

    assert!(plane.is_refined);

    // Verify that the data has been modified by the refinement process
    // (This is a simple check - the actual refinement should change some values)
    let original_pattern_intact = plane_data
        .iter()
        .enumerate()
        .all(|(i, &val)| val == (i % 256) as u8);
    assert!(
        !original_pattern_intact,
        "Refinement should have modified the data"
    );
}

#[test]
fn test_mvplane_refine_different_pel_methods_no_clones() {
    // Test all subpel methods work correctly without clones
    for pel in [Subpel::Half, Subpel::Quarter] {
        for method in [
            SubpelMethod::Bilinear,
            SubpelMethod::Bicubic,
            SubpelMethod::Wiener,
        ] {
            let mut plane = create_test_mvplane(16, 16, pel, 4, 4, 8, 0, 24);

            let max_offset = plane.subpel_window_offsets.iter().max().unwrap_or(&0);
            let window_size = plane.pitch.get() * (plane.height.get() + 2 * plane.vpad);
            let total_size = max_offset + window_size + 1000;
            let mut plane_data = vec![100u8; total_size];

            assert!(!plane.is_refined);
            plane.refine(method, &mut plane_data);
            assert!(plane.is_refined);
        }
    }
}
