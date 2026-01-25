use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::Result;
use fftw::{
    plan::{R2RPlan, R2RPlan32},
    types::{Flag, R2RKind},
};
use num_traits::clamp;

use crate::util::{Pixel, round_ties_to_even};

#[derive(Clone)]
pub struct DctHelper {
    size_x: NonZeroUsize,
    size_y: NonZeroUsize,
    bits_per_sample: NonZeroU8,
    dct_shift: usize,
    dct_shift0: usize,

    src: Box<[f32]>,
    src_dct: Box<[f32]>,
}

impl DctHelper {
    pub fn new(
        size_x: NonZeroUsize,
        size_y: NonZeroUsize,
        bits_per_sample: NonZeroU8,
    ) -> Result<Self> {
        let size_2d = size_y.saturating_mul(size_x);
        let mut cur_size = 1usize;
        let mut dct_shift = 0usize;
        while cur_size < size_2d.get() {
            dct_shift += 1;
            cur_size <<= 1;
        }
        let dct_shift0 = dct_shift + 2;

        let src = vec![0.0; size_2d.get()].into_boxed_slice();
        let src_dct = vec![0.0; size_2d.get()].into_boxed_slice();
        let this = DctHelper {
            size_x,
            size_y,
            bits_per_sample,
            dct_shift,
            dct_shift0,
            src,
            src_dct,
        };
        Ok(this)
    }

    pub fn bytes_2d<T: Pixel>(
        &mut self,
        src_plane: &[T],
        src_pitch: NonZeroUsize,
        dct_plane: &mut [T],
        dct_pitch: NonZeroUsize,
    ) -> Result<()> {
        // TODO: Do we need to cache this?
        let mut plan = R2RPlan32::aligned(
            &[self.size_x.get(), self.size_y.get()],
            R2RKind::FFTW_REDFT10,
            Flag::ESTIMATE,
        )?;

        self.pixels_to_float_src(src_plane, src_pitch);
        plan.r2r(&mut self.src, &mut self.src_dct)?;
        self.float_src_to_pixels(dct_plane, dct_pitch);

        Ok(())
    }

    fn pixels_to_float_src<T: Pixel>(&mut self, src_plane: &[T], src_pitch: NonZeroUsize) {
        for j in 0..(self.size_y.get()) {
            let f_src = &mut self.src[j * self.size_x.get()..][..self.size_x.get()];
            let p_src = &src_plane[j * src_pitch.get()..][..self.size_x.get()];
            for (f, p) in f_src.iter_mut().zip(p_src.iter()) {
                *f = p.to_f32().expect("fits in f32");
            }
        }
    }

    fn float_src_to_pixels<T: Pixel>(&self, dst: &mut [T], dst_pitch: NonZeroUsize) {
        let sqrt_2_div_2: f32 = (2f32).sqrt() / 2.0;
        let real_data = &self.src_dct;

        // Have to do math in larger type to avoid overflow
        let pixel_max = (1 << self.bits_per_sample.get() as usize) - 1;
        let pixel_half = 1 << (self.bits_per_sample.get() as usize - 1);

        for j in 0..(self.size_y.get()) {
            let real_data = &real_data[j * self.size_x.get()..][..self.size_x.get()];
            let dst = &mut dst[j * dst_pitch.get()..][..self.size_x.get()];
            for (f, p) in real_data.iter().zip(dst.iter_mut()) {
                // to be compatible with integer DCTINT8
                let f = *f * sqrt_2_div_2;
                let integ = round_ties_to_even(f) as i32;
                *p = T::from(clamp((integ >> self.dct_shift) + pixel_half, 0, pixel_max))
                    .expect("clamp guarantees in range");
            }
        }

        // to be compatible with integer DCTINT8
        let f = real_data[0] * 0.5;
        let integ = round_ties_to_even(f) as i32;
        // DC
        dst[0] = T::from(clamp((integ >> self.dct_shift0) + pixel_half, 0, pixel_max))
            .expect("clamp guarantees in range");
    }
}
