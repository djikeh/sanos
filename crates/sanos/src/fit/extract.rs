use crate::density::{DensityTolerances, MarginalDensity, MartingaleDensity};
use crate::error::{SanosError, SanosResult};
use crate::fit::kernels::KernelSet;
use crate::fit::lp::builder::LpLayout;
use crate::fit::solver::LpSolution;

pub fn extract_density(
    layout: &LpLayout,
    sol: &LpSolution,
    kernels: &KernelSet,
) -> SanosResult<MartingaleDensity> {
    if layout.q_var_ids.len() != kernels.c.len() {
        return Err(SanosError::InvalidOrdering {
            msg: "layout.q_var_ids must align with kernel slices",
        });
    }

    let tol = DensityTolerances::from_tol(1e-6)?;

    let mut marginals = Vec::with_capacity(kernels.c.len());

    for (j, kc) in kernels.c.iter().enumerate() {
        let q_ids = &layout.q_var_ids[j];
        if q_ids.len() != kc.model_strikes.len() {
            return Err(SanosError::InvalidOrdering {
                msg: "q vector length must equal kernel model strikes length",
            });
        }

        let mut atoms = Vec::with_capacity(q_ids.len());
        for (&vid, &k) in q_ids.iter().zip(kc.model_strikes.iter()) {
            let q = *sol.values.get(vid).ok_or(SanosError::InvalidOrdering {
                msg: "solution vector too short",
            })?;

            atoms.push((k, q));
        }

        let m = MarginalDensity::new(kc.maturity, atoms, tol)?;
        marginals.push(m);
    }

    MartingaleDensity::new(marginals)
}
