// tests/density.rs
use sanos::density::{DensityTolerances, MarginalDensity, MartingaleDensity};

#[test]
fn marginal_density_validates_mass_and_mean() {
    let tol = DensityTolerances::from_tol(1e-12).unwrap();

    // Two-point distribution with mean 1:
    // k = 0.8 (q=0.5), k = 1.2 (q=0.5) -> mean = 1.0
    let m = MarginalDensity::new(1.0, vec![(0.8, 0.5), (1.2, 0.5)], tol).unwrap();
    let c_at_1 = m.call(1.0).unwrap();
    assert!(c_at_1 >= 0.0);
}

#[test]
fn marginal_density_rejects_bad_mass() {
    let tol = DensityTolerances::from_tol(1e-12).unwrap();

    // Mass != 1
    let err = MarginalDensity::new(1.0, vec![(1.0, 0.9)], tol).err().unwrap();
    let msg = format!("{err}");
    assert!(msg.contains("marginal mass constraint violated"));
}

#[test]
fn marginal_density_rejects_bad_mean() {
    let tol = DensityTolerances::from_tol(1e-12).unwrap();

    // Mean != 1
    let err = MarginalDensity::new(1.0, vec![(1.1, 1.0)], tol).err().unwrap();
    let msg = format!("{err}");
    assert!(msg.contains("marginal mean constraint violated"));
}

#[test]
fn martingale_density_convex_order_passes_for_same_marginals() {
    let tol = DensityTolerances::from_tol(1e-12).unwrap();

    let m1 = MarginalDensity::new(0.5, vec![(0.9, 0.5), (1.1, 0.5)], tol).unwrap();
    let m2 = MarginalDensity::new(1.0, vec![(0.9, 0.5), (1.1, 0.5)], tol).unwrap();

    let md = MartingaleDensity::new(vec![m2.clone(), m1.clone()]).unwrap(); // intentionally unsorted input
    md.validate_marginals(tol).unwrap();
    md.validate_convex_order(tol).unwrap();
}

#[test]
fn martingale_density_convex_order_fails_when_more_concentrated_later() {
    let tol = DensityTolerances::new(1e-12, 1e-12, 1e-12).unwrap();

    // Early: spread distribution around 1 (gives higher call-transform for many kappa)
    let early = MarginalDensity::new(0.5, vec![(0.8, 0.5), (1.2, 0.5)], tol).unwrap();

    // Late: degenerate at 1 (still mean 1) => call-transform is smaller for kappa < 1
    let late = MarginalDensity::new(1.0, vec![(1.0, 1.0)], tol).unwrap();

    let md = MartingaleDensity::new(vec![early, late]).unwrap();
    let err = md.validate_convex_order(tol).err().unwrap();
    let msg = format!("{err}");
    assert!(msg.contains("convex order violated"));
}
