// src/density/mod.rs
pub mod marginal;
pub mod martingale;
pub mod tol;

pub use marginal::MarginalDensity;
pub use martingale::MartingaleDensity;
pub use tol::DensityTolerances;
