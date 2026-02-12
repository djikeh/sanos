// src/backbone/bs.rs
//
// Minimal Black–Scholes utilities for SANOS.
//
// Conventions:
// - Forward-normalized calls: F = 1
// - Strike is moneyness k > 0
// - Total variance var >= 0
//
// Formula:
//   C(k, var) = N(d1) - k N(d2)
//   d1 = (-ln k + 0.5 var) / sqrt(var)
//   d2 = d1 - sqrt(var)
//
// Edge case:
// - If var == 0, C = max(1 - k, 0)

use crate::error::{SanosError, SanosResult};
use statrs::distribution::{ContinuousCDF, Normal};

/// Standard normal CDF N(x).
#[inline]
pub fn norm_cdf(x: f64) -> SanosResult<f64> {
    if !x.is_finite() {
        return Err(SanosError::NonFinite { field: "x", value: x });
    }
    // Safe unwrap: std normal params are valid.
    let n = Normal::new(0.0, 1.0).expect("Normal(0,1) must be constructible");
    Ok(n.cdf(x))
}

/// Forward-normalized Black–Scholes call price with forward F = 1.
///
/// Inputs:
/// - `k`: strike in forward moneyness (k > 0)
/// - `var`: total variance (var >= 0)
///
/// Returns:
/// - `C` in [0, 1] for k >= 0 (numerically, may have tiny eps violations)
pub fn bs_call_forward_norm(k: f64, var: f64) -> SanosResult<f64> {
    if !k.is_finite() {
        return Err(SanosError::NonFinite { field: "k", value: k });
    }
    if !var.is_finite() {
        return Err(SanosError::NonFinite { field: "var", value: var });
    }
    if k <= 0.0 {
        return Err(SanosError::InvalidBound {
            field: "k",
            value: k,
            min: f64::MIN_POSITIVE,
            max: f64::INFINITY,
        });
    }
    if var < 0.0 {
        return Err(SanosError::InvalidBound {
            field: "var",
            value: var,
            min: 0.0,
            max: f64::INFINITY,
        });
    }

    // Deterministic limit as var -> 0.
    if var == 0.0 {
        return Ok((1.0 - k).max(0.0));
    }

    let sqrt_var = var.sqrt();
    // d1, d2 with forward-normalized convention.
    let ln_k = k.ln();
    let d1 = (-ln_k + 0.5 * var) / sqrt_var;
    let d2 = d1 - sqrt_var;

    let n = Normal::new(0.0, 1.0).expect("Normal(0,1) must be constructible");
    let nd1 = n.cdf(d1);
    let nd2 = n.cdf(d2);

    Ok(nd1 - k * nd2)
}
