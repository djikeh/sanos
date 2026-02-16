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
    var_scale: f64,
}

impl TimeChangedLognormal {
    pub fn new(var_curve: PiecewiseLinearCurve, var_scale: f64) -> Self {
        Self { var_curve, var_scale }
    }

    #[inline]
    pub fn var_scale(&self) -> f64 {
        self.var_scale
    }

    #[inline]
    pub fn var_curve_knots(&self) -> &[(f64, f64)] {
        self.var_curve.knots()
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
    fn call(&self, maturity: f64, a: f64, b: f64) -> SanosResult<f64> {
        self.call_eta(maturity, a, b, self.var_scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backbone::YModel;
    use crate::term::PiecewiseLinearCurve;

    #[test]
    fn time_changed_lognormal_unit_mean_kernel() {
        let curve = PiecewiseLinearCurve::new(vec![(0.5, 0.04), (1.0, 0.09)]).unwrap();
        let y = TimeChangedLognormal::new(curve, 1.0);

        let v = y.call(1.0, 2.0, 0.0).unwrap();
        assert!((v - 2.0).abs() < 1e-12);
    }

    #[test]
    fn time_changed_lognormal_bounds() {
        let curve = PiecewiseLinearCurve::new(vec![(0.5, 0.04), (1.0, 0.09)]).unwrap();
        let y = TimeChangedLognormal::new(curve, 1.0);

        let a = 1.5;
        let val = y.call(1.0, a, 1.0).unwrap();
        assert!(val >= -1e-12);
        assert!(val <= a + 1e-12);
    }
}
