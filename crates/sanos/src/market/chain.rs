// src/market/chain.rs
use crate::error::{SanosError, SanosResult};

use super::atm::AtmMidPolicy;
use super::quote::CallQuote;

#[derive(Debug, Clone)]
pub struct OptionChain {
    maturity: f64,
    quotes: Vec<CallQuote>, // strictly increasing in k
}

impl OptionChain {
    pub fn new(maturity: f64, mut quotes: Vec<CallQuote>) -> SanosResult<Self> {
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
        if quotes.is_empty() {
            return Err(SanosError::EmptyCollection {
                what: "OptionChain.quotes",
            });
        }

        // Sort and ensure strict uniqueness.
        quotes.sort_by(|a, b| a.k.partial_cmp(&b.k).unwrap());

        for w in quotes.windows(2) {
            let k0 = w[0].k;
            let k1 = w[1].k;
            if k1 <= k0 {
                if (k1 - k0).abs() == 0.0 {
                    return Err(SanosError::DuplicateKey {
                        what: "strike k",
                        value: k0,
                    });
                }
                return Err(SanosError::InvalidOrdering {
                    msg: "strikes must be strictly increasing",
                });
            }
        }

        Ok(Self { maturity, quotes })
    }

    #[inline]
    pub fn maturity(&self) -> f64 {
        self.maturity
    }

    #[inline]
    pub fn quotes(&self) -> &[CallQuote] {
        &self.quotes
    }

    pub fn atm_mid(&self, policy: &dyn AtmMidPolicy) -> SanosResult<f64> {
        policy.atm_mid(self)
    }

    /// Returns the index of a quote whose strike is "equal" to k in log-space within tol_log.
    pub fn find_strike(&self, k: f64, tol_log: f64) -> Option<usize> {
        if k <= 0.0 || !k.is_finite() || !tol_log.is_finite() || tol_log < 0.0 {
            return None;
        }
        let target = k.ln();
        self.quotes
            .iter()
            .position(|q| (q.k.ln() - target).abs() <= tol_log)
    }
}
