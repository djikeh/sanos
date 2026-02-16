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

/// Black–Scholes ATM implied total variance W from forward-normalized ATM call price C(1, W).
///
/// For k = 1 (ATM forward), we have:
///   C = N(d1) - N(d2) with d1 = 0.5*sqrt(W), d2 = -0.5*sqrt(W)
/// => C = 2*N(0.5*sqrt(W)) - 1
/// => N(0.5*sqrt(W)) = (1 + C)/2
/// => W = 4 * (Phi^{-1}((1+C)/2))^2
pub fn bs_implied_atm_var_from_call(call_atm: f64) -> SanosResult<f64> {
    if !call_atm.is_finite() {
        return Err(SanosError::NonFinite { field: "call_atm", value: call_atm });
    }
    if !(0.0..=1.0).contains(&call_atm) {
        return Err(SanosError::InvalidBound {
            field: "call_atm",
            value: call_atm,
            min: 0.0,
            max: 1.0,
        });
    }

    let p = 0.5 * (1.0 + call_atm);

    // clamp away from {0,1} to avoid +/- infinity inverse_cdf
    let eps = 1e-15;
    let p = p.clamp(eps, 1.0 - eps);

    let n = Normal::new(0.0, 1.0).expect("Normal(0,1) must be constructible");
    let x = n.inverse_cdf(p); // x = 0.5*sqrt(W)
    Ok(4.0 * x * x)
}
