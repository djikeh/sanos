# Changelog

All notable changes to the public `sanos` crate will be documented in this file.

The format is inspired by Keep a Changelog and the project follows SemVer for the
published crate API.

## [0.2.0] - 2026-03-13

### Added

- Richer crate-level documentation with a calibration gallery drawn from the current
  benchmark study.
- Benchmark summary documenting representative calibration outcomes across 30
  snapshots and multiple config regimes.
- Release-oriented repository docs with a clearer crates.io publication workflow.

### Changed

- `crates/sanos/src/lib.rs` now mirrors the crate README into docs.rs for a more
  complete public API landing page.
- Workspace helper crates are explicitly marked `publish = false` while the public
  release target remains the `sanos` library crate.

## [0.1.1] - 2026-03-13

Initial public crates.io release of `sanos`.

### Added

- Core SANOS calibration library with public modules for market data, calibration,
  surface queries, backbone models, fitting, strike grids, interpolation, density,
  and term structures.
- `calibrate` / `calibrate_with_stats` entry points and the `SanosSurface` runtime
  object for arbitrage-aware option surface construction.
- Optional `serde` support for config/runtime types.
- Optional `iv-jaeckel` implied-vol inversion feature enabled by default.
- End-to-end test coverage for calibration, snapshot fixtures, and grid policies.

### Packaging

- Published crate metadata for docs.rs and crates.io.
- Release checklist and PowerShell release helper for reproducible validation and
  publication.
