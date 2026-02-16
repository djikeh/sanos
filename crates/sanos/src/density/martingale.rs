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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn martingale_density_convex_order_passes_for_same_marginals() {
        let tol = DensityTolerances::from_tol(1e-12).unwrap();

        let m1 = MarginalDensity::new(0.5, vec![(0.9, 0.5), (1.1, 0.5)], tol).unwrap();
        let m2 = MarginalDensity::new(1.0, vec![(0.9, 0.5), (1.1, 0.5)], tol).unwrap();

        let md = MartingaleDensity::new(vec![m2.clone(), m1.clone()]).unwrap();
        md.validate_marginals(tol).unwrap();
        md.validate_convex_order(tol).unwrap();
    }

    #[test]
    fn martingale_density_convex_order_fails_when_more_concentrated_later() {
        let tol = DensityTolerances::new(1e-12, 1e-12, 1e-12).unwrap();

        let early = MarginalDensity::new(0.5, vec![(0.8, 0.5), (1.2, 0.5)], tol).unwrap();
        let late = MarginalDensity::new(1.0, vec![(1.0, 1.0)], tol).unwrap();

        let md = MartingaleDensity::new(vec![early, late]).unwrap();
        let err = md.validate_convex_order(tol).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("convex order violated"));
    }
}
