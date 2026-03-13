use crate::density::{DensityTolerances, MarginalDensity, MartingaleDensity};
use crate::error::{SanosError, SanosResult};
use crate::fit::builder::QLayout;
use crate::fit::kernels::KernelSet;

/// Extract a `MartingaleDensity` from the raw solution vector x and layout.
pub fn extract_density(
    layout: &QLayout,
    x: &[f64],
    kernels: &KernelSet,
) -> SanosResult<MartingaleDensity> {
    if layout.sizes.len() != kernels.c.len() {
        return Err(SanosError::InvalidOrdering {
            msg: "layout sizes must align with kernel slices",
        });
    }
    if x.len() < layout.total {
        return Err(SanosError::InvalidOrdering {
            msg: "solution vector too short for layout",
        });
    }

    let tol = DensityTolerances::from_tol(1e-6)?;

    let mut marginals = Vec::with_capacity(kernels.c.len());

    for (j, kc) in kernels.c.iter().enumerate() {
        if layout.sizes[j] != kc.model_strikes.len() {
            return Err(SanosError::InvalidOrdering {
                msg: "q vector length must equal kernel model strikes length",
            });
        }

        let offset = layout.offsets[j];
        let mut atoms = Vec::with_capacity(layout.sizes[j]);
        for i in 0..layout.sizes[j] {
            let q = x[offset + i];
            let k = kc.model_strikes[i];
            atoms.push((k, q));
        }

        let m = MarginalDensity::new(kc.maturity, atoms, tol)?;
        marginals.push(m);
    }

    MartingaleDensity::new(marginals)
}
