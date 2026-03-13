use crate::backbone::builder::build_time_changed_lognormal_from_book;
use crate::backbone::config::BsTimeChangedConfig;
use crate::backbone::y_model::YModel;
use crate::error::SanosResult;
use crate::market::{AtmMidPolicy, OptionBook};
use std::sync::Arc;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum BackboneConfig {
    /// Black-Scholes backbone in the "lognormal time-changed" form.
    BsTimeChanged(BsTimeChangedConfig),
    // Future extensions:
    // NormalTimeChanged(...),
    // HestonApprox(...),
    // LocalVolDLV(...),
}

impl BackboneConfig {
    pub fn build_atm_mid_policy(&self) -> SanosResult<Box<dyn AtmMidPolicy>> {
        match self {
            BackboneConfig::BsTimeChanged(bs_cfg) => bs_cfg.atm_policy.build(),
        }
    }

    pub fn eta(&self) -> f64 {
        match self {
            BackboneConfig::BsTimeChanged(bs_cfg) => bs_cfg.eta,
        }
    }
}

pub fn build_backbone(book: &OptionBook, cfg: &BackboneConfig) -> SanosResult<Arc<dyn YModel>> {
    Ok(build_backbone_with_total_variances(book, cfg)?.0)
}

/// Build the runtime backbone and return ATM total variances `W(T_j)` on book maturities.
pub fn build_backbone_with_total_variances(
    book: &OptionBook,
    cfg: &BackboneConfig,
) -> SanosResult<(Arc<dyn YModel>, Vec<f64>)> {
    match cfg {
        BackboneConfig::BsTimeChanged(bs_cfg) => {
            let model = build_time_changed_lognormal_from_book(book, bs_cfg)?;
            let mut total_variances = Vec::with_capacity(book.len());
            for chain in book.chains() {
                total_variances.push(model.var(chain.maturity())?);
            }
            Ok((Arc::new(model), total_variances))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backbone::bs::bs_call_forward_norm;
    use crate::backbone::BsTimeChangedConfig;
    use crate::error::SanosError;
    use crate::market::{CallQuote, OptionChain};

    fn chain_from_atm_var(maturity: f64, atm_var: f64) -> OptionChain {
        let atm_call = bs_call_forward_norm(1.0, atm_var).unwrap();
        let q = CallQuote::new(1.0, atm_call, atm_call, 1.0).unwrap();
        OptionChain::new(maturity, vec![q]).unwrap()
    }

    fn book_from_pairs(pairs: &[(f64, f64)]) -> OptionBook {
        let chains: Vec<OptionChain> = pairs
            .iter()
            .map(|&(t, w)| chain_from_atm_var(t, w))
            .collect();
        OptionBook::new(chains).unwrap()
    }

    #[test]
    fn build_backbone_returns_model_that_prices_calls() {
        let book = book_from_pairs(&[(0.5, 0.04), (1.0, 0.09)]);
        let cfg = BackboneConfig::BsTimeChanged(BsTimeChangedConfig {
            eta: 1.0 - 1e-12,
            var_floor: 0.0,
            enforce_non_decreasing: false,
            ..BsTimeChangedConfig::default()
        });

        let built = build_backbone(&book, &cfg).unwrap();
        let c = built.call(0.5, 1.0, 1.0).unwrap();
        let expected = bs_call_forward_norm(1.0, 0.04).unwrap();
        assert!((c - expected).abs() < 1e-12);
    }

    #[test]
    fn build_backbone_uses_eta_scaling() {
        let book = book_from_pairs(&[(0.5, 0.04)]);
        let cfg = BackboneConfig::BsTimeChanged(BsTimeChangedConfig {
            var_floor: 0.0,
            enforce_non_decreasing: false,
            ..BsTimeChangedConfig::default()
        });
        let built = build_backbone(&book, &cfg).unwrap();
        let c = built.call(0.5, 1.0, 1.0).unwrap();
        let expected = bs_call_forward_norm(1.0, 0.01).unwrap();
        assert!((c - expected).abs() < 1e-12);
    }

    #[test]
    fn build_backbone_propagates_invalid_var_floor() {
        let book = book_from_pairs(&[(0.5, 0.04)]);
        let cfg = BackboneConfig::BsTimeChanged(BsTimeChangedConfig {
            var_floor: -1e-6,
            ..BsTimeChangedConfig::default()
        });

        let err = build_backbone(&book, &cfg).unwrap_err();
        match err {
            SanosError::InvalidBound { field, .. } => {
                assert_eq!(field, "bs_time_changed.var_floor")
            }
            _ => panic!("unexpected error variant: {err:?}"),
        }
    }
}
