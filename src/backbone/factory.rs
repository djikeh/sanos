use crate::backbone::builder::build_time_changed_lognormal_from_book;
use crate::backbone::config::BsTimeChangedConfig;
use crate::backbone::lognormal_tc::TimeChangedLognormal;
use crate::error::SanosResult;
use crate::market::OptionBook;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum BackboneConfig {
    /// Black–Scholes backbone in the "lognormal time-changed" form.
    BsTimeChanged(BsTimeChangedConfig),

    // Future extensions:
    // NormalTimeChanged(...),
    // HestonApprox(...),
    // LocalVolDLV(...),
}

/// Runtime backbone instance built from market data (OptionBook) and config.
///
/// Note: we intentionally keep this as an enum rather than a `dyn YModel`,
/// because `YModel` is not object-safe due to its associated `Smoothing` type.
#[derive(Debug, Clone)]
pub enum BuiltBackbone {
    BsTimeChanged(TimeChangedLognormal),
}

impl BuiltBackbone {
    pub fn name(&self) -> &'static str {
        match self {
            BuiltBackbone::BsTimeChanged(_) => "bs_time_changed",
        }
    }
}

pub fn build_backbone(book: &OptionBook, cfg: &BackboneConfig) -> SanosResult<BuiltBackbone> {
    match cfg {
        BackboneConfig::BsTimeChanged(bs_cfg) => {
            let model = build_time_changed_lognormal_from_book(book, bs_cfg)?;
            Ok(BuiltBackbone::BsTimeChanged(model))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backbone::bs::bs_call_forward_norm;
    use crate::backbone::{AtmMidPolicyConfig, BsTimeChangedConfig};
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
    fn build_backbone_returns_bs_time_changed_variant() {
        let book = book_from_pairs(&[(0.5, 0.04), (1.0, 0.09)]);
        let cfg = BackboneConfig::BsTimeChanged(BsTimeChangedConfig::default());

        let built = build_backbone(&book, &cfg).unwrap();
        assert_eq!(built.name(), "bs_time_changed");
        assert!(matches!(built, BuiltBackbone::BsTimeChanged(_)));
    }

    #[test]
    fn built_backbone_name_is_stable_for_bs_variant() {
        let book = book_from_pairs(&[(0.5, 0.04)]);
        let cfg = BackboneConfig::BsTimeChanged(BsTimeChangedConfig::default());
        let built = build_backbone(&book, &cfg).unwrap();
        assert_eq!(built.name(), "bs_time_changed");
    }

    #[test]
    fn build_backbone_propagates_invalid_var_floor() {
        let book = book_from_pairs(&[(0.5, 0.04)]);
        let cfg = BackboneConfig::BsTimeChanged(BsTimeChangedConfig {
            atm_policy: AtmMidPolicyConfig::default(),
            var_floor: -1e-6,
            enforce_non_decreasing: true,
        });

        let err = build_backbone(&book, &cfg).unwrap_err();
        match err {
            SanosError::InvalidBound { field, .. } => assert_eq!(field, "bs_time_changed.var_floor"),
            _ => panic!("unexpected error variant: {err:?}"),
        }
    }
}
