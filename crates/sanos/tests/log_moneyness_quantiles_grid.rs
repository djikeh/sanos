use sanos::backbone::bs_call_forward_norm;
use sanos::backbone::{build_backbone_with_total_variances, BackboneConfig, BsTimeChangedConfig};
#[cfg(feature = "lp-microlp")]
use sanos::calibration::{calibrate, CalibrationConfig, ConvexOrderValidationMode};
#[cfg(feature = "lp-microlp")]
use sanos::fit::{FitConfig, LpSolverConfig};
use sanos::grid::{
    build_strike_grids_with_variances, LogMoneynessQuantilesGridConfig, StrikeGridPolicyConfig,
};
#[cfg(feature = "lp-microlp")]
use sanos::interp::TimeInterpConfig;
use sanos::market::{CallQuote, OptionBook, OptionChain};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct IvSurfaceSnapshot {
    maturities: Vec<MaturityNode>,
}

#[derive(Debug, Deserialize)]
struct MaturityNode {
    t: f64,
    quotes: Vec<IvQuote>,
}

#[derive(Debug, Deserialize)]
struct IvQuote {
    k: f64,
    bid_iv: f64,
    ask_iv: f64,
}

fn load_book_from_snapshot() -> OptionBook {
    let json = include_str!("fixtures/tv_equity_like_001.snapshot.json");
    let snap: IvSurfaceSnapshot = serde_json::from_str(json).expect("snapshot must parse");

    let mut chains = Vec::with_capacity(snap.maturities.len());
    for m in snap.maturities {
        let mut quotes = Vec::with_capacity(m.quotes.len());

        for q in m.quotes {
            let bid_var = q.bid_iv * q.bid_iv * m.t;
            let ask_var = q.ask_iv * q.ask_iv * m.t;

            let bid = bs_call_forward_norm(q.k, bid_var).expect("bid price must be computable");
            let ask = bs_call_forward_norm(q.k, ask_var).expect("ask price must be computable");

            quotes.push(CallQuote::new(q.k, bid, ask, 1.0).expect("quote must validate"));
        }

        chains.push(OptionChain::new(m.t, quotes).expect("chain must validate"));
    }

    OptionBook::new(chains).expect("book must validate")
}

fn quantile_grid_config() -> StrikeGridPolicyConfig {
    StrikeGridPolicyConfig::LogMoneynessQuantiles(LogMoneynessQuantilesGridConfig {
        n: 81,
        left_sigmas: 4.5,
        right_sigmas: 3.0,
        alpha_left: 1.8,
        alpha_right: 1.2,
        include_market_strikes: true,
        min_spacing: Some(1e-4),
        k_min: Some(0.65),
        k_max: Some(1.35),
    })
}

#[test]
fn snapshot_quantile_grid_includes_market_strikes_and_stays_bounded() {
    let book = load_book_from_snapshot();
    let backbone_cfg = BackboneConfig::BsTimeChanged(BsTimeChangedConfig::default());
    let (_, total_variances) =
        build_backbone_with_total_variances(&book, &backbone_cfg).expect("backbone must build");
    let atm_policy = match &backbone_cfg {
        BackboneConfig::BsTimeChanged(bs_cfg) => {
            bs_cfg.atm_policy.build().expect("ATM policy must build")
        }
    };
    let grid_cfg = quantile_grid_config();

    let grids = build_strike_grids_with_variances(
        &book,
        atm_policy.as_ref(),
        &grid_cfg,
        Some(&total_variances),
    )
    .expect("grid build must succeed");

    let chain_idx = 2;
    let chain = &book.chains()[chain_idx];
    let grid = &grids[chain_idx];

    assert!(grid.strikes().windows(2).all(|w| w[1] > w[0]));
    assert!(grid.strikes().iter().all(|k| *k >= 0.65 && *k <= 1.35));

    for q in chain.quotes() {
        let found = grid
            .strikes()
            .binary_search_by(|k| k.partial_cmp(&q.k).unwrap())
            .is_ok();
        assert!(
            found,
            "grid must include market strike k={} at T={}",
            q.k,
            chain.maturity()
        );
    }
}

#[cfg(feature = "lp-microlp")]
#[test]
fn calibrate_runs_with_log_moneyness_quantiles_grid_policy() {
    let book = load_book_from_snapshot();
    let mut fit = FitConfig::default();
    fit.solver = LpSolverConfig::Microlp;

    let candidates = [
        LogMoneynessQuantilesGridConfig {
            n: 31,
            left_sigmas: 4.5,
            right_sigmas: 3.0,
            alpha_left: 1.8,
            alpha_right: 1.2,
            include_market_strikes: true,
            min_spacing: None,
            k_min: None,
            k_max: None,
        },
        LogMoneynessQuantilesGridConfig {
            n: 81,
            left_sigmas: 4.5,
            right_sigmas: 3.0,
            alpha_left: 1.8,
            alpha_right: 1.2,
            include_market_strikes: true,
            min_spacing: None,
            k_min: None,
            k_max: None,
        },
        LogMoneynessQuantilesGridConfig {
            n: 101,
            left_sigmas: 4.5,
            right_sigmas: 3.0,
            alpha_left: 1.8,
            alpha_right: 1.2,
            include_market_strikes: true,
            min_spacing: Some(1e-4),
            k_min: None,
            k_max: None,
        },
    ];

    let mut surface = None;
    let mut last_error = None;
    for grid_cfg in candidates {
        let cfg = CalibrationConfig {
            backbone: BackboneConfig::BsTimeChanged(BsTimeChangedConfig::default()),
            grid: StrikeGridPolicyConfig::LogMoneynessQuantiles(grid_cfg),
            fit: fit.clone(),
            market_completion: None,
            time_interp: TimeInterpConfig::AtmVarianceTime,
            convex_order_validation: ConvexOrderValidationMode::Error,
        };

        match calibrate(&book, &cfg) {
            Ok(s) => {
                surface = Some(s);
                break;
            }
            Err(err) => last_error = Some(err),
        }
    }
    let surface = surface.unwrap_or_else(|| {
        panic!(
            "calibration must succeed with at least one LogMoneynessQuantiles configuration; last_error={:?}",
            last_error
        )
    });
    for chain in book.chains() {
        for q in chain.quotes() {
            let c = surface
                .call(chain.maturity(), q.k)
                .expect("surface call must be computable");
            assert!(
                c.is_finite(),
                "non-finite value at T={}, k={}",
                chain.maturity(),
                q.k
            );
            assert!(
                c >= -1e-10,
                "negative value at T={}, k={}",
                chain.maturity(),
                q.k
            );
        }
    }
}
