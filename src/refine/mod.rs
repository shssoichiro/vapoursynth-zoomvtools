mod bicubic;
mod bilinear;
mod wiener;

#[cfg(test)]
mod tests;

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
pub type RefineFn<T> = fn(&mut [T], &[T], NonZeroUsize, NonZeroUsize, NonZeroUsize, NonZeroU8);

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
