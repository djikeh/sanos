use resopt::{DefaultSolver, Solver};

use crate::density::MartingaleDensity;
use crate::error::{SanosError, SanosResult};
use crate::fit::builder::build_resopt_problem;
use crate::fit::config::FitConfig;
use crate::fit::extract::extract_density;
use crate::fit::kernels::KernelSet;
use crate::market::OptionBook;

#[derive(Debug, Clone)]
pub struct SolveResult {
    pub density: MartingaleDensity,
    pub objective_value: f64,
}

/// Solve the SANOS calibration problem using resopt (L2 objective with Clarabel).
///
/// 1. Builds the constrained residual problem from (book, kernels, config).
/// 2. Solves with the default resopt solver (Clarabel).
/// 3. Extracts the martingale density from the solution vector.
pub fn solve(
    book: &OptionBook,
    kernels: &KernelSet,
    cfg: &FitConfig,
) -> SanosResult<SolveResult> {
    cfg.validate()?;
    kernels.validate()?;

    let (problem, layout) = build_resopt_problem(book, kernels, cfg)?;

    let result = DefaultSolver::new().solve(&problem).map_err(|e| {
        SanosError::External {
            msg: format!("resopt solve failed: {e}"),
        }
    })?;

    if !result.is_solved() {
        return Err(SanosError::External {
            msg: format!("resopt solver did not converge: {:?}", result.status()),
        });
    }

    let solution = result.solution().ok_or_else(|| SanosError::External {
        msg: "resopt returned solved status but no solution".to_string(),
    })?;

    let x = solution.x();
    let objective_value = result.objective_value().unwrap_or(0.0);
    let density = extract_density(&layout, x, kernels)?;

    Ok(SolveResult {
        density,
        objective_value,
    })
}
