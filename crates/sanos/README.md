# sanos

Rust implementation of SANOS: smooth, arbitrage-aware option surface calibration.

This crate provides:
- market data structures (`OptionBook`, `OptionChain`, `CallQuote`)
- calibration pipeline (`calibrate`, `calibrate_with_stats`)
- resulting surface object (`SanosSurface`)

## Installation

```toml
[dependencies]
sanos = "0.1"
```

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

## Feature Flags

- `lp-microlp` (default): pure-Rust LP solver backend.
- `lp-cbc`: CBC backend via `good_lp/lp-solvers` (requires CBC runtime).
- `iv-jaeckel` (default): implied-vol inversion support.
- `serde`: serialization support for config/runtime types.

When selecting a solver in configuration, the matching crate feature must be enabled.

## Project Notes

This repository also contains higher-level tooling (`sanos-cli`, `sanos-io`, Python orchestration).
The `sanos` crate itself is the core Rust calibration engine.

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
