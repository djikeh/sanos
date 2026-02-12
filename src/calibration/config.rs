// src/calibration/config.rs
use crate::error::{SanosError, SanosResult};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmegaConfig {
    Zero,
    One,
}

impl OmegaConfig {
    #[inline]
    pub fn as_u8(self) -> u8 {
        match self {
            OmegaConfig::Zero => 0,
            OmegaConfig::One => 1,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConstraintKernelConfig {
    /// ω = 0 => use linear_call
    /// ω = 1 => use call
    pub omega: OmegaConfig,
}

impl Default for ConstraintKernelConfig {
    fn default() -> Self {
        Self { omega: OmegaConfig::One } // recommandé
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct CalibrationConfig {
    pub constraints: ConstraintKernelConfig,
}

impl Default for CalibrationConfig {
    fn default() -> Self {
        Self {
            constraints: ConstraintKernelConfig::default(),
        }
    }
}

impl CalibrationConfig {
    pub fn validate(&self) -> SanosResult<()> {
        Ok(())
    }
}
