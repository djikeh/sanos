// src/interp/mod.rs
pub mod linear;
pub mod time_interpolator;

pub use linear::LinearTime;
pub use time_interpolator::TimeInterpolator;
