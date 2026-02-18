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

#[cfg(test)]
mod tests {
    use super::CalibrationConfig;
    use sanos::backbone::{BackboneConfig, BsTimeChangedConfig};
    use sanos::calibration::ConvexOrderValidationMode;
    use sanos::fit::FitConfig;
    use sanos::grid::StrikeGridPolicyConfig;
    use sanos::interp::TimeInterpConfig;

    #[test]
    fn calibration_config_reexport_matches_runtime_type() {
        let cfg = CalibrationConfig {
            backbone: BackboneConfig::BsTimeChanged(BsTimeChangedConfig::default()),
            grid: StrikeGridPolicyConfig::default(),
            fit: FitConfig::default(),
            market_completion: None,
            time_interp: TimeInterpConfig::default(),
            convex_order_validation: ConvexOrderValidationMode::Error,
        };
        match cfg.backbone {
            BackboneConfig::BsTimeChanged(_) => {}
        }
    }
}
