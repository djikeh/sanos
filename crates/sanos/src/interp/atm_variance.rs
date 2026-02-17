// src/interp/atm_variance.rs
use crate::backbone::{bs_call_forward_norm, bs_implied_atm_var_from_call};
use crate::error::{SanosError, SanosResult};

use super::time_interpolator::{bracket_maturity, TimeInterpolator};

/// Interpolation in implied Black-Scholes ATM variance (Remark 2.13 style).
///
/// Inputs:
/// - `maturities`: strictly increasing grid `T_0 < ... < T_{M-1}`
/// - `atm_calls`: ATM call prices `C_j(1)` at node maturities (same length)
///
/// For `T in [T_j, T_{j+1}]`:
/// - `W(T)` linearly interpolated between `W_j` and `W_{j+1}`
/// - `alpha_j(T) = (Call(1,1,W(T)) - C_j(1)) / (C_{j+1}(1) - C_j(1))`
#[derive(Debug, Clone, Copy, Default)]
pub struct AtmVarianceTime;

impl TimeInterpolator for AtmVarianceTime {
    fn alpha(&self, maturity: f64, maturities: &[f64], atm_calls: &[f64]) -> SanosResult<(usize, f64)> {
        let j = bracket_maturity(maturity, maturities)?;

        if atm_calls.len() != maturities.len() {
            return Err(SanosError::InvalidOrdering {
                msg: "atm_calls must have same length as maturities",
            });
        }

        let t0 = maturities[j];
        let t1 = maturities[j + 1];

        let c0 = atm_calls[j];
        let c1 = atm_calls[j + 1];

        // If boundary clamping:
        if maturity <= t0 {
            return Ok((j, 0.0));
        }
        if maturity >= t1 {
            return Ok((j, 1.0));
        }

        // Convert node ATM calls to node ATM variances.
        let w0 = bs_implied_atm_var_from_call(c0)?;
        let w1 = bs_implied_atm_var_from_call(c1)?;

        // Linear interpolation in time for total variance W(T).
        let w_t = if (t1 - t0).abs() <= 1e-16 {
            0.5 * (w0 + w1)
        } else {
            w0 + (w1 - w0) * (maturity - t0) / (t1 - t0)
        };

        let c_t = bs_call_forward_norm(1.0, w_t)?;

        let denom = c1 - c0;
        if denom.abs() <= 1e-16 {
            // If ATM calls are equal, any alpha works; choose 0 for stability.
            return Ok((j, 0.0));
        }

        let mut a = (c_t - c0) / denom;

        // Safety clamp to [0,1] for numerical noise.
        if !a.is_finite() {
            a = 0.0;
        }
        a = a.clamp(0.0, 1.0);

        Ok((j, a))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atm_var_inversion_roundtrip() {
        let w = 0.09;
        let c = bs_call_forward_norm(1.0, w).unwrap();
        let w2 = bs_implied_atm_var_from_call(c).unwrap();
        assert!((w - w2).abs() < 1e-10);
    }

    #[test]
    fn atm_variance_time_returns_alpha_in_0_1() {
        let interp = AtmVarianceTime;

        let maturities = vec![0.5, 1.0];
        let c0 = bs_call_forward_norm(1.0, 0.04).unwrap();
        let c1 = bs_call_forward_norm(1.0, 0.09).unwrap();
        let atm_calls = vec![c0, c1];

        let (_j, a) = interp.alpha(0.75, &maturities, &atm_calls).unwrap();
        assert!((0.0..=1.0).contains(&a));
    }
}
