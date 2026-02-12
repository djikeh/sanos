// src/term/curve.rs
use crate::error::{SanosError, SanosResult};

/// A simple piecewise-linear curve defined by knots (t_i, y_i).
///
/// - Knots are strictly increasing in time.
/// - Values must be finite.
/// - Extrapolation is *flat* outside the knot range:
///   - for T <= t_0: returns y_0
///   - for T >= t_last: returns y_last
#[derive(Debug, Clone)]
pub struct PiecewiseLinearCurve {
    knots: Vec<(f64, f64)>,
}

impl PiecewiseLinearCurve {
    pub fn new(knots: Vec<(f64, f64)>) -> SanosResult<Self> {
        if knots.is_empty() {
            return Err(SanosError::EmptyCollection { what: "curve.knots" });
        }

        // Validate + ensure strictly increasing times
        for (i, (t, y)) in knots.iter().enumerate() {
            if !t.is_finite() {
                return Err(SanosError::NonFinite { field: "t", value: *t });
            }
            if !y.is_finite() {
                return Err(SanosError::NonFinite { field: "y", value: *y });
            }
            if *t <= 0.0 {
                return Err(SanosError::InvalidBound {
                    field: "t",
                    value: *t,
                    min: f64::MIN_POSITIVE,
                    max: f64::INFINITY,
                });
            }
            if i > 0 {
                let t_prev = knots[i - 1].0;
                if *t <= t_prev {
                    if (*t - t_prev).abs() == 0.0 {
                        return Err(SanosError::DuplicateKey {
                            what: "curve time knot",
                            value: *t,
                        });
                    }
                    return Err(SanosError::InvalidOrdering {
                        msg: "curve times must be strictly increasing",
                    });
                }
            }
        }

        Ok(Self { knots })
    }

    #[inline]
    pub fn knots(&self) -> &[(f64, f64)] {
        &self.knots
    }

    /// Evaluate the curve at maturity T using piecewise-linear interpolation and flat extrapolation.
    pub fn value(&self, t: f64) -> SanosResult<f64> {
        if !t.is_finite() {
            return Err(SanosError::NonFinite { field: "t", value: t });
        }
        if t <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "t",
                value: t,
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }

        let (t0, y0) = self.knots[0];
        let (tn, yn) = self.knots[self.knots.len() - 1];

        if t <= t0 {
            return Ok(y0);
        }
        if t >= tn {
            return Ok(yn);
        }

        // Find interval [t_i, t_{i+1}] containing t (linear scan MVP; can binary search later)
        for w in self.knots.windows(2) {
            let (ta, ya) = w[0];
            let (tb, yb) = w[1];
            if ta <= t && t <= tb {
                let denom = tb - ta;
                if denom.abs() <= 1e-16 {
                    return Ok(0.5 * (ya + yb));
                }
                let w = (t - ta) / denom;
                return Ok((1.0 - w) * ya + w * yb);
            }
        }

        // Should be unreachable due to boundary checks.
        Err(SanosError::InvalidOrdering {
            msg: "failed to bracket t in curve evaluation",
        })
    }
}
