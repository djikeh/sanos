use std::sync::Arc;
// src/calibration/kernel_builder.rs
use log::debug;

use crate::backbone::YModel;
use crate::calibration::config::{CalibrationConfig, OmegaConfig};
use crate::calibration::kernels::{DenseMat, KernelC, KernelSet, KernelTransition};
use crate::error::{SanosError, SanosResult};
use crate::grid::StrikeGrid;
use crate::market::OptionBook;

pub trait KernelBuilder: Send + Sync {
    fn build(
        &self,
        book: &OptionBook,
        grids: &[StrikeGrid],
        y: &Arc<dyn YModel>,
        cfg: &CalibrationConfig,
    ) -> SanosResult<KernelSet>;
}

/// BS / LN time-changed kernel builder (works with TimeChangedLognormal today).
/// Kept generic as long as `YModel<Smoothing=f64>` holds.
#[derive(Debug, Default, Clone)]
pub struct BsKernelBuilder;

impl BsKernelBuilder {
    fn constraint_kernel_value(
        y: &Arc<dyn YModel>,
        omega: OmegaConfig,
        maturity: f64,
        a: f64,
        b: f64,
    ) -> SanosResult<f64> {
        match omega {
            OmegaConfig::Zero => y.linear_call(maturity, a, b),
            OmegaConfig::One => y.call(maturity, a, b),
        }
    }
}

impl KernelBuilder for BsKernelBuilder {
    fn build(
        &self,
        book: &OptionBook,
        grids: &[StrikeGrid],
        y: &Arc<dyn YModel>,
        cfg: &CalibrationConfig,
    ) -> SanosResult<KernelSet> {
        cfg.validate()?;

        if grids.len() != book.len() {
            return Err(SanosError::InvalidOrdering { msg: "grids.len() must match book.len()" });
        }

        let omega = cfg.constraints.omega;

        let mut c_out = Vec::with_capacity(book.len());
        let mut t_out = Vec::with_capacity(book.len().saturating_sub(1));

        for (j, chain) in book.chains().iter().enumerate() {
            let grid = &grids[j];
            if (grid.maturity() - chain.maturity()).abs() > 0.0 {
                // You can loosen this later with a tolerance if needed.
                return Err(SanosError::InvalidOrdering { msg: "grid maturity must equal chain maturity" });
            }

            let maturity = chain.maturity();

            let market_strikes: Vec<f64> = chain.quotes().iter().map(|q| q.k).collect();
            let model_strikes: Vec<f64> = grid.strikes().to_vec();

            let n_mkt = market_strikes.len();
            let n_mod = model_strikes.len();

            if n_mkt == 0 || n_mod == 0 {
                return Err(SanosError::EmptyCollection { what: "market/model strikes" });
            }

            // C_j matrix: use smoothed_call with η (pricing kernel for fit)
            let mut data = Vec::with_capacity(n_mkt * n_mod);
            for &k_mkt in &market_strikes {
                for &k_mod in &model_strikes {
                    // E[(k_mod * Y_T - k_mkt)^+]
                    let v = y.call(maturity, k_mod, k_mkt)?;
                    data.push(v);
                }
            }

            debug!("Built C kernel j={j}, T={maturity}, n_mkt={n_mkt}, n_mod={n_mod}");
            let cmat = DenseMat::new(n_mkt, n_mod, data)?;
            c_out.push(KernelC {
                maturity,
                market_strikes,
                model_strikes,
                c: cmat,
            });

            // Transitions: for j>=1 build U_j and R_j
            if j >= 1 {
                let prev_grid = &grids[j - 1];
                let prev_strikes = prev_grid.strikes();

                let nj = n_mod;
                let njm1 = prev_strikes.len();

                // U_j: Nj x Nj
                let mut u_data = Vec::with_capacity(nj * nj);
                for &a in grid.strikes() {
                    for &b in grid.strikes() {
                        // U_j = (a - b)^+ if omega=0, else use call kernel
                        let v = Self::constraint_kernel_value(y, omega, maturity, a, b)?;
                        u_data.push(v);
                    }
                }
                let u = DenseMat::new(nj, nj, u_data)?;

                // R_j: Nj x N(j-1)
                let mut r_data = Vec::with_capacity(nj * njm1);
                for &a in grid.strikes() {
                    for &b in prev_strikes {
                        let v = Self::constraint_kernel_value(y, omega, maturity, a, b)?;
                        r_data.push(v);
                    }
                }
                let r = DenseMat::new(nj, njm1, r_data)?;

                t_out.push(KernelTransition { maturity, u, r });
            }
        }

        let ks = KernelSet { c: c_out, transitions: t_out };
        ks.validate()?;
        Ok(ks)
    }
}
