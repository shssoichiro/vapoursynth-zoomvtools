mod bicubic;
mod bilinear;
mod wiener;

use std::num::{NonZeroU8, NonZeroUsize};

pub use bicubic::{refine_horizontal_bicubic, refine_vertical_bicubic};
pub use bilinear::{
    refine_diagonal_bilinear, refine_horizontal_bilinear, refine_vertical_bilinear,
};
pub use wiener::{refine_horizontal_wiener, refine_vertical_wiener};

use crate::{mv_plane::MVPlane, pad::pad_reference_frame, util::Pixel};

/// Function pointer type for sub-pixel refinement functions.
///
/// This type alias defines the signature for all refinement functions that perform
/// sub-pixel interpolation for motion estimation. All refinement functions follow
/// this common interface for consistency and interchangeability.
pub type RefineFn<T> = fn(&[T], &mut [T], NonZeroUsize, NonZeroUsize, NonZeroUsize, NonZeroU8);

impl MVPlane {
    /// Refines motion vector plane to half-pixel precision using 2x upsampled reference.
    ///
    /// This method creates sub-pixel samples at half-pixel positions (0.5 horizontal,
    /// 0.5 vertical, and 0.5 diagonal) by extracting every other pixel from a 2x
    /// upsampled reference frame. This is used in hierarchical motion estimation
    /// where higher resolution references provide sub-pixel accuracy.
    ///
    /// The method populates three sub-pixel windows:
    /// - Window 1: (0.5, 0) - horizontal half-pixel positions
    /// - Window 2: (0, 0.5) - vertical half-pixel positions  
    /// - Window 3: (0.5, 0.5) - diagonal half-pixel positions
    ///
    /// # Parameters
    /// - `src_2x`: Source buffer containing 2x upsampled reference frame
    /// - `src_2x_pitch`: Number of pixels per row in the upsampled source
    /// - `is_ext_padded`: Whether external padding has already been applied
    /// - `dest`: Destination buffer to store sub-pixel samples
    pub fn refine_ext_pel2<T: Pixel>(
        &mut self,
        mut src_2x: &[T],
        src_2x_pitch: NonZeroUsize,
        is_ext_padded: bool,
        dest: &mut [T],
    ) {
        let mut p1 = self.subpel_window_offsets[1];
        let mut p2 = self.subpel_window_offsets[2];
        let mut p3 = self.subpel_window_offsets[3];

        // pel clip may be already padded (i.e. is finest clip)
        if !is_ext_padded {
            let offset = self.pitch.get() * self.vpad + self.hpad;
            p1 += offset;
            p2 += offset;
            p3 += offset;
        }

        for _h in 0..self.height.get() {
            for w in 0..self.width.get() {
                dest[p1 + w] = src_2x[(w << 1) + 1];
                dest[p2 + w] = src_2x[(w << 1) + src_2x_pitch.get()];
                dest[p3 + w] = src_2x[(w << 1) + src_2x_pitch.get() + 1];
            }
            p1 += self.pitch.get();
            p2 += self.pitch.get();
            p3 += self.pitch.get();
            src_2x = &src_2x[src_2x_pitch.get() * 2..];
        }

        if !is_ext_padded {
            for i in 1..4 {
                pad_reference_frame(
                    self.subpel_window_offsets[i],
                    self.pitch,
                    self.hpad,
                    self.vpad,
                    self.width,
                    self.height,
                    dest,
                );
            }
        }
        self.is_padded = true;
    }

    /// Refines motion vector plane to quarter-pixel precision using 4x upsampled reference.
    ///
    /// This method creates sub-pixel samples at quarter-pixel positions by extracting
    /// appropriately spaced pixels from a 4x upsampled reference frame. This provides
    /// the highest sub-pixel precision for motion estimation, creating 15 additional
    /// sub-pixel windows beyond the original integer positions.
    ///
    /// The method populates 15 sub-pixel windows covering all quarter-pixel positions:
    /// - (0.25, 0), (0.5, 0), (0.75, 0) - horizontal quarter-pixel positions
    /// - (0, 0.25), (0, 0.5), (0, 0.75) - vertical quarter-pixel positions
    /// - All diagonal combinations of the above positions
    ///
    /// # Parameters
    /// - `src_2x`: Source buffer containing 4x upsampled reference frame
    /// - `src_2x_pitch`: Number of pixels per row in the upsampled source
    /// - `is_ext_padded`: Whether external padding has already been applied
    /// - `dest`: Destination buffer to store sub-pixel samples
    pub fn refine_ext_pel4<T: Pixel>(
        &mut self,
        mut src_2x: &[T],
        src_2x_pitch: NonZeroUsize,
        is_ext_padded: bool,
        dest: &mut [T],
    ) {
        let mut pp = [0; 16];
        for (ppi, offset) in pp
            .iter_mut()
            .zip(self.subpel_window_offsets.iter_mut())
            .take(16)
            .skip(1)
        {
            *ppi = *offset;
        }

        // pel clip may be already padded (i.e. is finest clip)
        if !is_ext_padded {
            let offset = self.pitch.get() * self.vpad + self.hpad;
            for ppi in pp[1..16].iter_mut() {
                *ppi += offset;
            }
        }

        for _h in 0..self.height.get() {
            for w in 0..self.width.get() {
                dest[pp[1] + w] = src_2x[(w << 2) + 1];
                dest[pp[2] + w] = src_2x[(w << 2) + 2];
                dest[pp[3] + w] = src_2x[(w << 2) + 3];
                dest[pp[4] + w] = src_2x[(w << 2) + src_2x_pitch.get()];
                dest[pp[5] + w] = src_2x[(w << 2) + src_2x_pitch.get() + 1];
                dest[pp[6] + w] = src_2x[(w << 2) + src_2x_pitch.get() + 2];
                dest[pp[7] + w] = src_2x[(w << 2) + src_2x_pitch.get() + 3];
                dest[pp[8] + w] = src_2x[(w << 2) + src_2x_pitch.get() * 2];
                dest[pp[9] + w] = src_2x[(w << 2) + src_2x_pitch.get() * 2 + 1];
                dest[pp[10] + w] = src_2x[(w << 2) + src_2x_pitch.get() * 2 + 2];
                dest[pp[11] + w] = src_2x[(w << 2) + src_2x_pitch.get() * 2 + 3];
                dest[pp[12] + w] = src_2x[(w << 2) + src_2x_pitch.get() * 3];
                dest[pp[13] + w] = src_2x[(w << 2) + src_2x_pitch.get() * 3 + 1];
                dest[pp[14] + w] = src_2x[(w << 2) + src_2x_pitch.get() * 3 + 2];
                dest[pp[15] + w] = src_2x[(w << 2) + src_2x_pitch.get() * 3 + 3];
            }
            for ppi in pp[1..16].iter_mut() {
                *ppi += self.pitch.get();
            }
            src_2x = &src_2x[src_2x_pitch.get() * 4..];
        }

        if !is_ext_padded {
            for i in 1..16 {
                pad_reference_frame(
                    self.subpel_window_offsets[i],
                    self.pitch,
                    self.hpad,
                    self.vpad,
                    self.width,
                    self.height,
                    dest,
                );
            }
        }
        self.is_padded = true;
    }
}

#[cfg(test)]
mod tests {
    use std::num::{NonZeroU8, NonZeroUsize};

    use crate::{mv_plane::MVPlane, params::Subpel};

    /// Helper function to create an MVPlane for testing
    fn create_test_mv_plane(
        width: usize,
        height: usize,
        pel: Subpel,
        hpad: usize,
        vpad: usize,
        plane_offset: usize,
    ) -> MVPlane {
        let pitch = width + 2 * hpad;
        MVPlane::new(
            NonZeroUsize::new(width).unwrap(),
            NonZeroUsize::new(height).unwrap(),
            pel,
            hpad,
            vpad,
            NonZeroU8::new(8).unwrap(), // 8 bits per sample
            plane_offset,
            NonZeroUsize::new(pitch).unwrap(),
        )
        .unwrap()
    }

    /// Helper function to create source data for 2x upsampled frame
    fn create_2x_upsampled_frame<T: Copy + From<u8>>(
        width: usize,
        height: usize,
        pitch: usize,
    ) -> Vec<T> {
        let mut frame = vec![T::from(0); pitch * height];

        // Fill with a pattern where each 2x2 block has different values
        // This helps us verify the interpolation is working correctly
        for y in 0..height {
            for x in 0..width {
                let block_x = x / 2;
                let block_y = y / 2;
                let sub_x = x % 2;
                let sub_y = y % 2;

                let base_value = ((block_y * (width / 2) + block_x) % 256) as u8;
                let offset = (sub_y * 2 + sub_x) * 10;
                let value = base_value.saturating_add(offset as u8);

                frame[y * pitch + x] = T::from(value);
            }
        }

        frame
    }

    /// Helper function to create destination buffer with sufficient space for all
    /// subpel windows
    fn create_dest_buffer<T: Copy + From<u8>>(plane: &MVPlane, total_windows: usize) -> Vec<T> {
        let total_size = plane
            .subpel_window_offsets
            .get(total_windows - 1)
            .copied()
            .unwrap_or(0)
            + (plane.height.get() + 2 * plane.vpad) * plane.pitch.get();
        vec![T::from(0); total_size]
    }

    /// Verify that the interpolated values are correctly placed in subpel windows
    fn verify_pel2_interpolation<T: Copy + PartialEq + std::fmt::Debug + From<u8>>(
        plane: &MVPlane,
        src_2x: &[T],
        src_2x_pitch: usize,
        dest: &[T],
        is_ext_padded: bool,
    ) {
        // Calculate base offsets for each subpel window
        let mut p1 = plane.subpel_window_offsets[1];
        let mut p2 = plane.subpel_window_offsets[2];
        let mut p3 = plane.subpel_window_offsets[3];

        if !is_ext_padded {
            let offset = plane.pitch.get() * plane.vpad + plane.hpad;
            p1 += offset;
            p2 += offset;
            p3 += offset;
        }

        // Check each row
        for h in 0..plane.height.get() {
            let src_row_base = h * 2 * src_2x_pitch;

            for w in 0..plane.width.get() {
                let src_col_base = w * 2;

                // Verify subpel window 1: (0.5, 0) - horizontally interpolated
                let expected_p1 = src_2x[src_row_base + src_col_base + 1];
                assert_eq!(
                    dest[p1 + w],
                    expected_p1,
                    "Subpel window 1 mismatch at ({}, {})",
                    w,
                    h
                );

                // Verify subpel window 2: (0, 0.5) - vertically interpolated
                let expected_p2 = src_2x[src_row_base + src_2x_pitch + src_col_base];
                assert_eq!(
                    dest[p2 + w],
                    expected_p2,
                    "Subpel window 2 mismatch at ({}, {})",
                    w,
                    h
                );

                // Verify subpel window 3: (0.5, 0.5) - diagonally interpolated
                let expected_p3 = src_2x[src_row_base + src_2x_pitch + src_col_base + 1];
                assert_eq!(
                    dest[p3 + w],
                    expected_p3,
                    "Subpel window 3 mismatch at ({}, {})",
                    w,
                    h
                );
            }

            // Move to next row
            p1 += plane.pitch.get();
            p2 += plane.pitch.get();
            p3 += plane.pitch.get();
        }
    }

    /// Verify that the interpolated values are correctly placed in subpel windows
    /// for pel4
    fn verify_pel4_interpolation<T: Copy + PartialEq + std::fmt::Debug + From<u8>>(
        plane: &MVPlane,
        src_2x: &[T],
        src_2x_pitch: usize,
        dest: &[T],
        is_ext_padded: bool,
    ) {
        let mut pp = [0; 16];
        for (ppi, offset) in pp
            .iter_mut()
            .zip(plane.subpel_window_offsets.iter())
            .take(16)
            .skip(1)
        {
            *ppi = *offset;
        }

        if !is_ext_padded {
            let offset = plane.pitch.get() * plane.vpad + plane.hpad;
            for ppi in pp[1..16].iter_mut() {
                *ppi += offset;
            }
        }

        // Check each row
        for h in 0..plane.height.get() {
            let src_row_base = h * 4 * src_2x_pitch;

            for w in 0..plane.width.get() {
                let src_col_base = w * 4;

                // Verify all 15 subpel windows (skip window 0 which is the original)
                let expected_values = [
                    src_2x[src_row_base + src_col_base + 1],            // pp[1]
                    src_2x[src_row_base + src_col_base + 2],            // pp[2]
                    src_2x[src_row_base + src_col_base + 3],            // pp[3]
                    src_2x[src_row_base + src_2x_pitch + src_col_base], // pp[4]
                    src_2x[src_row_base + src_2x_pitch + src_col_base + 1], // pp[5]
                    src_2x[src_row_base + src_2x_pitch + src_col_base + 2], // pp[6]
                    src_2x[src_row_base + src_2x_pitch + src_col_base + 3], // pp[7]
                    src_2x[src_row_base + src_2x_pitch * 2 + src_col_base], // pp[8]
                    src_2x[src_row_base + src_2x_pitch * 2 + src_col_base + 1], // pp[9]
                    src_2x[src_row_base + src_2x_pitch * 2 + src_col_base + 2], // pp[10]
                    src_2x[src_row_base + src_2x_pitch * 2 + src_col_base + 3], // pp[11]
                    src_2x[src_row_base + src_2x_pitch * 3 + src_col_base], // pp[12]
                    src_2x[src_row_base + src_2x_pitch * 3 + src_col_base + 1], // pp[13]
                    src_2x[src_row_base + src_2x_pitch * 3 + src_col_base + 2], // pp[14]
                    src_2x[src_row_base + src_2x_pitch * 3 + src_col_base + 3], // pp[15]
                ];

                for i in 1..16 {
                    assert_eq!(
                        dest[pp[i] + w],
                        expected_values[i - 1],
                        "Subpel window {} mismatch at ({}, {})",
                        i,
                        w,
                        h
                    );
                }
            }

            // Move to next row
            for ppi in pp[1..16].iter_mut() {
                *ppi += plane.pitch.get();
            }
        }
    }

    #[test]
    fn test_refine_ext_pel2_u8_basic() {
        let width = 4;
        let height = 4;
        let hpad = 2;
        let vpad = 2;

        let mut plane = create_test_mv_plane(width, height, Subpel::Half, hpad, vpad, 0);

        // Create 2x upsampled source (8x8 for 4x4 dest)
        let src_2x_width = width * 2;
        let src_2x_height = height * 2;
        let src_2x_pitch = src_2x_width;
        let src_2x = create_2x_upsampled_frame::<u8>(src_2x_width, src_2x_height, src_2x_pitch);

        let mut dest = create_dest_buffer::<u8>(&plane, 4);

        plane.refine_ext_pel2(
            &src_2x,
            NonZeroUsize::new(src_2x_pitch).unwrap(),
            true, // is_ext_padded
            &mut dest,
        );

        verify_pel2_interpolation(&plane, &src_2x, src_2x_pitch, &dest, true);
        assert!(plane.is_padded);
    }

    #[test]
    fn test_refine_ext_pel2_u16_basic() {
        let width = 3;
        let height = 3;
        let hpad = 1;
        let vpad = 1;

        let mut plane = create_test_mv_plane(width, height, Subpel::Half, hpad, vpad, 0);

        // Create 2x upsampled source (6x6 for 3x3 dest)
        let src_2x_width = width * 2;
        let src_2x_height = height * 2;
        let src_2x_pitch = src_2x_width;
        let src_2x = create_2x_upsampled_frame::<u16>(src_2x_width, src_2x_height, src_2x_pitch);

        let mut dest = create_dest_buffer::<u16>(&plane, 4);

        plane.refine_ext_pel2(
            &src_2x,
            NonZeroUsize::new(src_2x_pitch).unwrap(),
            true, // is_ext_padded
            &mut dest,
        );

        verify_pel2_interpolation(&plane, &src_2x, src_2x_pitch, &dest, true);
        assert!(plane.is_padded);
    }

    #[test]
    fn test_refine_ext_pel2_not_padded() {
        let width = 2;
        let height = 2;
        let hpad = 1;
        let vpad = 1;

        let mut plane = create_test_mv_plane(width, height, Subpel::Half, hpad, vpad, 0);

        let src_2x_width = width * 2;
        let src_2x_height = height * 2;
        let src_2x_pitch = src_2x_width;
        let src_2x = create_2x_upsampled_frame::<u8>(src_2x_width, src_2x_height, src_2x_pitch);

        let mut dest = create_dest_buffer::<u8>(&plane, 4);

        plane.refine_ext_pel2(
            &src_2x,
            NonZeroUsize::new(src_2x_pitch).unwrap(),
            false, // is_ext_padded = false, will add padding
            &mut dest,
        );

        verify_pel2_interpolation(&plane, &src_2x, src_2x_pitch, &dest, false);
        assert!(plane.is_padded);
    }

    #[test]
    fn test_refine_ext_pel4_u8_basic() {
        let width = 2;
        let height = 2;
        let hpad = 1;
        let vpad = 1;

        let mut plane = create_test_mv_plane(width, height, Subpel::Quarter, hpad, vpad, 0);

        // Create 4x upsampled source (8x8 for 2x2 dest)
        let src_2x_width = width * 4;
        let src_2x_height = height * 4;
        let src_2x_pitch = src_2x_width;
        let src_2x = create_2x_upsampled_frame::<u8>(src_2x_width, src_2x_height, src_2x_pitch);

        let mut dest = create_dest_buffer::<u8>(&plane, 16);

        plane.refine_ext_pel4(
            &src_2x,
            NonZeroUsize::new(src_2x_pitch).unwrap(),
            true, // is_ext_padded
            &mut dest,
        );

        verify_pel4_interpolation(&plane, &src_2x, src_2x_pitch, &dest, true);
        assert!(plane.is_padded);
    }

    #[test]
    fn test_refine_ext_pel4_u16_basic() {
        let width = 3;
        let height = 2;
        let hpad = 2;
        let vpad = 1;

        let mut plane = create_test_mv_plane(width, height, Subpel::Quarter, hpad, vpad, 0);

        // Create 4x upsampled source (12x8 for 3x2 dest)
        let src_2x_width = width * 4;
        let src_2x_height = height * 4;
        let src_2x_pitch = src_2x_width;
        let src_2x = create_2x_upsampled_frame::<u16>(src_2x_width, src_2x_height, src_2x_pitch);

        let mut dest = create_dest_buffer::<u16>(&plane, 16);

        plane.refine_ext_pel4(
            &src_2x,
            NonZeroUsize::new(src_2x_pitch).unwrap(),
            true, // is_ext_padded
            &mut dest,
        );

        verify_pel4_interpolation(&plane, &src_2x, src_2x_pitch, &dest, true);
        assert!(plane.is_padded);
    }

    #[test]
    fn test_refine_ext_pel4_not_padded() {
        let width = 2;
        let height = 1;
        let hpad = 1;
        let vpad = 1;

        let mut plane = create_test_mv_plane(width, height, Subpel::Quarter, hpad, vpad, 0);

        let src_2x_width = width * 4;
        let src_2x_height = height * 4;
        let src_2x_pitch = src_2x_width;
        let src_2x = create_2x_upsampled_frame::<u8>(src_2x_width, src_2x_height, src_2x_pitch);

        let mut dest = create_dest_buffer::<u8>(&plane, 16);

        plane.refine_ext_pel4(
            &src_2x,
            NonZeroUsize::new(src_2x_pitch).unwrap(),
            false, // is_ext_padded = false, will add padding
            &mut dest,
        );

        verify_pel4_interpolation(&plane, &src_2x, src_2x_pitch, &dest, false);
        assert!(plane.is_padded);
    }

    #[test]
    fn test_refine_ext_pel2_larger_frame() {
        let width = 8;
        let height = 6;
        let hpad = 4;
        let vpad = 3;

        let mut plane = create_test_mv_plane(width, height, Subpel::Half, hpad, vpad, 0);

        let src_2x_width = width * 2;
        let src_2x_height = height * 2;
        let src_2x_pitch = src_2x_width;
        let src_2x = create_2x_upsampled_frame::<u8>(src_2x_width, src_2x_height, src_2x_pitch);

        let mut dest = create_dest_buffer::<u8>(&plane, 4);

        plane.refine_ext_pel2(
            &src_2x,
            NonZeroUsize::new(src_2x_pitch).unwrap(),
            true,
            &mut dest,
        );

        verify_pel2_interpolation(&plane, &src_2x, src_2x_pitch, &dest, true);
        assert!(plane.is_padded);
    }

    #[test]
    fn test_refine_ext_pel4_larger_frame() {
        let width = 4;
        let height = 3;
        let hpad = 2;
        let vpad = 2;

        let mut plane = create_test_mv_plane(width, height, Subpel::Quarter, hpad, vpad, 0);

        let src_2x_width = width * 4;
        let src_2x_height = height * 4;
        let src_2x_pitch = src_2x_width;
        let src_2x = create_2x_upsampled_frame::<u16>(src_2x_width, src_2x_height, src_2x_pitch);

        let mut dest = create_dest_buffer::<u16>(&plane, 16);

        plane.refine_ext_pel4(
            &src_2x,
            NonZeroUsize::new(src_2x_pitch).unwrap(),
            true,
            &mut dest,
        );

        verify_pel4_interpolation(&plane, &src_2x, src_2x_pitch, &dest, true);
        assert!(plane.is_padded);
    }

    #[test]
    fn test_refine_ext_pel2_minimal_size() {
        let width = 1;
        let height = 1;
        let hpad = 1;
        let vpad = 1;

        let mut plane = create_test_mv_plane(width, height, Subpel::Half, hpad, vpad, 0);

        let src_2x_width = width * 2;
        let src_2x_height = height * 2;
        let src_2x_pitch = src_2x_width;
        let mut src_2x = vec![0u8; src_2x_pitch * src_2x_height];

        // Set specific values for the 2x2 block
        src_2x[0] = 10; // (0,0)
        src_2x[1] = 20; // (1,0)
        src_2x[src_2x_pitch] = 30; // (0,1)
        src_2x[src_2x_pitch + 1] = 40; // (1,1)

        let mut dest = create_dest_buffer::<u8>(&plane, 4);

        plane.refine_ext_pel2(
            &src_2x,
            NonZeroUsize::new(src_2x_pitch).unwrap(),
            true,
            &mut dest,
        );

        verify_pel2_interpolation(&plane, &src_2x, src_2x_pitch, &dest, true);
        assert!(plane.is_padded);
    }

    #[test]
    fn test_refine_ext_pel4_minimal_size() {
        let width = 1;
        let height = 1;
        let hpad = 1;
        let vpad = 1;

        let mut plane = create_test_mv_plane(width, height, Subpel::Quarter, hpad, vpad, 0);

        let src_2x_width = width * 4;
        let src_2x_height = height * 4;
        let src_2x_pitch = src_2x_width;
        let mut src_2x = vec![0u8; src_2x_pitch * src_2x_height];

        // Set specific values for the 4x4 block
        for i in 0..16 {
            let y = i / 4;
            let x = i % 4;
            src_2x[y * src_2x_pitch + x] = ((i + 1) * 10) as u8;
        }

        let mut dest = create_dest_buffer::<u8>(&plane, 16);

        plane.refine_ext_pel4(
            &src_2x,
            NonZeroUsize::new(src_2x_pitch).unwrap(),
            true,
            &mut dest,
        );

        verify_pel4_interpolation(&plane, &src_2x, src_2x_pitch, &dest, true);
        assert!(plane.is_padded);
    }

    #[test]
    fn test_refine_ext_pel2_with_plane_offset() {
        let width = 2;
        let height = 2;
        let hpad = 1;
        let vpad = 1;
        let plane_offset = 100; // Non-zero offset

        let mut plane = create_test_mv_plane(width, height, Subpel::Half, hpad, vpad, plane_offset);

        let src_2x_width = width * 2;
        let src_2x_height = height * 2;
        let src_2x_pitch = src_2x_width;
        let src_2x = create_2x_upsampled_frame::<u8>(src_2x_width, src_2x_height, src_2x_pitch);

        let mut dest = create_dest_buffer::<u8>(&plane, 4);

        plane.refine_ext_pel2(
            &src_2x,
            NonZeroUsize::new(src_2x_pitch).unwrap(),
            true,
            &mut dest,
        );

        verify_pel2_interpolation(&plane, &src_2x, src_2x_pitch, &dest, true);
        assert!(plane.is_padded);
    }

    #[test]
    fn test_refine_ext_pel4_with_plane_offset() {
        let width = 2;
        let height = 1;
        let hpad = 1;
        let vpad = 1;
        let plane_offset = 200; // Non-zero offset

        let mut plane =
            create_test_mv_plane(width, height, Subpel::Quarter, hpad, vpad, plane_offset);

        let src_2x_width = width * 4;
        let src_2x_height = height * 4;
        let src_2x_pitch = src_2x_width;
        let src_2x = create_2x_upsampled_frame::<u8>(src_2x_width, src_2x_height, src_2x_pitch);

        let mut dest = create_dest_buffer::<u8>(&plane, 16);

        plane.refine_ext_pel4(
            &src_2x,
            NonZeroUsize::new(src_2x_pitch).unwrap(),
            true,
            &mut dest,
        );

        verify_pel4_interpolation(&plane, &src_2x, src_2x_pitch, &dest, true);
        assert!(plane.is_padded);
    }

    #[test]
    fn test_refine_ext_pel2_different_pitch() {
        let width = 4;
        let height = 2;
        let hpad = 2;
        let vpad = 1;

        let mut plane = create_test_mv_plane(width, height, Subpel::Half, hpad, vpad, 0);

        let src_2x_width = width * 2;
        let src_2x_height = height * 2;
        let src_2x_pitch = src_2x_width + 4; // Different pitch (with extra bytes)
        let mut src_2x = vec![0u8; src_2x_pitch * src_2x_height];

        // Fill only the used portion of each row
        for y in 0..src_2x_height {
            for x in 0..src_2x_width {
                let value = ((y * src_2x_width + x) % 256) as u8;
                src_2x[y * src_2x_pitch + x] = value;
            }
        }

        let mut dest = create_dest_buffer::<u8>(&plane, 4);

        plane.refine_ext_pel2(
            &src_2x,
            NonZeroUsize::new(src_2x_pitch).unwrap(),
            true,
            &mut dest,
        );

        verify_pel2_interpolation(&plane, &src_2x, src_2x_pitch, &dest, true);
        assert!(plane.is_padded);
    }

    #[test]
    fn test_refine_ext_pel4_different_pitch() {
        let width = 2;
        let height = 2;
        let hpad = 1;
        let vpad = 1;

        let mut plane = create_test_mv_plane(width, height, Subpel::Quarter, hpad, vpad, 0);

        let src_2x_width = width * 4;
        let src_2x_height = height * 4;
        let src_2x_pitch = src_2x_width + 2; // Different pitch (with extra bytes)
        let mut src_2x = vec![0u16; src_2x_pitch * src_2x_height];

        // Fill only the used portion of each row
        for y in 0..src_2x_height {
            for x in 0..src_2x_width {
                let value = ((y * src_2x_width + x) % 1000) as u16;
                src_2x[y * src_2x_pitch + x] = value;
            }
        }

        let mut dest = create_dest_buffer::<u16>(&plane, 16);

        plane.refine_ext_pel4(
            &src_2x,
            NonZeroUsize::new(src_2x_pitch).unwrap(),
            true,
            &mut dest,
        );

        verify_pel4_interpolation(&plane, &src_2x, src_2x_pitch, &dest, true);
        assert!(plane.is_padded);
    }
}
