// src/grid/specs.rs
use crate::error::{SanosError, SanosResult};

#[derive(Debug, Clone, Copy)]
pub struct WingsSpec {
    pub n_left: usize,
    pub n_right: usize,
    pub ratio: f64, // > 1
}

impl Default for WingsSpec {
    fn default() -> Self {
        Self { n_left: 2, n_right: 2, ratio: 1.2 }
    }
}

impl WingsSpec {
    pub fn validate(&self) -> SanosResult<()> {
        if !self.ratio.is_finite() {
            return Err(SanosError::NonFinite { field: "ratio", value: self.ratio });
        }
        if self.ratio <= 1.0 {
            return Err(SanosError::InvalidBound {
                field: "ratio",
                value: self.ratio,
                min: 1.0 + f64::EPSILON,
                max: f64::INFINITY,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AtmRefineSpec {
    pub enabled: bool,
    pub steps: usize,
    pub delta_log: f64, // > 0
}

impl Default for AtmRefineSpec {
    fn default() -> Self {
        Self { enabled: false, steps: 2, delta_log: 0.05 }
    }
}

impl AtmRefineSpec {
    pub fn validate(&self) -> SanosResult<()> {
        if !self.delta_log.is_finite() {
            return Err(SanosError::NonFinite { field: "delta_log", value: self.delta_log });
        }
        if self.enabled && self.delta_log <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "delta_log",
                value: self.delta_log,
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GridSizeControl {
    pub max_points: usize,
    pub keep_all_market_strikes: bool,
}

impl Default for GridSizeControl {
    fn default() -> Self {
        Self { max_points: 80, keep_all_market_strikes: true }
    }
}

impl GridSizeControl {
    pub fn validate(&self) -> SanosResult<()> {
        if self.max_points == 0 {
            return Err(SanosError::InvalidOrdering { msg: "max_points must be > 0" });
        }
        Ok(())
    }
}
