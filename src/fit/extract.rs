use crate::density::{DensityTolerances, MarginalDensity, MartingaleDensity};
use crate::error::{SanosError, SanosResult};
use crate::fit::lp::builder::LpLayout;
use crate::fit::solver::LpSolution;
use crate::grid::StrikeGrid;

pub fn extract_density(
    layout: &LpLayout,
    sol: &LpSolution,
    grids: &[StrikeGrid],
) -> SanosResult<MartingaleDensity> {
    if layout.q_var_ids.len() != grids.len() {
        return Err(SanosError::InvalidOrdering {
            msg: "layout.q_var_ids must align with grids",
        });
    }

    let tol = DensityTolerances::from_tol(1e-10)?; // pragmatic default

    let mut marginals = Vec::with_capacity(grids.len());

    for (j, grid) in grids.iter().enumerate() {
        let q_ids = &layout.q_var_ids[j];
        if q_ids.len() != grid.strikes().len() {
            return Err(SanosError::InvalidOrdering {
                msg: "q vector length must equal strike grid length",
            });
        }

        let mut atoms = Vec::with_capacity(q_ids.len());
        for (i, &vid) in q_ids.iter().enumerate() {
            let k = grid.strikes()[i];
            let q = *sol.values.get(vid).ok_or(SanosError::InvalidOrdering {
                msg: "solution vector too short",
            })?;

            atoms.push((k, q));
        }

        let m = MarginalDensity::new(grid.maturity(), atoms, tol)?;
        marginals.push(m);
    }

    MartingaleDensity::new(marginals)
}
