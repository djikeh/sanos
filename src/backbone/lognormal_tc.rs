// src/backbone/lognormal_tc.rs
use crate::backbone::bs::bs_call_forward_norm;
use crate::backbone::y_model::YModel;
use crate::error::{SanosError, SanosResult};
use crate::term::PiecewiseLinearCurve;

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
}

impl YModel for TimeChangedLognormal {
    fn call(&self, maturity: f64, a: f64, b: f64) -> SanosResult<f64> {
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

        let v = self.var(maturity)?;
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
