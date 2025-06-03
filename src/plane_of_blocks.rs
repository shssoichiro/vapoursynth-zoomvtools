use crate::{
    mv::MotionVector,
    params::{DivideMode, MotionFlags, Subpel},
    util::Pixel,
};
use aligned::{A64, Aligned};
use smallvec::SmallVec;
use std::num::{NonZeroU8, NonZeroUsize};

// max block width * max block height
const MAX_BLOCK_SIZE: usize = 128 * 128;

#[derive(Debug, Clone)]
pub struct PlaneOfBlocks<T: Pixel> {
    pel: Subpel,
    log_pel: u8,
    log_scale: usize,
    scale: usize,
    blk_size_x: NonZeroUsize,
    blk_size_y: NonZeroUsize,
    overlap_x: usize,
    overlap_y: usize,
    blk_x: NonZeroUsize,
    blk_y: NonZeroUsize,
    blk_count: NonZeroUsize,
    x_ratio_uv: NonZeroU8,
    y_ratio_uv: NonZeroU8,
    log_x_ratio_uv: u8,
    log_y_ratio_uv: u8,
    bits_per_sample: NonZeroU8,
    smallest_plane: bool,
    chroma: bool,
    can_use_satd: bool,
    global_mv_predictor: MotionVector,
    vectors: Vec<MotionVector>,
    dct_pitch: NonZeroUsize,
    freq_size: NonZeroUsize,
    freq_array: Vec<i32>,
    very_big_sad: NonZeroUsize,

    // TODO: We might want to move these away from this struct
    dct_src: Aligned<A64, SmallVec<[T; MAX_BLOCK_SIZE]>>,
    dct_ref: Aligned<A64, SmallVec<[T; MAX_BLOCK_SIZE]>>,
    src_pitch_temp: [NonZeroUsize; 3],
    src_temp: [Aligned<A64, SmallVec<[T; MAX_BLOCK_SIZE]>>; 3],
}

impl<T: Pixel> PlaneOfBlocks<T> {
    pub fn new(
        blk_x: NonZeroUsize,
        blk_y: NonZeroUsize,
        blk_size_x: NonZeroUsize,
        blk_size_y: NonZeroUsize,
        pel: Subpel,
        level: usize,
        motion_flags: MotionFlags,
        overlap_x: usize,
        overlap_y: usize,
        x_ratio_uv: NonZeroU8,
        y_ratio_uv: NonZeroU8,
        bits_per_sample: NonZeroU8,
    ) -> Self {
        debug_assert!(
            bits_per_sample.get() as usize > (size_of::<T>() - 1) * 8
                && (bits_per_sample.get() as usize <= size_of::<T>() * 8)
        );

        let blk_count = blk_x.saturating_mul(blk_y);
        // SAFETY: pel can never be 0, so this can never be 0
        let freq_size = unsafe { NonZeroUsize::new_unchecked(8192 * u8::from(pel) as usize * 2) };
        let dct_pitch = blk_size_x;
        // SAFETY: valid values will never result in this being 0
        let chroma_src_pitch =
            unsafe { NonZeroUsize::new_unchecked(blk_size_x.get() / x_ratio_uv.get() as usize) };
        let src_pitch_temp = [blk_size_x, chroma_src_pitch, chroma_src_pitch];
        Self {
            pel,
            log_pel: u8::from(pel).ilog2() as u8,
            log_scale: level,
            scale: 2usize.pow(level as u32),
            blk_size_x,
            blk_size_y,
            overlap_x,
            overlap_y,
            blk_x,
            blk_y,
            blk_count,
            x_ratio_uv,
            y_ratio_uv,
            log_x_ratio_uv: x_ratio_uv.ilog2() as u8,
            log_y_ratio_uv: y_ratio_uv.ilog2() as u8,
            bits_per_sample,
            smallest_plane: motion_flags.contains(MotionFlags::SMALLEST_PLANE),
            chroma: motion_flags.contains(MotionFlags::USE_CHROMA_MOTION),
            can_use_satd: !(blk_size_x.get() == 16 && blk_size_y.get() == 2),
            global_mv_predictor: MotionVector::zero(),
            vectors: vec![MotionVector::zero(); blk_count.get()],
            dct_pitch,
            freq_size,
            freq_array: vec![0; freq_size.get()],
            // SAFETY: constant can never be 0
            very_big_sad: blk_size_x
                .saturating_mul(blk_size_y)
                .saturating_mul(unsafe { NonZeroUsize::new_unchecked(1 << bits_per_sample.get()) }),
            dct_src: Aligned(SmallVec::from_elem(
                T::from(0),
                blk_size_y.get() * dct_pitch.get(),
            )),
            dct_ref: Aligned(SmallVec::from_elem(
                T::from(0),
                blk_size_y.get() * dct_pitch.get(),
            )),
            src_pitch_temp,
            src_temp: [
                Aligned(SmallVec::from_elem(
                    T::from(0),
                    blk_size_y.get() * src_pitch_temp[0].get(),
                )),
                Aligned(SmallVec::from_elem(
                    T::from(0),
                    blk_size_y.get() / y_ratio_uv.get() as usize * src_pitch_temp[1].get(),
                )),
                Aligned(SmallVec::from_elem(
                    T::from(0),
                    blk_size_y.get() / y_ratio_uv.get() as usize * src_pitch_temp[2].get(),
                )),
            ],
        }
    }
}
