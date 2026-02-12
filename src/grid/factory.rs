// src/grid/factory.rs
use crate::error::SanosResult;
use crate::grid::config::StrikeGridPolicyConfig;
use crate::grid::policy::{MarketAnchored, StrikeGridPolicy};
use crate::grid::StrikeGrid;
use crate::market::{AtmMidPolicy, OptionBook};

pub fn build_strike_grids(
    book: &OptionBook,
    atm: &dyn AtmMidPolicy,
    cfg: &StrikeGridPolicyConfig,
) -> SanosResult<Vec<StrikeGrid>> {
    match cfg {
        StrikeGridPolicyConfig::MarketAnchored(c) => {
            let policy: MarketAnchored = c.to_runtime()?;
            policy.build(book, atm)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SanosError;
    use crate::grid::config::{
        AtmRefineConfig, GridSizeConfig, MarketAnchoredGridConfig, WingsConfig,
    };
    use crate::market::{CallQuote, NearestOrLinearLogMoneyness, OptionChain};

    fn sample_book() -> OptionBook {
        let c1 = OptionChain::new(
            0.5,
            vec![
                CallQuote::new(0.9, 0.22, 0.24, 1.0).unwrap(),
                CallQuote::new(1.1, 0.15, 0.17, 1.0).unwrap(),
            ],
        )
        .unwrap();
        let c2 = OptionChain::new(
            1.0,
            vec![
                CallQuote::new(0.85, 0.28, 0.30, 1.0).unwrap(),
                CallQuote::new(1.15, 0.11, 0.13, 1.0).unwrap(),
            ],
        )
        .unwrap();
        OptionBook::new(vec![c2, c1]).unwrap()
    }

    #[test]
    fn build_strike_grids_market_anchored_dispatches_successfully() {
        let book = sample_book();
        let atm = NearestOrLinearLogMoneyness::default();
        let cfg = StrikeGridPolicyConfig::MarketAnchored(MarketAnchoredGridConfig::default());

        let grids = build_strike_grids(&book, &atm, &cfg).unwrap();
        assert_eq!(grids.len(), book.len());
    }

    #[test]
    fn build_strike_grids_propagates_invalid_config() {
        let book = sample_book();
        let atm = NearestOrLinearLogMoneyness::default();
        let cfg = StrikeGridPolicyConfig::MarketAnchored(MarketAnchoredGridConfig {
            ensure_atm: true,
            wings: WingsConfig { n_left: 2, n_right: 2, ratio: 1.0 },
            atm_refine: AtmRefineConfig::default(),
            grid_size: GridSizeConfig::default(),
            min_strike: 1e-4,
            max_strike: 1e4,
            min_spacing_log: 1e-3,
        });

        let err = build_strike_grids(&book, &atm, &cfg).unwrap_err();
        match err {
            SanosError::InvalidBound { field, .. } => assert_eq!(field, "grid.wings.ratio"),
            _ => panic!("unexpected error variant: {err:?}"),
        }
    }
}
