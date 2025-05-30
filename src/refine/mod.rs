#[cfg(test)]
mod tests;

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

pub type RefineFn<T> = fn(&[T], &mut [T], NonZeroUsize, NonZeroUsize, NonZeroUsize, NonZeroU8);

impl MVPlane {
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
