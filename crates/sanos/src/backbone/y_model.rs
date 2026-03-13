use std::fmt::Debug;
// src/backbone/y_model.rs
use crate::error::{SanosError, SanosResult};

/// A single input case for `YModel::sanity_check`.
#[derive(Debug, Clone, Copy)]
pub struct SanityCase {
    pub maturity: f64,
    pub a: f64,
    pub b: f64,
}

/// A minimal report for sanity checks (non-blocking diagnostics).
#[derive(Debug, Clone)]
pub struct SanityReport {
    pub case: SanityCase,
    pub value: f64,
    pub issues: Vec<&'static str>,
}

/// Backbone model for SANOS: exposes the call-kernel
///     call(T, a, b) = E[(a Y_T - b)^+]
///
/// Conventions:
/// - maturity T > 0
/// - a > 0
/// - b >= 0
pub trait YModel: Send + Sync + Debug {
    /// Kernel call: E[(a Y_T - b)^+]
    fn call(&self, maturity: f64, a: f64, b: f64) -> SanosResult<f64>;

    /// A simple linear model for null volatility and sanity checks:
    /// Y_T = 1 deterministically, so call(T, a, b) = (a - b)^+
    fn linear_call(&self, maturity: f64, a: f64, b: f64) -> SanosResult<f64> {
        if !maturity.is_finite() {
            return Err(SanosError::NonFinite {
                field: "maturity",
                value: maturity,
            });
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
            return Err(SanosError::NonFinite {
                field: "a",
                value: a,
            });
        }
        if !b.is_finite() {
            return Err(SanosError::NonFinite {
                field: "b",
                value: b,
            });
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

        Ok((a - b).max(0.0))
    }

    /// Lightweight, non-blocking sanity checks.
    ///
    /// This method never fails unless the underlying `call` fails; it returns a list
    /// of reports with flags if invariants appear violated beyond `tol`.
    fn sanity_check(&self, cases: &[SanityCase], tol: f64) -> SanosResult<Vec<SanityReport>> {
        if !tol.is_finite() || tol < 0.0 {
            return Err(SanosError::InvalidBound {
                field: "tol",
                value: tol,
                min: 0.0,
                max: f64::INFINITY,
            });
        }

        let mut reports = Vec::with_capacity(cases.len());
        for &case in cases {
            let kernel_value = self.call(case.maturity, case.a, case.b)?;
            let mut issues: Vec<&'static str> = Vec::new();

            // Basic finiteness
            if !kernel_value.is_finite() {
                issues.push("non_finite_value");
            }

            // Bounds: 0 <= call <= a (since Y is positive with E[Y]=1)
            if kernel_value < -tol {
                issues.push("below_zero");
            }
            if kernel_value > case.a + tol {
                issues.push("above_a");
            }

            // Unit mean check: call(T, a, 0) should be ~ a (because (aY)^+ = aY and E[Y]=1)
            if case.b == 0.0 && (kernel_value - case.a).abs() > tol.max(tol * case.a.abs().max(1.0))
            {
                issues.push("unit_mean_violation");
            }

            reports.push(SanityReport {
                case,
                value: kernel_value,
                issues,
            });
        }

        Ok(reports)
    }
}
