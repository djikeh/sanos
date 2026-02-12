use crate::error::{SanosError, SanosResult};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmegaConfig {
    Zero,
    One,
}

impl Default for OmegaConfig {
    fn default() -> Self {
        OmegaConfig::One // recommandé
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct KernelConfig {
    /// ω = 0 => linear_call
    /// ω = 1 => call
    pub omega: OmegaConfig,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self { omega: OmegaConfig::default() }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectiveConfig {
    /// Hard constraints: model prices must lie within bid/ask.
    HardBidAsk,

    /// Soft bid/ask constraints with hinge slacks.
    HingeBidAsk {
        slack_penalty: f64,
        epsilon_inside: f64,
    },

    /// L1 fit to mid prices (typically with optional extra constraints).
    L1Mid { weight: f64 },
}

impl Default for ObjectiveConfig {
    fn default() -> Self {
        ObjectiveConfig::HardBidAsk
    }
}

impl ObjectiveConfig {
    pub fn validate(&self) -> SanosResult<()> {
        match self {
            ObjectiveConfig::HardBidAsk => Ok(()),
            ObjectiveConfig::HingeBidAsk { slack_penalty, epsilon_inside } => {
                if !slack_penalty.is_finite() {
                    return Err(SanosError::NonFinite { field: "objective.slack_penalty", value: *slack_penalty });
                }
                if *slack_penalty <= 0.0 {
                    return Err(SanosError::InvalidBound {
                        field: "objective.slack_penalty",
                        value: *slack_penalty,
                        min: f64::MIN_POSITIVE,
                        max: f64::INFINITY,
                    });
                }
                if !epsilon_inside.is_finite() {
                    return Err(SanosError::NonFinite { field: "objective.epsilon_inside", value: *epsilon_inside });
                }
                if *epsilon_inside < 0.0 {
                    return Err(SanosError::InvalidBound {
                        field: "objective.epsilon_inside",
                        value: *epsilon_inside,
                        min: 0.0,
                        max: f64::INFINITY,
                    });
                }
                Ok(())
            }
            ObjectiveConfig::L1Mid { weight } => {
                if !weight.is_finite() {
                    return Err(SanosError::NonFinite { field: "objective.weight", value: *weight });
                }
                if *weight <= 0.0 {
                    return Err(SanosError::InvalidBound {
                        field: "objective.weight",
                        value: *weight,
                        min: f64::MIN_POSITIVE,
                        max: f64::INFINITY,
                    });
                }
                Ok(())
            }
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct LpConfig {
    pub enforce_simplex: bool,          // sum_i q_{j,i} = 1
    pub enforce_nonnegativity: bool,    // q_{j,i} >= 0
    pub include_time_constraints: bool, // U/R blocks
}

impl Default for LpConfig {
    fn default() -> Self {
        Self {
            enforce_simplex: true,
            enforce_nonnegativity: true,
            include_time_constraints: true,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum LpSolverConfig {
    Cbc { msg: bool, time_limit_sec: Option<u64> },
    // Highs { presolve: bool, time_limit_sec: Option<u64> },
}

impl Default for LpSolverConfig {
    fn default() -> Self {
        LpSolverConfig::Cbc { msg: true, time_limit_sec: None }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct FitConfig {
    pub kernel: KernelConfig,
    pub objective: ObjectiveConfig,
    pub lp: LpConfig,
    pub solver: LpSolverConfig,
}

impl Default for FitConfig {
    fn default() -> Self {
        Self {
            kernel: KernelConfig::default(),
            objective: ObjectiveConfig::default(),
            lp: LpConfig::default(),
            solver: LpSolverConfig::default(),
        }
    }
}

impl FitConfig {
    pub fn validate(&self) -> SanosResult<()> {
        self.objective.validate()?;
        Ok(())
    }
}
