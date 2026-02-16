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

    /// SANOS paper-style robust objective:
    /// - w * (epsilon_inside * |mid - model|
    ///      + slack_penalty * (max(0, bid-model) + max(0, model-ask)))
    /// where default `w` follows Eq. (26): `quote.weight / (ask-bid)`.
    HingeBidAsk {
        slack_penalty: f64,
        epsilon_inside: f64,
    },

    /// L1 fit to mid prices (typically with optional extra constraints).
    L1Mid { weight: f64 },
}

impl Default for ObjectiveConfig {
    fn default() -> Self {
        ObjectiveConfig::HingeBidAsk {
            slack_penalty: 1.0,
            epsilon_inside: 1e-8,
        }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitPriceProxyConfig {
    Mid,
    Bid,
    Ask,
}

impl Default for InitPriceProxyConfig {
    fn default() -> Self {
        InitPriceProxyConfig::Mid
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct InitializationConfig {
    /// Enable linear-density-based initialization.
    pub enabled: bool,
    /// Market quote proxy used to build the raw call curve.
    pub price_proxy: InitPriceProxyConfig,
    /// Tolerance used to decide whether the raw density is already feasible.
    pub feasibility_tol: f64,
    /// Tolerance used for projection constraints.
    pub projection_tol: f64,
    /// Threshold below which atoms are considered numerically zero in diagnostics.
    pub near_zero_tol: f64,
    /// L1 anchor strength: adds sum_i |q_i - p*_i| to the LP objective.
    pub anchor_l1_weight: f64,
}

impl Default for InitializationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            price_proxy: InitPriceProxyConfig::Mid,
            feasibility_tol: 1e-8,
            projection_tol: 1e-10,
            near_zero_tol: 1e-10,
            anchor_l1_weight: 1e-3,
        }
    }
}

impl InitializationConfig {
    pub fn validate(&self) -> SanosResult<()> {
        for (field, value, min) in [
            ("initialization.feasibility_tol", self.feasibility_tol, 0.0),
            ("initialization.projection_tol", self.projection_tol, 0.0),
            ("initialization.near_zero_tol", self.near_zero_tol, 0.0),
            (
                "initialization.anchor_l1_weight",
                self.anchor_l1_weight,
                0.0,
            ),
        ] {
            if !value.is_finite() {
                return Err(SanosError::NonFinite { field, value });
            }
            if value < min {
                return Err(SanosError::InvalidBound {
                    field,
                    value,
                    min,
                    max: f64::INFINITY,
                });
            }
        }
        Ok(())
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum LpSolverConfig {
    /// Pure Rust LP solver backend (no external binary required).
    Microlp,

    /// External CBC solver binary (requires `cbc` installed in PATH).
    Cbc { msg: bool, time_limit_sec: Option<u64> },
}

impl Default for LpSolverConfig {
    fn default() -> Self {
        LpSolverConfig::Microlp
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct FitConfig {
    pub kernel: KernelConfig,
    pub objective: ObjectiveConfig,
    pub lp: LpConfig,
    pub solver: LpSolverConfig,
    #[cfg_attr(feature = "serde", serde(default))]
    pub initialization: InitializationConfig,
}

impl Default for FitConfig {
    fn default() -> Self {
        Self {
            kernel: KernelConfig::default(),
            objective: ObjectiveConfig::default(),
            lp: LpConfig::default(),
            solver: LpSolverConfig::default(),
            initialization: InitializationConfig::default(),
        }
    }
}

impl FitConfig {
    pub fn validate(&self) -> SanosResult<()> {
        self.objective.validate()?;
        self.initialization.validate()?;
        Ok(())
    }
}
