use sanos::backbone::bs::bs_call_forward_norm;
use sanos::backbone::{BackboneConfig, BsTimeChangedConfig};
use sanos::calibration::{calibrate, CalibrationConfig};
use sanos::density::DensityTolerances;
use sanos::error::SanosError;
use sanos::fit::{FitConfig, LpSolverConfig, ObjectiveConfig};
use sanos::grid::StrikeGridPolicyConfig;
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

fn default_calibration_config_for_snapshot() -> CalibrationConfig {
    let backbone = BackboneConfig::BsTimeChanged(BsTimeChangedConfig {
        eta: 0.25,
        ..BsTimeChangedConfig::default()
    });

    let mut fit = FitConfig::default();
    fit.objective = ObjectiveConfig::HardBidAsk;
    fit.solver = LpSolverConfig::Cbc { msg: false, time_limit_sec: Some(10) };

    CalibrationConfig {
        backbone,
        grid: StrikeGridPolicyConfig::default(),
        fit,
        time_interp: TimeInterpConfig::AtmVarianceTime,
    }
}

fn is_missing_cbc(err: &SanosError) -> bool {
    match err {
        SanosError::External { msg } => {
            let m = msg.to_ascii_lowercase();
            m.contains("cbc") && m.contains("program not found")
        }
        _ => false,
    }
}

#[test]
fn calibrate_snapshot_produces_valid_martingale_density() {
    let book = load_book_from_snapshot();
    let cfg = default_calibration_config_for_snapshot();

    let surface = match calibrate(&book, &cfg) {
        Ok(surface) => surface,
        Err(err) if is_missing_cbc(&err) => return,
        Err(err) => panic!("calibration must succeed: {err:?}"),
    };

    let tol = DensityTolerances::from_tol(1e-10).unwrap();
    let q = surface.martingale_density();
    q.validate_marginals(tol).expect("marginals must be valid");
    q.validate_convex_order(tol).expect("convex order must hold");
}

#[test]
fn calibrated_surface_respects_market_bid_ask_on_nodes() {
    let book = load_book_from_snapshot();
    let cfg = default_calibration_config_for_snapshot();

    let surface = match calibrate(&book, &cfg) {
        Ok(surface) => surface,
        Err(err) if is_missing_cbc(&err) => return,
        Err(err) => panic!("calibration must succeed: {err:?}"),
    };
    let eps = 5e-6;

    for chain in book.chains() {
        let t = chain.maturity();
        for q in chain.quotes() {
            let c = surface.call(t, q.k).expect("surface call must be computable");
            assert!(c + eps >= q.bid, "T={t}, k={}, c={c}, bid={}", q.k, q.bid);
            assert!(c <= q.ask + eps, "T={t}, k={}, c={c}, ask={}", q.k, q.ask);
        }
    }
}
