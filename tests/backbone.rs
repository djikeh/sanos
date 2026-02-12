// tests/backbone.rs
use sanos::backbone::{bs_call_forward_norm, TimeChangedLognormal, YModel};
use sanos::term::PiecewiseLinearCurve;

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
