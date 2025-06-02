use bitflags::bitflags;
use std::num::NonZeroUsize;

use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Subpel {
    Full = 1,
    Half = 2,
    Quarter = 4,
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

impl From<Subpel> for NonZeroUsize {
    fn from(value: Subpel) -> Self {
        // SAFETY: the int value of this enum can never be zero
        unsafe { NonZeroUsize::new_unchecked(usize::from(value)) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubpelMethod {
    Bilinear = 0,
    Bicubic = 1,
    Wiener = 2,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReduceFilter {
    Average = 0,
    Triangle = 1,
    Bilinear = 2,
    Quadratic = 3,
    Cubic = 4,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchType {
    Onetime = 0,
    Nstep = 1,
    Logarithmic = 2,
    Exhaustive = 3,
    Hex2 = 4,
    UnevenMultiHexagon = 5,
    Horizontal = 6,
    Vertical = 7,
}

impl TryFrom<i64> for SearchType {
    type Error = anyhow::Error;

    fn try_from(val: i64) -> Result<Self> {
        Ok(match val {
            0 => Self::Onetime,
            1 => Self::Nstep,
            2 => Self::Logarithmic,
            3 => Self::Exhaustive,
            4 => Self::Hex2,
            5 => Self::UnevenMultiHexagon,
            6 => Self::Horizontal,
            7 => Self::Vertical,
            _ => bail!("Invalid value for 'search', must be 0-7, got {val}."),
        })
    }
}

/// Specifies how block differences (SAD) are calculated between frames.
/// Can use spatial data, DCT coefficients, SATD, or combinations to improve motion estimation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DctMode {
    /// Calculate differences using raw pixel values in spatial domain.
    Spatial = 0,
    /// Calculate differences using DCT coefficients. Slower, especially for block sizes other than 8x8.
    Dct = 1,
    /// Use both spatial and DCT data, weighted based on the average luma difference between frames.
    MixedSpatialDct = 2,
    /// Adaptively choose between spatial data or an equal mix of spatial and DCT data for each block.
    AdaptiveSpatialMixed = 3,
    /// Adaptively choose between spatial data or DCT-weighted mixed mode for each block.
    AdaptiveSpatialDct = 4,
    /// Use Sum of Absolute Transformed Differences (SATD) instead of SAD for luma comparison.
    Satd = 5,
    /// Use both SATD and DCT data, weighted based on the average luma difference between frames.
    MixedSatdDct = 6,
    /// Adaptively choose between SATD data or an equal mix of SATD and DCT data for each block.
    AdaptiveSatdMixed = 7,
    /// Adaptively choose between SATD data or DCT-weighted mixed mode for each block.
    AdaptiveSatdDct = 8,
    /// Mix of SAD, SATD and DCT data. Weight varies from SAD-only to equal SAD/SATD mix.
    MixedSadEqSatdDct = 9,
    /// Adaptively use SATD weighted by SAD, but only when there are significant luma changes.
    AdaptiveSatdLuma = 10,
}

impl TryFrom<i64> for DctMode {
    type Error = anyhow::Error;

    fn try_from(val: i64) -> Result<Self> {
        Ok(match val {
            0 => Self::Spatial,
            1 => Self::Dct,
            2 => Self::MixedSpatialDct,
            3 => Self::AdaptiveSpatialMixed,
            4 => Self::AdaptiveSpatialDct,
            5 => Self::Satd,
            6 => Self::MixedSatdDct,
            7 => Self::AdaptiveSatdMixed,
            8 => Self::AdaptiveSatdDct,
            9 => Self::MixedSadEqSatdDct,
            10 => Self::AdaptiveSatdLuma,
            _ => bail!("Invalid value for 'dct', must be 0-10, got {val}."),
        })
    }
}

impl DctMode {
    pub fn uses_satd(&self) -> bool {
        match self {
            DctMode::Satd
            | DctMode::MixedSatdDct
            | DctMode::AdaptiveSatdMixed
            | DctMode::AdaptiveSatdDct
            | DctMode::MixedSadEqSatdDct
            | DctMode::AdaptiveSatdLuma => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PenaltyScaling {
    None = 0,
    Linear = 1,
    /// Quadratic dependence from hierarchical level size
    Quadratic = 2,
}

impl TryFrom<i64> for PenaltyScaling {
    type Error = anyhow::Error;

    fn try_from(val: i64) -> Result<Self> {
        Ok(match val {
            0 => Self::None,
            1 => Self::Linear,
            2 => Self::Quadratic,
            _ => bail!("Invalid value for 'plevel', must be 0-2, got {val}."),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DivideMode {
    /// do not divide
    None = 0,
    /// divide blocks and assign the original vector to all 4 subblocks
    Original = 1,
    /// divide blocks and assign median (with 2 neighbors) vectors to subblocks
    Median = 2,
}

impl TryFrom<i64> for DivideMode {
    type Error = anyhow::Error;

    fn try_from(val: i64) -> Result<Self> {
        Ok(match val {
            0 => Self::None,
            1 => Self::Original,
            2 => Self::Median,
            _ => bail!("Invalid value for 'divide', must be 0-2, got {val}."),
        })
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MotionFlags: u8 {
        const IS_BACKWARD = 0x00000002;
        const SMALLEST_PLANE = 0x00000004;
        const USE_CHROMA_MOTION = 0x00000008;
        const USE_SSD = 0x00000010;
        const USE_SATD =  0x00000020;
    }
}

pub const MV_DEFAULT_SCD1: usize = 400;
pub const MV_DEFAULT_SCD2: usize = 130;
