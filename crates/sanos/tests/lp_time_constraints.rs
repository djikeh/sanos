use std::sync::Arc;

use sanos::backbone::bs::bs_call_forward_norm;
use sanos::backbone::{TimeChangedLognormal, YModel};
use sanos::fit::{
    build_kernels, FitConfig, LpBuilder, ObjectiveConfig, SanosLpBuilder,
};
use sanos::grid::StrikeGrid;
use sanos::market::{CallQuote, OptionBook, OptionChain};
use sanos::term::PiecewiseLinearCurve;

fn sample_book_and_grids() -> (OptionBook, Vec<StrikeGrid>, Arc<TimeChangedLognormal>) {
    let t0 = 0.5;
    let t1 = 1.0;
    let w0 = 0.04;
    let w1 = 0.16;

    let q0 = vec![
        CallQuote::new(0.9, bs_call_forward_norm(0.9, w0).unwrap(), bs_call_forward_norm(0.9, w0).unwrap(), 1.0).unwrap(),
        CallQuote::new(1.0, bs_call_forward_norm(1.0, w0).unwrap(), bs_call_forward_norm(1.0, w0).unwrap(), 1.0).unwrap(),
        CallQuote::new(1.1, bs_call_forward_norm(1.1, w0).unwrap(), bs_call_forward_norm(1.1, w0).unwrap(), 1.0).unwrap(),
    ];
    let q1 = vec![
        CallQuote::new(0.9, bs_call_forward_norm(0.9, w1).unwrap(), bs_call_forward_norm(0.9, w1).unwrap(), 1.0).unwrap(),
        CallQuote::new(1.0, bs_call_forward_norm(1.0, w1).unwrap(), bs_call_forward_norm(1.0, w1).unwrap(), 1.0).unwrap(),
        CallQuote::new(1.1, bs_call_forward_norm(1.1, w1).unwrap(), bs_call_forward_norm(1.1, w1).unwrap(), 1.0).unwrap(),
    ];

    let c0 = OptionChain::new(t0, q0).unwrap();
    let c1 = OptionChain::new(t1, q1).unwrap();
    let book = OptionBook::new(vec![c0, c1]).unwrap();

    let grid0 = StrikeGrid::new(t0, vec![0.9, 1.0, 1.1]).unwrap();
    let grid1 = StrikeGrid::new(t1, vec![0.9, 1.0, 1.1]).unwrap();

    let curve = PiecewiseLinearCurve::new(vec![(t0, w0), (t1, w1)]).unwrap();
    let y = Arc::new(TimeChangedLognormal::new(curve, 1.0));

    (book, vec![grid0, grid1], y)
}

#[test]
fn transition_r_uses_previous_maturity() {
    let (_book, grids, y) = sample_book_and_grids();

    // Build a minimal book aligned with the same maturities for kernel construction.
    let t0 = grids[0].maturity();
    let t1 = grids[1].maturity();
    let q0 = vec![CallQuote::new(1.0, y.call(t0, 1.0, 1.0).unwrap(), y.call(t0, 1.0, 1.0).unwrap(), 1.0).unwrap()];
    let q1 = vec![CallQuote::new(1.0, y.call(t1, 1.0, 1.0).unwrap(), y.call(t1, 1.0, 1.0).unwrap(), 1.0).unwrap()];
    let book = OptionBook::new(vec![OptionChain::new(t0, q0).unwrap(), OptionChain::new(t1, q1).unwrap()]).unwrap();

    let kernels = build_kernels(&book, &grids, &(y.clone() as Arc<dyn sanos::backbone::YModel>), &FitConfig::default().kernel).unwrap();
    let tr = &kernels.transitions[0];

    // row/col index 1 corresponds to strike 1.0 in our grid [0.9, 1.0, 1.1]
    let r_11 = tr.r.get(1, 1);
    let expected_prev = y.call(t0, 1.0, 1.0).unwrap();
    let wrong_current = y.call(t1, 1.0, 1.0).unwrap();

    assert!((r_11 - expected_prev).abs() < 1e-12);
    assert!((r_11 - wrong_current).abs() > 1e-6);
}

#[test]
fn lp_builder_adds_time_constraints_when_enabled() {
    let (book, grids, y) = sample_book_and_grids();
    let y_dyn = y as Arc<dyn sanos::backbone::YModel>;
    let fit = FitConfig {
        objective: ObjectiveConfig::HardBidAsk,
        ..FitConfig::default()
    };
    let kernels = build_kernels(&book, &grids, &y_dyn, &fit.kernel).unwrap();

    let builder = SanosLpBuilder::default();
    let built = builder.build(&book, &kernels, &fit).unwrap();

    let expected: usize = kernels.transitions.iter().map(|tr| tr.u.nrows).sum();
    let actual = built
        .model
        .constraints
        .iter()
        .filter(|c| c.name.starts_with("time_"))
        .count();

    assert_eq!(actual, expected);
}

#[test]
fn lp_builder_skips_time_constraints_when_disabled() {
    let (book, grids, y) = sample_book_and_grids();
    let y_dyn = y as Arc<dyn sanos::backbone::YModel>;
    let mut fit = FitConfig {
        objective: ObjectiveConfig::HardBidAsk,
        ..FitConfig::default()
    };
    fit.lp.include_time_constraints = false;

    let kernels = build_kernels(&book, &grids, &y_dyn, &fit.kernel).unwrap();

    let builder = SanosLpBuilder::default();
    let built = builder.build(&book, &kernels, &fit).unwrap();

    let actual = built
        .model
        .constraints
        .iter()
        .filter(|c| c.name.starts_with("time_"))
        .count();
    assert_eq!(actual, 0);
}

#[test]
fn hinge_bid_ask_objective_builds() {
    let (book, grids, y) = sample_book_and_grids();
    let y_dyn = y as Arc<dyn sanos::backbone::YModel>;
    let fit = FitConfig {
        objective: ObjectiveConfig::HingeBidAsk {
            slack_penalty: 1000.0,
            epsilon_inside: 0.0,
        },
        ..FitConfig::default()
    };
    let kernels = build_kernels(&book, &grids, &y_dyn, &fit.kernel).unwrap();

    let builder = SanosLpBuilder::default();
    let built = builder.build(&book, &kernels, &fit).unwrap();

    // Hinge objective introduces non-empty objective terms (slack penalties).
    assert!(!built.model.objective.terms.is_empty());
}

#[test]
fn hinge_objective_uses_inverse_spread_weights() {
    let t = 0.5;

    // Quote 0 has tighter spread => larger paper weight 1/(ask-bid)
    let q0 = CallQuote::new(0.9, 0.25, 0.26, 1.0).unwrap(); // spread 0.01
    let q1 = CallQuote::new(1.1, 0.08, 0.18, 1.0).unwrap(); // spread 0.10

    let book = OptionBook::new(vec![OptionChain::new(t, vec![q0, q1]).unwrap()]).unwrap();
    let grids = vec![StrikeGrid::new(t, vec![0.9, 1.0, 1.1]).unwrap()];

    let curve = PiecewiseLinearCurve::new(vec![(t, 0.04)]).unwrap();
    let y = Arc::new(TimeChangedLognormal::new(curve, 0.25)) as Arc<dyn sanos::backbone::YModel>;

    let fit = FitConfig {
        objective: ObjectiveConfig::HingeBidAsk {
            slack_penalty: 1.0,
            epsilon_inside: 1.0,
        },
        ..FitConfig::default()
    };
    let kernels = build_kernels(&book, &grids, &y, &fit.kernel).unwrap();
    let built = SanosLpBuilder::default().build(&book, &kernels, &fit).unwrap();

    let coef = |var_name: &str| -> f64 {
        let vid = built
            .model
            .vars
            .iter()
            .position(|v| v.name == var_name)
            .expect("variable must exist");
        built
            .model
            .objective
            .terms
            .iter()
            .find(|t| t.var == vid)
            .map(|t| t.coef)
            .unwrap_or(0.0)
    };

    let e0 = coef("e_mid_0_0");
    let e1 = coef("e_mid_0_1");
    let s0 = coef("s_bid_0_0");
    let s1 = coef("s_bid_0_1");

    // With equal quote.weight and epsilon/slack = 1:
    // ratio = (1/0.01) / (1/0.10) = 10
    assert!((e0 / e1 - 10.0).abs() < 1e-12, "e_mid ratio expected 10, got {}", e0 / e1);
    assert!((s0 / s1 - 10.0).abs() < 1e-12, "slack ratio expected 10, got {}", s0 / s1);
}
