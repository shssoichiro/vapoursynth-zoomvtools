use bitflags::bitflags;
use std::mem::size_of;

pub const MV_SIZE: usize = size_of::<MotionVector>();

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MotionVector {
    pub x: i32,
    pub y: i32,
    pub sad: i64,
}

impl MotionVector {
    /// computes square distance between two vectors
    #[must_use]
    pub fn square_difference_norm(&self, v2x: i32, v2y: i32) -> u64 {
        ((self.x - v2x).pow(2) + (self.y - v2y).pow(2)) as u64
    }
}

impl MotionVector {
    #[must_use]
    pub(crate) fn bytes(&self) -> &[u8] {
        // SAFETY: We've added `repr(c)` to ensure a predictable size of the struct
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
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
