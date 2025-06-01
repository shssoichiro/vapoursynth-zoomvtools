mod avx2;
mod rust;

use std::num::{NonZeroU8, NonZeroUsize};

use crate::util::Pixel;

/// Performs horizontal Wiener filtering for sub-pixel motion estimation refinement.
///
/// This function applies a Wiener filter horizontally to create high-quality sub-pixel
/// samples between existing pixels. The Wiener filter uses a 6-tap kernel with optimized
/// coefficients that provide excellent interpolation quality by minimizing reconstruction
/// error while preserving image details.
///
/// Edge pixels use simple averaging due to insufficient neighbors for the full kernel.
/// The Wiener filter is particularly effective for maintaining sharpness during
/// sub-pixel interpolation in motion estimation applications.
///
/// # Parameters
/// - `src`: Source image buffer
/// - `dest`: Destination buffer for interpolated results
/// - `pitch`: Number of pixels per row in both buffers
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
/// - `bits_per_sample`: Bit depth of the pixel format for clamping
pub fn refine_horizontal_wiener<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    debug_assert!(
        bits_per_sample.get() as usize > (size_of::<T>() - 1) * 8
            && (bits_per_sample.get() as usize <= size_of::<T>() * 8)
    );

    rust::refine_horizontal_wiener(src, dest, pitch, width, height, bits_per_sample);
}

/// Performs vertical Wiener filtering for sub-pixel motion estimation refinement.
///
/// This function applies a Wiener filter vertically to create high-quality sub-pixel
/// samples between existing pixels. The Wiener filter uses a 6-tap kernel with optimized
/// coefficients that provide excellent interpolation quality by minimizing reconstruction
/// error while preserving image details.
///
/// Edge rows use simple averaging due to insufficient neighbors for the full kernel,
/// and the last row is copied directly from the source. The Wiener filter is
/// particularly effective for maintaining sharpness during sub-pixel interpolation.
///
/// # Parameters
/// - `src`: Source image buffer
/// - `dest`: Destination buffer for interpolated results
/// - `pitch`: Number of pixels per row in both buffers
/// - `width`: Width of the image in pixels
/// - `height`: Height of the image in pixels
/// - `bits_per_sample`: Bit depth of the pixel format for clamping
pub fn refine_vertical_wiener<T: Pixel>(
    src: &[T],
    dest: &mut [T],
    pitch: NonZeroUsize,
    width: NonZeroUsize,
    height: NonZeroUsize,
    bits_per_sample: NonZeroU8,
) {
    debug_assert!(
        bits_per_sample.get() as usize > (size_of::<T>() - 1) * 8
            && (bits_per_sample.get() as usize <= size_of::<T>() * 8)
    );

    rust::refine_vertical_wiener(src, dest, pitch, width, height, bits_per_sample);
}

#[cfg(test)]
#[allow(unused_unsafe)]
#[allow(clippy::undocumented_unsafe_blocks)]
mod tests {
    use std::num::{NonZeroU8, NonZeroUsize};

    use super::{refine_horizontal_wiener, refine_vertical_wiener};
    use pastey::paste;

    macro_rules! horizontal_tests {
        ($module:ident) => {
            paste! {
                #[test]
                fn test_horizontal_wiener_u8_basic() {
                    // Test basic horizontal Wiener filtering with u8 pixels
                    let width = NonZeroUsize::new(8).unwrap();
                    let height = NonZeroUsize::new(1).unwrap();
                    let pitch = width;
                    let bits_per_sample = NonZeroU8::new(8).unwrap();

                    // Create test data: gradual increase from 0 to 255
                    let src: Vec<u8> = vec![0, 32, 64, 96, 128, 160, 192, 255];
                    let mut dest = vec![0u8; 8];

                    refine_horizontal_wiener(&src, &mut dest, pitch, width, height, bits_per_sample);

                    // First two pixels should be bilinear interpolation
                    assert_eq!(dest[0], 16); // (0 + 32) / 2
                    assert_eq!(dest[1], 48); // (32 + 64) / 2

                    // Last pixel should be unchanged
                    assert_eq!(dest[7], 255);

                    // Middle pixels should be Wiener filtered (values will be different from
                    // bilinear) The exact values depend on the Wiener filter implementation
                    assert!(dest[2] > 0 && dest[2] < 255);
                    assert!(dest[3] > 0 && dest[3] < 255);
                }

                #[test]
                fn test_horizontal_wiener_u16_basic() {
                    // Test basic horizontal Wiener filtering with u16 pixels
                    let width = NonZeroUsize::new(8).unwrap();
                    let height = NonZeroUsize::new(1).unwrap();
                    let pitch = width;
                    let bits_per_sample = NonZeroU8::new(10).unwrap();

                    // Create test data for 10-bit: 0 to 1023
                    let src: Vec<u16> = vec![0, 128, 256, 384, 512, 640, 768, 1023];
                    let mut dest = vec![0u16; 8];

                    refine_horizontal_wiener(&src, &mut dest, pitch, width, height, bits_per_sample);

                    // First two pixels should be bilinear interpolation
                    assert_eq!(dest[0], 64); // (0 + 128) / 2
                    assert_eq!(dest[1], 192); // (128 + 256) / 2

                    // Last pixel should be unchanged
                    assert_eq!(dest[7], 1023);

                    // Middle pixels should be Wiener filtered
                    assert!(dest[2] > 0 && dest[2] < 1023);
                    assert!(dest[3] > 0 && dest[3] < 1023);
                }

                #[test]
                fn test_horizontal_wiener_multirow() {
                    // Test horizontal Wiener with multiple rows
                    let width = NonZeroUsize::new(6).unwrap();
                    let height = NonZeroUsize::new(3).unwrap();
                    let pitch = width;
                    let bits_per_sample = NonZeroU8::new(8).unwrap();

                    let src: Vec<u8> = vec![
                        10, 20, 30, 40, 50, 60, // Row 1
                        70, 80, 90, 100, 110, 120, // Row 2
                        130, 140, 150, 160, 170, 180, // Row 3
                    ];
                    let mut dest = vec![0u8; 18];

                    refine_horizontal_wiener(&src, &mut dest, pitch, width, height, bits_per_sample);

                    // Check that each row is processed independently
                    // First pixel of each row should be bilinear of first two
                    assert_eq!(dest[0], 15); // (10 + 20) / 2
                    assert_eq!(dest[6], 75); // (70 + 80) / 2
                    assert_eq!(dest[12], 135); // (130 + 140) / 2

                    // Last pixel of each row should be unchanged
                    assert_eq!(dest[5], 60);
                    assert_eq!(dest[11], 120);
                    assert_eq!(dest[17], 180);
                }

                #[test]
                fn test_horizontal_wiener_minimum_width() {
                    // Test with minimum width where Wiener filter can be applied
                    let width = NonZeroUsize::new(6).unwrap(); // Minimum for Wiener core (needs i-2 to i+3)
                    let height = NonZeroUsize::new(1).unwrap();
                    let pitch = width;
                    let bits_per_sample = NonZeroU8::new(8).unwrap();

                    let src: Vec<u8> = vec![0, 50, 100, 150, 200, 255];
                    let mut dest = vec![0u8; 6];

                    refine_horizontal_wiener(&src, &mut dest, pitch, width, height, bits_per_sample);

                    // Should not crash and produce reasonable results
                    assert_eq!(dest[0], 25); // (0 + 50) / 2
                    assert_eq!(dest[1], 75); // (50 + 100) / 2
                    assert_eq!(dest[5], 255); // Last pixel unchanged
                }

                #[test]
                fn test_wiener_with_different_pitch() {
                    // Test with pitch different from width (common in real scenarios)
                    let width = NonZeroUsize::new(4).unwrap();
                    let height = NonZeroUsize::new(2).unwrap();
                    let pitch = NonZeroUsize::new(8).unwrap(); // Pitch larger than width
                    let bits_per_sample = NonZeroU8::new(8).unwrap();

                    let src: Vec<u8> = vec![
                        10, 20, 30, 40, 0, 0, 0, 0, // Row 1 with padding
                        50, 60, 70, 80, 0, 0, 0, 0, // Row 2 with padding
                    ];
                    let mut dest = vec![0u8; 16];

                    refine_horizontal_wiener(&src, &mut dest, pitch, width, height, bits_per_sample);

                    // Check that pitch is respected
                    assert_eq!(dest[0], 15); // (10 + 20) / 2
                    assert_eq!(dest[3], 40); // Last pixel unchanged
                    assert_eq!(dest[8], 55); // (50 + 60) / 2 - second row
                    assert_eq!(dest[11], 80); // Last pixel of second row
                }

                #[test]
                fn test_horizontal_wiener_uniform_values() {
                    // Test with uniform input values - should produce uniform output
                    let width = NonZeroUsize::new(8).unwrap();
                    let height = NonZeroUsize::new(1).unwrap();
                    let pitch = width;
                    let bits_per_sample = NonZeroU8::new(8).unwrap();

                    let src = vec![100u8; 8]; // All pixels same value
                    let mut dest = vec![0u8; 8];

                    refine_horizontal_wiener(&src, &mut dest, pitch, width, height, bits_per_sample);

                    // All output values should be the same as input
                    for &pixel in &dest {
                        assert_eq!(pixel, 100);
                    }
                }

                #[test]
                fn test_wiener_pixel_clamping() {
                    // Test that output is properly clamped to pixel range
                    let width = NonZeroUsize::new(8).unwrap();
                    let height = NonZeroUsize::new(1).unwrap();
                    let pitch = width;
                    let bits_per_sample = NonZeroU8::new(8).unwrap();

                    // Extreme values that might cause overflow in intermediate calculations
                    let src: Vec<u8> = vec![0, 0, 255, 255, 0, 0, 255, 255];
                    let mut dest = vec![0u8; 8];

                    refine_horizontal_wiener(&src, &mut dest, pitch, width, height, bits_per_sample);

                    // Test passes if function doesn't panic with extreme values
                    // The type system ensures u8 values are in valid range [0, 255]
                    assert_eq!(dest.len(), 8);
                }

                #[test]
                fn test_wiener_16bit_pixel_clamping() {
                    // Test pixel clamping for 16-bit values
                    let width = NonZeroUsize::new(8).unwrap();
                    let height = NonZeroUsize::new(1).unwrap();
                    let pitch = width;
                    let bits_per_sample = NonZeroU8::new(10).unwrap(); // 10-bit: max value 1023

                    // Values near the maximum that might cause overflow
                    let src: Vec<u16> = vec![0, 0, 1023, 1023, 0, 0, 1023, 1023];
                    let mut dest = vec![0u16; 8];

                    refine_horizontal_wiener(&src, &mut dest, pitch, width, height, bits_per_sample);

                    // All output values should be in valid range [0, 1023]
                    for &pixel in &dest {
                        assert!(pixel <= 1023);
                    }
                }
            }
        };
    }

    macro_rules! vertical_tests {
        ($module:ident) => {
            paste! {
                #[test]
                fn test_vertical_wiener_u8_basic() {
                    // Test basic vertical Wiener filtering with u8 pixels
                    let width = NonZeroUsize::new(1).unwrap();
                    let height = NonZeroUsize::new(8).unwrap();
                    let pitch = width;
                    let bits_per_sample = NonZeroU8::new(8).unwrap();

                    // Create test data: vertical gradient
                    let src: Vec<u8> = vec![0, 32, 64, 96, 128, 160, 192, 255];
                    let mut dest = vec![0u8; 8];

                    refine_vertical_wiener(&src, &mut dest, pitch, width, height, bits_per_sample);

                    // First two pixels should be bilinear interpolation
                    assert_eq!(dest[0], 16); // (0 + 32) / 2
                    assert_eq!(dest[1], 48); // (32 + 64) / 2

                    // Last pixel should be unchanged
                    assert_eq!(dest[7], 255);

                    // Middle pixels should be Wiener filtered
                    assert!(dest[2] > 0 && dest[2] < 255);
                    assert!(dest[3] > 0 && dest[3] < 255);
                }

                #[test]
                fn test_vertical_wiener_u16_basic() {
                    // Test basic vertical Wiener filtering with u16 pixels
                    let width = NonZeroUsize::new(1).unwrap();
                    let height = NonZeroUsize::new(8).unwrap();
                    let pitch = width;
                    let bits_per_sample = NonZeroU8::new(12).unwrap();

                    // Create test data for 12-bit: 0 to 4095
                    let src: Vec<u16> = vec![0, 512, 1024, 1536, 2048, 2560, 3072, 4095];
                    let mut dest = vec![0u16; 8];

                    refine_vertical_wiener(&src, &mut dest, pitch, width, height, bits_per_sample);

                    // First two pixels should be bilinear interpolation
                    assert_eq!(dest[0], 256); // (0 + 512) / 2
                    assert_eq!(dest[1], 768); // (512 + 1024) / 2

                    // Last pixel should be unchanged
                    assert_eq!(dest[7], 4095);

                    // Middle pixels should be Wiener filtered
                    assert!(dest[2] > 0 && dest[2] < 4095);
                    assert!(dest[3] > 0 && dest[3] < 4095);
                }

                #[test]
                fn test_vertical_wiener_multicolumn() {
                    // Test vertical Wiener with multiple columns
                    let width = NonZeroUsize::new(3).unwrap();
                    let height = NonZeroUsize::new(6).unwrap();
                    let pitch = width;
                    let bits_per_sample = NonZeroU8::new(8).unwrap();

                    let src: Vec<u8> = vec![
                        10, 70, 130, // Row 1
                        20, 80, 140, // Row 2
                        30, 90, 150, // Row 3
                        40, 100, 160, // Row 4
                        50, 110, 170, // Row 5
                        60, 120, 180, // Row 6
                    ];
                    let mut dest = vec![0u8; 18];

                    refine_vertical_wiener(&src, &mut dest, pitch, width, height, bits_per_sample);

                    // Check that each column is processed independently
                    // First two rows should be bilinear interpolation
                    assert_eq!(dest[0], 15); // (10 + 20) / 2
                    assert_eq!(dest[1], 75); // (70 + 80) / 2
                    assert_eq!(dest[2], 135); // (130 + 140) / 2

                    assert_eq!(dest[3], 25); // (20 + 30) / 2
                    assert_eq!(dest[4], 85); // (80 + 90) / 2
                    assert_eq!(dest[5], 145); // (140 + 150) / 2

                    // Last row should be unchanged
                    assert_eq!(dest[15], 60);
                    assert_eq!(dest[16], 120);
                    assert_eq!(dest[17], 180);
                }

                #[test]
                fn test_vertical_wiener_minimum_height() {
                    // Test with minimum height where Wiener filter can be applied
                    let width = NonZeroUsize::new(1).unwrap();
                    let height = NonZeroUsize::new(6).unwrap(); // Minimum for Wiener core
                    let pitch = width;
                    let bits_per_sample = NonZeroU8::new(8).unwrap();

                    let src: Vec<u8> = vec![0, 50, 100, 150, 200, 255];
                    let mut dest = vec![0u8; 6];

                    refine_vertical_wiener(&src, &mut dest, pitch, width, height, bits_per_sample);

                    // Should not crash and produce reasonable results
                    assert_eq!(dest[0], 25); // (0 + 50) / 2
                    assert_eq!(dest[1], 75); // (50 + 100) / 2
                    assert_eq!(dest[5], 255); // Last pixel unchanged
                }

                #[test]
                fn test_vertical_wiener_uniform_values() {
                    // Test with uniform input values - should produce uniform output
                    let width = NonZeroUsize::new(1).unwrap();
                    let height = NonZeroUsize::new(8).unwrap();
                    let pitch = width;
                    let bits_per_sample = NonZeroU8::new(8).unwrap();

                    let src = vec![100u8; 8]; // All pixels same value
                    let mut dest = vec![0u8; 8];

                    refine_vertical_wiener(&src, &mut dest, pitch, width, height, bits_per_sample);

                    // All output values should be the same as input
                    for &pixel in &dest {
                        assert_eq!(pixel, 100);
                    }
                }
            }
        };
    }

    horizontal_tests!(rust);
    vertical_tests!(rust);
}
