// src/interp/linear.rs
use crate::error::SanosResult;

use super::time_interpolator::{bracket_maturity, TimeInterpolator};

/// Linear interpolation in calendar time:
///   alpha = (T - T_j) / (T_{j+1} - T_j)
#[derive(Debug, Clone, Copy, Default)]
pub struct LinearTime;

impl TimeInterpolator for LinearTime {
    fn alpha(&self, maturity: f64, maturities: &[f64], _atm_calls: &[f64]) -> SanosResult<(usize, f64)> {
        let j = bracket_maturity(maturity, maturities)?;
        let t0 = maturities[j];
        let t1 = maturities[j + 1];

        // Boundary clamping behavior:
        if maturity <= t0 {
            return Ok((j, 0.0));
        }
        if maturity >= t1 {
            return Ok((j, 1.0));
        }

        let denom = t1 - t0;
        let a = (maturity - t0) / denom;
        Ok((j, a))
    }
}
