// src/grid/config.rs
use crate::error::{SanosError, SanosResult};
use crate::grid::policy::MarketAnchored;

/// How to extend the grid beyond market strikes.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WingsConfig {
    pub n_left: usize,
    pub n_right: usize,
    pub ratio: f64, // must be > 1
}

impl Default for WingsConfig {
    fn default() -> Self {
        Self { n_left: 2, n_right: 2, ratio: 1.2 }
    }
}

impl WingsConfig {
    pub fn validate(&self) -> SanosResult<()> {
        if !self.ratio.is_finite() {
            return Err(SanosError::NonFinite { field: "grid.wings.ratio", value: self.ratio });
        }
        if self.ratio <= 1.0 {
            return Err(SanosError::InvalidBound {
                field: "grid.wings.ratio",
                value: self.ratio,
                min: 1.0 + f64::EPSILON,
                max: f64::INFINITY,
            });
        }
        Ok(())
    }
}

/// Optional ATM refinement around k=1.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtmRefineConfig {
    pub enabled: bool,
    pub steps: usize,
    pub delta_log: f64, // > 0 when enabled
}

impl Default for AtmRefineConfig {
    fn default() -> Self {
        Self { enabled: false, steps: 2, delta_log: 0.05 }
    }
}

impl AtmRefineConfig {
    pub fn validate(&self) -> SanosResult<()> {
        if !self.delta_log.is_finite() {
            return Err(SanosError::NonFinite { field: "grid.atm_refine.delta_log", value: self.delta_log });
        }
        if self.enabled && self.delta_log <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "grid.atm_refine.delta_log",
                value: self.delta_log,
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }
        Ok(())
    }
}

/// Controls the maximum number of grid points.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridSizeConfig {
    pub max_points: usize,
    pub keep_all_market_strikes: bool,
}

impl Default for GridSizeConfig {
    fn default() -> Self {
        Self { max_points: 80, keep_all_market_strikes: true }
    }
}

impl GridSizeConfig {
    pub fn validate(&self) -> SanosResult<()> {
        if self.max_points == 0 {
            return Err(SanosError::InvalidOrdering { msg: "grid.grid_size.max_points must be > 0" });
        }
        Ok(())
    }
}

/// User-facing, serializable strike grid policy config.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum StrikeGridPolicyConfig {
    MarketAnchored(MarketAnchoredGridConfig),
}

impl Default for StrikeGridPolicyConfig {
    fn default() -> Self {
        Self::MarketAnchored(MarketAnchoredGridConfig::default())
    }
}

/// Serializable config for the runtime `MarketAnchored` policy.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct MarketAnchoredGridConfig {
    pub ensure_atm: bool,
    pub wings: WingsConfig,
    pub atm_refine: AtmRefineConfig,
    pub grid_size: GridSizeConfig,
    pub min_strike: f64,
    pub max_strike: f64,
    pub min_spacing_log: f64,
}

impl Default for MarketAnchoredGridConfig {
    fn default() -> Self {
        Self {
            ensure_atm: true,
            wings: WingsConfig::default(),
            atm_refine: AtmRefineConfig::default(),
            grid_size: GridSizeConfig::default(),
            min_strike: 1e-4,
            max_strike: 1e4,
            min_spacing_log: 1e-3,
        }
    }
}

impl MarketAnchoredGridConfig {
    pub fn validate(&self) -> SanosResult<()> {
        self.wings.validate()?;
        self.atm_refine.validate()?;
        self.grid_size.validate()?;

        for (field, v) in [
            ("grid.market_anchored.min_strike", self.min_strike),
            ("grid.market_anchored.max_strike", self.max_strike),
            ("grid.market_anchored.min_spacing_log", self.min_spacing_log),
        ] {
            if !v.is_finite() {
                return Err(SanosError::NonFinite { field, value: v });
            }
        }

        if self.min_strike <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "grid.market_anchored.min_strike",
                value: self.min_strike,
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }
        if self.max_strike <= self.min_strike {
            return Err(SanosError::InvalidOrdering { msg: "grid.market_anchored.max_strike must be > min_strike" });
        }
        if self.min_spacing_log < 0.0 {
            return Err(SanosError::InvalidBound {
                field: "grid.market_anchored.min_spacing_log",
                value: self.min_spacing_log,
                min: 0.0,
                max: f64::INFINITY,
            });
        }

        Ok(())
    }

    pub fn to_runtime(&self) -> SanosResult<MarketAnchored> {
        self.validate()?;
        Ok(MarketAnchored {
            ensure_atm: self.ensure_atm,
            // NOTE: we will update `MarketAnchored` fields to use the same names/types:
            wings: self.wings,
            atm_refine: self.atm_refine,
            size_control: self.grid_size,
            min_strike: self.min_strike,
            max_strike: self.max_strike,
            min_spacing_log: self.min_spacing_log,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wings_validate_rejects_ratio_not_greater_than_one() {
        let cfg = WingsConfig { n_left: 1, n_right: 1, ratio: 1.0 };
        let err = cfg.validate().unwrap_err();
        match err {
            SanosError::InvalidBound { field, .. } => assert_eq!(field, "grid.wings.ratio"),
            _ => panic!("unexpected error variant: {err:?}"),
        }
    }

    #[test]
    fn atm_refine_validate_requires_positive_delta_when_enabled() {
        let cfg = AtmRefineConfig { enabled: true, steps: 2, delta_log: 0.0 };
        let err = cfg.validate().unwrap_err();
        match err {
            SanosError::InvalidBound { field, .. } => assert_eq!(field, "grid.atm_refine.delta_log"),
            _ => panic!("unexpected error variant: {err:?}"),
        }
    }

    #[test]
    fn grid_size_validate_rejects_zero_max_points() {
        let cfg = GridSizeConfig { max_points: 0, keep_all_market_strikes: true };
        let err = cfg.validate().unwrap_err();
        match err {
            SanosError::InvalidOrdering { msg } => {
                assert_eq!(msg, "grid.grid_size.max_points must be > 0")
            }
            _ => panic!("unexpected error variant: {err:?}"),
        }
    }

    #[test]
    fn market_anchored_validate_rejects_invalid_ordering() {
        let cfg = MarketAnchoredGridConfig {
            min_strike: 2.0,
            max_strike: 2.0,
            ..MarketAnchoredGridConfig::default()
        };
        let err = cfg.validate().unwrap_err();
        match err {
            SanosError::InvalidOrdering { msg } => {
                assert_eq!(msg, "grid.market_anchored.max_strike must be > min_strike")
            }
            _ => panic!("unexpected error variant: {err:?}"),
        }
    }

    #[test]
    fn market_anchored_to_runtime_maps_fields() {
        let cfg = MarketAnchoredGridConfig {
            ensure_atm: false,
            wings: WingsConfig { n_left: 3, n_right: 4, ratio: 1.3 },
            atm_refine: AtmRefineConfig { enabled: true, steps: 5, delta_log: 0.02 },
            grid_size: GridSizeConfig { max_points: 17, keep_all_market_strikes: false },
            min_strike: 0.2,
            max_strike: 3.5,
            min_spacing_log: 0.004,
        };

        let runtime = cfg.to_runtime().unwrap();
        assert_eq!(runtime.ensure_atm, cfg.ensure_atm);
        assert_eq!(runtime.wings, cfg.wings);
        assert_eq!(runtime.atm_refine, cfg.atm_refine);
        assert_eq!(runtime.size_control, cfg.grid_size);
        assert_eq!(runtime.min_strike, cfg.min_strike);
        assert_eq!(runtime.max_strike, cfg.max_strike);
        assert_eq!(runtime.min_spacing_log, cfg.min_spacing_log);
    }
}
