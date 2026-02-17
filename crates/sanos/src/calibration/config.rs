use crate::backbone::BackboneConfig;
use crate::fit::FitConfig;
use crate::grid::StrikeGridPolicyConfig;
use crate::interp::TimeInterpConfig;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvexOrderValidationMode {
    Error,
    Warn,
}

impl Default for ConvexOrderValidationMode {
    fn default() -> Self {
        Self::Error
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct CalibrationConfig {
    pub backbone: BackboneConfig,
    pub grid: StrikeGridPolicyConfig,
    pub fit: FitConfig,
    pub time_interp: TimeInterpConfig,
    #[cfg_attr(feature = "serde", serde(default))]
    pub convex_order_validation: ConvexOrderValidationMode,
}
