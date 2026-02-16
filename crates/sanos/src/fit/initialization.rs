use log::info;

use crate::error::{SanosError, SanosResult};
use crate::fit::config::{InitPriceProxyConfig, InitializationConfig, LpSolverConfig};
use crate::fit::lp::model::{LinTerm, LpModel, Sense};
use crate::fit::solve_lp;
use crate::grid::StrikeGrid;
use crate::market::{CallQuote, OptionBook};

#[derive(Debug, Clone)]
pub struct RawLinearDensity {
    /// Discrete strike slopes d_i = (C_{i+1} - C_i) / (K_{i+1} - K_i).
    pub slopes: Vec<f64>,
    /// Node-based raw density on the same strike nodes.
    pub density: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct FeasibleDensityDiagnostics {
    pub mass: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub near_zero_atoms: usize,
}

#[derive(Debug, Clone)]
pub struct LinearDensitySliceDiagnostics {
    pub maturity: f64,
    pub projection_needed: bool,
    pub raw_mass: f64,
    pub raw_mean: f64,
    pub raw_min: f64,
    pub raw_max: f64,
    pub projected: FeasibleDensityDiagnostics,
    pub l1_distance: f64,
    pub l2_distance: f64,
}

#[derive(Debug, Clone)]
pub struct LinearDensityInitialization {
    pub projected: Vec<Vec<f64>>, // projected[j][i]
    pub diagnostics: Vec<LinearDensitySliceDiagnostics>,
}

/// Build raw node-based linear density from discrete call prices.
///
/// Discretization used (node-based):
/// - Given strikes `K_0 < ... < K_{n-1}` and calls `C_i = C(K_i)`, define slopes
///   `d_i = (C_{i+1} - C_i) / (K_{i+1} - K_i)` for `i=0..n-2`.
/// - Raw node masses are:
///   `p_0 = 1 + d_0`,
///   `p_i = d_i - d_{i-1}` for interior nodes,
///   `p_{n-1} = -d_{n-2}`.
/// This is the discrete second-difference construction in the spirit of SANOS Remark 2.11.
pub fn compute_raw_linear_density(strikes: &[f64], calls: &[f64]) -> SanosResult<RawLinearDensity> {
    validate_strikes_and_calls(strikes, calls)?;

    let n = strikes.len();
    let mut slopes = Vec::with_capacity(n - 1);
    for i in 0..(n - 1) {
        let dk = strikes[i + 1] - strikes[i];
        let slope = (calls[i + 1] - calls[i]) / dk;
        slopes.push(slope);
    }

    let mut density = vec![0.0; n];
    density[0] = 1.0 + slopes[0];
    for i in 1..(n - 1) {
        density[i] = slopes[i] - slopes[i - 1];
    }
    density[n - 1] = -slopes[n - 2];

    Ok(RawLinearDensity { slopes, density })
}

/// Project a raw density vector onto:
/// - p_i >= 0
/// - sum_i p_i = 1
/// - sum_i p_i * K_i = 1
///
/// via L1 projection:
///   minimize sum_i u_i
///   s.t. -u_i <= p_i - raw_i <= u_i
pub fn project_density_with_martingale_constraints(
    strikes: &[f64],
    raw: &[f64],
    solver_cfg: &LpSolverConfig,
    tol: f64,
) -> SanosResult<Vec<f64>> {
    validate_density_inputs(strikes, raw)?;
    if !tol.is_finite() || tol < 0.0 {
        return Err(SanosError::InvalidBound {
            field: "projection_tol",
            value: tol,
            min: 0.0,
            max: f64::INFINITY,
        });
    }

    let k_min = strikes[0];
    let k_max = strikes[strikes.len() - 1];
    if 1.0 < k_min - tol || 1.0 > k_max + tol {
        return Err(SanosError::InvalidOrdering {
            msg: "cannot enforce mean=1: strike grid does not bracket 1",
        });
    }

    let mut lp = LpModel::new();
    let mut p_vars = Vec::with_capacity(raw.len());
    let mut u_vars = Vec::with_capacity(raw.len());

    for i in 0..raw.len() {
        p_vars.push(lp.add_var(format!("proj_p_{i}"), 0.0, f64::INFINITY)?);
        u_vars.push(lp.add_var(format!("proj_u_{i}"), 0.0, f64::INFINITY)?);
    }

    let simplex_terms = p_vars
        .iter()
        .map(|&v| LinTerm { var: v, coef: 1.0 })
        .collect::<Vec<_>>();
    lp.add_constraint("proj_simplex", simplex_terms, Sense::Eq, 1.0)?;

    let mean_terms = p_vars
        .iter()
        .enumerate()
        .map(|(i, &v)| LinTerm {
            var: v,
            coef: strikes[i],
        })
        .collect::<Vec<_>>();
    lp.add_constraint("proj_mean", mean_terms, Sense::Eq, 1.0)?;

    for i in 0..raw.len() {
        // p_i - u_i <= raw_i
        lp.add_constraint(
            format!("proj_dev_pos_{i}"),
            vec![
                LinTerm {
                    var: p_vars[i],
                    coef: 1.0,
                },
                LinTerm {
                    var: u_vars[i],
                    coef: -1.0,
                },
            ],
            Sense::Le,
            raw[i],
        )?;

        // raw_i - p_i <= u_i  <=> -p_i - u_i <= -raw_i
        lp.add_constraint(
            format!("proj_dev_neg_{i}"),
            vec![
                LinTerm {
                    var: p_vars[i],
                    coef: -1.0,
                },
                LinTerm {
                    var: u_vars[i],
                    coef: -1.0,
                },
            ],
            Sense::Le,
            -raw[i],
        )?;

        lp.add_obj_term(u_vars[i], 1.0)?;
    }

    let sol = solve_lp(&lp, solver_cfg)?;
    let mut out = Vec::with_capacity(p_vars.len());
    for &v in &p_vars {
        out.push(sol.values[v]);
    }
    Ok(out)
}

pub fn build_linear_density_initialization(
    book: &OptionBook,
    grids: &[StrikeGrid],
    init_cfg: &InitializationConfig,
    solver_cfg: &LpSolverConfig,
) -> SanosResult<Option<LinearDensityInitialization>> {
    if !init_cfg.enabled {
        return Ok(None);
    }
    if book.len() != grids.len() {
        return Err(SanosError::InvalidOrdering {
            msg: "book and grids must have same length",
        });
    }

    let mut projected = Vec::with_capacity(grids.len());
    let mut diagnostics = Vec::with_capacity(grids.len());

    for (chain, grid) in book.chains().iter().zip(grids.iter()) {
        let market_strikes: Vec<f64> = chain.quotes().iter().map(|q| q.k).collect();
        let market_calls: Vec<f64> = chain
            .quotes()
            .iter()
            .map(|q| quote_proxy_value(*q, init_cfg.price_proxy))
            .collect();
        let model_calls =
            interpolate_calls_on_grid(&market_strikes, &market_calls, grid.strikes())?;

        let raw = compute_raw_linear_density(grid.strikes(), &model_calls)?;
        let raw_stats = summarize_density(&raw.density, grid.strikes(), init_cfg.near_zero_tol)?;
        let already_feasible = is_feasible(
            &raw.density,
            grid.strikes(),
            init_cfg.feasibility_tol,
            init_cfg.feasibility_tol,
        )?;

        let projected_slice = if already_feasible {
            raw.density.clone()
        } else {
            project_density_with_martingale_constraints(
                grid.strikes(),
                &raw.density,
                solver_cfg,
                init_cfg.projection_tol,
            )?
        };

        let proj_stats =
            summarize_density(&projected_slice, grid.strikes(), init_cfg.near_zero_tol)?;
        let projected_feasible = is_feasible(
            &projected_slice,
            grid.strikes(),
            init_cfg.projection_tol.max(1e-10),
            init_cfg.projection_tol.max(1e-10),
        )?;
        if !projected_feasible {
            return Err(SanosError::InvalidOrdering {
                msg: "projected initialization density is not feasible",
            });
        }

        let l1_distance = projected_slice
            .iter()
            .zip(raw.density.iter())
            .map(|(a, b)| (a - b).abs())
            .sum::<f64>();
        let l2_distance = projected_slice
            .iter()
            .zip(raw.density.iter())
            .map(|(a, b)| {
                let d = a - b;
                d * d
            })
            .sum::<f64>()
            .sqrt();

        if !already_feasible {
            info!(
                "linear-density projection at T={:.6}: moved L1={:.3e}, L2={:.3e}",
                chain.maturity(),
                l1_distance,
                l2_distance
            );
        }

        diagnostics.push(LinearDensitySliceDiagnostics {
            maturity: chain.maturity(),
            projection_needed: !already_feasible,
            raw_mass: raw_stats.mass,
            raw_mean: raw_stats.mean,
            raw_min: raw_stats.min,
            raw_max: raw_stats.max,
            projected: proj_stats,
            l1_distance,
            l2_distance,
        });
        projected.push(projected_slice);
    }

    Ok(Some(LinearDensityInitialization {
        projected,
        diagnostics,
    }))
}

pub fn add_l1_density_anchor(
    model: &mut LpModel,
    q_var_ids: &[Vec<usize>],
    anchors: &[Vec<f64>],
    weight: f64,
) -> SanosResult<()> {
    if !weight.is_finite() {
        return Err(SanosError::NonFinite {
            field: "initialization.anchor_l1_weight",
            value: weight,
        });
    }
    if weight <= 0.0 {
        return Ok(());
    }
    if q_var_ids.len() != anchors.len() {
        return Err(SanosError::InvalidOrdering {
            msg: "q layout and anchor slices must align",
        });
    }

    for j in 0..q_var_ids.len() {
        if q_var_ids[j].len() != anchors[j].len() {
            return Err(SanosError::InvalidOrdering {
                msg: "q slice and anchor slice sizes must align",
            });
        }

        for i in 0..q_var_ids[j].len() {
            let qid = q_var_ids[j][i];
            let anchor = anchors[j][i];
            if !anchor.is_finite() {
                return Err(SanosError::NonFinite {
                    field: "initialization.anchor",
                    value: anchor,
                });
            }

            let dev = model.add_var(format!("anchor_dev_{j}_{i}"), 0.0, f64::INFINITY)?;

            // q - dev <= anchor
            model.add_constraint(
                format!("anchor_pos_{j}_{i}"),
                vec![
                    LinTerm {
                        var: qid,
                        coef: 1.0,
                    },
                    LinTerm {
                        var: dev,
                        coef: -1.0,
                    },
                ],
                Sense::Le,
                anchor,
            )?;

            // anchor - q <= dev  <=>  -q - dev <= -anchor
            model.add_constraint(
                format!("anchor_neg_{j}_{i}"),
                vec![
                    LinTerm {
                        var: qid,
                        coef: -1.0,
                    },
                    LinTerm {
                        var: dev,
                        coef: -1.0,
                    },
                ],
                Sense::Le,
                -anchor,
            )?;

            model.add_obj_term(dev, weight)?;
        }
    }

    Ok(())
}

fn validate_strikes_and_calls(strikes: &[f64], calls: &[f64]) -> SanosResult<()> {
    if strikes.len() != calls.len() {
        return Err(SanosError::InvalidOrdering {
            msg: "strikes and calls must have same length",
        });
    }
    if strikes.len() < 2 {
        return Err(SanosError::InvalidOrdering {
            msg: "at least 2 strike nodes are required",
        });
    }
    for i in 0..strikes.len() {
        let k = strikes[i];
        let c = calls[i];
        if !k.is_finite() {
            return Err(SanosError::NonFinite {
                field: "strike",
                value: k,
            });
        }
        if !c.is_finite() {
            return Err(SanosError::NonFinite {
                field: "call",
                value: c,
            });
        }
        if k <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "strike",
                value: k,
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }
        if i > 0 && strikes[i] <= strikes[i - 1] {
            return Err(SanosError::InvalidOrdering {
                msg: "strikes must be strictly increasing",
            });
        }
    }
    Ok(())
}

fn validate_density_inputs(strikes: &[f64], density: &[f64]) -> SanosResult<()> {
    if strikes.len() != density.len() {
        return Err(SanosError::InvalidOrdering {
            msg: "strikes and density must have same length",
        });
    }
    if strikes.len() < 2 {
        return Err(SanosError::InvalidOrdering {
            msg: "at least 2 strike nodes are required",
        });
    }
    for i in 0..strikes.len() {
        let k = strikes[i];
        let q = density[i];
        if !k.is_finite() {
            return Err(SanosError::NonFinite {
                field: "strike",
                value: k,
            });
        }
        if !q.is_finite() {
            return Err(SanosError::NonFinite {
                field: "density",
                value: q,
            });
        }
        if i > 0 && strikes[i] <= strikes[i - 1] {
            return Err(SanosError::InvalidOrdering {
                msg: "strikes must be strictly increasing",
            });
        }
    }
    Ok(())
}

fn summarize_density(
    density: &[f64],
    strikes: &[f64],
    near_zero_tol: f64,
) -> SanosResult<FeasibleDensityDiagnostics> {
    validate_density_inputs(strikes, density)?;
    if !near_zero_tol.is_finite() || near_zero_tol < 0.0 {
        return Err(SanosError::InvalidBound {
            field: "near_zero_tol",
            value: near_zero_tol,
            min: 0.0,
            max: f64::INFINITY,
        });
    }

    let mass = density.iter().sum::<f64>();
    let mean = density
        .iter()
        .zip(strikes.iter())
        .map(|(q, k)| q * k)
        .sum::<f64>();
    let min = density
        .iter()
        .copied()
        .fold(f64::INFINITY, |a, b| a.min(b));
    let max = density
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, |a, b| a.max(b));
    let near_zero_atoms = density
        .iter()
        .filter(|&&q| q.abs() <= near_zero_tol)
        .count();

    Ok(FeasibleDensityDiagnostics {
        mass,
        mean,
        min,
        max,
        near_zero_atoms,
    })
}

fn is_feasible(density: &[f64], strikes: &[f64], mass_tol: f64, mean_tol: f64) -> SanosResult<bool> {
    validate_density_inputs(strikes, density)?;
    if !mass_tol.is_finite() || mass_tol < 0.0 {
        return Err(SanosError::InvalidBound {
            field: "mass_tol",
            value: mass_tol,
            min: 0.0,
            max: f64::INFINITY,
        });
    }
    if !mean_tol.is_finite() || mean_tol < 0.0 {
        return Err(SanosError::InvalidBound {
            field: "mean_tol",
            value: mean_tol,
            min: 0.0,
            max: f64::INFINITY,
        });
    }

    let mass = density.iter().sum::<f64>();
    let mean = density
        .iter()
        .zip(strikes.iter())
        .map(|(q, k)| q * k)
        .sum::<f64>();
    let min = density.iter().copied().fold(f64::INFINITY, |a, b| a.min(b));

    Ok(min >= -mass_tol && (mass - 1.0).abs() <= mass_tol && (mean - 1.0).abs() <= mean_tol)
}

fn quote_proxy_value(quote: CallQuote, proxy: InitPriceProxyConfig) -> f64 {
    match proxy {
        InitPriceProxyConfig::Mid => quote.mid(),
        InitPriceProxyConfig::Bid => quote.bid,
        InitPriceProxyConfig::Ask => quote.ask,
    }
}

fn interpolate_calls_on_grid(
    market_strikes: &[f64],
    market_calls: &[f64],
    grid_strikes: &[f64],
) -> SanosResult<Vec<f64>> {
    if market_strikes.len() != market_calls.len() {
        return Err(SanosError::InvalidOrdering {
            msg: "market strikes and calls must have same length",
        });
    }
    if market_strikes.is_empty() {
        return Err(SanosError::EmptyCollection {
            what: "market strikes",
        });
    }
    for i in 0..market_strikes.len() {
        let k = market_strikes[i];
        let c = market_calls[i];
        if !k.is_finite() {
            return Err(SanosError::NonFinite {
                field: "market strike",
                value: k,
            });
        }
        if !c.is_finite() {
            return Err(SanosError::NonFinite {
                field: "market call",
                value: c,
            });
        }
        if i > 0 && market_strikes[i] <= market_strikes[i - 1] {
            return Err(SanosError::InvalidOrdering {
                msg: "market strikes must be strictly increasing",
            });
        }
    }
    if grid_strikes.is_empty() {
        return Err(SanosError::EmptyCollection {
            what: "grid strikes",
        });
    }
    for i in 0..grid_strikes.len() {
        if !grid_strikes[i].is_finite() {
            return Err(SanosError::NonFinite {
                field: "grid strike",
                value: grid_strikes[i],
            });
        }
        if grid_strikes[i] <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "grid strike",
                value: grid_strikes[i],
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }
    }

    let n_mkt = market_strikes.len();
    let mut out = Vec::with_capacity(grid_strikes.len());

    let left_slope = if n_mkt >= 2 {
        (market_calls[1] - market_calls[0]) / (market_strikes[1] - market_strikes[0])
    } else {
        0.0
    };
    let right_slope = if n_mkt >= 2 {
        (market_calls[n_mkt - 1] - market_calls[n_mkt - 2])
            / (market_strikes[n_mkt - 1] - market_strikes[n_mkt - 2])
    } else {
        0.0
    };

    for &k in grid_strikes {
        let mut c = if n_mkt == 1 {
            market_calls[0]
        } else if k <= market_strikes[0] {
            market_calls[0] + left_slope * (k - market_strikes[0])
        } else if k >= market_strikes[n_mkt - 1] {
            market_calls[n_mkt - 1] + right_slope * (k - market_strikes[n_mkt - 1])
        } else {
            let hi = market_strikes.partition_point(|&x| x < k);
            let lo = hi - 1;
            let k0 = market_strikes[lo];
            let k1 = market_strikes[hi];
            let c0 = market_calls[lo];
            let c1 = market_calls[hi];
            let w = (k - k0) / (k1 - k0);
            (1.0 - w) * c0 + w * c1
        };

        // Enforce broad call bounds during interpolation/extrapolation.
        let intrinsic = (1.0 - k).max(0.0);
        c = c.clamp(intrinsic, 1.0);
        out.push(c);
    }

    Ok(out)
}
