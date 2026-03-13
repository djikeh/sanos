// src/surface/sanos_surface.rs
use std::fmt::Debug;
use std::sync::{Arc, OnceLock};

use crate::backbone::YModel;
use crate::density::MartingaleDensity;
use crate::error::{SanosError, SanosResult};
use crate::interp::TimeInterpolator;

#[derive(Debug, Default)]
struct SurfaceCache {
    atm_calls: OnceLock<SanosResult<Vec<f64>>>,
}

/// SANOS surface as per Theorem 3.1:
/// - Y: backbone model exposing call(T, a, b) = E[(a Y_T - b)^+]
/// - q: martingale density (sequence of discrete marginals)
/// - alpha: time interpolation weight
#[derive(Debug, Clone)]
pub struct SanosSurface {
    backbone_model: Arc<dyn YModel>,
    martingale_density: MartingaleDensity,
    time_interpolator: Arc<dyn TimeInterpolator>,
    maturities: Vec<f64>,
    cache: Arc<SurfaceCache>,
}

impl SanosSurface {
    pub fn new(
        backbone_model: Arc<dyn YModel>,
        martingale_density: MartingaleDensity,
        time_interpolator: Arc<dyn TimeInterpolator>,
    ) -> Self {
        let maturities: Vec<f64> = martingale_density
            .marginals()
            .iter()
            .map(|marginal| marginal.maturity())
            .collect();

        Self {
            backbone_model,
            martingale_density,
            time_interpolator,
            maturities,
            cache: Arc::new(SurfaceCache::default()),
        }
    }

    #[inline]
    pub fn y(&self) -> &Arc<dyn YModel> {
        self.backbone_model()
    }

    #[inline]
    pub fn backbone_model(&self) -> &Arc<dyn YModel> {
        &self.backbone_model
    }

    #[inline]
    pub fn martingale_density(&self) -> &MartingaleDensity {
        &self.martingale_density
    }

    /// Call price C(T, K) (normalized convention v0).
    ///
    /// Implements:
    ///   Ĉ_j(K) = sum_i q_{j,i} E[(K_{j,i} Y_{T_j} - K)^+]
    ///   C(T,K) = (1-alpha) Ĉ_j(K) + alpha Ĉ_{j+1}(K), T in [T_j, T_{j+1}]
    pub fn call(&self, maturity: f64, strike: f64) -> SanosResult<f64> {
        if !strike.is_finite() {
            return Err(SanosError::NonFinite {
                field: "strike",
                value: strike,
            });
        }
        if strike <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "strike",
                value: strike,
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }

        if self.maturities.len() < 2 {
            return Err(SanosError::InvalidOrdering {
                msg: "SanosSurface requires at least 2 marginals",
            });
        }

        let atm_calls = self.atm_calls_cached()?;
        let (j, alpha) = self
            .time_interpolator
            .alpha(maturity, &self.maturities, atm_calls)?;

        let c0 = self.slice_call_price(j, strike)?;
        let c1 = self.slice_call_price(j + 1, strike)?;

        // convex combination
        Ok((1.0 - alpha) * c0 + alpha * c1)
    }

    /// Compute slice call Ĉ_j(K) at node maturity T_j.
    fn slice_call_price(&self, j: usize, strike: f64) -> SanosResult<f64> {
        let marginal =
            self.martingale_density
                .marginals()
                .get(j)
                .ok_or(SanosError::InvalidOrdering {
                    msg: "slice index out of range",
                })?;

        let maturity = marginal.maturity();
        let mut acc = 0.0_f64;

        for &(model_strike, atom_weight) in marginal.atoms() {
            // E[(k_i Y_{Tj} - K)^+]
            let kernel_value = self.backbone_model.call(maturity, model_strike, strike)?;
            acc += atom_weight * kernel_value;
        }

        Ok(acc)
    }

    fn compute_atm_calls(&self) -> SanosResult<Vec<f64>> {
        let mut out = Vec::with_capacity(self.maturities.len());
        for j in 0..self.maturities.len() {
            out.push(self.slice_call_price(j, 1.0)?);
        }
        Ok(out)
    }

    fn atm_calls_cached(&self) -> SanosResult<&[f64]> {
        let res = self
            .cache
            .atm_calls
            .get_or_init(|| self.compute_atm_calls());
        match res {
            Ok(values) => Ok(values.as_slice()),
            Err(err) => Err(err.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backbone::TimeChangedLognormal;
    use crate::density::{DensityTolerances, MarginalDensity, MartingaleDensity};
    use crate::interp::LinearTime;
    use crate::term::PiecewiseLinearCurve;

    #[test]
    fn sanos_surface_matches_nodes_with_linear_time() {
        let tol = DensityTolerances::from_tol(1e-12).unwrap();
        let m1 = MarginalDensity::new(0.5, vec![(0.9, 0.5), (1.1, 0.5)], tol).unwrap();
        let m2 = MarginalDensity::new(1.0, vec![(0.9, 0.5), (1.1, 0.5)], tol).unwrap();
        let q = MartingaleDensity::new(vec![m2.clone(), m1.clone()]).unwrap();

        let var_curve = PiecewiseLinearCurve::new(vec![(0.5, 0.04), (1.0, 0.09)]).unwrap();
        let y = Arc::new(TimeChangedLognormal::new(var_curve, 1.0));

        let interp = Arc::new(LinearTime) as Arc<dyn crate::interp::TimeInterpolator>;
        let s = SanosSurface::new(y, q, interp);

        let k = 1.0;
        let c_at_05 = s.call(0.5, k).unwrap();

        let atoms = m1.atoms();
        let mut slice = 0.0;
        for &(ki, qi) in atoms {
            let v = s.y().call(0.5, ki, k).unwrap();
            slice += qi * v;
        }

        assert!((c_at_05 - slice).abs() < 1e-10);
        assert!(c_at_05 >= -1e-12);
    }

    #[test]
    fn sanos_surface_interpolates_between_nodes() {
        let tol = DensityTolerances::from_tol(1e-12).unwrap();
        let m1 = MarginalDensity::new(0.5, vec![(0.8, 0.5), (1.2, 0.5)], tol).unwrap();
        let m2 = MarginalDensity::new(1.0, vec![(0.9, 0.5), (1.1, 0.5)], tol).unwrap();
        let q = MartingaleDensity::new(vec![m1.clone(), m2.clone()]).unwrap();

        let var_curve = PiecewiseLinearCurve::new(vec![(0.5, 0.04), (1.0, 0.09)]).unwrap();
        let y = Arc::new(TimeChangedLognormal::new(var_curve, 1.0));

        let interp = Arc::new(LinearTime) as Arc<dyn crate::interp::TimeInterpolator>;
        let s = SanosSurface::new(y, q, interp);

        let k = 1.0;
        let c0 = s.call(0.5, k).unwrap();
        let cm = s.call(0.75, k).unwrap();
        let c1 = s.call(1.0, k).unwrap();

        let lo = c0.min(c1) - 1e-10;
        let hi = c0.max(c1) + 1e-10;
        assert!(cm >= lo && cm <= hi);
    }
}
