use std::fmt::Debug;
// src/interp/time_interpolator.rs
use crate::error::{SanosError, SanosResult};

/// Time interpolation policy for SANOS:
/// returns (j, alpha) such that
///   maturities[j] <= T <= maturities[j+1]
///   alpha in [0,1]
pub trait TimeInterpolator: Send + Sync + Debug {
    fn alpha(
        &self,
        maturity: f64,
        maturities: &[f64],
        atm_calls: &[f64], // not used by LinearTime but kept for future ATM-variance interpolation
    ) -> SanosResult<(usize, f64)>;
}

/// Shared helper: validate maturity grid and find bracketing index j.
pub fn bracket_maturity(maturity: f64, maturities: &[f64]) -> SanosResult<usize> {
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
    if maturities.len() < 2 {
        return Err(SanosError::InvalidOrdering { msg: "need at least 2 maturities for interpolation" });
    }
    for w in maturities.windows(2) {
        if w[1] <= w[0] {
            return Err(SanosError::InvalidOrdering { msg: "maturities must be strictly increasing" });
        }
    }

    // Clamp to boundary intervals:
    // - if maturity <= T0, use interval [T0, T1] with alpha=0 in LinearTime
    // - if maturity >= T_last, use interval [T_{n-2}, T_{n-1}] with alpha=1 in LinearTime
    if maturity <= maturities[0] {
        return Ok(0);
    }
    if maturity >= maturities[maturities.len() - 1] {
        return Ok(maturities.len() - 2);
    }

    // Find j such that T_j <= T <= T_{j+1}
    for (j, w) in maturities.windows(2).enumerate() {
        if w[0] <= maturity && maturity <= w[1] {
            return Ok(j);
        }
    }

    Err(SanosError::InvalidOrdering { msg: "failed to bracket maturity" })
}
