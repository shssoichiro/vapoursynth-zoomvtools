use bitflags::bitflags;
use std::mem::size_of;

pub const MV_SIZE: usize = size_of::<MotionVector>();

#[derive(Debug, Clone, Copy)]
pub struct MotionVector {
    pub x: isize,
    pub y: isize,
    pub sad: i64,
}

bitflags! {
    /// Bitflags for motion vector checking options.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CheckMVFlags: u32 {
        /// Penalty for new motion vectors.
        const PENALTY_NEW = 1 << 1;
        /// Update direction during motion vector checking.
        const UPDATE_DIR = 1 << 2;
        /// Update best motion vector during checking.
        const UPDATE_BEST_MV = 1 << 3;
    }
}

impl CheckMVFlags {
    /// Creates a new instance with no flags set.
    #[must_use]
    pub fn new() -> Self {
        Self::empty()
    }
}

impl Default for CheckMVFlags {
    fn default() -> Self {
        Self::new()
    }
}
impl MotionVector {
    #[must_use]
    pub fn zero() -> Self {
        MotionVector {
            x: 0,
            y: 0,
            sad: -1,
        }
    }
}

impl Default for MotionVector {
    fn default() -> Self {
        Self::zero()
    }
}
