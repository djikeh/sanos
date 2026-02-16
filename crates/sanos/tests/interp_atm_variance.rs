// tests/interp_atm_variance.rs
use sanos::backbone::bs::{bs_call_forward_norm, bs_implied_atm_var_from_call};
use sanos::interp::{AtmVarianceTime, TimeInterpolator};

#[test]
fn atm_var_inversion_roundtrip() {
    let w = 0.09;
    let c = bs_call_forward_norm(1.0, w).unwrap();
    let w2 = bs_implied_atm_var_from_call(c).unwrap();
    assert!((w - w2).abs() < 1e-10);
}

#[test]
fn atm_variance_time_returns_alpha_in_0_1() {
    let interp = AtmVarianceTime;

    let maturities = vec![0.5, 1.0];
    let c0 = bs_call_forward_norm(1.0, 0.04).unwrap();
    let c1 = bs_call_forward_norm(1.0, 0.09).unwrap();
    let atm_calls = vec![c0, c1];

    let (_j, a) = interp.alpha(0.75, &maturities, &atm_calls).unwrap();
    assert!(a >= 0.0 && a <= 1.0);
}
