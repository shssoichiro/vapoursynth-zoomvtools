use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::Result;
use ndrustfft::DctHandler;

#[derive(Clone)]
pub struct DctHelper {
    size_x: NonZeroUsize,
    size_y: NonZeroUsize,
    bits_per_sample: NonZeroU8,
    dct_shift: usize,
    dct_shift0: usize,
    dct_handler_x: DctHandler<f32>,
    dct_handler_y: DctHandler<f32>,
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

        let this = DctHelper {
            size_x,
            size_y,
            bits_per_sample,
            dct_shift,
            dct_shift0,
            dct_handler_x: DctHandler::new(size_x.get()),
            dct_handler_y: DctHandler::new(size_y.get()),
        };
        Ok(this)
    }

    pub fn bytes_2d(&self) {
        todo!()
    }
}
