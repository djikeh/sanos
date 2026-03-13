use log::debug;
use std::sync::Arc;

use crate::backbone::YModel;
use crate::error::{SanosError, SanosResult};
use crate::fit::config::{KernelConfig, OmegaConfig};
use crate::fit::kernels::{DenseMat, KernelC, KernelSet, KernelTransition};
use crate::grid::StrikeGrid;
use crate::market::OptionBook;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConstraintKernelKind {
    Zero,
    One,
}

fn call_for_constraints(
    y: &Arc<dyn YModel>,
    kind: ConstraintKernelKind,
    t: f64,
    a: f64,
    b: f64,
) -> SanosResult<f64> {
    match kind {
        ConstraintKernelKind::Zero => y.linear_call(t, a, b),
        ConstraintKernelKind::One => y.call(t, a, b),
    }
}

fn build_transition_mats(
    y: &Arc<dyn YModel>,
    kind: ConstraintKernelKind,
    maturity: f64,
    prev_maturity: f64,
    strikes: &[f64],
    prev_strikes: &[f64],
) -> SanosResult<(DenseMat, DenseMat)> {
    let nj = strikes.len();
    let njm1 = prev_strikes.len();

    let mut u_data = Vec::with_capacity(nj * nj);
    for &a in strikes {
        for &b in strikes {
            u_data.push(call_for_constraints(y, kind, maturity, a, b)?);
        }
    }
    let u = DenseMat::new(nj, nj, u_data)?;

    let mut r_data = Vec::with_capacity(nj * njm1);
    for &a in strikes {
        for &b in prev_strikes {
            r_data.push(call_for_constraints(y, kind, prev_maturity, a, b)?);
        }
    }
    let r = DenseMat::new(nj, njm1, r_data)?;

    Ok((u, r))
}

fn primary_and_secondary_kinds(
    omega: OmegaConfig,
) -> (ConstraintKernelKind, Option<ConstraintKernelKind>) {
    match omega {
        OmegaConfig::Zero => (ConstraintKernelKind::Zero, None),
        OmegaConfig::One => (ConstraintKernelKind::One, None),
        OmegaConfig::Both => (ConstraintKernelKind::One, Some(ConstraintKernelKind::Zero)),
    }
}

/// Build SANOS kernel blocks from book, strike grids, and backbone.
///
/// - C_j uses `y.call` (market fit kernel).
/// - U/R transitions use the `omega` switch:
///   `omega = Zero` -> `linear_call`
///   `omega = One` -> `call`
pub fn build_kernels(
    book: &OptionBook,
    grids: &[StrikeGrid],
    y: &Arc<dyn YModel>,
    cfg: &KernelConfig,
) -> SanosResult<KernelSet> {
    if grids.len() != book.len() {
        return Err(SanosError::InvalidOrdering {
            msg: "grids.len() must match book.len()",
        });
    }

    let (primary_kind, secondary_kind) = primary_and_secondary_kinds(cfg.omega);

    let mut c_out = Vec::with_capacity(book.len());
    let mut t_out = Vec::with_capacity(book.len().saturating_sub(1));

    for (j, chain) in book.chains().iter().enumerate() {
        let grid = &grids[j];
        let maturity = chain.maturity();

        // You can relax this later if needed.
        if grid.maturity() != maturity {
            return Err(SanosError::InvalidOrdering {
                msg: "grid maturity must equal chain maturity",
            });
        }

        let market_strikes: Vec<f64> = chain.quotes().iter().map(|q| q.k).collect();
        let model_strikes: Vec<f64> = grid.strikes().to_vec();

        let n_mkt = market_strikes.len();
        let n_mod = model_strikes.len();

        if n_mkt == 0 || n_mod == 0 {
            return Err(SanosError::EmptyCollection {
                what: "market/model strikes",
            });
        }

        // C_j: n_mkt x n_mod
        let mut c_data = Vec::with_capacity(n_mkt * n_mod);
        for &k_mkt in &market_strikes {
            for &k_mod in &model_strikes {
                // E[(k_mod Y_T - k_mkt)^+]
                c_data.push(y.call(maturity, k_mod, k_mkt)?);
            }
        }

        debug!("Built C kernel j={j}, T={maturity}, n_mkt={n_mkt}, n_mod={n_mod}");
        let cmat = DenseMat::new(n_mkt, n_mod, c_data)?;
        c_out.push(KernelC {
            maturity,
            market_strikes,
            model_strikes,
            c: cmat,
        });

        // Transitions (U/R) for j>=1
        if j >= 1 {
            let prev_grid = &grids[j - 1];
            let prev_strikes = prev_grid.strikes();
            let prev_maturity = prev_grid.maturity();

            let (u, r) = build_transition_mats(
                y,
                primary_kind,
                maturity,
                prev_maturity,
                grid.strikes(),
                prev_strikes,
            )?;
            let (u_alt, r_alt) = match secondary_kind {
                Some(kind) => {
                    let (u2, r2) = build_transition_mats(
                        y,
                        kind,
                        maturity,
                        prev_maturity,
                        grid.strikes(),
                        prev_strikes,
                    )?;
                    (Some(u2), Some(r2))
                }
                None => (None, None),
            };

            t_out.push(KernelTransition {
                maturity,
                u,
                r,
                u_alt,
                r_alt,
            });
        }
    }

    let ks = KernelSet {
        c: c_out,
        transitions: t_out,
    };
    ks.validate()?;
    Ok(ks)
}
