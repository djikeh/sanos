use std::sync::Arc;

use crate::backbone::YModel;
use crate::density::{DensityTolerances, MarginalDensity, MartingaleDensity};
use crate::error::{SanosError, SanosResult};
use crate::fit::config::{InitPriceProxyConfig, InitializationConfig, WarmStartMode};
use crate::grid::StrikeGrid;
use crate::market::{CallQuote, OptionBook, OptionChain};

pub fn build_warm_start(
    grids: &[StrikeGrid],
    y: &Arc<dyn YModel>,
    init_cfg: &InitializationConfig,
) -> SanosResult<Option<MartingaleDensity>> {
    if !init_cfg.uses_warm_start() || init_cfg.mode == WarmStartMode::None {
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
