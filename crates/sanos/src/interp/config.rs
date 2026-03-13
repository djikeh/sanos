use super::{AtmVarianceTime, LinearTime, TimeInterpolator};
use crate::error::SanosResult;
use std::sync::Arc;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeInterpConfig {
    LinearTime,
    #[default]
    AtmVarianceTime,
}

impl TimeInterpConfig {
    pub fn build(self) -> SanosResult<Arc<dyn TimeInterpolator>> {
        Ok(match self {
            TimeInterpConfig::LinearTime => Arc::new(LinearTime),
            TimeInterpConfig::AtmVarianceTime => Arc::new(AtmVarianceTime),
        })
    }
}
