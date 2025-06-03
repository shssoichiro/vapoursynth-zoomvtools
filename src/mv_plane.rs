#[cfg(test)]
mod tests;

use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::Result;
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
    #[allow(dead_code)]
    pub hpad_pel: usize,
    #[allow(dead_code)]
    pub vpad_pel: usize,
    pub bits_per_sample: NonZeroU8,
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
        let offset_padding = pitch.get() * vpad + hpad;

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
        if reduced_plane.is_filled {
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

    /// Helper function to safely perform refinement operation without cloning
    /// by ensuring non-overlapping source and destination slices.
    fn refine_with_split<T: Pixel>(
        plane: &mut [T],
        src_offset: usize,
        dest_offset: usize,
        refine_fn: RefineFn<T>,
        pitch: NonZeroUsize,
        padded_width: NonZeroUsize,
        padded_height: NonZeroUsize,
        bits_per_sample: NonZeroU8,
    ) {
        debug_assert!(
            bits_per_sample.get() as usize > (size_of::<T>() - 1) * 8
                && (bits_per_sample.get() as usize <= size_of::<T>() * 8)
        );

        if src_offset <= dest_offset {
            // Source comes before destination, split at destination
            let (left, right) = plane.split_at_mut(dest_offset);
            refine_fn(
                right,
                &left[src_offset..],
                pitch,
                padded_width,
                padded_height,
                bits_per_sample,
            );
        } else {
            // Destination comes before source, split at source
            let (left, right) = plane.split_at_mut(src_offset);
            refine_fn(
                &mut left[dest_offset..],
                right,
                pitch,
                padded_width,
                padded_height,
                bits_per_sample,
            );
        }
    }

    /// Helper function to safely perform average2 operation without cloning
    /// by ensuring non-overlapping source and destination slices.
    fn average2_with_split<T: Pixel>(
        plane: &mut [T],
        src1_offset: usize,
        src2_offset: usize,
        dest_offset: usize,
        pitch: NonZeroUsize,
        width: NonZeroUsize,
        height: NonZeroUsize,
    ) {
        // Find the ordering of the three offsets
        let mut offsets = [
            (src1_offset, 1u8), // 1 = src1
            (src2_offset, 2u8), // 2 = src2
            (dest_offset, 0u8), // 0 = dest
        ];
        offsets.sort_by_key(|&(offset, _)| offset);

        let _first_offset = offsets[0].0;
        let second_offset = offsets[1].0;
        let third_offset = offsets[2].0;

        let first_type = offsets[0].1;
        let second_type = offsets[1].1;
        let third_type = offsets[2].1;

        // Split the slice to get non-overlapping regions
        let (first_part, rest) = plane.split_at_mut(second_offset);
        let (second_part, third_part) = rest.split_at_mut(third_offset - second_offset);

        // Determine which slice corresponds to which source/destination
        let (src1_slice, src2_slice, dest_slice) = match (first_type, second_type, third_type) {
            (1, 2, 0) => (
                &first_part[src1_offset..],
                &second_part[0..],
                &mut third_part[0..],
            ),
            (1, 0, 2) => (
                &first_part[src1_offset..],
                &third_part[src2_offset - third_offset..],
                &mut second_part[dest_offset - second_offset..],
            ),
            (2, 1, 0) => (
                &second_part[src1_offset - second_offset..],
                &first_part[src2_offset..],
                &mut third_part[0..],
            ),
            (2, 0, 1) => (
                &third_part[src1_offset - third_offset..],
                &first_part[src2_offset..],
                &mut second_part[dest_offset - second_offset..],
            ),
            (0, 1, 2) => (
                &second_part[src1_offset - second_offset..],
                &third_part[src2_offset - third_offset..],
                &mut first_part[dest_offset..],
            ),
            (0, 2, 1) => (
                &third_part[src1_offset - third_offset..],
                &second_part[src2_offset - second_offset..],
                &mut first_part[dest_offset..],
            ),
            _ => unreachable!("Invalid offset ordering"),
        };

        average2(src1_slice, src2_slice, dest_slice, pitch, width, height);
    }

    pub fn refine<T: Pixel>(&mut self, method: SubpelMethod, plane: &mut [T]) {
        if self.is_refined {
            return;
        }

        if self.pel == Subpel::Full {
            self.is_refined = true;
            return;
        }

        let refine: [RefineFn<T>; 3] = match method {
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
                if method == SubpelMethod::Bilinear {
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
                if method == SubpelMethod::Bilinear {
                    src_offsets[2] = self.subpel_window_offsets[0];
                } else {
                    src_offsets[2] = self.subpel_window_offsets[8];
                }
            }
        }

        // Use the helper function to avoid clones
        for i in 0..3 {
            Self::refine_with_split(
                plane,
                src_offsets[i],
                dest_offsets[i],
                refine[i],
                self.pitch,
                self.padded_width,
                self.padded_height,
                self.bits_per_sample,
            );
        }

        // Use helper function to avoid clones in average2 calls
        if self.pel == Subpel::Quarter {
            Self::average2_with_split(
                plane,
                self.subpel_window_offsets[0],
                self.subpel_window_offsets[2],
                self.subpel_window_offsets[1],
                self.pitch,
                self.padded_width,
                self.padded_height,
            );
            Self::average2_with_split(
                plane,
                self.subpel_window_offsets[8],
                self.subpel_window_offsets[10],
                self.subpel_window_offsets[9],
                self.pitch,
                self.padded_width,
                self.padded_height,
            );
            Self::average2_with_split(
                plane,
                self.subpel_window_offsets[0],
                self.subpel_window_offsets[8],
                self.subpel_window_offsets[4],
                self.pitch,
                self.padded_width,
                self.padded_height,
            );
            Self::average2_with_split(
                plane,
                self.subpel_window_offsets[2],
                self.subpel_window_offsets[10],
                self.subpel_window_offsets[6],
                self.pitch,
                self.padded_width,
                self.padded_height,
            );
            Self::average2_with_split(
                plane,
                self.subpel_window_offsets[4],
                self.subpel_window_offsets[6],
                self.subpel_window_offsets[5],
                self.pitch,
                self.padded_width,
                self.padded_height,
            );

            Self::average2_with_split(
                plane,
                self.subpel_window_offsets[0] + 1,
                self.subpel_window_offsets[2],
                self.subpel_window_offsets[3],
                self.pitch,
                // SAFETY: Since we are doing qpel refinement, we know res is at least 4x4
                unsafe { NonZeroUsize::new_unchecked(self.padded_width.get() - 1) },
                self.padded_height,
            );
            Self::average2_with_split(
                plane,
                self.subpel_window_offsets[8] + 1,
                self.subpel_window_offsets[10],
                self.subpel_window_offsets[11],
                self.pitch,
                // SAFETY: Since we are doing qpel refinement, we know res is at least 4x4
                unsafe { NonZeroUsize::new_unchecked(self.padded_width.get() - 1) },
                self.padded_height,
            );
            Self::average2_with_split(
                plane,
                self.subpel_window_offsets[0] + self.pitch.get(),
                self.subpel_window_offsets[8],
                self.subpel_window_offsets[12],
                self.pitch,
                self.padded_width,
                // SAFETY: Since we are doing qpel refinement, we know res is at least 4x4
                unsafe { NonZeroUsize::new_unchecked(self.padded_height.get() - 1) },
            );
            Self::average2_with_split(
                plane,
                self.subpel_window_offsets[2] + self.pitch.get(),
                self.subpel_window_offsets[10],
                self.subpel_window_offsets[14],
                self.pitch,
                self.padded_width,
                // SAFETY: Since we are doing qpel refinement, we know res is at least 4x4
                unsafe { NonZeroUsize::new_unchecked(self.padded_height.get() - 1) },
            );
            Self::average2_with_split(
                plane,
                self.subpel_window_offsets[12],
                self.subpel_window_offsets[14],
                self.subpel_window_offsets[13],
                self.pitch,
                self.padded_width,
                self.padded_height,
            );
            Self::average2_with_split(
                plane,
                self.subpel_window_offsets[4] + 1,
                self.subpel_window_offsets[6],
                self.subpel_window_offsets[7],
                self.pitch,
                // SAFETY: Since we are doing qpel refinement, we know res is at least 4x4
                unsafe { NonZeroUsize::new_unchecked(self.padded_width.get() - 1) },
                self.padded_height,
            );
            Self::average2_with_split(
                plane,
                self.subpel_window_offsets[12] + 1,
                self.subpel_window_offsets[14],
                self.subpel_window_offsets[15],
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
    level: usize,
    y_ratio_uv: NonZeroU8,
    vpad: usize,
) -> NonZeroUsize {
    let mut height = src_height.get();
    let y_ratio_uv_val = y_ratio_uv.get() as usize;

    for _i in 1..=level {
        height = if vpad >= y_ratio_uv_val {
            (height / y_ratio_uv_val + 1) / 2 * y_ratio_uv_val
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
    level: usize,
    x_ratio_uv: NonZeroU8,
    hpad: usize,
) -> NonZeroUsize {
    let mut width = src_width.get();
    let x_ratio_uv_val = x_ratio_uv.get() as usize;

    for _i in 1..=level {
        width = if hpad >= x_ratio_uv_val {
            (width / x_ratio_uv_val + 1) / 2 * x_ratio_uv_val
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
/// This function computes the pixel offset where a specific plane begins within
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
/// The pixel offset where the specified plane begins in the superframe
pub fn plane_super_offset(
    chroma: bool,
    src_height: NonZeroUsize,
    level: usize,
    pel: Subpel,
    vpad: usize,
    plane_pitch: NonZeroUsize,
    y_ratio_uv: NonZeroU8,
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
        let y_ratio_uv_val = y_ratio_uv.get() as usize;
        offset = pel * pel * plane_pitch_val * (src_height_val + vpad * 2);

        for i in 1..level {
            // NOTE: We use `src_height` here (not a running `height` variable) because
            // plane_height_luma internally handles the hierarchical scaling by applying
            // the division `level` times in its own loop. Each call calculates the height
            // at the specific level `i` starting from the original source dimensions.
            height = if chroma {
                plane_height_luma(
                    src_height.saturating_mul(y_ratio_uv.into()),
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
