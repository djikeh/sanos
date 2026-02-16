//! Schema crate for SANOS.
//!
//! This crate defines stable, versioned JSON schemas for:
//! - market data snapshots (e.g., implied-vol surfaces)
//! - calibration configuration objects
//!
//! The goal is to keep I/O and schema concerns out of the core `sanos` crate.

pub mod v1;

// Re-export the runtime calibration config types from the core crate.
// This ensures the JSON config stays in sync with the actual calibrator.
pub use sanos::calibration::CalibrationConfig;
