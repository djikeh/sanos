// src/interp/mod.rs
pub mod linear;
pub mod time_interpolator;
mod atm_variance;
mod config;

pub use linear::LinearTime;
pub use time_interpolator::TimeInterpolator;
pub use atm_variance::AtmVarianceTime;
pub use config::TimeInterpConfig;