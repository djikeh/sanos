// src/interp/mod.rs
pub mod linear;
pub mod time_interpolator;
mod atm_variance;

pub use linear::LinearTime;
pub use time_interpolator::TimeInterpolator;
pub use atm_variance::AtmVarianceTime;
