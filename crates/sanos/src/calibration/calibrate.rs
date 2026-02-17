use crate::backbone::{build_backbone_with_total_variances, BackboneConfig};
use crate::density::DensityTolerances;
use crate::error::SanosResult;
use crate::fit::lp::builder::{LpBuilder, SanosLpBuilder};
use crate::fit::{add_martingale_density_warm_start, build_kernels, build_warm_start, solve};
use crate::grid::build_strike_grids_with_variances;
use crate::market::OptionBook;
use crate::surface::SanosSurface;
use log::warn;

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
    let grids = build_strike_grids_with_variances(
        book,
        atm_policy.as_ref(),
        &cfg.grid,
        Some(&total_variances),
    )?;

    // 3) kernels
    let kernels = build_kernels(book, &grids, &y, &cfg.fit.kernel)?;

    // 4) LP build
    let lp_builder = SanosLpBuilder;
    let mut built_lp = lp_builder.build(book, &kernels, &cfg.fit)?;

    // 5) optional warm-start density
    let warm_start_density = build_warm_start(&grids, &y, &cfg.fit.initialization)?;
    if let Some(warm_start) = warm_start_density.as_ref() {
        add_martingale_density_warm_start(&mut built_lp.model, &built_lp.layout, warm_start)?;
    }

    // 6) solve LP and 7) extract martingale density + objective value
    let solved = solve(&built_lp.model, &built_lp.layout, &kernels, &cfg.fit.solver)?;
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

    // 8) time interpolator
    let interp = cfg.time_interp.build()?;

    Ok(CalibrationResult {
        surface: SanosSurface::new(y, q, interp),
        stats: CalibrationRunStats { objective_value },
    })
}

pub fn calibrate(book: &OptionBook, cfg: &CalibrationConfig) -> SanosResult<SanosSurface> {
    Ok(calibrate_with_stats(book, cfg)?.surface)
}
