// tests/backbone.rs
use sanos::error::SanosError;
use sanos::market::{CallQuote, OptionBook, OptionChain};
use sanos::term::PiecewiseLinearCurve;
use sanos::backbone::{build_backbone, bs_call_forward_norm, TimeChangedLognormal, YModel, AtmMidPolicyConfig, BackboneConfig, BsTimeChangedConfig, BuiltBackbone};

#[test]
fn bs_call_var_zero_limit() {
    let c = bs_call_forward_norm(1.2, 0.0).unwrap();
    assert!((c - 0.0).abs() < 1e-15);

    let c = bs_call_forward_norm(0.8, 0.0).unwrap();
    assert!((c - 0.2).abs() < 1e-15);
}

#[test]
fn time_changed_lognormal_unit_mean_kernel() {
    let curve = PiecewiseLinearCurve::new(vec![(0.5, 0.04), (1.0, 0.09)]).unwrap();
    let y = TimeChangedLognormal::new(curve);

    let v = y.call(1.0, 2.0, 0.0).unwrap();
    assert!((v - 2.0).abs() < 1e-12);
}

#[test]
fn time_changed_lognormal_bounds() {
    let curve = PiecewiseLinearCurve::new(vec![(0.5, 0.04), (1.0, 0.09)]).unwrap();
    let y = TimeChangedLognormal::new(curve);

    let a = 1.5;
    let val = y.call(1.0, a, 1.0).unwrap();
    assert!(val >= -1e-12);
    assert!(val <= a + 1e-12);
}

fn assert_close(actual: f64, expected: f64, tol: f64) {
    assert!(
        (actual - expected).abs() <= tol,
        "expected {expected}, got {actual}, tol={tol}"
    );
}

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
fn build_backbone_returns_bs_time_changed_and_name() {
    let book = book_from_pairs(&[(0.5, 0.04), (1.0, 0.09)]);
    let cfg = BackboneConfig::BsTimeChanged(BsTimeChangedConfig::default());

    let built = build_backbone(&book, &cfg).unwrap();
    assert_eq!(built.name(), "bs_time_changed");

    match built {
        BuiltBackbone::BsTimeChanged(model) => {
            assert_close(model.var(0.5).unwrap(), 0.04, 1e-10);
            assert_close(model.var(1.0).unwrap(), 0.09, 1e-10);
        }
    }
}

#[test]
fn build_backbone_applies_var_floor() {
    let book = book_from_pairs(&[(0.5, 0.0)]);
    let cfg = BackboneConfig::BsTimeChanged(BsTimeChangedConfig {
        atm_policy: AtmMidPolicyConfig::default(),
        var_floor: 0.02,
        enforce_non_decreasing: true,
    });

    let built = build_backbone(&book, &cfg).unwrap();
    match built {
        BuiltBackbone::BsTimeChanged(model) => {
            assert_close(model.var(0.5).unwrap(), 0.02, 1e-14);
        }
    }
}

#[test]
fn build_backbone_clamps_decreasing_variance_when_enabled() {
    let book = book_from_pairs(&[(0.5, 0.09), (1.0, 0.04)]);
    let cfg = BackboneConfig::BsTimeChanged(BsTimeChangedConfig {
        atm_policy: AtmMidPolicyConfig::default(),
        var_floor: 0.0,
        enforce_non_decreasing: true,
    });

    let built = build_backbone(&book, &cfg).unwrap();
    match built {
        BuiltBackbone::BsTimeChanged(model) => {
            assert_close(model.var(0.5).unwrap(), 0.09, 1e-10);
            assert_close(model.var(1.0).unwrap(), 0.09, 1e-10);
        }
    }
}

#[test]
fn build_backbone_keeps_decreasing_variance_when_disabled() {
    let book = book_from_pairs(&[(0.5, 0.09), (1.0, 0.04)]);
    let cfg = BackboneConfig::BsTimeChanged(BsTimeChangedConfig {
        atm_policy: AtmMidPolicyConfig::default(),
        var_floor: 0.0,
        enforce_non_decreasing: false,
    });

    let built = build_backbone(&book, &cfg).unwrap();
    match built {
        BuiltBackbone::BsTimeChanged(model) => {
            assert_close(model.var(0.5).unwrap(), 0.09, 1e-10);
            assert_close(model.var(1.0).unwrap(), 0.04, 1e-10);
        }
    }
}

#[test]
fn build_backbone_rejects_negative_var_floor() {
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

#[test]
fn build_backbone_rejects_non_finite_var_floor() {
    let book = book_from_pairs(&[(0.5, 0.04)]);
    let cfg = BackboneConfig::BsTimeChanged(BsTimeChangedConfig {
        atm_policy: AtmMidPolicyConfig::default(),
        var_floor: f64::NAN,
        enforce_non_decreasing: true,
    });

    let err = build_backbone(&book, &cfg).unwrap_err();
    match err {
        SanosError::NonFinite { field, .. } => assert_eq!(field, "bs_time_changed.var_floor"),
        _ => panic!("unexpected error variant: {err:?}"),
    }
}

#[test]
fn build_backbone_rejects_negative_atm_tol_log() {
    let book = book_from_pairs(&[(0.5, 0.04)]);
    let cfg = BackboneConfig::BsTimeChanged(BsTimeChangedConfig {
        atm_policy: AtmMidPolicyConfig::NearestOrLinearLogMoneyness { tol_log: -1e-3 },
        var_floor: 0.0,
        enforce_non_decreasing: true,
    });

    let err = build_backbone(&book, &cfg).unwrap_err();
    match err {
        SanosError::InvalidBound { field, .. } => assert_eq!(field, "atm_policy.tol_log"),
        _ => panic!("unexpected error variant: {err:?}"),
    }
}

#[test]
fn build_backbone_rejects_non_finite_atm_tol_log() {
    let book = book_from_pairs(&[(0.5, 0.04)]);
    let cfg = BackboneConfig::BsTimeChanged(BsTimeChangedConfig {
        atm_policy: AtmMidPolicyConfig::NearestOrLinearLogMoneyness { tol_log: f64::NAN },
        var_floor: 0.0,
        enforce_non_decreasing: true,
    });

    let err = build_backbone(&book, &cfg).unwrap_err();
    match err {
        SanosError::NonFinite { field, .. } => assert_eq!(field, "atm_policy.tol_log"),
        _ => panic!("unexpected error variant: {err:?}"),
    }
}

