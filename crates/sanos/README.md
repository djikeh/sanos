# sanos

Rust implementation of SANOS: smooth, arbitrage-aware option surface calibration.

This crate provides:
- market data structures (`OptionBook`, `OptionChain`, `CallQuote`)
- calibration pipeline (`calibrate`, `calibrate_with_stats`)
- resulting surface object (`SanosSurface`)

API docs: https://docs.rs/sanos

## Installation

```toml
[dependencies]
sanos = "0.1"
```

## Input Conventions

- Quotes are **forward-normalized call prices** in `[0, 1]`.
- Strikes are forward moneyness (`k > 0`).
- Maturities must be strictly increasing in `OptionBook`.
- Use at least **2 maturities** if you want to query interpolated surface prices (`surface.call`).

## Quick Start

```rust
use sanos::calibration::{calibrate, CalibrationConfig};
use sanos::error::SanosResult;
use sanos::market::OptionBook;

fn run(book: &OptionBook, cfg: &CalibrationConfig) -> SanosResult<f64> {
    let surface = calibrate(book, cfg)?;
    surface.call(1.0, 1.0)
}
```

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

## Scope

This crate is self-contained and usable as-is for building option books, calibrating SANOS
surfaces, and querying interpolated prices.

## Roadmap

Additional tooling and integrations may be published later, after further validation.

## Research Attribution

This crate is an independent implementation of the SANOS methodology described in:

- *"SANOS: Smooth strictly Arbitrage-free Non-parametric Option Surfaces"* (arXiv:2601.11209)
- URL: https://arxiv.org/abs/2601.11209

The code in this repository is original Rust code released under MIT (`LICENSE`), and is not a copy of the paper text.

## Non-Affiliation

This project is not affiliated with, endorsed by, or maintained by the authors of the SANOS paper.

## Pre-publish Checklist

```bash
cargo test -p sanos
cargo test -p sanos --no-default-features
RUSTDOCFLAGS="-D warnings" cargo doc -p sanos --no-deps
cargo package -p sanos
```
