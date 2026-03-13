# sanos

Smooth, arbitrage-aware option surface calibration for Rust.

`sanos` is a Rust crate for building smooth option surfaces from quoted option
markets while keeping tight control over arbitrage consistency, strike-grid design,
and calibration behavior.

It is aimed at developers building:

- derivatives analytics backends
- research pipelines for smile and surface fitting
- pricing or risk tools that need a queryable calibrated surface
- Rust-native alternatives to ad hoc notebook calibration stacks

API docs: <https://docs.rs/sanos>

## Why use `sanos`

What you get out of the box:

- validated market data types (`CallQuote`, `OptionChain`, `OptionBook`)
- a high-level calibration API (`calibrate`, `calibrate_with_stats`)
- a reusable calibrated surface object (`SanosSurface`)
- support for smooth term structures and strike interpolation
- no-arbitrage diagnostics you can expose in your own tooling
- optional serialization support via `serde`

Why it fits well in a Rust stack:

- strong type-level validation at the market-data boundary
- no Python dependency in the crate itself
- easy embedding in services, CLIs, batch jobs, or research binaries
- feature-gated optional components instead of a monolithic runtime

## What it looks like in practice

The examples below show the kind of calibration outputs you can generate with
`sanos`: price fit, IV fit, density reconstruction, and compact quality diagnostics.

### `sabr_catalog_02_equity_like_moderate_skew` with `tight_spread`

A clean equity-like skew fit.

Price fit:

![Equity-like price smile calibration](https://raw.githubusercontent.com/djikeh/sanos/master/crates/sanos/docs/assets/equity_like_smiles_fit.png)

IV fit:

![Equity-like IV smile calibration](https://raw.githubusercontent.com/djikeh/sanos/master/crates/sanos/docs/assets/equity_like_smiles_iv_fit.png)

Density:

![Equity-like density comparison](https://raw.githubusercontent.com/djikeh/sanos/master/crates/sanos/docs/assets/equity_like_density_comparison.png)

Performance summary:

![Equity-like quality summary](https://raw.githubusercontent.com/djikeh/sanos/master/crates/sanos/docs/assets/equity_like_quality_summary.png)

### `svi_raw_catalog_06_shifted_smile_center_right_long_end_focus` with `strong_wings`

A shifted smile with long-end structure.

Price fit:

![Shifted long-end price smile calibration](https://raw.githubusercontent.com/djikeh/sanos/master/crates/sanos/docs/assets/shifted_long_end_smiles_fit.png)

IV fit:

![Shifted long-end IV smile calibration](https://raw.githubusercontent.com/djikeh/sanos/master/crates/sanos/docs/assets/shifted_long_end_smiles_iv_fit.png)

Density:

![Shifted long-end density comparison](https://raw.githubusercontent.com/djikeh/sanos/master/crates/sanos/docs/assets/shifted_long_end_density_comparison.png)

Performance summary:

![Shifted long-end quality summary](https://raw.githubusercontent.com/djikeh/sanos/master/crates/sanos/docs/assets/shifted_long_end_quality_summary.png)

### `tv_catalog_07_inverted_term_structure` with `tight_spread`

An inverted term-structure example with a clean global fit.

Price fit:

![Inverted term-structure price smile calibration](https://raw.githubusercontent.com/djikeh/sanos/master/crates/sanos/docs/assets/inverted_term_smiles_fit.png)

IV fit:

![Inverted term-structure IV smile calibration](https://raw.githubusercontent.com/djikeh/sanos/master/crates/sanos/docs/assets/inverted_term_smiles_iv_fit.png)

Density:

![Inverted term-structure density comparison](https://raw.githubusercontent.com/djikeh/sanos/master/crates/sanos/docs/assets/inverted_term_density_comparison.png)

Performance summary:

![Inverted term-structure quality summary](https://raw.githubusercontent.com/djikeh/sanos/master/crates/sanos/docs/assets/inverted_term_quality_summary.png)

## Installation

```toml
[dependencies]
sanos = "0.2"
```

## 30-second example

```rust
use sanos::calibration::{calibrate, CalibrationConfig};
use sanos::error::SanosResult;
use sanos::market::OptionBook;

fn run(book: &OptionBook, cfg: &CalibrationConfig) -> SanosResult<f64> {
    let surface = calibrate(book, cfg)?;
    surface.call(1.0, 1.0)
}
```

## Core workflow

`sanos` is designed for workflows where you need more than a pointwise smile fit:

- enforce calendar / convexity / monotonicity consistency at the surface level
- keep control over the strike grid and regularization regime
- inspect fit quality through residual, density, and smoothness diagnostics
- serialize configs and surface outputs for offline research pipelines

## Input Conventions

- Quotes are **forward-normalized call prices** in `[0, 1]`.
- Strikes are forward moneyness (`k > 0`).
- Maturities must be strictly increasing in `OptionBook`.
- Use at least **2 maturities** if you want to query interpolated surface prices (`surface.call`).

## Public API at a glance

- `market`: validated option quotes, chains, and books
- `calibration`: top-level calibration entry points and runtime config
- `surface`: calibrated `SanosSurface` queries
- `backbone`: backbone models and ATM term-structure helpers
- `fit`: fitting config, weighting, regularization, and warm start controls
- `grid`: strike-grid policies for calibration and export
- `interp`: time interpolation choices
- `density`: marginal and martingale density diagnostics
- `term`: term-structure utilities

## Typical use cases

- Calibrate a surface once and query prices at arbitrary maturities and strikes.
- Build a research pipeline that compares calibration policies or regularization regimes.
- Export calibrated surfaces and diagnostics into your own storage or reporting layer.
- Validate whether a quoted book is fit for downstream pricing or risk calculations.

## End-to-End Example (`OptionBook` + Config)

```rust
use sanos::backbone::{bs_call_forward_norm, BackboneConfig, BsTimeChangedConfig};
use sanos::calibration::{calibrate, CalibrationConfig, ConvexOrderValidationMode};
use sanos::error::SanosResult;
use sanos::fit::FitConfig;
use sanos::grid::StrikeGridPolicyConfig;
use sanos::interp::TimeInterpConfig;
use sanos::market::{CallQuote, OptionBook, OptionChain};

fn build_synthetic_book() -> SanosResult<OptionBook> {
    // Two maturities with synthetic ATM total variances.
    let maturities = [0.5, 1.0];
    let total_vars = [0.04, 0.09];
    let strikes = [0.8, 0.9, 1.0, 1.1, 1.2];

    let mut chains = Vec::new();
    for (t, w) in maturities.into_iter().zip(total_vars) {
        let mut quotes = Vec::new();
        for k in strikes {
            let mid = bs_call_forward_norm(k, w)?;
            let spread = 0.01;
            let bid = (mid - 0.5 * spread).clamp(0.0, 1.0);
            let ask = (mid + 0.5 * spread).clamp(0.0, 1.0);
            quotes.push(CallQuote::new(k, bid, ask, 1.0)?);
        }
        chains.push(OptionChain::new(t, quotes)?);
    }

    OptionBook::new(chains)
}

fn build_config() -> CalibrationConfig {
    CalibrationConfig {
        backbone: BackboneConfig::BsTimeChanged(BsTimeChangedConfig::default()),
        grid: StrikeGridPolicyConfig::default(),
        fit: FitConfig::default(),
        time_interp: TimeInterpConfig::default(),
        convex_order_validation: ConvexOrderValidationMode::Error,
    }
}

fn main() -> SanosResult<()> {
    let book = build_synthetic_book()?;
    let cfg = build_config();

    let surface = calibrate(&book, &cfg)?;
    let c = surface.call(0.75, 1.0)?;
    println!("Interpolated call(T=0.75, K=1.0) = {c:.6}");
    Ok(())
}
```

## Config Example

```rust
use sanos::backbone::{BackboneConfig, BsTimeChangedConfig};
use sanos::calibration::{CalibrationConfig, ConvexOrderValidationMode};
use sanos::fit::FitConfig;
use sanos::grid::StrikeGridPolicyConfig;
use sanos::interp::TimeInterpConfig;

let cfg = CalibrationConfig {
    backbone: BackboneConfig::BsTimeChanged(BsTimeChangedConfig::default()),
    grid: StrikeGridPolicyConfig::default(),
    fit: FitConfig::default(), // default solver = Microlp
    time_interp: TimeInterpConfig::default(),
    convex_order_validation: ConvexOrderValidationMode::Error,
};
```

## Feature Flags

- `lp-microlp` (default): pure-Rust LP solver backend.
- `lp-cbc`: CBC backend via `good_lp/lp-solvers` (requires CBC runtime).
- `iv-jaeckel` (default): implied-vol inversion support.
- `serde`: serialization support for config/runtime types.

When selecting a solver in configuration, the matching crate feature must be enabled.

## Diagnostics you can expose in your application

The crate focuses on the calibration engine, and is designed to surface metrics that
are useful in production and research:

- inside bid/ask ratio for positive-spread books
- normalized residuals versus half-spread
- implied-vol MAE / RMSE
- total variation and max second difference of the model IV smile
- strike monotonicity, convexity, and calendar no-arbitrage violations
- density reconstruction quality and projection diagnostics

## Scope

This crate is self-contained and usable as-is for building option books, calibrating SANOS
surfaces, and querying interpolated prices.

## Known limitations

- The current public crate is the core library. The JSON schema, I/O helpers, and CLI
  remain workspace crates and are not yet published as stable crates.io APIs.
- Complex stressed markets can still require tuning of weighting, warm-start, and
  grid policies.
- For production rollout, you should keep your own acceptance thresholds on fit quality
  and no-arbitrage diagnostics.

## Research Attribution

This crate is an independent implementation of the SANOS methodology described in:

- *"SANOS: Smooth strictly Arbitrage-free Non-parametric Option Surfaces"* (arXiv:2601.11209)
- URL: <https://arxiv.org/abs/2601.11209>

The code in this repository is original Rust code released under MIT (`LICENSE`), and is not a copy of the paper text.

## Non-Affiliation

This project is not affiliated with, endorsed by, or maintained by the authors of the SANOS paper.
