// src/market/quote.rs
use crate::error::{SanosError, SanosResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CallQuote {
    pub k: f64,      // strike in forward moneyness (k > 0)
    pub bid: f64,    // normalized call bid in [0,1]
    pub ask: f64,    // normalized call ask in [0,1]
    pub weight: f64, // >= 0
}

impl CallQuote {
    pub fn new(k: f64, bid: f64, ask: f64, weight: f64) -> SanosResult<Self> {
        validate_finite("k", k)?;
        validate_finite("bid", bid)?;
        validate_finite("ask", ask)?;
        validate_finite("weight", weight)?;

        if k <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "k",
                value: k,
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }

        if !(0.0..=1.0).contains(&bid) {
            return Err(SanosError::InvalidBound {
                field: "bid",
                value: bid,
                min: 0.0,
                max: 1.0,
            });
        }
        if !(0.0..=1.0).contains(&ask) {
            return Err(SanosError::InvalidBound {
                field: "ask",
                value: ask,
                min: 0.0,
                max: 1.0,
            });
        }
        if bid > ask {
            return Err(SanosError::InvalidOrdering {
                msg: "bid must be <= ask",
            });
        }
        if weight < 0.0 {
            return Err(SanosError::InvalidBound {
                field: "weight",
                value: weight,
                min: 0.0,
                max: f64::INFINITY,
            });
        }

        Ok(Self {
            k,
            bid,
            ask,
            weight,
        })
    }

    #[inline]
    pub fn mid(&self) -> f64 {
        0.5 * (self.bid + self.ask)
    }
}

fn validate_finite(field: &'static str, value: f64) -> SanosResult<()> {
    if !value.is_finite() {
        return Err(SanosError::NonFinite { field, value });
    }
    Ok(())
}
