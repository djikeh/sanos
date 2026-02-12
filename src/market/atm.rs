// src/market/atm.rs
use crate::error::{SanosError, SanosResult};

use super::chain::OptionChain;

pub trait AtmMidPolicy: Send + Sync {
    fn atm_mid(&self, chain: &OptionChain) -> SanosResult<f64>;
}

/// v0: If k=1 exists (within tol_log), return its mid.
/// Else, if 1 is bracketed by two strikes, linearly interpolate in log-moneyness.
/// Else, fall back to the nearest strike in log-moneyness.
#[derive(Debug, Clone, Copy)]
pub struct NearestOrLinearLogMoneyness {
    pub tol_log: f64,
}

impl Default for NearestOrLinearLogMoneyness {
    fn default() -> Self {
        Self { tol_log: 1e-10 }
    }
}

impl AtmMidPolicy for NearestOrLinearLogMoneyness {
    fn atm_mid(&self, chain: &OptionChain) -> SanosResult<f64> {
        let quotes = chain.quotes();
        let ks: Vec<f64> = quotes.iter().map(|q| q.k).collect();
        let mids: Vec<f64> = quotes.iter().map(|q| q.mid()).collect();

        // 1) Exact ATM (within tol)
        if let Some(idx) = chain.find_strike(1.0, self.tol_log) {
            return Ok(mids[idx]);
        }

        // 2) Bracketed interpolation: find i such that k_i < 1 < k_{i+1}
        let mut bracket: Option<(usize, usize)> = None;
        for i in 0..ks.len().saturating_sub(1) {
            if ks[i] < 1.0 && 1.0 < ks[i + 1] {
                bracket = Some((i, i + 1));
                break;
            }
        }

        if let Some((i0, i1)) = bracket {
            let x0 = ks[i0].ln();
            let x1 = ks[i1].ln();
            let x = 0.0_f64; // ln(1) = 0

            if (x1 - x0).abs() <= 1e-16 {
                return Ok(0.5 * (mids[i0] + mids[i1]));
            }

            let w = (x - x0) / (x1 - x0); // in (0,1)
            let atm = (1.0 - w) * mids[i0] + w * mids[i1];
            return Ok(atm);
        }

        // 3) Fallback to nearest strike in log-space
        let mut best_idx = 0usize;
        let mut best_dist = f64::INFINITY;
        for (i, &k) in ks.iter().enumerate() {
            let d = k.ln().abs();
            if d < best_dist {
                best_dist = d;
                best_idx = i;
            }
        }

        if best_dist.is_finite() {
            Ok(mids[best_idx])
        } else {
            Err(SanosError::AtmNotComputable {
                maturity: chain.maturity(),
                reason: "no finite strikes available",
            })
        }
    }
}
