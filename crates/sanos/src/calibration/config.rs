use crate::backbone::BackboneConfig;
use crate::fit::FitConfig;
use crate::grid::StrikeGridPolicyConfig;
use crate::interp::TimeInterpConfig;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct CalibrationConfig {
    pub backbone: BackboneConfig,
    pub grid: StrikeGridPolicyConfig,
    pub fit: FitConfig,
    pub time_interp: TimeInterpConfig,
}
