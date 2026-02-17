//! Core SANOS calibration library.
//!
//! Public API is exposed through high-level modules (`market`, `calibration`,
//! `surface`, `backbone`, `fit`, `grid`, `interp`, `density`, `term`) and
//! through `prelude`.
//! Internal submodules are kept private to avoid locking unstable internals in
//! semver for the first public releases.

pub mod error;
pub mod market;
pub mod prelude;
pub mod backbone;
pub mod term;
pub mod density;
pub mod interp;
pub mod surface;
pub mod grid;
pub mod fit;
pub mod calibration;
