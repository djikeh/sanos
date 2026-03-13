use std::sync::Arc;

use crate::backbone::{build_backbone_with_total_variances, BackboneConfig, YModel};
use crate::density::{DensityTolerances, MarginalDensity, MartingaleDensity};
use crate::error::SanosResult;
use crate::fit::{build_kernels, solve, WarmStartMode};
use crate::grid::{build_strike_grids_with_variances, StrikeGrid};
use crate::market::{complete_slice_remark_2_8, CompletionConfig, OptionBook};
use crate::surface::SanosSurface;
use log::{info, warn};

use super::config::{CalibrationConfig, ConvexOrderValidationMode};

#[derive(Debug, Clone)]
pub struct CalibrationRunStats {
    pub objective_value: f64,
}

#[derive(Debug, Clone)]
pub struct CalibrationResult {
    pub surface: SanosSurface,
    pub stats: CalibrationRunStats,
}

pub fn calibrate_with_stats(
    book: &OptionBook,
    cfg: &CalibrationConfig,
) -> SanosResult<CalibrationResult> {
    cfg.fit.validate()?;

    // 1) backbone
    let (y, total_variances) = build_backbone_with_total_variances(book, &cfg.backbone)?;

    // 2) strike grids
    let atm_policy = match &cfg.backbone {
        BackboneConfig::BsTimeChanged(bs_cfg) => bs_cfg.atm_policy.build()?,
    };
    let mut grids = build_strike_grids_with_variances(
        book,
        atm_policy.as_ref(),
        &cfg.grid,
        Some(&total_variances),
    )?;

    // Practical completion (Remark 2.8) is used only for BackboneSynthetic warm-start.
    // K0=0 is used only in completion algebra and is not inserted in model grids.
    if cfg.fit.initialization.mode == WarmStartMode::BackboneSynthetic {
        let completion_cfg = &cfg.fit.initialization.market_completion;
        let (completed_grids, _completed_density) = build_completed_grids_and_warm_start(
            &grids,
            &y,
            completion_cfg,
            cfg.fit.initialization.feasibility_tol,
        )?;
        grids = completed_grids;
    }

    // 3) kernels
    let kernels = build_kernels(book, &grids, &y, &cfg.fit.kernel)?;

    // 4) solve with resopt
    let solved = solve(book, &kernels, &cfg.fit)?;
    let q = solved.density;
    let objective_value = solved.objective_value;

    let tol = DensityTolerances::from_tol(1e-6)?;
    q.validate_marginals(tol)?;
    match q.validate_convex_order(tol) {
        Ok(()) => {}
        Err(err) => match cfg.convex_order_validation {
            ConvexOrderValidationMode::Error => return Err(err),
            ConvexOrderValidationMode::Warn => {
                warn!(
                    "convex-order validation warning after calibration: {:?}",
                    err
                );
            }
        },
    }

    // 5) time interpolator
    let interp = cfg.time_interp.build()?;

    Ok(CalibrationResult {
        surface: SanosSurface::new(y, q, interp),
        stats: CalibrationRunStats { objective_value },
    })
}

pub fn calibrate(book: &OptionBook, cfg: &CalibrationConfig) -> SanosResult<SanosSurface> {
    Ok(calibrate_with_stats(book, cfg)?.surface)
}

fn build_completed_grids_and_warm_start(
    grids: &[StrikeGrid],
    y: &Arc<dyn YModel>,
    completion_cfg: &CompletionConfig,
    feasibility_tol: f64,
) -> SanosResult<(Vec<StrikeGrid>, MartingaleDensity)> {
    let tol = DensityTolerances::from_tol(feasibility_tol.max(1e-12))?;
    let mut completed_grids = Vec::with_capacity(grids.len());
    let mut marginals = Vec::with_capacity(grids.len());

    for grid in grids {
        let maturity = grid.maturity();
        let k_internal = grid.strikes();
        let calls_internal: Vec<f64> = k_internal
            .iter()
            .map(|&k| y.call(maturity, 1.0, k))
            .collect::<SanosResult<Vec<_>>>()?;

        let completed = complete_slice_remark_2_8(k_internal, &calls_internal, completion_cfg)?;
        let model_strikes = completed.k[1..].to_vec(); // drop K0=0 for model grids

        if model_strikes.len() != completed.density.len() {
            return Err(crate::error::SanosError::External {
                msg: format!(
                    "completion output mismatch at T={:.6}: strikes={}, density={}",
                    maturity,
                    model_strikes.len(),
                    completed.density.len()
                ),
            });
        }

        let grid_completed = StrikeGrid::new(maturity, model_strikes.clone())?;
        let atoms = model_strikes
            .iter()
            .copied()
            .zip(completed.density.iter().copied())
            .collect::<Vec<_>>();
        let marginal = MarginalDensity::new(maturity, atoms, tol)?;

        info!(
            "remark-2.8 completion at T={:.6}: K1={:.6}, KN={:.6}, dC0={:+.3e}, dC1={:+.3e}, dC2={:+.3e}, dC_last2={:+.3e}, dC_last1={:+.3e}, sum_p={:.6e}, mean_p={:.6e}, min_p={:+.3e}",
            maturity,
            completed.diagnostics.k1,
            completed.diagnostics.k_n,
            completed.diagnostics.d_c0,
            completed.diagnostics.d_c1,
            completed.diagnostics.d_c2,
            completed.diagnostics.d_c_last2,
            completed.diagnostics.d_c_last1,
            completed.diagnostics.sum_p,
            completed.diagnostics.mean_p,
            completed.diagnostics.min_p
        );

        completed_grids.push(grid_completed);
        marginals.push(marginal);
    }

    let q = MartingaleDensity::new(marginals)?;
    q.validate_marginals(tol)?;
    q.validate_convex_order(tol)?;

    Ok((completed_grids, q))
}
