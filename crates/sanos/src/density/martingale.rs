// src/density/martingale.rs
use crate::error::{SanosError, SanosResult};
use super::{DensityTolerances, MarginalDensity};

/// Time-indexed sequence of marginal densities, sorted by maturity.
///
/// Convex order constraint (discrete check):
/// For each adjacent pair (j, j+1),
///     C_j(kappa) <= C_{j+1}(kappa) for all kappa
/// where C_j is the call-transform of the marginal at maturity T_j.
///
/// We check this on kappa grid:
///     grid = strikes_j ∪ strikes_{j+1}
#[derive(Debug, Clone)]
pub struct MartingaleDensity {
    marginals: Vec<MarginalDensity>,
}

impl MartingaleDensity {
    pub fn new(mut marginals: Vec<MarginalDensity>) -> SanosResult<Self> {
        if marginals.is_empty() {
            return Err(SanosError::EmptyCollection { what: "MartingaleDensity.marginals" });
        }

        // Sort by maturity
        marginals.sort_by(|a, b| a.maturity().partial_cmp(&b.maturity()).unwrap());

        // Strictly increasing maturities
        for w in marginals.windows(2) {
            let t0 = w[0].maturity();
            let t1 = w[1].maturity();
            if t1 <= t0 {
                if (t1 - t0).abs() == 0.0 {
                    return Err(SanosError::DuplicateKey { what: "maturity", value: t0 });
                }
                return Err(SanosError::InvalidOrdering { msg: "maturities must be strictly increasing" });
            }
        }

        Ok(Self { marginals })
    }

    #[inline]
    pub fn marginals(&self) -> &[MarginalDensity] {
        &self.marginals
    }

    pub fn validate_marginals(&self, tol: DensityTolerances) -> SanosResult<()> {
        for m in &self.marginals {
            m.validate(tol)?;
        }
        Ok(())
    }

    pub fn validate_convex_order(&self, tol: DensityTolerances) -> SanosResult<()> {
        if self.marginals.len() < 2 {
            return Ok(());
        }

        for w in self.marginals.windows(2) {
            let m0 = &w[0];
            let m1 = &w[1];

            // Build kappa grid = union of strikes from both marginals
            let mut grid: Vec<f64> = m0.atoms().iter().map(|(k, _)| *k).collect();
            grid.extend(m1.atoms().iter().map(|(k, _)| *k));
            grid.sort_by(|a, b| a.partial_cmp(b).unwrap());
            grid.dedup_by(|a, b| (*a - *b).abs() == 0.0);

            for &kappa in &grid {
                let c0 = m0.call(kappa)?;
                let c1 = m1.call(kappa)?;
                if c0 > c1 + tol.order {
                    return Err(SanosError::InvalidOrdering {
                        msg: "convex order violated: call-transform decreases with maturity",
                    });
                }
            }
        }

        Ok(())
    }
}
