use sanos::backbone::bs::bs_call_forward_norm;
use sanos::backbone::{BackboneConfig, BsTimeChangedConfig};
use sanos::calibration::{calibrate_with_stats, CalibrationConfig};
use sanos::fit::{
    compute_raw_linear_density, project_density_with_martingale_constraints, FitConfig,
    LpSolverConfig,
};
use sanos::grid::StrikeGridPolicyConfig;
use sanos::interp::TimeInterpConfig;
use sanos::market::{CallQuote, OptionBook, OptionChain};

fn sample_book_with_local_arbitrage_noise() -> OptionBook {
    let strikes = [0.8, 0.9, 1.0, 1.1, 1.2];
    let maturities = [0.5, 1.0];
    let vars = [0.04, 0.09];

    let mut chains = Vec::new();
    for (j, &t) in maturities.iter().enumerate() {
        let mut quotes = Vec::new();
        for (i, &k) in strikes.iter().enumerate() {
            let mut mid = bs_call_forward_norm(k, vars[j]).unwrap();
            if j == 0 && i == 2 {
                // Intentional local bump to break discrete convexity on this slice.
                mid = (mid + 0.03).min(0.95);
            }
            let bid = (mid - 0.002).max(0.0);
            let ask = (mid + 0.002).min(1.0);
            quotes.push(CallQuote::new(k, bid, ask, 1.0).unwrap());
        }
        chains.push(OptionChain::new(t, quotes).unwrap());
    }

    OptionBook::new(chains).unwrap()
}

#[test]
fn projected_linear_density_is_feasible() {
    let strikes = vec![0.75, 0.9, 1.0, 1.1, 1.3];
    let calls = vec![0.34, 0.23, 0.20, 0.19, 0.11];

    let raw = compute_raw_linear_density(&strikes, &calls).unwrap();
    let projected = project_density_with_martingale_constraints(
        &strikes,
        &raw.density,
        &LpSolverConfig::Microlp,
        1e-10,
    )
    .unwrap();

    let mass: f64 = projected.iter().sum();
    let mean: f64 = projected
        .iter()
        .zip(strikes.iter())
        .map(|(p, k)| p * k)
        .sum();
    let min = projected
        .iter()
        .copied()
        .fold(f64::INFINITY, |a, b| a.min(b));

    assert!(min >= -1e-9, "projected density has negative atom: {min}");
    assert!((mass - 1.0).abs() <= 1e-8, "mass mismatch: {mass}");
    assert!((mean - 1.0).abs() <= 1e-8, "mean mismatch: {mean}");
}

#[test]
fn calibration_runs_with_linear_density_initialization_path() {
    let book = sample_book_with_local_arbitrage_noise();

    let mut fit = FitConfig::default();
    fit.solver = LpSolverConfig::Microlp;
    fit.initialization.enabled = true;
    fit.initialization.anchor_l1_weight = 1e-3;

    let cfg = CalibrationConfig {
        backbone: BackboneConfig::BsTimeChanged(BsTimeChangedConfig::default()),
        grid: StrikeGridPolicyConfig::default(),
        fit,
        time_interp: TimeInterpConfig::AtmVarianceTime,
    };

    let run =
        calibrate_with_stats(&book, &cfg).expect("calibration with initialization must succeed");
    let init = run
        .stats
        .initialization
        .expect("initialization diagnostics must be populated");
    assert_eq!(init.diagnostics.len(), book.len());
    assert!(init
        .diagnostics
        .iter()
        .all(|d| (d.projected.mass - 1.0).abs() <= 1e-8));
    assert!(init
        .diagnostics
        .iter()
        .all(|d| (d.projected.mean - 1.0).abs() <= 1e-8));

    for chain in book.chains() {
        for q in chain.quotes() {
            let c = run
                .surface
                .call(chain.maturity(), q.k)
                .expect("surface call should be finite");
            assert!(c.is_finite());
        }
    }
}
