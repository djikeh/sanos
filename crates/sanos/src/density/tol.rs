// src/density/tol.rs
use crate::error::{SanosError, SanosResult};

#[derive(Debug, Clone, Copy)]
pub struct DensityTolerances {
    pub mass: f64,
    pub mean: f64,
    pub order: f64,
}

impl DensityTolerances {
    pub fn new(mass: f64, mean: f64, order: f64) -> SanosResult<Self> {
        for (field, v) in [("mass", mass), ("mean", mean), ("order", order)] {
            if !v.is_finite() {
                return Err(SanosError::NonFinite { field, value: v });
            }
            if v < 0.0 {
                return Err(SanosError::InvalidBound {
                    field,
                    value: v,
                    min: 0.0,
                    max: f64::INFINITY,
                });
            }
        }
        Ok(Self { mass, mean, order })
    }

    /// Convenience constructor: set all tolerances to the same value.
    pub fn from_tol(tol: f64) -> SanosResult<Self> {
        Self::new(tol, tol, tol)
    }
}
