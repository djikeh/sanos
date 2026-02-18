use std::sync::Arc;

use crate::backbone::YModel;
use crate::density::{DensityTolerances, MarginalDensity, MartingaleDensity};
use crate::error::{SanosError, SanosResult};
use crate::fit::config::{InitPriceProxyConfig, InitializationConfig};
use crate::grid::StrikeGrid;
use crate::market::{CallQuote, OptionBook, OptionChain};

pub fn build_warm_start(
    grids: &[StrikeGrid],
    y: &Arc<dyn YModel>,
    init_cfg: &InitializationConfig,
) -> SanosResult<Option<MartingaleDensity>> {
    if !init_cfg.uses_warm_start() {
        return Ok(None);
    }

    let synthetic_option_book = build_synthetic_option_book(grids, y)?;
    Ok(Some(build_strict_linear_martingale_density(
        &synthetic_option_book,
        init_cfg.price_proxy,
        init_cfg.feasibility_tol,
    )?))
}

/// Build a synthetic `OptionBook` using market maturities/strikes and `y.call(t, 1.0, k)`.
/// Returned quotes have zero spreads (`bid == ask`).
pub fn build_synthetic_option_book(
    grids: &[StrikeGrid],
    y: &Arc<dyn YModel>,
) -> SanosResult<OptionBook> {
    let mut chains = Vec::with_capacity(grids.len());

    for strike_grid in grids {
        let t = strike_grid.maturity();
        let mut quotes = Vec::with_capacity(strike_grid.strikes().len());
        for &k in strike_grid.strikes() {
            let call = y.call(t, 1.0, k)?;
            quotes.push(CallQuote::new(k, call, call, 1.0)?);
        }
        chains.push(OptionChain::new(t, quotes)?);
    }

    OptionBook::new(chains)
}

/// Build a strict martingale density from an `OptionBook` via the direct
/// linear discrete method (no LP projection fallback).
pub fn build_strict_linear_martingale_density(
    book: &OptionBook,
    price_proxy: InitPriceProxyConfig,
    feasibility_tol: f64,
) -> SanosResult<MartingaleDensity> {
    let tol = DensityTolerances::from_tol(feasibility_tol.max(1e-12))?;
    let mut marginals = Vec::with_capacity(book.len());

    for chain in book.chains() {
        let strikes: Vec<f64> = chain.quotes().iter().map(|q| q.k).collect();
        let calls: Vec<f64> = chain
            .quotes()
            .iter()
            .map(|q| quote_proxy_value(*q, price_proxy))
            .collect();

        marginals.push(compute_raw_linear_density(
            chain.maturity(),
            &strikes,
            &calls,
            tol,
        )?);
    }

    let q = MartingaleDensity::new(marginals)?;
    q.validate_marginals(tol)?;
    q.validate_convex_order(tol)?;
    Ok(q)
}

fn quote_proxy_value(quote: CallQuote, proxy: InitPriceProxyConfig) -> f64 {
    match proxy {
        InitPriceProxyConfig::Mid => quote.mid(),
        InitPriceProxyConfig::Bid => quote.bid,
        InitPriceProxyConfig::Ask => quote.ask,
    }
}

/// Build strict raw linear-discrete marginal density from discrete call prices.
///
/// Discretization used (node-based):
/// - Given strikes `K_0 < ... < K_{n-1}` and calls `C_i = C(K_i)`, define slopes
///   `d_i = (C_{i+1} - C_i) / (K_{i+1} - K_i)` for `i=0..n-2`.
/// - Raw node masses are:
///   `p_0 = 1 + d_0`,
///   `p_i = d_i - d_{i-1}` for interior nodes,
///   `p_{n-1} = -d_{n-2}`.
///
/// This is the discrete second-difference construction in the spirit of SANOS Remark 2.11.
pub fn compute_raw_linear_density(
    maturity: f64,
    strikes: &[f64],
    calls: &[f64],
    tol: DensityTolerances,
) -> SanosResult<MarginalDensity> {
    validate_strikes_and_calls(strikes, calls)?;

    let n = strikes.len();
    let mut slopes = Vec::with_capacity(n - 1);
    for i in 0..(n - 1) {
        let dk = strikes[i + 1] - strikes[i];
        slopes.push((calls[i + 1] - calls[i]) / dk);
    }

    let mut density = vec![0.0; n];
    density[0] = 1.0 + slopes[0];
    for i in 1..(n - 1) {
        density[i] = slopes[i] - slopes[i - 1];
    }
    density[n - 1] = -slopes[n - 2];

    let atoms = strikes.iter().copied().zip(density).collect::<Vec<_>>();
    MarginalDensity::new(maturity, atoms, tol)
}

fn validate_strikes_and_calls(strikes: &[f64], calls: &[f64]) -> SanosResult<()> {
    if strikes.len() != calls.len() {
        return Err(SanosError::InvalidOrdering {
            msg: "strikes and calls must have same length",
        });
    }
    if strikes.len() < 2 {
        return Err(SanosError::InvalidOrdering {
            msg: "at least 2 strike nodes are required",
        });
    }
    for i in 0..strikes.len() {
        let k = strikes[i];
        let c = calls[i];
        if !k.is_finite() {
            return Err(SanosError::NonFinite {
                field: "strike",
                value: k,
            });
        }
        if !c.is_finite() {
            return Err(SanosError::NonFinite {
                field: "call",
                value: c,
            });
        }
        if k <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "strike",
                value: k,
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }
        if i > 0 && strikes[i] <= strikes[i - 1] {
            return Err(SanosError::InvalidOrdering {
                msg: "strikes must be strictly increasing",
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::error::SanosError;
    use crate::fit::config::WarmStartMode;

    #[derive(Debug)]
    struct LinearModel;

    impl YModel for LinearModel {
        fn call(&self, maturity: f64, a: f64, b: f64) -> SanosResult<f64> {
            self.linear_call(maturity, a, b)
        }
    }

    #[derive(Debug)]
    struct AlwaysErrModel;

    impl YModel for AlwaysErrModel {
        fn call(&self, _maturity: f64, _a: f64, _b: f64) -> SanosResult<f64> {
            Err(SanosError::External {
                msg: "synthetic failure".to_string(),
            })
        }
    }

    #[derive(Debug)]
    struct FlatHighCallModel;

    impl YModel for FlatHighCallModel {
        fn call(&self, _maturity: f64, _a: f64, _b: f64) -> SanosResult<f64> {
            Ok(0.9)
        }
    }

    fn tol() -> DensityTolerances {
        DensityTolerances::from_tol(1e-12).unwrap()
    }

    fn valid_strikes() -> Vec<f64> {
        vec![0.5, 1.0, 1.5]
    }

    fn valid_calls() -> Vec<f64> {
        vec![0.5, 0.125, 0.0]
    }

    fn make_grid(maturity: f64, strikes: Vec<f64>) -> StrikeGrid {
        StrikeGrid::new(maturity, strikes).unwrap()
    }

    fn make_book_with_calls(
        maturity: f64,
        strikes: &[f64],
        bids: &[f64],
        asks: &[f64],
    ) -> OptionBook {
        let quotes = strikes
            .iter()
            .zip(bids.iter().zip(asks.iter()))
            .map(|(&k, (&bid, &ask))| CallQuote::new(k, bid, ask, 1.0).unwrap())
            .collect::<Vec<_>>();
        let chain = OptionChain::new(maturity, quotes).unwrap();
        OptionBook::new(vec![chain]).unwrap()
    }

    #[test]
    fn compute_raw_linear_density_happy_path() {
        let marginal = compute_raw_linear_density(1.0, &valid_strikes(), &valid_calls(), tol()).unwrap();
        let probs = marginal.probabilities();
        assert_eq!(probs.len(), 3);
        assert!((probs[0] - 0.25).abs() < 1e-12);
        assert!((probs[1] - 0.5).abs() < 1e-12);
        assert!((probs[2] - 0.25).abs() < 1e-12);
    }

    #[test]
    fn compute_raw_linear_density_rejects_length_mismatch() {
        let err = compute_raw_linear_density(1.0, &[0.5, 1.0], &[0.5], tol()).unwrap_err();
        assert!(matches!(err, SanosError::InvalidOrdering { .. }));
    }

    #[test]
    fn compute_raw_linear_density_rejects_too_few_nodes() {
        let err = compute_raw_linear_density(1.0, &[1.0], &[0.0], tol()).unwrap_err();
        assert!(matches!(err, SanosError::InvalidOrdering { .. }));
    }

    #[test]
    fn compute_raw_linear_density_rejects_non_finite_strike() {
        let err = compute_raw_linear_density(1.0, &[f64::NAN, 1.0], &[0.5, 0.0], tol()).unwrap_err();
        assert!(matches!(
            err,
            SanosError::NonFinite {
                field: "strike",
                ..
            }
        ));
    }

    #[test]
    fn compute_raw_linear_density_rejects_non_finite_call() {
        let err = compute_raw_linear_density(1.0, &[0.5, 1.0], &[f64::INFINITY, 0.0], tol()).unwrap_err();
        assert!(matches!(
            err,
            SanosError::NonFinite {
                field: "call",
                ..
            }
        ));
    }

    #[test]
    fn compute_raw_linear_density_rejects_non_positive_strike() {
        let err = compute_raw_linear_density(1.0, &[0.0, 1.0], &[0.5, 0.0], tol()).unwrap_err();
        assert!(matches!(
            err,
            SanosError::InvalidBound {
                field: "strike",
                ..
            }
        ));
    }

    #[test]
    fn compute_raw_linear_density_rejects_non_increasing_strikes() {
        let err = compute_raw_linear_density(1.0, &[0.5, 0.5], &[0.5, 0.5], tol()).unwrap_err();
        assert!(matches!(err, SanosError::InvalidOrdering { .. }));
    }

    #[test]
    fn compute_raw_linear_density_propagates_marginal_validation_error() {
        // Produces p=[1,0] on strikes [0.5, 1.0], so mean != 1 and marginal validation fails.
        let err = compute_raw_linear_density(1.0, &[0.5, 1.0], &[0.9, 0.9], tol()).unwrap_err();
        assert!(matches!(err, SanosError::InvalidOrdering { .. }));
    }

    #[test]
    fn build_synthetic_option_book_happy_path() {
        let model: Arc<dyn YModel> = Arc::new(LinearModel);
        let grids = vec![
            make_grid(0.5, vec![0.8, 1.0, 1.2]),
            make_grid(1.0, vec![0.7, 1.1]),
        ];

        let book = build_synthetic_option_book(&grids, &model).unwrap();
        assert_eq!(book.len(), 2);
        for chain in book.chains() {
            for q in chain.quotes() {
                assert_eq!(q.bid, q.ask);
                assert_eq!(q.weight, 1.0);
            }
        }
    }

    #[test]
    fn build_synthetic_option_book_rejects_empty_grids() {
        let model: Arc<dyn YModel> = Arc::new(LinearModel);
        let err = build_synthetic_option_book(&[], &model).unwrap_err();
        assert!(matches!(err, SanosError::EmptyCollection { .. }));
    }

    #[test]
    fn build_synthetic_option_book_propagates_model_error() {
        let model: Arc<dyn YModel> = Arc::new(AlwaysErrModel);
        let grids = vec![make_grid(1.0, vec![0.5, 1.0, 1.5])];
        let err = build_synthetic_option_book(&grids, &model).unwrap_err();
        assert!(matches!(err, SanosError::External { .. }));
    }

    #[test]
    fn build_strict_linear_martingale_density_supports_all_price_proxies() {
        let strikes = valid_strikes();
        let calls = valid_calls();
        let book = make_book_with_calls(1.0, &strikes, &calls, &calls);

        for proxy in [
            InitPriceProxyConfig::Mid,
            InitPriceProxyConfig::Bid,
            InitPriceProxyConfig::Ask,
        ] {
            let q = build_strict_linear_martingale_density(&book, proxy, 1e-12).unwrap();
            assert_eq!(q.marginals().len(), 1);
        }
    }

    #[test]
    fn build_strict_linear_martingale_density_rejects_infinite_tolerance() {
        let strikes = valid_strikes();
        let calls = valid_calls();
        let book = make_book_with_calls(1.0, &strikes, &calls, &calls);
        let err =
            build_strict_linear_martingale_density(&book, InitPriceProxyConfig::Mid, f64::INFINITY)
                .unwrap_err();
        assert!(matches!(
            err,
            SanosError::NonFinite {
                field: "mass",
                ..
            }
        ));
    }

    #[test]
    fn build_strict_linear_martingale_density_detects_convex_order_violation() {
        let strikes = valid_strikes();
        let spread = vec![0.5, 0.125, 0.0];
        let degenerate = vec![0.5, 0.0, 0.0];

        let chain1_quotes = strikes
            .iter()
            .zip(spread.iter())
            .map(|(&k, &c)| CallQuote::new(k, c, c, 1.0).unwrap())
            .collect::<Vec<_>>();
        let chain2_quotes = strikes
            .iter()
            .zip(degenerate.iter())
            .map(|(&k, &c)| CallQuote::new(k, c, c, 1.0).unwrap())
            .collect::<Vec<_>>();

        let chain1 = OptionChain::new(0.5, chain1_quotes).unwrap();
        let chain2 = OptionChain::new(1.0, chain2_quotes).unwrap();
        let book = OptionBook::new(vec![chain1, chain2]).unwrap();

        let err =
            build_strict_linear_martingale_density(&book, InitPriceProxyConfig::Mid, 1e-12)
                .unwrap_err();
        assert!(matches!(err, SanosError::InvalidOrdering { .. }));
    }

    #[test]
    fn build_strict_linear_martingale_density_propagates_raw_density_error() {
        let book = make_book_with_calls(1.0, &[0.5, 1.0], &[0.9, 0.9], &[0.9, 0.9]);
        let err =
            build_strict_linear_martingale_density(&book, InitPriceProxyConfig::Mid, 1e-12)
                .unwrap_err();
        assert!(matches!(err, SanosError::InvalidOrdering { .. }));
    }

    #[test]
    fn build_warm_start_returns_none_when_mode_is_none() {
        let model: Arc<dyn YModel> = Arc::new(LinearModel);
        let grids = vec![make_grid(1.0, vec![0.5, 1.0, 1.5])];
        let cfg = InitializationConfig {
            mode: WarmStartMode::None,
            price_proxy: InitPriceProxyConfig::Mid,
            feasibility_tol: 1e-8,
            market_completion: crate::market::CompletionConfig::default(),
        };

        let out = build_warm_start(&grids, &model, &cfg).unwrap();
        assert!(out.is_none());
    }

    #[test]
    fn build_warm_start_happy_path() {
        let model: Arc<dyn YModel> = Arc::new(LinearModel);
        let grids = vec![
            make_grid(0.5, vec![0.6, 1.0, 1.4]),
            make_grid(1.0, vec![0.5, 1.0, 1.5]),
        ];
        let cfg = InitializationConfig {
            mode: WarmStartMode::BackboneSynthetic,
            price_proxy: InitPriceProxyConfig::Ask,
            feasibility_tol: 1e-8,
            market_completion: crate::market::CompletionConfig::default(),
        };

        let out = build_warm_start(&grids, &model, &cfg).unwrap();
        assert!(out.is_some());
        assert_eq!(out.unwrap().marginals().len(), 2);
    }

    #[test]
    fn build_warm_start_propagates_model_error() {
        let model: Arc<dyn YModel> = Arc::new(AlwaysErrModel);
        let grids = vec![make_grid(1.0, vec![0.5, 1.0, 1.5])];
        let cfg = InitializationConfig {
            mode: WarmStartMode::BackboneSynthetic,
            price_proxy: InitPriceProxyConfig::Mid,
            feasibility_tol: 1e-8,
            market_completion: crate::market::CompletionConfig::default(),
        };

        let err = build_warm_start(&grids, &model, &cfg).unwrap_err();
        assert!(matches!(err, SanosError::External { .. }));
    }

    #[test]
    fn build_warm_start_propagates_strict_density_error() {
        let model: Arc<dyn YModel> = Arc::new(FlatHighCallModel);
        let grids = vec![make_grid(1.0, vec![0.5, 1.0])];
        let cfg = InitializationConfig {
            mode: WarmStartMode::BackboneSynthetic,
            price_proxy: InitPriceProxyConfig::Mid,
            feasibility_tol: 1e-8,
            market_completion: crate::market::CompletionConfig::default(),
        };

        let err = build_warm_start(&grids, &model, &cfg).unwrap_err();
        assert!(matches!(err, SanosError::InvalidOrdering { .. }));
    }
}
