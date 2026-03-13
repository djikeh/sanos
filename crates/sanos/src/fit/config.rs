use crate::error::{SanosError, SanosResult};
use crate::market::CompletionConfig;

/// Finite-difference order for smoothing regularization.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SmoothingOrder {
    /// First-order differences: penalizes jumps in density.
    #[default]
    D1,
    /// Second-order differences: penalizes changes in curvature.
    D2,
}

/// Tikhonov regularization mode for the calibration problem.
///
/// Adds `lambda / 2 * ||L x - x_ref||_2^2` to the objective.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "mode"))]
#[derive(Debug, Clone, PartialEq, Default)]
pub enum RegularizationMode {
    /// No regularization (default, backward-compatible).
    #[default]
    None,
    /// Ridge: L = I, x_ref = 0.  Shrinks density toward zero.
    Ridge,
    /// Smoothing: L = block-diagonal finite-difference matrix, x_ref = 0.
    /// Penalizes irregular densities.
    Smoothing {
        #[cfg_attr(feature = "serde", serde(default))]
        order: SmoothingOrder,
    },
}

/// Per-maturity lambda scaling.
///
/// The effective lambda for maturity j is `base_lambda * scale_j`, where `scale_j`
/// is interpolated from the provided knots.  This allows stronger regularization
/// at short maturities where the kernel is ill-conditioned.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LambdaScaling {
    /// (maturity, scale_factor) knots, sorted by maturity.
    /// Interpolation is piecewise-linear with flat extrapolation.
    /// If empty, all maturities use scale = 1.0.
    #[cfg_attr(feature = "serde", serde(default))]
    pub knots: Vec<(f64, f64)>,
}

impl LambdaScaling {
    /// Return the scale factor at the given maturity.
    pub fn scale_at(&self, maturity: f64) -> f64 {
        if self.knots.is_empty() {
            return 1.0;
        }
        if self.knots.len() == 1 || maturity <= self.knots[0].0 {
            return self.knots[0].1;
        }
        let last = self.knots.len() - 1;
        if maturity >= self.knots[last].0 {
            return self.knots[last].1;
        }
        for w in self.knots.windows(2) {
            let (ta, sa) = w[0];
            let (tb, sb) = w[1];
            if ta <= maturity && maturity <= tb {
                let frac = (maturity - ta) / (tb - ta);
                return (1.0 - frac) * sa + frac * sb;
            }
        }
        1.0
    }

    pub fn is_uniform(&self) -> bool {
        self.knots.is_empty()
    }

    pub fn validate(&self) -> SanosResult<()> {
        for (i, &(t, s)) in self.knots.iter().enumerate() {
            if !t.is_finite() || t <= 0.0 {
                return Err(SanosError::InvalidBound {
                    field: "regularization.lambda_scaling.maturity",
                    value: t,
                    min: f64::MIN_POSITIVE,
                    max: f64::INFINITY,
                });
            }
            if !s.is_finite() || s <= 0.0 {
                return Err(SanosError::InvalidBound {
                    field: "regularization.lambda_scaling.scale",
                    value: s,
                    min: f64::MIN_POSITIVE,
                    max: f64::INFINITY,
                });
            }
            if i > 0 && t <= self.knots[i - 1].0 {
                return Err(SanosError::InvalidOrdering {
                    msg: "lambda_scaling knots must be strictly increasing in maturity",
                });
            }
        }
        Ok(())
    }
}

/// Configuration for Tikhonov regularization.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct RegularizationConfig {
    #[cfg_attr(feature = "serde", serde(default))]
    pub mode: RegularizationMode,
    /// Base regularization weight (lambda > 0).  Ignored when mode = None.
    pub lambda: f64,
    /// Optional per-maturity scaling of lambda.
    /// Effective lambda_j = lambda * scaling.scale_at(T_j).
    #[cfg_attr(feature = "serde", serde(default))]
    pub lambda_scaling: LambdaScaling,
}

impl Default for RegularizationConfig {
    fn default() -> Self {
        Self {
            mode: RegularizationMode::None,
            lambda: 1e-4,
            lambda_scaling: LambdaScaling::default(),
        }
    }
}

impl RegularizationConfig {
    pub fn is_active(&self) -> bool {
        self.mode != RegularizationMode::None
    }

    pub fn validate(&self) -> SanosResult<()> {
        if !self.is_active() {
            return Ok(());
        }
        if !self.lambda.is_finite() {
            return Err(SanosError::NonFinite {
                field: "regularization.lambda",
                value: self.lambda,
            });
        }
        if self.lambda <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "regularization.lambda",
                value: self.lambda,
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }
        self.lambda_scaling.validate()?;
        Ok(())
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OmegaConfig {
    Zero,
    #[default]
    One,
    Both,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KernelConfig {
    /// `omega = Zero` => `linear_call` constraints.
    /// `omega = One` => `call` constraints.
    /// `omega = Both` => enforce both blocks.
    pub omega: OmegaConfig,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintConfig {
    pub enforce_simplex: bool,          // sum_i q_{j,i} = 1
    pub enforce_nonnegativity: bool,    // q_{j,i} >= 0
    pub include_time_constraints: bool, // U/R blocks
}

impl Default for ConstraintConfig {
    fn default() -> Self {
        Self {
            enforce_simplex: true,
            enforce_nonnegativity: true,
            include_time_constraints: true,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InitPriceProxyConfig {
    #[default]
    Mid,
    Bid,
    Ask,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WarmStartMode {
    /// No warm-start: leave initialization entirely to the solver backend.
    #[default]
    None,
    /// Build warm-start from synthetic calls generated by the backbone
    /// at market strikes.
    BackboneSynthetic,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct InitializationConfig {
    /// Warm-start source policy.
    #[cfg_attr(feature = "serde", serde(default))]
    pub mode: WarmStartMode,
    /// Market quote proxy used to build the raw call curve.
    pub price_proxy: InitPriceProxyConfig,
    /// Numerical tolerance used for strict warm-start feasibility checks.
    pub feasibility_tol: f64,
    /// Practical market completion (Remark 2.8) config used for synthetic warm-start.
    #[cfg_attr(feature = "serde", serde(default))]
    pub market_completion: CompletionConfig,
}

impl Default for InitializationConfig {
    fn default() -> Self {
        Self {
            mode: WarmStartMode::default(),
            price_proxy: InitPriceProxyConfig::Mid,
            feasibility_tol: 1e-8,
            market_completion: CompletionConfig::default(),
        }
    }
}

impl InitializationConfig {
    pub fn uses_warm_start(&self) -> bool {
        self.mode != WarmStartMode::None
    }

    pub fn validate(&self) -> SanosResult<()> {
        if !self.feasibility_tol.is_finite() {
            return Err(SanosError::NonFinite {
                field: "initialization.feasibility_tol",
                value: self.feasibility_tol,
            });
        }
        if self.feasibility_tol < 0.0 {
            return Err(SanosError::InvalidBound {
                field: "initialization.feasibility_tol",
                value: self.feasibility_tol,
                min: 0.0,
                max: f64::INFINITY,
            });
        }
        if self.mode == WarmStartMode::BackboneSynthetic {
            self.market_completion.validate()?;
        }
        Ok(())
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuoteWeightMode {
    /// Equal weighting for all quotes, up to the per-quote `weight` multiplier.
    Identity,
    /// Paper-style weighting: inverse bid/ask spread.
    #[default]
    BidAskSpread,
    /// Black vega weighting computed from the quote mid.
    Vega,
    /// Combined vega/spread weighting.
    BidAskVega,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteWeightingConfig {
    #[cfg_attr(feature = "serde", serde(default))]
    pub mode: QuoteWeightMode,
    /// Lower bound for the bid/ask spread used in weight calculations.
    pub spread_floor: f64,
    /// Lower bound for Black vega used in weight calculations.
    pub vega_floor: f64,
}

impl Default for QuoteWeightingConfig {
    fn default() -> Self {
        Self {
            mode: QuoteWeightMode::default(),
            spread_floor: 1e-12,
            vega_floor: 1e-12,
        }
    }
}

impl QuoteWeightingConfig {
    pub fn validate(&self) -> SanosResult<()> {
        if !self.spread_floor.is_finite() {
            return Err(SanosError::NonFinite {
                field: "weighting.spread_floor",
                value: self.spread_floor,
            });
        }
        if self.spread_floor <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "weighting.spread_floor",
                value: self.spread_floor,
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }

        if !self.vega_floor.is_finite() {
            return Err(SanosError::NonFinite {
                field: "weighting.vega_floor",
                value: self.vega_floor,
            });
        }
        if self.vega_floor <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "weighting.vega_floor",
                value: self.vega_floor,
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }

        Ok(())
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FitConfig {
    pub kernel: KernelConfig,
    pub constraints: ConstraintConfig,
    #[cfg_attr(feature = "serde", serde(default))]
    pub initialization: InitializationConfig,
    #[cfg_attr(feature = "serde", serde(default))]
    pub weighting: QuoteWeightingConfig,
    #[cfg_attr(feature = "serde", serde(default))]
    pub regularization: RegularizationConfig,
}

impl FitConfig {
    pub fn uses_warm_start(&self) -> bool {
        self.initialization.uses_warm_start()
    }

    pub fn validate(&self) -> SanosResult<()> {
        self.initialization.validate()?;
        self.weighting.validate()?;
        self.regularization.validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_config_default_validates() {
        let cfg = FitConfig::default();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn weighting_defaults_match_legacy_behavior() {
        let cfg = FitConfig::default();
        assert_eq!(cfg.weighting.mode, QuoteWeightMode::BidAskSpread);
        assert_eq!(cfg.weighting.spread_floor, 1e-12);
        assert_eq!(cfg.weighting.vega_floor, 1e-12);
    }

    #[test]
    fn invalid_weighting_floor_is_rejected() {
        let mut cfg = FitConfig::default();
        cfg.weighting.vega_floor = 0.0;
        assert!(matches!(
            cfg.validate(),
            Err(SanosError::InvalidBound {
                field: "weighting.vega_floor",
                ..
            })
        ));
    }
}
