use std::num::NonZeroUsize;

use super::{pad_corner, pad_reference_frame};

/// Helper function to create a test frame with a specific pattern
/// Creates a frame with total size including padding, but only fills the inner
/// content area
fn create_test_frame<T: Copy + From<u8>>(
    width: usize,
    height: usize,
    hpad: usize,
    vpad: usize,
    pitch: usize,
) -> Vec<T> {
    let total_height = height + 2 * vpad;
    let mut frame = vec![T::from(0); pitch * total_height];

    // Fill the inner rectangle (actual content area) with a pattern for easy
    // verification
    for y in 0..height {
        for x in 0..width {
            let value = ((y * width + x) % 255) as u8;
            frame[(vpad + y) * pitch + hpad + x] = T::from(value);
        }
    }

    frame
}

/// Helper function to verify that padding was applied correctly
fn verify_padding<T: Copy + PartialEq + std::fmt::Debug>(
    frame: &[T],
    offset: usize,
    pitch: usize,
    hpad: usize,
    vpad: usize,
    width: usize,
    height: usize,
) {
    // Verify corners
    let top_left_value = frame[offset + vpad * pitch + hpad];
    let top_right_value = frame[offset + vpad * pitch + hpad + width - 1];
    let bottom_left_value = frame[offset + (vpad + height - 1) * pitch + hpad];
    let bottom_right_value = frame[offset + (vpad + height - 1) * pitch + hpad + width - 1];

    // Check top-left corner
    for y in 0..vpad {
        for x in 0..hpad {
            assert_eq!(
                frame[offset + y * pitch + x],
                top_left_value,
                "Top-left corner padding failed at ({}, {})",
                x,
                y
            );
        }
    }

    // Check top-right corner
    for y in 0..vpad {
        for x in 0..hpad {
            assert_eq!(
                frame[offset + y * pitch + hpad + width + x],
                top_right_value,
                "Top-right corner padding failed at ({}, {})",
                x,
                y
            );
        }
    }

    // Check bottom-left corner
    for y in 0..vpad {
        for x in 0..hpad {
            assert_eq!(
                frame[offset + (vpad + height + y) * pitch + x],
                bottom_left_value,
                "Bottom-left corner padding failed at ({}, {})",
                x,
                y
            );
        }
    }

    // Check bottom-right corner
    for y in 0..vpad {
        for x in 0..hpad {
            assert_eq!(
                frame[offset + (vpad + height + y) * pitch + hpad + width + x],
                bottom_right_value,
                "Bottom-right corner padding failed at ({}, {})",
                x,
                y
            );
        }
    }

    // Verify top edge
    for x in 0..width {
        let expected_value = frame[offset + vpad * pitch + hpad + x];
        for y in 0..vpad {
            assert_eq!(
                frame[offset + y * pitch + hpad + x],
                expected_value,
                "Top edge padding failed at ({}, {})",
                x,
                y
            );
        }
    }

    // Verify bottom edge
    for x in 0..width {
        let expected_value = frame[offset + (vpad + height - 1) * pitch + hpad + x];
        for y in 0..vpad {
            assert_eq!(
                frame[offset + (vpad + height + y) * pitch + hpad + x],
                expected_value,
                "Bottom edge padding failed at ({}, {})",
                x,
                y
            );
        }
    }

    // Verify left edge
    for y in 0..height {
        let expected_value = frame[offset + (vpad + y) * pitch + hpad];
        for x in 0..hpad {
            assert_eq!(
                frame[offset + (vpad + y) * pitch + x],
                expected_value,
                "Left edge padding failed at ({}, {})",
                x,
                y
            );
        }
    }

    // Verify right edge
    for y in 0..height {
        let expected_value = frame[offset + (vpad + y) * pitch + hpad + width - 1];
        for x in 0..hpad {
            assert_eq!(
                frame[offset + (vpad + y) * pitch + hpad + width + x],
                expected_value,
                "Right edge padding failed at ({}, {})",
                x,
                y
            );
        }
    }
}

#[test]
fn test_pad_reference_frame_u8_basic() {
    let width = 4;
    let height = 4;
    let hpad = 2;
    let vpad = 2;
    let pitch = width + 2 * hpad;

    let mut frame = create_test_frame::<u8>(width, height, hpad, vpad, pitch);
    let offset = 0;

    pad_reference_frame(
        offset,
        NonZeroUsize::new(pitch).unwrap(),
        hpad,
        vpad,
        NonZeroUsize::new(width).unwrap(),
        NonZeroUsize::new(height).unwrap(),
        &mut frame,
    );

    verify_padding(&frame, offset, pitch, hpad, vpad, width, height);
}

#[test]
fn test_pad_reference_frame_u16_basic() {
    let width = 4;
    let height = 4;
    let hpad = 2;
    let vpad = 2;
    let pitch = width + 2 * hpad;

    let mut frame = create_test_frame::<u16>(width, height, hpad, vpad, pitch);
    let offset = 0;

    pad_reference_frame(
        offset,
        NonZeroUsize::new(pitch).unwrap(),
        hpad,
        vpad,
        NonZeroUsize::new(width).unwrap(),
        NonZeroUsize::new(height).unwrap(),
        &mut frame,
    );

    verify_padding(&frame, offset, pitch, hpad, vpad, width, height);
}

#[test]
fn test_pad_reference_frame_with_offset() {
    let width = 3;
    let height = 3;
    let hpad = 1;
    let vpad = 1;
    let pitch = width + 2 * hpad;
    let offset = 10; // Non-zero offset

    let frame_size = offset + (height + 2 * vpad) * pitch;
    let mut frame = vec![0u8; frame_size];

    // Fill the actual frame area with test data
    for y in 0..height {
        for x in 0..width {
            let value = ((y * width + x + 1) * 10) as u8;
            frame[offset + (vpad + y) * pitch + hpad + x] = value;
        }
    }

    pad_reference_frame(
        offset,
        NonZeroUsize::new(pitch).unwrap(),
        hpad,
        vpad,
        NonZeroUsize::new(width).unwrap(),
        NonZeroUsize::new(height).unwrap(),
        &mut frame,
    );

    verify_padding(&frame, offset, pitch, hpad, vpad, width, height);
}

#[test]
fn test_pad_reference_frame_minimal_size() {
    let width = 1;
    let height = 1;
    let hpad = 1;
    let vpad = 1;
    let pitch = width + 2 * hpad;

    let mut frame = create_test_frame::<u8>(width, height, hpad, vpad, pitch);
    let offset = 0;

    // Set a specific value for the single pixel
    frame[vpad * pitch + hpad] = 42;

    pad_reference_frame(
        offset,
        NonZeroUsize::new(pitch).unwrap(),
        hpad,
        vpad,
        NonZeroUsize::new(width).unwrap(),
        NonZeroUsize::new(height).unwrap(),
        &mut frame,
    );

    // All padding should be the value of the single pixel
    for y in 0..(height + 2 * vpad) {
        for x in 0..(width + 2 * hpad) {
            if y == vpad && x == hpad {
                continue; // Skip the original pixel
            }
            assert_eq!(frame[y * pitch + x], 42, "Padding failed at ({}, {})", x, y);
        }
    }
}

#[test]
fn test_pad_reference_frame_asymmetric_padding() {
    let width = 3;
    let height = 2;
    let hpad = 3; // Different horizontal padding
    let vpad = 1; // Different vertical padding
    let pitch = width + 2 * hpad;

    let mut frame = create_test_frame::<u16>(width, height, hpad, vpad, pitch);
    let offset = 0;

    pad_reference_frame(
        offset,
        NonZeroUsize::new(pitch).unwrap(),
        hpad,
        vpad,
        NonZeroUsize::new(width).unwrap(),
        NonZeroUsize::new(height).unwrap(),
        &mut frame,
    );

    verify_padding(&frame, offset, pitch, hpad, vpad, width, height);
}

#[test]
fn test_pad_reference_frame_large_frame() {
    let width = 16;
    let height = 12;
    let hpad = 4;
    let vpad = 3;
    let pitch = width + 2 * hpad;

    let mut frame = create_test_frame::<u8>(width, height, hpad, vpad, pitch);
    let offset = 0;

    pad_reference_frame(
        offset,
        NonZeroUsize::new(pitch).unwrap(),
        hpad,
        vpad,
        NonZeroUsize::new(width).unwrap(),
        NonZeroUsize::new(height).unwrap(),
        &mut frame,
    );

    verify_padding(&frame, offset, pitch, hpad, vpad, width, height);
}

#[test]
fn test_pad_reference_frame_wide_frame() {
    let width = 20;
    let height = 2;
    let hpad = 2;
    let vpad = 2;
    let pitch = width + 2 * hpad;

    let mut frame = create_test_frame::<u8>(width, height, hpad, vpad, pitch);
    let offset = 0;

    pad_reference_frame(
        offset,
        NonZeroUsize::new(pitch).unwrap(),
        hpad,
        vpad,
        NonZeroUsize::new(width).unwrap(),
        NonZeroUsize::new(height).unwrap(),
        &mut frame,
    );

    verify_padding(&frame, offset, pitch, hpad, vpad, width, height);
}

#[test]
fn test_pad_reference_frame_tall_frame() {
    let width = 2;
    let height = 20;
    let hpad = 2;
    let vpad = 2;
    let pitch = width + 2 * hpad;

    let mut frame = create_test_frame::<u16>(width, height, hpad, vpad, pitch);
    let offset = 0;

    pad_reference_frame(
        offset,
        NonZeroUsize::new(pitch).unwrap(),
        hpad,
        vpad,
        NonZeroUsize::new(width).unwrap(),
        NonZeroUsize::new(height).unwrap(),
        &mut frame,
    );

    verify_padding(&frame, offset, pitch, hpad, vpad, width, height);
}

#[test]
fn test_pad_reference_frame_pitch_larger_than_padded_width() {
    let width = 4;
    let height = 3;
    let hpad = 2;
    let vpad = 2;
    let pitch = width + 2 * hpad + 4; // Extra pitch

    let mut frame = create_test_frame::<u8>(width, height, hpad, vpad, pitch);
    let offset = 0;

    pad_reference_frame(
        offset,
        NonZeroUsize::new(pitch).unwrap(),
        hpad,
        vpad,
        NonZeroUsize::new(width).unwrap(),
        NonZeroUsize::new(height).unwrap(),
        &mut frame,
    );

    verify_padding(&frame, offset, pitch, hpad, vpad, width, height);
}

#[test]
fn test_pad_corner_basic() {
    let hpad = 2;
    let vpad = 2;
    let pitch = 6;
    let mut dest = vec![0u8; pitch * vpad];
    let val = 42u8;
    let offset = 0;

    // SAFETY: test function
    unsafe {
        pad_corner(
            offset,
            val,
            hpad,
            vpad,
            NonZeroUsize::new(pitch).unwrap(),
            &mut dest,
        );
    }

    // Verify all pixels in the corner are set to the expected value
    for y in 0..vpad {
        for x in 0..hpad {
            assert_eq!(
                dest[y * pitch + x],
                val,
                "Corner padding failed at ({}, {})",
                x,
                y
            );
        }
    }
}

#[test]
fn test_pad_corner_u16() {
    let hpad = 3;
    let vpad = 1;
    let pitch = 8;
    let mut dest = vec![0u16; pitch * vpad];
    let val = 1337u16;
    let offset = 0;

    // SAFETY: test function
    unsafe {
        pad_corner(
            offset,
            val,
            hpad,
            vpad,
            NonZeroUsize::new(pitch).unwrap(),
            &mut dest,
        );
    }

    // Verify all pixels in the corner are set to the expected value
    for y in 0..vpad {
        for x in 0..hpad {
            assert_eq!(
                dest[y * pitch + x],
                val,
                "Corner padding failed at ({}, {})",
                x,
                y
            );
        }
    }
}

#[test]
fn test_pad_corner_with_offset() {
    let hpad = 2;
    let vpad = 2;
    let pitch = 6;
    let offset = 12;
    let total_size = offset + pitch * vpad;
    let mut dest = vec![0u8; total_size];
    let val = 99u8;

    // SAFETY: test function
    unsafe {
        pad_corner(
            offset,
            val,
            hpad,
            vpad,
            NonZeroUsize::new(pitch).unwrap(),
            &mut dest,
        );
    }

    // Verify the area before offset is unchanged
    for (i, &value) in dest.iter().take(offset).enumerate() {
        assert_eq!(value, 0, "Data before offset was modified at index {}", i);
    }

    // Verify corner padding
    for y in 0..vpad {
        for x in 0..hpad {
            assert_eq!(
                dest[offset + y * pitch + x],
                val,
                "Corner padding failed at ({}, {})",
                x,
                y
            );
        }
    }
}

#[test]
fn test_pad_reference_frame_preserves_original_data() {
    let width = 3;
    let height = 3;
    let hpad = 1;
    let vpad = 1;
    let pitch = width + 2 * hpad;

    let mut frame = create_test_frame::<u8>(width, height, hpad, vpad, pitch);
    let original_data: Vec<u8> = frame
        .iter()
        .enumerate()
        .skip(vpad * pitch + hpad)
        .take(height * pitch - hpad)
        .filter(|(i, _)| {
            let row = (*i - vpad * pitch) / pitch;
            let col = (*i - vpad * pitch) % pitch;
            col >= hpad && col < hpad + width && row < height
        })
        .map(|(_, &val)| val)
        .collect();

    let offset = 0;

    pad_reference_frame(
        offset,
        NonZeroUsize::new(pitch).unwrap(),
        hpad,
        vpad,
        NonZeroUsize::new(width).unwrap(),
        NonZeroUsize::new(height).unwrap(),
        &mut frame,
    );

    // Verify original data is preserved
    let mut original_iter = original_data.iter();
    for y in 0..height {
        for x in 0..width {
            let actual = frame[offset + (vpad + y) * pitch + hpad + x];
            let expected = *original_iter.next().unwrap();
            assert_eq!(
                actual, expected,
                "Original data was modified at ({}, {})",
                x, y
            );
        }
    }
}

#[test]
fn test_pad_reference_frame_edge_values_correctness() {
    let width = 3;
    let height = 3;
    let hpad = 2;
    let vpad = 1;
    let pitch = width + 2 * hpad;

    let mut frame = vec![0u8; (height + 2 * vpad) * pitch];

    // Set specific values in the original frame area
    let values = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];

    for y in 0..height {
        for x in 0..width {
            frame[(vpad + y) * pitch + hpad + x] = values[y][x];
        }
    }

    let offset = 0;

    pad_reference_frame(
        offset,
        NonZeroUsize::new(pitch).unwrap(),
        hpad,
        vpad,
        NonZeroUsize::new(width).unwrap(),
        NonZeroUsize::new(height).unwrap(),
        &mut frame,
    );

    // Check specific edge values
    // Top edge should replicate first row
    for x in 0..width {
        for y in 0..vpad {
            assert_eq!(
                frame[y * pitch + hpad + x],
                values[0][x],
                "Top edge incorrect at ({}, {})",
                x,
                y
            );
        }
    }

    // Bottom edge should replicate last row
    for x in 0..width {
        for y in 0..vpad {
            assert_eq!(
                frame[(vpad + height + y) * pitch + hpad + x],
                values[height - 1][x],
                "Bottom edge incorrect at ({}, {})",
                x,
                y
            );
        }
    }

    // Left edge should replicate first column
    for y in 0..height {
        for x in 0..hpad {
            assert_eq!(
                frame[(vpad + y) * pitch + x],
                values[y][0],
                "Left edge incorrect at ({}, {})",
                x,
                y
            );
        }
    }

    // Right edge should replicate last column
    for y in 0..height {
        for x in 0..hpad {
            assert_eq!(
                frame[(vpad + y) * pitch + hpad + width + x],
                values[y][width - 1],
                "Right edge incorrect at ({}, {})",
                x,
                y
            );
        }
    }
}
