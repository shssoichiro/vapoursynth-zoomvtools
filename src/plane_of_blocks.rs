use crate::params::{MotionFlags, Subpel};
use anyhow::Result;
use std::num::{NonZeroU8, NonZeroUsize};

#[derive(Debug, Clone)]
pub struct PlaneOfBlocks {
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
}

impl PlaneOfBlocks {
    pub fn new(
        blk_x_current: NonZeroUsize,
        blk_y_current: NonZeroUsize,
        blk_size_x: NonZeroUsize,
        blk_size_y: NonZeroUsize,
        pel_current: Subpel,
        i: usize,
        motion_flags_current: MotionFlags,
        overlap_x: usize,
        overlap_y: usize,
        x_ratio_uv: NonZeroU8,
        y_ratio_uv: NonZeroU8,
        bits_per_sample: NonZeroU8,
    ) -> Result<Self> {
        todo!()
    }
}
