// tests/grid.rs
use sanos::grid::{MarketAnchored, StrikeGridPolicy};
use sanos::market::{CallQuote, NearestOrLinearLogMoneyness, OptionBook, OptionChain};

#[test]
fn market_anchored_builds_grid_with_atm_and_wings() {
    let q1 = CallQuote::new(0.9, 0.25, 0.27, 1.0).unwrap();
    let q2 = CallQuote::new(1.1, 0.16, 0.18, 1.0).unwrap();
    let c1 = OptionChain::new(0.5, vec![q1, q2]).unwrap();

    let q3 = CallQuote::new(0.85, 0.30, 0.31, 1.0).unwrap();
    let q4 = CallQuote::new(1.15, 0.12, 0.13, 1.0).unwrap();
    let c2 = OptionChain::new(1.0, vec![q3, q4]).unwrap();

    let book = OptionBook::new(vec![c2, c1]).unwrap();

    let pol = NearestOrLinearLogMoneyness::default();
    let grid_pol = MarketAnchored::default();

    let grids = grid_pol.build(&book, &pol).unwrap();
    assert_eq!(grids.len(), 2);

    for g in grids {
        // Must include ATM (1.0) within tolerance, and have at least market points + wings
        let strikes = g.strikes();
        assert!(strikes.len() >= 2);
        assert!(strikes.windows(2).all(|w| w[1] > w[0]));

        let has_atm = strikes.iter().any(|&k| (k.ln()).abs() < 1e-2);
        assert!(has_atm);
    }
}
