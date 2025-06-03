use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::{Result, anyhow};

use crate::{
    params::{DivideMode, MotionFlags, Subpel},
    plane_of_blocks::PlaneOfBlocks,
};

#[derive(Debug, Clone)]
pub struct GroupOfPlanes {
    pub blk_size_x: NonZeroUsize,
    pub blk_size_y: NonZeroUsize,
    pub level_count: usize,
    pub overlap_x: usize,
    pub overlap_y: usize,
    pub x_ratio_uv: NonZeroU8,
    pub y_ratio_uv: NonZeroU8,
    pub divide_extra: DivideMode,
    pub planes: Vec<PlaneOfBlocks>,
}

impl GroupOfPlanes {
    pub fn new(
        blk_size_x: NonZeroUsize,
        blk_size_y: NonZeroUsize,
        level_count: usize,
        pel: Subpel,
        motion_flags: MotionFlags,
        overlap_x: usize,
        overlap_y: usize,
        blk_x: NonZeroUsize,
        blk_y: NonZeroUsize,
        x_ratio_uv: NonZeroU8,
        y_ratio_uv: NonZeroU8,
        divide_extra: DivideMode,
        bits_per_sample: NonZeroU8,
    ) -> Result<Self> {
        let mut planes = Vec::with_capacity(level_count as usize);

        let mut pel_current = pel;
        let mut motion_flags_current = motion_flags;
        let width_b = NonZeroUsize::new((blk_size_x.get() - overlap_x) * blk_x.get() + overlap_x)
            .ok_or(anyhow!("invalid width calculation"))?;
        let height_b = NonZeroUsize::new((blk_size_y.get() - overlap_y) * blk_y.get() + overlap_y)
            .ok_or(anyhow!("invalid height calculation"))?;

        for i in 0..level_count {
            if i == level_count - 1 {
                motion_flags_current |= MotionFlags::SMALLEST_PLANE;
            }

            let blk_x_current = NonZeroUsize::new(
                ((width_b.get() >> i) - overlap_x) / (blk_size_x.get() - overlap_x),
            )
            .ok_or(anyhow!("invalid block x calculation"))?;
            let blk_y_current = NonZeroUsize::new(
                ((height_b.get() >> i) - overlap_y) / (blk_size_y.get() - overlap_y),
            )
            .ok_or(anyhow!("invalid block y calculation"))?;

            planes.push(PlaneOfBlocks::new(
                blk_x_current,
                blk_y_current,
                blk_size_x,
                blk_size_y,
                pel_current,
                i,
                motion_flags_current,
                overlap_x,
                overlap_y,
                x_ratio_uv,
                y_ratio_uv,
                bits_per_sample,
            )?);
            pel_current = Subpel::Full;
        }

        Ok(Self {
            blk_size_x,
            blk_size_y,
            level_count,
            overlap_x,
            overlap_y,
            x_ratio_uv,
            y_ratio_uv,
            divide_extra,
            planes,
        })
    }
}
