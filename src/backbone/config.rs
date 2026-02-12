use crate::error::{SanosError, SanosResult};
use crate::market::{AtmMidPolicy, NearestOrLinearLogMoneyness};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AtmMidPolicyConfig {
    NearestOrLinearLogMoneyness { tol_log: f64 },
}

impl Default for AtmMidPolicyConfig {
    fn default() -> Self {
        Self::NearestOrLinearLogMoneyness { tol_log: 1e-10 }
    }
}

impl AtmMidPolicyConfig {
    pub fn build(self) -> SanosResult<Box<dyn AtmMidPolicy>> {
        match self {
            AtmMidPolicyConfig::NearestOrLinearLogMoneyness { tol_log } => {
                if !tol_log.is_finite() {
                    return Err(SanosError::NonFinite { field: "atm_policy.tol_log", value: tol_log });
                }
                if tol_log < 0.0 {
                    return Err(SanosError::InvalidBound {
                        field: "atm_policy.tol_log",
                        value: tol_log,
                        min: 0.0,
                        max: f64::INFINITY,
                    });
                }
                Ok(Box::new(NearestOrLinearLogMoneyness { tol_log }))
            }
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BsTimeChangedConfig {
    pub atm_policy: AtmMidPolicyConfig,
    pub var_floor: f64,
    pub enforce_non_decreasing: bool,
    pub eta: f64
}

impl Default for BsTimeChangedConfig {
    fn default() -> Self {
        Self {
            atm_policy: AtmMidPolicyConfig::default(),
            var_floor: 1e-12,
            enforce_non_decreasing: true,
            eta: 0.25,
        }
    }
}

impl BsTimeChangedConfig {
    pub fn validate(&self) -> SanosResult<()> {
        if !self.var_floor.is_finite() {
            return Err(SanosError::NonFinite { field: "bs_time_changed.var_floor", value: self.var_floor });
        }
        if self.var_floor < 0.0 {
            return Err(SanosError::InvalidBound {
                field: "bs_time_changed.var_floor",
                value: self.var_floor,
                min: 0.0,
                max: f64::INFINITY,
            });
        }
        if self.eta < 0.0 || self.eta > 1.0 {
            return Err(SanosError::InvalidBound {
                field: "bs_time_changed.eta",
                value: self.eta,
                min: 0.0,
                max: 1.0,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atm_policy_build_accepts_non_negative_tol() {
        let cfg = AtmMidPolicyConfig::NearestOrLinearLogMoneyness { tol_log: 0.0 };
        assert!(cfg.build().is_ok());
    }

    #[test]
    fn atm_policy_build_rejects_negative_tol() {
        let cfg = AtmMidPolicyConfig::NearestOrLinearLogMoneyness { tol_log: -1e-6 };
        match cfg.build() {
            Err(SanosError::InvalidBound { field, .. }) => assert_eq!(field, "atm_policy.tol_log"),
            Err(err) => panic!("unexpected error variant: {err:?}"),
            Ok(_) => panic!("expected error for negative tol_log"),
        }
    }

    #[test]
    fn atm_policy_build_rejects_non_finite_tol() {
        let cfg = AtmMidPolicyConfig::NearestOrLinearLogMoneyness { tol_log: f64::NAN };
        match cfg.build() {
            Err(SanosError::NonFinite { field, .. }) => assert_eq!(field, "atm_policy.tol_log"),
            Err(err) => panic!("unexpected error variant: {err:?}"),
            Ok(_) => panic!("expected error for non-finite tol_log"),
        }
    }

    #[test]
    fn bs_time_changed_validate_accepts_non_negative_floor() {
        let cfg = BsTimeChangedConfig {
            atm_policy: AtmMidPolicyConfig::default(),
            var_floor: 0.0,
            enforce_non_decreasing: true,
            eta: 0.25,
        };
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn bs_time_changed_validate_rejects_negative_floor() {
        let cfg = BsTimeChangedConfig {
            atm_policy: AtmMidPolicyConfig::default(),
            var_floor: -1e-6,
            enforce_non_decreasing: true,
            eta: 0.25,
        };
        let err = cfg.validate().unwrap_err();
        match err {
            SanosError::InvalidBound { field, .. } => assert_eq!(field, "bs_time_changed.var_floor"),
            _ => panic!("unexpected error variant: {err:?}"),
        }
    }

    #[test]
    fn bs_time_changed_validate_rejects_non_finite_floor() {
        let cfg = BsTimeChangedConfig {
            atm_policy: AtmMidPolicyConfig::default(),
            var_floor: f64::NAN,
            enforce_non_decreasing: true,
            eta: 0.25,
        };
        let err = cfg.validate().unwrap_err();
        match err {
            SanosError::NonFinite { field, .. } => assert_eq!(field, "bs_time_changed.var_floor"),
            _ => panic!("unexpected error variant: {err:?}"),
        }
    }
}
