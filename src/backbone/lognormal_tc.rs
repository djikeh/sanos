// src/backbone/lognormal_tc.rs
use crate::backbone::bs::bs_call_forward_norm;
use crate::backbone::y_model::YModel;
use crate::error::{SanosError, SanosResult};
use crate::term::PiecewiseLinearCurve;

/// Smoothing specification for time-changed lognormal: scale factor for variance.
/// This is a simple parametric form that preserves the lognormal structure of the kernel.
/// The smoothed kernel is E[(a Y_T^eta - b)^+] = a * BSCall(F=1, K=b/a, var=eta * v(T)).
/// Note that eta=1 recovers the original kernel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TCLnSmoothing {
    var_scale: f64,
}

/// Time-changed lognormal martingale backbone:
///     Y_T = exp(B_{v(T)} - 0.5 v(T))
///
/// Kernel:
///     E[(a Y_T - b)^+] = a * BSCall(F=1, K=b/a, var=v(T))
#[derive(Debug, Clone)]
pub struct TimeChangedLognormal {
    var_curve: PiecewiseLinearCurve,
}

impl TimeChangedLognormal {
    pub fn new(var_curve: PiecewiseLinearCurve) -> Self {
        Self { var_curve }
    }

    #[inline]
    pub fn var(&self, maturity: f64) -> SanosResult<f64> {
        let v = self.var_curve.value(maturity)?;
        if v < 0.0 {
            return Err(SanosError::InvalidBound {
                field: "var(T)",
                value: v,
                min: 0.0,
                max: f64::INFINITY,
            });
        }
        Ok(v)
    }

    fn call_eta(&self, maturity: f64, a: f64, b: f64, eta: f64) -> SanosResult<f64> {
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
        if !a.is_finite() {
            return Err(SanosError::NonFinite { field: "a", value: a });
        }
        if !b.is_finite() {
            return Err(SanosError::NonFinite { field: "b", value: b });
        }
        if !eta.is_finite() {
            return Err(SanosError::NonFinite { field: "eta", value: eta });
        }
        if a <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "a",
                value: a,
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }
        if b < 0.0 {
            return Err(SanosError::InvalidBound {
                field: "b",
                value: b,
                min: 0.0,
                max: f64::INFINITY,
            });
        }
        if eta < 0.0 {
            return Err(SanosError::InvalidBound { field: "eta", value: eta, min: 0.0, max: f64::INFINITY, });
        }

        let v = self.var(maturity)?;
        let v = v * eta;
        let k = b / a;

        // If b == 0, then k = 0. But bs_call_forward_norm requires k>0.
        // Use the identity E[(aY)^+] = a E[Y] = a for a martingale with unit mean.
        if b == 0.0 {
            return Ok(a);
        }

        // If k <= 0 due to rounding, treat as near-zero.
        if k <= 0.0 {
            return Ok(a);
        }

        let c = bs_call_forward_norm(k, v)?;
        Ok(a * c)
    }
}

impl YModel for TimeChangedLognormal {
    type Smoothing = TCLnSmoothing;

    fn call(&self, maturity: f64, a: f64, b: f64) -> SanosResult<f64> {
        self.call_eta(maturity, a, b, 1.0)
    }

    fn smoothed_call(&self, maturity: f64, a: f64, b: f64, smoothing: Self::Smoothing) -> SanosResult<f64> {
        self.call_eta(maturity, a, b, smoothing.var_scale)
    }
}
