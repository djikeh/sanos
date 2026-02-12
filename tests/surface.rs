// tests/surface.rs
use std::sync::Arc;

use sanos::backbone::{TimeChangedLognormal, YModel};
use sanos::density::{DensityTolerances, MarginalDensity, MartingaleDensity};
use sanos::interp::LinearTime;
use sanos::surface::SanosSurface;
use sanos::term::PiecewiseLinearCurve;

#[test]
fn sanos_surface_matches_nodes_with_linear_time() {
    let tol = DensityTolerances::from_tol(1e-12).unwrap();

    // Simple two-marginal martingale density (same marginal here for simplicity)
    let m1 = MarginalDensity::new(0.5, vec![(0.9, 0.5), (1.1, 0.5)], tol).unwrap();
    let m2 = MarginalDensity::new(1.0, vec![(0.9, 0.5), (1.1, 0.5)], tol).unwrap();
    let q = MartingaleDensity::new(vec![m2.clone(), m1.clone()]).unwrap();

    // Backbone: time-changed lognormal with some total variance curve
    let var_curve = PiecewiseLinearCurve::new(vec![(0.5, 0.04), (1.0, 0.09)]).unwrap();
    let y = Arc::new(TimeChangedLognormal::new(var_curve)) as Arc<dyn YModel>;

    let interp = Arc::new(LinearTime) as Arc<dyn sanos::interp::TimeInterpolator>;
    let s = SanosSurface::new(y, q, interp);

    let k = 1.0;

    // At T=0.5: should coincide with slice j=0 (after sorting, first maturity is 0.5)
    let c_at_05 = s.call(0.5, k).unwrap();

    // Recompute slice explicitly via the density atoms to cross-check:
    // Ĉ_0(K) = Σ q_i E[(K_i Y_{0.5} - K)^+]
    // We'll rebuild same expression.
    let atoms = m1.atoms();
    let mut slice = 0.0;
    for &(ki, qi) in atoms {
        let v = s.y().call(0.5, ki, k).unwrap();
        slice += qi * v;
    }

    assert!((c_at_05 - slice).abs() < 1e-10);
    assert!(c_at_05 >= -1e-12);
}

#[test]
fn sanos_surface_interpolates_between_nodes() {
    let tol = DensityTolerances::from_tol(1e-12).unwrap();

    // Two different marginals (still mean 1 each) to see interpolation effect
    let m1 = MarginalDensity::new(0.5, vec![(0.8, 0.5), (1.2, 0.5)], tol).unwrap();
    let m2 = MarginalDensity::new(1.0, vec![(0.9, 0.5), (1.1, 0.5)], tol).unwrap();
    let q = MartingaleDensity::new(vec![m1.clone(), m2.clone()]).unwrap();

    let var_curve = PiecewiseLinearCurve::new(vec![(0.5, 0.04), (1.0, 0.09)]).unwrap();
    let y = Arc::new(TimeChangedLognormal::new(var_curve)) as Arc<dyn YModel>;

    let interp = Arc::new(LinearTime) as Arc<dyn sanos::interp::TimeInterpolator>;
    let s = SanosSurface::new(y, q, interp);

    let k = 1.0;
    let c0 = s.call(0.5, k).unwrap();
    let cm = s.call(0.75, k).unwrap();
    let c1 = s.call(1.0, k).unwrap();

    // Linear interpolation implies middle lies between endpoints (not strictly guaranteed numerically but should)
    let lo = c0.min(c1) - 1e-10;
    let hi = c0.max(c1) + 1e-10;
    assert!(cm >= lo && cm <= hi);
}
