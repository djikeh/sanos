use resopt::{
    Bounds, ConstrainedResidualProblem, ConstrainedResidualProblemBuilder,
    LinearEqualities, LinearInequalities, Loss, Matrix,
};

use crate::error::{SanosError, SanosResult};
use crate::fit::config::FitConfig;
use crate::fit::kernels::KernelSet;
use crate::market::OptionBook;

/// Maps the position of each q_j block within the concatenated decision vector x.
#[derive(Debug, Clone)]
pub struct QLayout {
    /// offset[j] = starting index of q_j in x
    pub offsets: Vec<usize>,
    /// sizes[j] = N_j (number of model strikes at maturity j)
    pub sizes: Vec<usize>,
    /// Total number of decision variables (sum of all N_j)
    pub total: usize,
}

/// Weighted quote weight: w = quote.weight / (ask - bid), clamped away from zero.
fn paper_quote_weight(bid: f64, ask: f64, weight: f64) -> f64 {
    let spread = (ask - bid).max(1e-12);
    weight / spread
}

/// Build the resopt constrained residual optimization problem from market data and kernels.
///
/// Problem formulation (Section 4.2 of the SANOS paper):
///
///   minimize   ||W * (A * x - b)||_2^2
///   subject to:
///     x >= 0                          (non-negativity)
///     E_eq * x = d_eq                 (simplex + mean constraints)
///     E_ineq * x <= d_ineq            (calendar arbitrage constraints)
///
/// where x = [q_1; q_2; ...; q_M] is the concatenated density vector.
pub fn build_resopt_problem(
    book: &OptionBook,
    kernels: &KernelSet,
    cfg: &FitConfig,
) -> SanosResult<(ConstrainedResidualProblem, QLayout)> {
    if kernels.c.len() != book.len() {
        return Err(SanosError::InvalidOrdering {
            msg: "kernels.c.len() must match book.len()",
        });
    }

    // --- Build QLayout ---
    let m = kernels.c.len();
    let mut offsets = Vec::with_capacity(m);
    let mut sizes = Vec::with_capacity(m);
    let mut total = 0usize;
    for kc in &kernels.c {
        offsets.push(total);
        let nj = kc.model_strikes.len();
        if nj == 0 {
            return Err(SanosError::EmptyCollection {
                what: "model_strikes",
            });
        }
        sizes.push(nj);
        total += nj;
    }
    let layout = QLayout { offsets, sizes, total };

    // --- Build residual matrix A and target vector b ---
    // A is block-diagonal: each block j is (n_mkt_j x N_j), placed at columns offset[j]
    // b is the concatenation of mid-prices
    // W is applied by pre-multiplying rows of A and b by the weight
    let n_residuals: usize = kernels.c.iter().map(|kc| kc.market_strikes.len()).sum();
    let mut a_data = vec![0.0; n_residuals * total];
    let mut b_data = Vec::with_capacity(n_residuals);

    let mut row = 0;
    for (j, chain) in book.chains().iter().enumerate() {
        let kc = &kernels.c[j];
        let quotes = chain.quotes();
        let n_mkt = quotes.len();

        if kc.market_strikes.len() != n_mkt {
            return Err(SanosError::InvalidOrdering {
                msg: "kernel market_strikes must align with chain quotes",
            });
        }
        if kc.c.nrows != n_mkt || kc.c.ncols != layout.sizes[j] {
            return Err(SanosError::InvalidOrdering {
                msg: "kernel matrix dims mismatch",
            });
        }

        for (r, quote) in quotes.iter().enumerate() {
            let w = paper_quote_weight(quote.bid, quote.ask, quote.weight);
            let mid = quote.mid();

            // Fill row of A (weighted): W * C_j
            for i in 0..layout.sizes[j] {
                let col = layout.offsets[j] + i;
                a_data[row * total + col] = w * kc.c.get(r, i);
            }

            // Fill b (weighted): W * mid
            b_data.push(w * mid);

            row += 1;
        }
    }

    let a_matrix = Matrix::from_row_major(n_residuals, total, a_data).map_err(|e| {
        SanosError::External {
            msg: format!("resopt Matrix::from_row_major failed: {e}"),
        }
    })?;

    // --- Build equality constraints ---
    // For each maturity j:
    //   simplex: 1' . q_j = 1
    //   mean:    K_j' . q_j = 1
    let n_eq_rows = if cfg.constraints.enforce_simplex { m } else { 0 } + m; // always enforce mean
    let mut eq_data = vec![0.0; n_eq_rows * total];
    let mut eq_rhs = Vec::with_capacity(n_eq_rows);

    let mut eq_row = 0;

    // Simplex constraints
    if cfg.constraints.enforce_simplex {
        for j in 0..m {
            for i in 0..layout.sizes[j] {
                let col = layout.offsets[j] + i;
                eq_data[eq_row * total + col] = 1.0;
            }
            eq_rhs.push(1.0);
            eq_row += 1;
        }
    }

    // Mean constraints: K' . q_j = 1
    for j in 0..m {
        for i in 0..layout.sizes[j] {
            let col = layout.offsets[j] + i;
            eq_data[eq_row * total + col] = kernels.c[j].model_strikes[i];
        }
        eq_rhs.push(1.0);
        eq_row += 1;
    }

    // --- Build inequality constraints (calendar / time) ---
    // U_j . q_j >= R_j . q_{j-1}  <=>  -U_j . q_j + R_j . q_{j-1} <= 0
    let mut ineq_data: Vec<f64> = Vec::new();
    let mut ineq_rhs: Vec<f64> = Vec::new();
    let mut n_ineq_rows = 0usize;

    if cfg.constraints.include_time_constraints {
        if kernels.transitions.len() + 1 != m {
            return Err(SanosError::InvalidOrdering {
                msg: "time constraints require transitions.len() = q.len()-1",
            });
        }

        // First pass: count rows
        for tr in &kernels.transitions {
            n_ineq_rows += tr.u.nrows;
            if tr.u_alt.is_some() {
                n_ineq_rows += tr.u.nrows;
            }
        }

        ineq_data.resize(n_ineq_rows * total, 0.0);
        ineq_rhs.resize(n_ineq_rows, 0.0);

        let mut ineq_row = 0;

        for (idx, tr) in kernels.transitions.iter().enumerate() {
            let j = idx + 1;
            let nj = layout.sizes[j];
            let nj_prev = layout.sizes[j - 1];

            if tr.u.nrows != nj || tr.u.ncols != nj {
                return Err(SanosError::InvalidOrdering {
                    msg: "U dimensions must be Nj x Nj",
                });
            }
            if tr.r.nrows != nj || tr.r.ncols != nj_prev {
                return Err(SanosError::InvalidOrdering {
                    msg: "R dimensions must be Nj x N(j-1)",
                });
            }

            // -U_j . q_j + R_j . q_{j-1} <= 0
            add_time_block(
                &mut ineq_data, &mut ineq_row, total,
                &tr.u, &tr.r, &layout, j,
            );

            // Optional second block (omega=Both)
            if let (Some(u_alt), Some(r_alt)) = (&tr.u_alt, &tr.r_alt) {
                if u_alt.nrows != nj || u_alt.ncols != nj {
                    return Err(SanosError::InvalidOrdering {
                        msg: "U_alt dimensions must be Nj x Nj",
                    });
                }
                if r_alt.nrows != nj || r_alt.ncols != nj_prev {
                    return Err(SanosError::InvalidOrdering {
                        msg: "R_alt dimensions must be Nj x N(j-1)",
                    });
                }

                add_time_block(
                    &mut ineq_data, &mut ineq_row, total,
                    u_alt, r_alt, &layout, j,
                );
            } else if tr.u_alt.is_some() || tr.r_alt.is_some() {
                return Err(SanosError::InvalidOrdering {
                    msg: "u_alt and r_alt must both be present or both absent",
                });
            }
        }
    }

    // --- Build bounds ---
    let bounds = if cfg.constraints.enforce_nonnegativity {
        Bounds::nonnegative(total)
    } else {
        Bounds::free(total)
    };

    // --- Assemble the problem ---
    let mut builder = ConstrainedResidualProblemBuilder::new()
        .matrix(a_matrix)
        .target(b_data)
        .loss(Loss::L2Squared)
        .bounds(bounds);

    // Add equality constraints
    if n_eq_rows > 0 {
        let eq_matrix = Matrix::from_row_major(n_eq_rows, total, eq_data).map_err(|e| {
            SanosError::External {
                msg: format!("resopt equality Matrix failed: {e}"),
            }
        })?;
        let equalities = LinearEqualities::new(eq_matrix, eq_rhs).map_err(|e| {
            SanosError::External {
                msg: format!("resopt LinearEqualities failed: {e}"),
            }
        })?;
        builder = builder.add_equalities(equalities);
    }

    // Add inequality constraints
    if n_ineq_rows > 0 {
        let ineq_matrix =
            Matrix::from_row_major(n_ineq_rows, total, ineq_data).map_err(|e| {
                SanosError::External {
                    msg: format!("resopt inequality Matrix failed: {e}"),
                }
            })?;
        let inequalities = LinearInequalities::new(ineq_matrix, ineq_rhs).map_err(|e| {
            SanosError::External {
                msg: format!("resopt LinearInequalities failed: {e}"),
            }
        })?;
        builder = builder.add_inequalities(inequalities);
    }

    let problem = builder.build().map_err(|e| SanosError::External {
        msg: format!("resopt problem build failed: {e}"),
    })?;

    Ok((problem, layout))
}

/// Write one time-constraint block: -U . q_j + R . q_{j-1} <= 0
fn add_time_block(
    ineq_data: &mut [f64],
    ineq_row: &mut usize,
    total_cols: usize,
    u: &crate::fit::kernels::DenseMat,
    r: &crate::fit::kernels::DenseMat,
    layout: &QLayout,
    j: usize,
) {
    let nj = layout.sizes[j];
    let nj_prev = layout.sizes[j - 1];

    for row_k in 0..u.nrows {
        // -U_j columns (current maturity)
        for i in 0..nj {
            let col = layout.offsets[j] + i;
            ineq_data[*ineq_row * total_cols + col] = -u.get(row_k, i);
        }
        // +R_j columns (previous maturity)
        for i in 0..nj_prev {
            let col = layout.offsets[j - 1] + i;
            ineq_data[*ineq_row * total_cols + col] = r.get(row_k, i);
        }
        *ineq_row += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::backbone::bs::bs_call_forward_norm;
    use crate::backbone::{TimeChangedLognormal, YModel};
    use crate::fit::kernel_builder::build_kernels;
    use crate::grid::StrikeGrid;
    use crate::market::{CallQuote, OptionBook, OptionChain};
    use crate::term::PiecewiseLinearCurve;

    fn sample_book_and_grids() -> (OptionBook, Vec<StrikeGrid>, Arc<TimeChangedLognormal>) {
        let t0 = 0.5;
        let t1 = 1.0;
        let w0 = 0.04;
        let w1 = 0.16;

        let q0 = vec![
            CallQuote::new(0.9, bs_call_forward_norm(0.9, w0).unwrap(), bs_call_forward_norm(0.9, w0).unwrap(), 1.0).unwrap(),
            CallQuote::new(1.0, bs_call_forward_norm(1.0, w0).unwrap(), bs_call_forward_norm(1.0, w0).unwrap(), 1.0).unwrap(),
            CallQuote::new(1.1, bs_call_forward_norm(1.1, w0).unwrap(), bs_call_forward_norm(1.1, w0).unwrap(), 1.0).unwrap(),
        ];
        let q1 = vec![
            CallQuote::new(0.9, bs_call_forward_norm(0.9, w1).unwrap(), bs_call_forward_norm(0.9, w1).unwrap(), 1.0).unwrap(),
            CallQuote::new(1.0, bs_call_forward_norm(1.0, w1).unwrap(), bs_call_forward_norm(1.0, w1).unwrap(), 1.0).unwrap(),
            CallQuote::new(1.1, bs_call_forward_norm(1.1, w1).unwrap(), bs_call_forward_norm(1.1, w1).unwrap(), 1.0).unwrap(),
        ];

        let c0 = OptionChain::new(t0, q0).unwrap();
        let c1 = OptionChain::new(t1, q1).unwrap();
        let book = OptionBook::new(vec![c0, c1]).unwrap();

        let grid0 = StrikeGrid::new(t0, vec![0.9, 1.0, 1.1]).unwrap();
        let grid1 = StrikeGrid::new(t1, vec![0.9, 1.0, 1.1]).unwrap();

        let curve = PiecewiseLinearCurve::new(vec![(t0, w0), (t1, w1)]).unwrap();
        let y = Arc::new(TimeChangedLognormal::new(curve, 1.0));

        (book, vec![grid0, grid1], y)
    }

    #[test]
    fn build_problem_succeeds() {
        let (book, grids, y) = sample_book_and_grids();
        let y_dyn = y as Arc<dyn YModel>;
        let cfg = FitConfig::default();
        let kernels = build_kernels(&book, &grids, &y_dyn, &cfg.kernel).unwrap();

        let (problem, layout) = build_resopt_problem(&book, &kernels, &cfg).unwrap();

        assert_eq!(layout.total, 6); // 3 + 3
        assert_eq!(layout.offsets, vec![0, 3]);
        assert_eq!(layout.sizes, vec![3, 3]);

        let summary = problem.summary();
        assert_eq!(summary.x_dim, 6);
    }

    #[test]
    fn layout_offsets_are_correct() {
        let (book, grids, y) = sample_book_and_grids();
        let y_dyn = y as Arc<dyn YModel>;
        let cfg = FitConfig::default();
        let kernels = build_kernels(&book, &grids, &y_dyn, &cfg.kernel).unwrap();

        let (_problem, layout) = build_resopt_problem(&book, &kernels, &cfg).unwrap();

        for j in 0..layout.sizes.len() {
            assert_eq!(layout.offsets[j], layout.sizes[..j].iter().sum::<usize>());
        }
    }
}
