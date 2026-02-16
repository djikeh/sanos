// src/density/marginal.rs
use crate::error::{SanosError, SanosResult};
use super::DensityTolerances;

/// Discrete marginal distribution at a given maturity:
///     mu = sum_i q_i * delta_{k_i}
///
/// Invariants (validated):
/// - maturity > 0
/// - k_i > 0, strictly increasing
/// - q_i >= 0
/// - sum_i q_i == 1 (within mass tol)
/// - sum_i q_i * k_i == 1 (within mean tol)
#[derive(Debug, Clone)]
pub struct MarginalDensity {
    maturity: f64,
    atoms: Vec<(f64, f64)>, // (k, q)
}

impl MarginalDensity {
    pub fn new(maturity: f64, mut atoms: Vec<(f64, f64)>, tol: DensityTolerances) -> SanosResult<Self> {
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
        if atoms.is_empty() {
            return Err(SanosError::EmptyCollection { what: "MarginalDensity.atoms" });
        }

        // Sort by strike
        atoms.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let md = Self { maturity, atoms };
        md.validate(tol)?;
        Ok(md)
    }

    #[inline]
    pub fn maturity(&self) -> f64 {
        self.maturity
    }

    #[inline]
    pub fn atoms(&self) -> &[(f64, f64)] {
        &self.atoms
    }

    /// Call-transform:
    ///     C(kappa) = E[(K - kappa)^+] = sum_i q_i * (k_i - kappa)^+
    pub fn call(&self, kappa: f64) -> SanosResult<f64> {
        if !kappa.is_finite() {
            return Err(SanosError::NonFinite { field: "kappa", value: kappa });
        }
        let mut acc = 0.0_f64;
        for &(k, q) in &self.atoms {
            let payoff = (k - kappa).max(0.0);
            acc += q * payoff;
        }
        Ok(acc)
    }

    pub fn validate(&self, tol: DensityTolerances) -> SanosResult<()> {
        // Validate atoms are finite and strictly increasing in k
        for (i, &(k, q)) in self.atoms.iter().enumerate() {
            if !k.is_finite() {
                return Err(SanosError::NonFinite { field: "k", value: k });
            }
            if !q.is_finite() {
                return Err(SanosError::NonFinite { field: "q", value: q });
            }
            if k <= 0.0 {
                return Err(SanosError::InvalidBound {
                    field: "k",
                    value: k,
                    min: f64::MIN_POSITIVE,
                    max: f64::INFINITY,
                });
            }
            if q < 0.0 {
                return Err(SanosError::InvalidBound {
                    field: "q",
                    value: q,
                    min: 0.0,
                    max: f64::INFINITY,
                });
            }
            if i > 0 {
                let k_prev = self.atoms[i - 1].0;
                if k <= k_prev {
                    if (k - k_prev).abs() == 0.0 {
                        return Err(SanosError::DuplicateKey { what: "atom strike", value: k });
                    }
                    return Err(SanosError::InvalidOrdering { msg: "atom strikes must be strictly increasing" });
                }
            }
        }

        // Validate mass and mean
        let mut mass = 0.0_f64;
        let mut mean = 0.0_f64;
        for &(k, q) in &self.atoms {
            mass += q;
            mean += q * k;
        }

        if (mass - 1.0).abs() > tol.mass {
            return Err(SanosError::InvalidOrdering {
                msg: "marginal mass constraint violated (sum q != 1 within tolerance)",
            });
        }
        if (mean - 1.0).abs() > tol.mean {
            return Err(SanosError::InvalidOrdering {
                msg: "marginal mean constraint violated (sum q*k != 1 within tolerance)",
            });
        }

        Ok(())
    }
}
