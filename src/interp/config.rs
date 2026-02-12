use crate::error::SanosResult;
use super::{AtmVarianceTime, LinearTime, TimeInterpolator};
use std::sync::Arc;

#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeInterpConfig {
    LinearTime,
    AtmVarianceTime,
}

impl Default for TimeInterpConfig {
    fn default() -> Self {
        TimeInterpConfig::AtmVarianceTime
    }
}

impl TimeInterpConfig {
    pub fn build(self) -> SanosResult<Arc<dyn TimeInterpolator>> {
        Ok(match self {
            TimeInterpConfig::LinearTime => Arc::new(LinearTime),
            TimeInterpConfig::AtmVarianceTime => Arc::new(AtmVarianceTime),
        })
    }
}
