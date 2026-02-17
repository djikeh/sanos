// src/density/mod.rs
mod marginal;
mod martingale;
mod tol;

pub use marginal::MarginalDensity;
pub use martingale::MartingaleDensity;
pub use tol::DensityTolerances;
