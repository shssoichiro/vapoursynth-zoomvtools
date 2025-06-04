#[derive(Debug, Clone, Copy)]
pub struct MotionVector {
    pub x: isize,
    pub y: isize,
    pub sad: i64,
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
