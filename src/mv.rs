#[derive(Debug, Clone, Copy)]
pub struct MotionVector {
    pub x: isize,
    pub y: isize,
    pub sad: Option<u64>,
}

impl MotionVector {
    pub fn zero() -> Self {
        MotionVector {
            x: 0,
            y: 0,
            sad: None,
        }
    }
}
