// src/surface/sanos_surface.rs
use std::sync::{Arc, OnceLock};
use std::fmt::Debug;

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
/// - q: MartingaleDensity (sequence of discrete marginals)
/// - interp: time interpolation (alpha)
#[derive(Debug, Clone)]
pub struct SanosSurface<M: YModel> {
    y: Arc<M>,
    q: MartingaleDensity,
    interp: Arc<dyn TimeInterpolator>,
    maturities: Vec<f64>,
    cache: Arc<SurfaceCache>,
}

impl <M: YModel> SanosSurface<M> {
    pub fn new(y: Arc<M>, q: MartingaleDensity, interp: Arc<dyn TimeInterpolator>) -> Self {
        let maturities: Vec<f64> = q.marginals().iter().map(|m| m.maturity()).collect();

        Self {
            y,
            q,
            interp,
            maturities,
            cache: Arc::new(SurfaceCache::default()),
        }
    }

    #[inline]
    pub fn y(&self) -> &Arc<M> {
        &self.y
    }

    #[inline]
    pub fn martingale_density(&self) -> &MartingaleDensity {
        &self.q
    }

    /// Call price C(T, K) (normalized convention v0).
    ///
    /// Implements:
    ///   Ĉ_j(K) = sum_i q_{j,i} E[(K_{j,i} Y_{T_j} - K)^+]
    ///   C(T,K) = (1-alpha) Ĉ_j(K) + alpha Ĉ_{j+1}(K), T in [T_j, T_{j+1}]
    pub fn call(&self, maturity: f64, strike: f64) -> SanosResult<f64> {
        if !strike.is_finite() {
            return Err(SanosError::NonFinite { field: "strike", value: strike });
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
            return Err(SanosError::InvalidOrdering { msg: "SanosSurface requires at least 2 marginals" });
        }

        let atm_calls = self.atm_calls_cached()?;
        let (j, alpha) = self.interp.alpha(maturity, &self.maturities, atm_calls)?;

        let c0 = self.slice_call(j, strike)?;
        let c1 = self.slice_call(j + 1, strike)?;

        // convex combination
        Ok((1.0 - alpha) * c0 + alpha * c1)
    }

    /// Compute slice call Ĉ_j(K) at node maturity T_j.
    fn slice_call(&self, j: usize, strike: f64) -> SanosResult<f64> {
        let m = self
            .q
            .marginals()
            .get(j)
            .ok_or(SanosError::InvalidOrdering { msg: "slice index out of range" })?;

        let tj = m.maturity();
        let mut acc = 0.0_f64;

        for &(k_i, q_i) in m.atoms() {
            // E[(k_i Y_{Tj} - K)^+]
            let v = self.y.call(tj, k_i, strike)?;
            acc += q_i * v;
        }

        Ok(acc)
    }

    fn compute_atm_calls(&self) -> SanosResult<Vec<f64>> {
        let mut out = Vec::with_capacity(self.maturities.len());
        for j in 0..self.maturities.len() {
            out.push(self.slice_call(j, 1.0)?);
        }
        Ok(out)
    }

    fn atm_calls_cached(&self) -> SanosResult<&[f64]> {
        let res = self.cache.atm_calls.get_or_init(|| self.compute_atm_calls());
        match res {
            Ok(values) => Ok(values.as_slice()),
            Err(err) => Err(err.clone()),
        }
    }
}
