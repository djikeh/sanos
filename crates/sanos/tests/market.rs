// tests/market.rs
use sanos::prelude::*;

#[test]
fn call_quote_validation() {
    assert!(CallQuote::new(1.0, 0.2, 0.3, 1.0).is_ok());
    assert!(CallQuote::new(1.0, 0.4, 0.3, 1.0).is_err());
    assert!(CallQuote::new(0.0, 0.2, 0.3, 1.0).is_err());
    assert!(CallQuote::new(1.0, -0.1, 0.3, 1.0).is_err());
    assert!(CallQuote::new(1.0, 0.2, 1.1, 1.0).is_err());
}

#[test]
fn option_chain_sorts_and_rejects_duplicates() {
    let q1 = CallQuote::new(1.2, 0.10, 0.12, 1.0).unwrap();
    let q2 = CallQuote::new(0.9, 0.25, 0.27, 1.0).unwrap();
    let chain = OptionChain::new(0.5, vec![q1, q2]).unwrap();
    assert!(chain.quotes()[0].k < chain.quotes()[1].k);

    let q3 = CallQuote::new(1.0, 0.20, 0.21, 1.0).unwrap();
    let q4 = CallQuote::new(1.0, 0.19, 0.22, 1.0).unwrap();
    assert!(OptionChain::new(0.5, vec![q3, q4]).is_err());
}

#[test]
fn option_book_sorts_and_rejects_duplicates() {
    let q = CallQuote::new(1.0, 0.2, 0.3, 1.0).unwrap();
    let c1 = OptionChain::new(1.0, vec![q]).unwrap();
    let c2 = OptionChain::new(0.5, vec![q]).unwrap();
    let book = OptionBook::new(vec![c1.clone(), c2.clone()]).unwrap();
    let ts: Vec<f64> = book.maturities().collect();
    assert!(ts[0] < ts[1]);

    let c3 = OptionChain::new(1.0, vec![q]).unwrap();
    assert!(OptionBook::new(vec![c1, c3]).is_err());
}

#[test]
fn atm_mid_policy_exact_or_interpolated_or_nearest() {
    let p = NearestOrLinearLogMoneyness { tol_log: 1e-12 };

    // exact k=1
    let q1 = CallQuote::new(0.9, 0.25, 0.27, 1.0).unwrap();
    let q2 = CallQuote::new(1.0, 0.20, 0.22, 1.0).unwrap();
    let q3 = CallQuote::new(1.1, 0.16, 0.18, 1.0).unwrap();
    let chain = OptionChain::new(1.0, vec![q1, q2, q3]).unwrap();
    let atm = chain.atm_mid(&p).unwrap();
    assert!((atm - q2.mid()).abs() < 1e-15);

    // bracketed interpolation (no k=1)
    let q1 = CallQuote::new(0.95, 0.23, 0.25, 1.0).unwrap();
    let q2 = CallQuote::new(1.05, 0.18, 0.20, 1.0).unwrap();
    let chain = OptionChain::new(1.0, vec![q1, q2]).unwrap();
    let atm = chain.atm_mid(&p).unwrap();
    assert!(atm <= q1.mid() && atm >= q2.mid()); // decreasing in k assumption not required, but typical

    // nearest (all strikes > 1)
    let q1 = CallQuote::new(1.10, 0.16, 0.18, 1.0).unwrap();
    let q2 = CallQuote::new(1.30, 0.08, 0.10, 1.0).unwrap();
    let chain = OptionChain::new(1.0, vec![q1, q2]).unwrap();
    let atm = chain.atm_mid(&p).unwrap();
    assert!((atm - q1.mid()).abs() < 1e-15);
}
