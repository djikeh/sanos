use crate::backbone::{build_backbone, BackboneConfig};
use crate::error::SanosResult;
use crate::fit::{build_kernels, extract_density, solve_lp};
use crate::fit::lp::builder::{LpBuilder, SanosLpBuilder};
use crate::grid::build_strike_grids;
use crate::market::OptionBook;
use crate::surface::SanosSurface;

use super::config::CalibrationConfig;

pub fn calibrate(book: &OptionBook, cfg: &CalibrationConfig) -> SanosResult<SanosSurface> {
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
    let built_lp = lp_builder.build(book, &kernels, &cfg.fit)?;

    // 5) Solve LP
    let sol = solve_lp(&built_lp.model, &cfg.fit.solver)?;

    // 6) Extract martingale density
    let q = extract_density(&built_lp.layout, &sol, &grids)?;

    // 7) Time interpolator
    let interp = cfg.time_interp.build()?;

    Ok(SanosSurface::new(y, q, interp))
}
