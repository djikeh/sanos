// src/market/book.rs
use crate::error::{SanosError, SanosResult};

use super::atm::AtmMidPolicy;
use super::chain::OptionChain;

#[derive(Debug, Clone)]
pub struct OptionBook {
    chains: Vec<OptionChain>, // strictly increasing maturities
}

impl OptionBook {
    pub fn new(mut chains: Vec<OptionChain>) -> SanosResult<Self> {
        if chains.is_empty() {
            return Err(SanosError::EmptyCollection {
                what: "OptionBook.chains",
            });
        }

        chains.sort_by(|a, b| a.maturity().partial_cmp(&b.maturity()).unwrap());

        for w in chains.windows(2) {
            let t0 = w[0].maturity();
            let t1 = w[1].maturity();
            if t1 <= t0 {
                if (t1 - t0).abs() == 0.0 {
                    return Err(SanosError::DuplicateKey {
                        what: "maturity",
                        value: t0,
                    });
                }
                return Err(SanosError::InvalidOrdering {
                    msg: "maturities must be strictly increasing",
                });
            }
        }

        Ok(Self { chains })
    }

    #[inline]
    pub fn chains(&self) -> &[OptionChain] {
        &self.chains
    }

    pub fn maturities(&self) -> impl Iterator<Item = f64> + '_ {
        self.chains.iter().map(|c| c.maturity())
    }

    pub fn len(&self) -> usize {
        self.chains.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chains.is_empty()
    }

    pub fn atm_mids(&self, policy: &dyn AtmMidPolicy) -> SanosResult<Vec<(f64, f64)>> {
        let mut out = Vec::with_capacity(self.chains.len());
        for c in &self.chains {
            out.push((c.maturity(), c.atm_mid(policy)?));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::CallQuote;

    #[test]
    fn option_book_sorts_and_rejects_duplicates() {
        let q = CallQuote::new(1.0, 0.2, 0.3, 1.0).unwrap();
        let c1 = OptionChain::new(1.0, vec![q]).unwrap();
        let c2 = OptionChain::new(0.5, vec![q]).unwrap();
        let book = OptionBook::new(vec![c1.clone(), c2.clone()]).unwrap();
        let ts: Vec<f64> = book.maturities().collect();
        assert!(ts[0] < ts[1]);

        let c3 = OptionChain::new(1.0, vec![q]).unwrap();
        assert!(OptionBook::new(vec![c1, c3]).is_err());
    }
}
