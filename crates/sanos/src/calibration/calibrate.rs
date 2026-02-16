use crate::backbone::{build_backbone, BackboneConfig};
use crate::density::DensityTolerances;
use crate::error::SanosResult;
use crate::fit::lp::builder::{LpBuilder, SanosLpBuilder};
use crate::fit::{
    add_l1_density_anchor, build_kernels, build_linear_density_initialization, extract_density,
    solve_lp, LinearDensityInitialization,
};
use crate::grid::build_strike_grids;
use crate::market::OptionBook;
use crate::surface::SanosSurface;

use super::config::CalibrationConfig;

#[derive(Debug, Clone)]
pub struct CalibrationRunStats {
    pub objective_value: f64,
    pub initialization: Option<LinearDensityInitialization>,
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
    let y = build_backbone(book, &cfg.backbone)?;

    // 2) strike grids (reuse existing ATM policy config build)
    let atm_policy = match &cfg.backbone {
        BackboneConfig::BsTimeChanged(bs_cfg) => bs_cfg.atm_policy.build()?,
    };
    let grids = build_strike_grids(book, atm_policy.as_ref(), &cfg.grid)?;

    // 3) kernels
    let kernels = build_kernels(book, &grids, &y, &cfg.fit.kernel)?;

    // 4) LP build
    let lp_builder = SanosLpBuilder::default();
    let mut built_lp = lp_builder.build(book, &kernels, &cfg.fit)?;

    // 5) Optional linear-density initialization + LP anchor.
    let initialization = build_linear_density_initialization(
        book,
        &grids,
        &cfg.fit.initialization,
        &cfg.fit.solver,
    )?;
    if let Some(init) = initialization.as_ref() {
        add_l1_density_anchor(
            &mut built_lp.model,
            &built_lp.layout.q_var_ids,
            &init.projected,
            cfg.fit.initialization.anchor_l1_weight,
        )?;
    }

    // 6) Solve LP
    let sol = solve_lp(&built_lp.model, &cfg.fit.solver)?;
    let objective_value = evaluate_objective_value(&built_lp.model, &sol.values);

    // 7) Extract martingale density
    let q = extract_density(&built_lp.layout, &sol, &grids)?;
    let tol = DensityTolerances::from_tol(1e-10)?;
    q.validate_marginals(tol)?;
    q.validate_convex_order(tol)?;

    // 8) Time interpolator
    let interp = cfg.time_interp.build()?;

    Ok(CalibrationResult {
        surface: SanosSurface::new(y, q, interp),
        stats: CalibrationRunStats {
            objective_value,
            initialization,
        },
    })
}

pub fn calibrate(book: &OptionBook, cfg: &CalibrationConfig) -> SanosResult<SanosSurface> {
    Ok(calibrate_with_stats(book, cfg)?.surface)
}

fn evaluate_objective_value(model: &crate::fit::lp::model::LpModel, values: &[f64]) -> f64 {
    model
        .objective
        .terms
        .iter()
        .map(|t| t.coef * values[t.var])
        .sum()
}
