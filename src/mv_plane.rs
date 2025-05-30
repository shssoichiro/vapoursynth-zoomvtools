use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::Result;
use bitflags::bitflags;
use smallvec::SmallVec;

use crate::{
    average::average2,
    pad::pad_reference_frame,
    params::{ReduceFilter, Subpel, SubpelMethod},
    reduce::{
        ReduceFn, reduce_average, reduce_bilinear, reduce_cubic, reduce_quadratic, reduce_triangle,
    },
    refine::{
        RefineFn, refine_diagonal_bilinear, refine_horizontal_bicubic, refine_horizontal_bilinear,
        refine_horizontal_wiener, refine_vertical_bicubic, refine_vertical_bilinear,
        refine_vertical_wiener,
    },
    util::{Pixel, vs_bitblt},
};

#[derive(Debug, Clone)]
pub struct MVPlane {
    pub subpel_window_offsets: SmallVec<[usize; 16]>,
    pub width: NonZeroUsize,
    pub height: NonZeroUsize,
    pub padded_width: NonZeroUsize,
    pub padded_height: NonZeroUsize,
    pub pitch: NonZeroUsize,
    pub hpad: usize,
    pub vpad: usize,
    pub offset_padding: usize,
    pub hpad_pel: usize,
    pub vpad_pel: usize,
    pub bits_per_sample: NonZeroU8,
    pub bytes_per_sample: NonZeroU8,
    pub pel: Subpel,
    pub is_padded: bool,
    pub is_refined: bool,
    pub is_filled: bool,
}

impl MVPlane {
    pub fn new(
        width: NonZeroUsize,
        height: NonZeroUsize,
        pel: Subpel,
        hpad: usize,
        vpad: usize,
        bits_per_sample: NonZeroU8,
        plane_offset: usize,
        pitch: NonZeroUsize,
    ) -> Result<Self> {
        let pel_val = usize::from(pel);
        let padded_width = width.saturating_add(2 * hpad);
        let padded_height = height.saturating_add(2 * vpad);
        let bytes_per_sample = NonZeroU8::try_from(bits_per_sample.saturating_add(7).get() / 8)?;
        let offset_padding = pitch.get() * vpad + hpad * bytes_per_sample.get() as usize;

        let windows = pel_val * pel_val;
        let mut offsets = SmallVec::with_capacity(windows);
        for i in 0..windows {
            let offset = i * pitch.get() * padded_height.get();
            offsets.push(plane_offset + offset);
        }

        Ok(Self {
            width,
            height,
            padded_width,
            padded_height,
            hpad,
            vpad,
            hpad_pel: hpad * pel_val,
            vpad_pel: vpad * pel_val,
            subpel_window_offsets: offsets,
            offset_padding,
            pitch,
            bits_per_sample,
            bytes_per_sample,
            pel,
            is_padded: false,
            is_refined: false,
            is_filled: false,
        })
    }

    pub fn fill_plane<T: Pixel>(&mut self, src: &[T], src_pitch: NonZeroUsize, dest: &mut [T]) {
        if self.is_filled {
            return;
        }

        let offset = self.subpel_window_offsets[0] + self.offset_padding;
        vs_bitblt(
            &mut dest[offset..],
            self.pitch,
            src,
            src_pitch,
            self.width,
            self.height,
        );

        self.is_filled = true;
    }

    pub fn refine_ext<T: Pixel>(
        &mut self,
        src_2x: &[T],
        src_2x_pitch: NonZeroUsize,
        is_ext_padded: bool,
        dest: &mut [T],
    ) {
        if !self.is_refined {
            match self.pel {
                Subpel::Full => {
                    // No refinement needed
                }
                Subpel::Half => {
                    self.refine_ext_pel2(src_2x, src_2x_pitch, is_ext_padded, dest);
                }
                Subpel::Quarter => {
                    self.refine_ext_pel4(src_2x, src_2x_pitch, is_ext_padded, dest);
                }
            }
        }
        self.is_refined = true;
    }

    pub fn reduce_to<T: Pixel>(
        &self,
        reduced_plane: &mut MVPlane,
        filter: ReduceFilter,
        dest: &mut [T],
        src: &[T],
        dest_pitch: NonZeroUsize,
        src_pitch: NonZeroUsize,
        dest_width: NonZeroUsize,
        dest_height: NonZeroUsize,
    ) {
        if self.is_filled {
            return;
        }

        let dest =
            &mut dest[reduced_plane.subpel_window_offsets[0] + reduced_plane.offset_padding..];
        let src = &src[self.subpel_window_offsets[0] + self.offset_padding..];
        let reduce: ReduceFn<T> = match filter {
            ReduceFilter::Average => reduce_average,
            ReduceFilter::Triangle => reduce_triangle,
            ReduceFilter::Bilinear => reduce_bilinear,
            ReduceFilter::Quadratic => reduce_quadratic,
            ReduceFilter::Cubic => reduce_cubic,
        };

        reduce(dest, src, dest_pitch, src_pitch, dest_width, dest_height);

        reduced_plane.is_filled = true;
    }

    pub fn pad<T: Pixel>(&mut self, src: &mut [T]) {
        if !self.is_padded {
            pad_reference_frame(
                self.subpel_window_offsets[0],
                self.pitch,
                self.hpad,
                self.vpad,
                self.width,
                self.height,
                src,
            );
            self.is_padded = true;
        }
    }

    pub fn refine<T: Pixel>(&mut self, subpel: SubpelMethod, plane: &mut [T]) {
        if self.is_refined {
            return;
        }

        if self.pel == Subpel::Full {
            self.is_refined = true;
            return;
        }

        let refine: [RefineFn<T>; 3] = match subpel {
            SubpelMethod::Bilinear => [
                refine_horizontal_bilinear,
                refine_vertical_bilinear,
                refine_diagonal_bilinear,
            ],
            SubpelMethod::Bicubic => [
                refine_horizontal_bicubic,
                refine_vertical_bicubic,
                refine_horizontal_bicubic,
            ],
            SubpelMethod::Wiener => [
                refine_horizontal_wiener,
                refine_vertical_wiener,
                refine_horizontal_wiener,
            ],
        };

        let mut src_offsets = [0; 3];
        let mut dest_offsets = [0; 3];
        match self.pel {
            Subpel::Full => unreachable!(),
            Subpel::Half => {
                dest_offsets[0] = self.subpel_window_offsets[1];
                dest_offsets[1] = self.subpel_window_offsets[2];
                dest_offsets[2] = self.subpel_window_offsets[3];
                src_offsets[0] = self.subpel_window_offsets[0];
                src_offsets[1] = self.subpel_window_offsets[0];
                if subpel == SubpelMethod::Bilinear {
                    src_offsets[2] = self.subpel_window_offsets[0];
                } else {
                    src_offsets[2] = self.subpel_window_offsets[2];
                }
            }
            Subpel::Quarter => {
                dest_offsets[0] = self.subpel_window_offsets[2];
                dest_offsets[1] = self.subpel_window_offsets[8];
                dest_offsets[2] = self.subpel_window_offsets[10];
                src_offsets[0] = self.subpel_window_offsets[0];
                src_offsets[1] = self.subpel_window_offsets[0];
                if subpel == SubpelMethod::Bilinear {
                    src_offsets[2] = self.subpel_window_offsets[0];
                } else {
                    src_offsets[2] = self.subpel_window_offsets[8];
                }
            }
        }

        // FIXME: Avoid these clones
        for i in 0..3 {
            refine[i](
                &plane[src_offsets[i]..].to_vec(),
                &mut plane[dest_offsets[i]..],
                self.pitch,
                self.padded_width,
                self.padded_height,
                self.bits_per_sample,
            );
        }

        // FIXME: Avoid all of these clones
        if self.pel == Subpel::Quarter {
            average2(
                &plane[self.subpel_window_offsets[0]..].to_vec(),
                &plane[self.subpel_window_offsets[2]..].to_vec(),
                &mut plane[self.subpel_window_offsets[1]..],
                self.pitch,
                self.padded_width,
                self.padded_height,
            );
            average2(
                &plane[self.subpel_window_offsets[8]..].to_vec(),
                &plane[self.subpel_window_offsets[10]..].to_vec(),
                &mut plane[self.subpel_window_offsets[9]..],
                self.pitch,
                self.padded_width,
                self.padded_height,
            );
            average2(
                &plane[self.subpel_window_offsets[0]..].to_vec(),
                &plane[self.subpel_window_offsets[8]..].to_vec(),
                &mut plane[self.subpel_window_offsets[4]..],
                self.pitch,
                self.padded_width,
                self.padded_height,
            );
            average2(
                &plane[self.subpel_window_offsets[2]..].to_vec(),
                &plane[self.subpel_window_offsets[10]..].to_vec(),
                &mut plane[self.subpel_window_offsets[6]..],
                self.pitch,
                self.padded_width,
                self.padded_height,
            );
            average2(
                &plane[self.subpel_window_offsets[4]..].to_vec(),
                &plane[self.subpel_window_offsets[6]..].to_vec(),
                &mut plane[self.subpel_window_offsets[5]..],
                self.pitch,
                self.padded_width,
                self.padded_height,
            );

            average2(
                &plane[self.subpel_window_offsets[0] + 1..].to_vec(),
                &plane[self.subpel_window_offsets[2]..].to_vec(),
                &mut plane[self.subpel_window_offsets[3]..],
                self.pitch,
                // SAFETY: Since we are doing qpel refinement, we know res is at least 4x4
                unsafe { NonZeroUsize::new_unchecked(self.padded_width.get() - 1) },
                self.padded_height,
            );
            average2(
                &plane[self.subpel_window_offsets[8] + 1..].to_vec(),
                &plane[self.subpel_window_offsets[10]..].to_vec(),
                &mut plane[self.subpel_window_offsets[11]..],
                self.pitch,
                // SAFETY: Since we are doing qpel refinement, we know res is at least 4x4
                unsafe { NonZeroUsize::new_unchecked(self.padded_width.get() - 1) },
                self.padded_height,
            );
            average2(
                &plane[self.subpel_window_offsets[0] + self.pitch.get()..].to_vec(),
                &plane[self.subpel_window_offsets[8]..].to_vec(),
                &mut plane[self.subpel_window_offsets[12]..],
                self.pitch,
                self.padded_width,
                // SAFETY: Since we are doing qpel refinement, we know res is at least 4x4
                unsafe { NonZeroUsize::new_unchecked(self.padded_height.get() - 1) },
            );
            average2(
                &plane[self.subpel_window_offsets[2] + self.pitch.get()..].to_vec(),
                &plane[self.subpel_window_offsets[10]..].to_vec(),
                &mut plane[self.subpel_window_offsets[14]..],
                self.pitch,
                self.padded_width,
                // SAFETY: Since we are doing qpel refinement, we know res is at least 4x4
                unsafe { NonZeroUsize::new_unchecked(self.padded_height.get() - 1) },
            );
            average2(
                &plane[self.subpel_window_offsets[12]..].to_vec(),
                &plane[self.subpel_window_offsets[14]..].to_vec(),
                &mut plane[self.subpel_window_offsets[13]..],
                self.pitch,
                self.padded_width,
                self.padded_height,
            );
            average2(
                &plane[self.subpel_window_offsets[4] + 1..].to_vec(),
                &plane[self.subpel_window_offsets[6]..].to_vec(),
                &mut plane[self.subpel_window_offsets[7]..],
                self.pitch,
                // SAFETY: Since we are doing qpel refinement, we know res is at least 4x4
                unsafe { NonZeroUsize::new_unchecked(self.padded_width.get() - 1) },
                self.padded_height,
            );
            average2(
                &plane[self.subpel_window_offsets[12] + 1..].to_vec(),
                &plane[self.subpel_window_offsets[14]..].to_vec(),
                &mut plane[self.subpel_window_offsets[15]..],
                self.pitch,
                // SAFETY: Since we are doing qpel refinement, we know res is at least 4x4
                unsafe { NonZeroUsize::new_unchecked(self.padded_width.get() - 1) },
                self.padded_height,
            );
        }

        self.is_refined = true;
    }
}

/// Calculates the height of a luma plane at a specific hierarchical level.
///
/// This function computes the height of a luma plane after downscaling through
/// multiple hierarchical levels in a motion estimation pyramid. At each level,
/// the height is divided by the UV ratio and then by 2, creating progressively
/// smaller reference frames for coarse-to-fine motion estimation.
///
/// The function accounts for chroma subsampling ratios and padding requirements
/// to ensure proper alignment at each level of the hierarchy.
///
/// # Parameters
/// - `src_height`: Original source image height
/// - `level`: Hierarchical level (0 = original size, higher = more downscaled)
/// - `y_ratio_uv`: Vertical chroma subsampling ratio (1 for 4:4:4, 2 for 4:2:0, etc.)
/// - `vpad`: Vertical padding amount in pixels
///
/// # Returns
/// The calculated height for the luma plane at the specified level
pub fn plane_height_luma(
    src_height: NonZeroUsize,
    level: u16,
    y_ratio_uv: NonZeroUsize,
    vpad: usize,
) -> NonZeroUsize {
    let mut height = src_height.get();
    let y_ratio_uv_val = y_ratio_uv.get();

    for _i in 1..=level {
        height = if vpad >= y_ratio_uv_val {
            (height / y_ratio_uv_val).div_ceil(2) * y_ratio_uv_val
        } else {
            ((height / y_ratio_uv_val) / 2) * y_ratio_uv_val
        };
    }

    // Call sites guarantee that the result will be non-zero
    debug_assert!(
        height > 0,
        "Calculated height must be non-zero. src_height: {}, level: {}, y_ratio_uv: {}, vpad: {}",
        src_height,
        level,
        y_ratio_uv,
        vpad
    );

    // SAFETY: Call sites enforce that this will never produce a 0 output height
    unsafe { NonZeroUsize::new_unchecked(height) }
}

/// Calculates the width of a luma plane at a specific hierarchical level.
///
/// This function computes the width of a luma plane after downscaling through
/// multiple hierarchical levels in a motion estimation pyramid. At each level,
/// the width is divided by the UV ratio and then by 2, creating progressively
/// smaller reference frames for coarse-to-fine motion estimation.
///
/// The function accounts for chroma subsampling ratios and padding requirements
/// to ensure proper alignment at each level of the hierarchy.
///
/// # Parameters
/// - `src_width`: Original source image width
/// - `level`: Hierarchical level (0 = original size, higher = more downscaled)
/// - `x_ratio_uv`: Horizontal chroma subsampling ratio (1 for 4:4:4, 2 for 4:2:0, etc.)
/// - `hpad`: Horizontal padding amount in pixels
///
/// # Returns
/// The calculated width for the luma plane at the specified level
pub fn plane_width_luma(
    src_width: NonZeroUsize,
    level: u16,
    x_ratio_uv: NonZeroUsize,
    hpad: usize,
) -> NonZeroUsize {
    let mut width = src_width.get();
    let x_ratio_uv_val = x_ratio_uv.get();

    for _i in 1..=level {
        width = if hpad >= x_ratio_uv_val {
            (width / x_ratio_uv_val).div_ceil(2) * x_ratio_uv_val
        } else {
            ((width / x_ratio_uv_val) / 2) * x_ratio_uv_val
        };
    }

    // Call sites guarantee that the result will be non-zero
    debug_assert!(
        width > 0,
        "Calculated width must be non-zero. src_width: {}, level: {}, x_ratio_uv: {}, hpad: {}",
        src_width,
        level,
        x_ratio_uv,
        hpad
    );

    // SAFETY: Call sites enforce that this will never produce a 0 output width
    unsafe { NonZeroUsize::new_unchecked(width) }
}

/// Calculates the memory offset for a plane within a hierarchical superframe structure.
///
/// This function computes the byte offset where a specific plane begins within
/// a superframe that contains multiple hierarchical levels and sub-pixel refinements.
/// Superframes store multiple downscaled versions of the same image along with
/// sub-pixel interpolated versions for efficient hierarchical motion estimation.
///
/// The offset calculation accounts for:
/// - Sub-pixel precision levels (pel parameter)
/// - Multiple hierarchical levels with different dimensions
/// - Chroma vs luma plane differences
/// - Padding requirements at each level
///
/// # Parameters
/// - `chroma`: Whether this is a chroma plane (affects subsampling calculations)
/// - `src_height`: Original source image height
/// - `level`: Target hierarchical level for the offset calculation
/// - `pel`: Sub-pixel precision level (1=integer, 2=half-pixel, 4=quarter-pixel)
/// - `vpad`: Vertical padding amount in pixels
/// - `plane_pitch`: Number of pixels per row in the plane buffer
/// - `y_ratio_uv`: Vertical chroma subsampling ratio
///
/// # Returns
/// The byte offset where the specified plane begins in the superframe
pub fn plane_super_offset(
    chroma: bool,
    src_height: NonZeroUsize,
    level: u16,
    pel: Subpel,
    vpad: usize,
    plane_pitch: NonZeroUsize,
    y_ratio_uv: NonZeroUsize,
) -> usize {
    // storing subplanes in superframes may be implemented by various ways
    let mut height; // luma or chroma

    let mut offset;

    if level == 0 {
        offset = 0;
    } else {
        let pel = usize::from(pel);
        let plane_pitch_val = plane_pitch.get();
        let src_height_val = src_height.get();
        let y_ratio_uv_val = y_ratio_uv.get();
        offset = pel * pel * plane_pitch_val * (src_height_val + vpad * 2);

        for i in 1..level {
            // NOTE: We use `src_height` here (not a running `height` variable) because
            // plane_height_luma internally handles the hierarchical scaling by applying
            // the division `level` times in its own loop. Each call calculates the height
            // at the specific level `i` starting from the original source dimensions.
            height = if chroma {
                plane_height_luma(
                    src_height.saturating_mul(y_ratio_uv),
                    i,
                    y_ratio_uv,
                    vpad * y_ratio_uv_val,
                )
                .get()
                    / y_ratio_uv_val
            } else {
                plane_height_luma(src_height, i, y_ratio_uv, vpad).get()
            };

            offset += plane_pitch_val * (height + vpad * 2);
        }
    }

    offset
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::Subpel;
    use std::num::NonZeroUsize;

    #[test]
    fn test_plane_height_luma_level_0() {
        let src_height = NonZeroUsize::new(100).unwrap();
        let y_ratio_uv = NonZeroUsize::new(2).unwrap();
        let vpad = 8;

        let result = plane_height_luma(src_height, 0, y_ratio_uv, vpad);

        // Level 0 should return original height
        assert_eq!(result.get(), 100);
    }

    #[test]
    fn test_plane_height_luma_downscaling() {
        let src_height = NonZeroUsize::new(200).unwrap();
        let y_ratio_uv = NonZeroUsize::new(2).unwrap();
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
        let y_ratio_uv = NonZeroUsize::new(2).unwrap();
        let vpad = 8; // vpad >= y_ratio_uv

        // Level 1: height = (200 / 2).div_ceil(2) * 2 = 100.div_ceil(2) * 2 = 50 * 2 = 100
        let result = plane_height_luma(src_height, 1, y_ratio_uv, vpad);
        assert_eq!(result.get(), 100);
    }

    #[test]
    fn test_plane_height_luma_multiple_levels() {
        let src_height = NonZeroUsize::new(1600).unwrap();
        let y_ratio_uv = NonZeroUsize::new(1).unwrap(); // 4:4:4 format
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
        let y_ratio_uv = NonZeroUsize::new(2).unwrap(); // 4:2:0 format
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
        let x_ratio_uv = NonZeroUsize::new(2).unwrap();
        let hpad = 8;

        let result = plane_width_luma(src_width, 0, x_ratio_uv, hpad);

        // Level 0 should return original width
        assert_eq!(result.get(), 100);
    }

    #[test]
    fn test_plane_width_luma_downscaling() {
        let src_width = NonZeroUsize::new(400).unwrap();
        let x_ratio_uv = NonZeroUsize::new(2).unwrap();
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
        let x_ratio_uv = NonZeroUsize::new(2).unwrap();
        let hpad = 4; // hpad >= x_ratio_uv

        // Level 1: width = (400 / 2).div_ceil(2) * 2 = 200.div_ceil(2) * 2 = 100 * 2 = 200
        let result = plane_width_luma(src_width, 1, x_ratio_uv, hpad);
        assert_eq!(result.get(), 200);
    }

    #[test]
    fn test_plane_width_luma_4_4_4_format() {
        let src_width = NonZeroUsize::new(1920).unwrap();
        let x_ratio_uv = NonZeroUsize::new(1).unwrap(); // 4:4:4 format
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
        let x_ratio_uv = NonZeroUsize::new(2).unwrap(); // 4:2:0 format
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
        let y_ratio_uv = NonZeroUsize::new(2).unwrap();
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
        let y_ratio_uv = NonZeroUsize::new(2).unwrap();
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
        let y_ratio_uv = NonZeroUsize::new(2).unwrap();
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
        let y_ratio_uv = NonZeroUsize::new(2).unwrap();
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
        let y_ratio_uv = NonZeroUsize::new(2).unwrap();
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
        let y_ratio_uv = NonZeroUsize::new(2).unwrap();
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
        let y_ratio_uv = NonZeroUsize::new(2).unwrap();
        let vpad = 4;
        let pel = Subpel::Full;

        let offset_luma =
            plane_super_offset(false, src_height, 2, pel, vpad, plane_pitch, y_ratio_uv);
        let offset_chroma =
            plane_super_offset(true, src_height, 2, pel, vpad, plane_pitch, y_ratio_uv);

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
        let y_ratio_uv = NonZeroUsize::new(2).unwrap(); // 4:2:0
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
        let ratio_uv = NonZeroUsize::new(2).unwrap();
        let pad = 8;

        let height = plane_height_luma(src_dim, 1, ratio_uv, pad);
        let width = plane_width_luma(src_dim, 1, ratio_uv, pad);

        // Both should give the same result for same input parameters
        assert_eq!(height, width);
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MVPlaneSet: u8 {
        const YPLANE = (1 << 0);
        const UPLANE = (1 << 1);
        const VPLANE = (1 << 2);
        const YUPLANES = Self::YPLANE.bits() | Self::UPLANE.bits();
        const YVPLANES = Self::YPLANE.bits() | Self::VPLANE.bits();
        const UVPLANES = Self::UPLANE.bits() | Self::VPLANE.bits();
        const YUVPLANES = Self::YPLANE.bits() | Self::UPLANE.bits() | Self::VPLANE.bits();
    }
}
