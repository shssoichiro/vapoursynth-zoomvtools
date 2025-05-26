use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Subpel {
    Full,
    Half,
    Quarter,
}

impl TryFrom<i64> for Subpel {
    type Error = anyhow::Error;

    fn try_from(val: i64) -> Result<Self> {
        Ok(match val {
            1 => Self::Full,
            2 => Self::Half,
            4 => Self::Quarter,
            _ => bail!("Invalid value for 'pel', must be 1, 2, or 4, got {val}."),
        })
    }
}

impl From<Subpel> for usize {
    fn from(value: Subpel) -> Self {
        match value {
            Subpel::Full => 1,
            Subpel::Half => 2,
            Subpel::Quarter => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubpelMethod {
    Bilinear,
    Bicubic,
    Wiener,
}

impl TryFrom<i64> for SubpelMethod {
    type Error = anyhow::Error;

    fn try_from(val: i64) -> Result<Self> {
        Ok(match val {
            0 => Self::Bilinear,
            1 => Self::Bicubic,
            2 => Self::Wiener,
            _ => bail!("Invalid value for 'sharp', must be 0-2, got {val}."),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReduceFilter {
    Average,
    Triangle,
    Bilinear,
    Quadratic,
    Cubic,
}

impl TryFrom<i64> for ReduceFilter {
    type Error = anyhow::Error;

    fn try_from(val: i64) -> Result<Self> {
        Ok(match val {
            0 => Self::Average,
            1 => Self::Triangle,
            2 => Self::Bilinear,
            3 => Self::Quadratic,
            4 => Self::Cubic,
            _ => bail!("Invalid value for 'rfilter', must be 0-4, got {val}."),
        })
    }
}
