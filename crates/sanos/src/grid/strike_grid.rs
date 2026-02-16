// src/grid/strike_grid.rs
use crate::error::{SanosError, SanosResult};

#[derive(Debug, Clone)]
pub struct StrikeGrid {
    maturity: f64,
    strikes: Vec<f64>, // strictly increasing
}

impl StrikeGrid {
    pub fn new(maturity: f64, strikes: Vec<f64>) -> SanosResult<Self> {
        if !maturity.is_finite() {
            return Err(SanosError::NonFinite { field: "maturity", value: maturity });
        }
        if maturity <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "maturity",
                value: maturity,
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }
        if strikes.is_empty() {
            return Err(SanosError::EmptyCollection { what: "StrikeGrid.strikes" });
        }
        for w in strikes.windows(2) {
            if w[1] <= w[0] {
                return Err(SanosError::InvalidOrdering { msg: "StrikeGrid strikes must be strictly increasing" });
            }
        }

        Ok(Self { maturity, strikes })
    }

    #[inline]
    pub fn maturity(&self) -> f64 {
        self.maturity
    }

    #[inline]
    pub fn strikes(&self) -> &[f64] {
        &self.strikes
    }
}
