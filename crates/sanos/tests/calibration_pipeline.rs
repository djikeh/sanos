use sanos::backbone::bs::{bs_call_forward_norm, bs_implied_atm_var_from_call};
use sanos::backbone::{build_backbone, BackboneConfig, BsTimeChangedConfig};
use sanos::calibration::{calibrate, CalibrationConfig};
use sanos::density::DensityTolerances;
use sanos::error::SanosError;
use sanos::fit::{
    build_kernels, extract_density, solve_lp, FitConfig, LpBuilder, LpSolverConfig, ObjectiveConfig,
    SanosLpBuilder,
};
use sanos::grid::{build_strike_grids, StrikeGridPolicyConfig};
use sanos::interp::TimeInterpConfig;
use sanos::market::{CallQuote, OptionBook, OptionChain};
use sanos::surface::SanosSurface;

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

fn snapshot_calibration_config() -> CalibrationConfig {
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
fn snapshot_fixture_is_realistic_option_book() {
    let book = load_book_from_snapshot();
    assert_eq!(book.len(), 5);

    for chain in book.chains() {
        assert_eq!(chain.quotes().len(), 31);
        assert!(chain.quotes().windows(2).all(|w| w[1].k > w[0].k));
        assert!(chain.quotes().iter().any(|q| (q.k - 1.0).abs() < 1e-12));
    }
}

#[test]
fn calibrate_pipeline_step_by_step_from_real_book() {
    let book = load_book_from_snapshot();
    let cfg = snapshot_calibration_config();
    let eta = match &cfg.backbone {
        BackboneConfig::BsTimeChanged(bs_cfg) => bs_cfg.eta,
    };

    // Step 1: backbone
    let y = build_backbone(&book, &cfg.backbone).expect("backbone build must succeed");
    for chain in book.chains() {
        let t = chain.maturity();
        let atm = chain
            .quotes()
            .iter()
            .find(|q| (q.k - 1.0).abs() < 1e-12)
            .expect("ATM quote k=1.0 must exist");
        let y_atm = y.call(t, 1.0, 1.0).expect("ATM call must be computable");
        let w_mkt = bs_implied_atm_var_from_call(atm.mid()).expect("ATM mid must map to implied variance");
        let expected = bs_call_forward_norm(1.0, eta * w_mkt).expect("scaled ATM call must be computable");
        assert!(
            (y_atm - expected).abs() <= 5e-8,
            "backbone ATM mismatch at T={t}: model={y_atm}, expected={expected}, eta={eta}"
        );
    }

    // Step 2: strike grids
    let atm_policy = match &cfg.backbone {
        BackboneConfig::BsTimeChanged(bs_cfg) => bs_cfg.atm_policy.build().expect("ATM policy must build"),
    };
    let grids = build_strike_grids(&book, atm_policy.as_ref(), &cfg.grid).expect("grid build must succeed");
    assert_eq!(grids.len(), book.len());
    for (chain, grid) in book.chains().iter().zip(grids.iter()) {
        assert_eq!(chain.maturity(), grid.maturity());
        assert!(grid.strikes().windows(2).all(|w| w[1] > w[0]));
        assert!(grid.strikes().iter().any(|&k| (k - 1.0).abs() < 1e-12));
        for q in chain.quotes() {
            let found = grid.strikes().binary_search_by(|k| k.partial_cmp(&q.k).unwrap()).is_ok();
            assert!(found, "grid must keep market strike k={} at T={}", q.k, chain.maturity());
        }
    }

    // Step 3: kernels
    let kernels = build_kernels(&book, &grids, &y, &cfg.fit.kernel).expect("kernel build must succeed");
    assert_eq!(kernels.c.len(), book.len());
    assert_eq!(kernels.transitions.len(), book.len() - 1);
    for (j, kc) in kernels.c.iter().enumerate() {
        assert_eq!(kc.c.nrows, book.chains()[j].quotes().len());
        assert_eq!(kc.c.ncols, grids[j].strikes().len());
        assert!(kc.c.data.iter().all(|&x| x.is_finite() && x >= 0.0));
    }
    for tr in &kernels.transitions {
        assert!(tr.u.data.iter().all(|&x| x.is_finite() && x >= 0.0));
        assert!(tr.r.data.iter().all(|&x| x.is_finite() && x >= 0.0));
    }

    // Step 4: LP build
    let lp_builder = SanosLpBuilder::default();
    let built_lp = lp_builder
        .build(&book, &kernels, &cfg.fit)
        .expect("LP build must succeed");
    assert!(!built_lp.model.vars.is_empty());
    assert!(!built_lp.model.constraints.is_empty());
    assert_eq!(built_lp.layout.q_var_ids.len(), book.len());
    for (qj, grid) in built_lp.layout.q_var_ids.iter().zip(grids.iter()) {
        assert_eq!(qj.len(), grid.strikes().len());
    }

    // Step 5: LP solve
    let sol = match solve_lp(&built_lp.model, &cfg.fit.solver) {
        Ok(sol) => sol,
        Err(err) if is_missing_cbc(&err) => return,
        Err(err) => panic!("LP solve must succeed: {err:?}"),
    };
    assert_eq!(sol.values.len(), built_lp.model.vars.len());
    assert!(sol.values.iter().all(|x| x.is_finite()));

    // Step 6: density extraction
    let q = extract_density(&built_lp.layout, &sol, &grids).expect("density extraction must succeed");
    let tol = DensityTolerances::from_tol(1e-10).unwrap();
    q.validate_marginals(tol).expect("marginals must be valid");
    q.validate_convex_order(tol).expect("convex order must hold");

    // Step 7: surface assembly and nodal checks
    let interp = cfg.time_interp.build().expect("time interpolator must build");
    let surface = SanosSurface::new(y.clone(), q, interp);

    let eps = 5e-6;
    for chain in book.chains() {
        let t = chain.maturity();
        for quote in chain.quotes() {
            let c = surface.call(t, quote.k).expect("surface call must be computable");
            assert!(c + eps >= quote.bid, "T={t}, k={}, c={c}, bid={}", quote.k, quote.bid);
            assert!(c <= quote.ask + eps, "T={t}, k={}, c={c}, ask={}", quote.k, quote.ask);
        }
    }

    // Full orchestrator should also work on the same real book.
    let full = match calibrate(&book, &cfg) {
        Ok(surface) => surface,
        Err(err) if is_missing_cbc(&err) => return,
        Err(err) => panic!("full calibrate must succeed: {err:?}"),
    };
    for chain in book.chains() {
        let t = chain.maturity();
        for quote in chain.quotes() {
            let c = full.call(t, quote.k).expect("surface call must be computable");
            assert!(c + eps >= quote.bid, "T={t}, k={}, c={c}, bid={}", quote.k, quote.bid);
            assert!(c <= quote.ask + eps, "T={t}, k={}, c={c}, ask={}", quote.k, quote.ask);
        }
    }
}
