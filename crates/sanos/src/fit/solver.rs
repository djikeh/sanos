use crate::error::{SanosError, SanosResult};
use crate::density::MartingaleDensity;
use crate::fit::config::LpSolverConfig;
use crate::fit::kernels::KernelSet;
use crate::fit::lp::builder::LpLayout;
use crate::fit::lp::model::LpModel;

#[derive(Debug, Clone)]
pub struct LpSolution {
    pub values: Vec<f64>, // values[var_id]
}

#[derive(Debug, Clone)]
pub struct SolveResult {
    pub density: MartingaleDensity,
    pub objective_value: f64,
}

pub fn solve_lp(model: &LpModel, cfg: &LpSolverConfig) -> SanosResult<LpSolution> {
    cfg.validate_available()?;
    match cfg {
        LpSolverConfig::Microlp => solve_with_microlp(model),
        LpSolverConfig::Cbc {
            msg,
            time_limit_sec,
        } => solve_with_cbc(model, *msg, *time_limit_sec),
    }
}

pub fn add_martingale_density_warm_start(
    model: &mut LpModel,
    layout: &LpLayout,
    warm_start: &MartingaleDensity,
) -> SanosResult<()> {
    if layout.q_var_ids.len() != warm_start.marginals().len() {
        return Err(SanosError::InvalidOrdering {
            msg: "layout.q_var_ids must align with warm-start marginals",
        });
    }

    for (q_ids, marginal) in layout.q_var_ids.iter().zip(warm_start.marginals().iter()) {
        if q_ids.len() != marginal.atoms().len() {
            return Err(SanosError::InvalidOrdering {
                msg: "number of q_var_ids does not match number of atoms in warm-start marginal",
            });
        }

        for (&q_id, &weight) in q_ids.iter().zip(marginal.probabilities().iter()) {
            model.set_var_initial(q_id, weight)?;
        }
    }

    Ok(())
}

pub fn solve(
    model: &LpModel,
    layout: &LpLayout,
    kernels: &KernelSet,
    cfg: &LpSolverConfig,
) -> SanosResult<SolveResult> {
    let lp_solution = solve_lp(model, cfg)?;
    let objective_value = evaluate_objective_value(model, &lp_solution.values);
    let density = crate::fit::extract::extract_density(layout, &lp_solution, kernels)?;

    Ok(SolveResult {
        density,
        objective_value,
    })
}

fn evaluate_objective_value(model: &LpModel, values: &[f64]) -> f64 {
    model
        .objective
        .terms
        .iter()
        .map(|t| t.coef * values[t.var])
        .sum()
}

#[cfg(feature = "lp-cbc")]
fn solve_with_cbc(model: &LpModel, msg: bool, time_limit_sec: Option<u64>) -> SanosResult<LpSolution> {
    use good_lp::solvers::{
        lp_solvers::{CbcSolver, LpSolver},
        WithTimeLimit,
    };
    use good_lp::{variable, Expression, ProblemVariables, Solution, SolverModel};
    use crate::fit::lp::model::Sense;

    let mut vars = ProblemVariables::new();
    let mut vhandles = Vec::with_capacity(model.vars.len());

    for v in &model.vars {
        let mut spec = variable();
        if v.lb.is_finite() {
            spec = spec.min(v.lb);
        }
        if v.ub.is_finite() {
            spec = spec.max(v.ub);
        }
        if let Some(initial) = v.initial {
            spec = spec.initial(initial);
        }
        vhandles.push(vars.add(spec));
    }

    let mut obj = Expression::from(0.0);
    for t in &model.objective.terms {
        let vh = vhandles[t.var];
        obj += t.coef * vh;
    }

    let solver = LpSolver(CbcSolver::new());
    let mut pb = vars.minimise(obj).using(solver);

    // lp-solvers backend does not expose a portable verbosity toggle here.
    let _ = msg;
    if let Some(sec) = time_limit_sec {
        pb = pb.with_time_limit(sec as f64);
    }

    for c in &model.constraints {
        let mut expr = Expression::from(0.0);
        for t in &c.terms {
            let vh = vhandles[t.var];
            expr += t.coef * vh;
        }

        pb = match c.sense {
            Sense::Le => pb.with(expr.leq(c.rhs)),
            Sense::Ge => pb.with(expr.geq(c.rhs)),
            Sense::Eq => pb.with(expr.eq(c.rhs)),
        };
    }

    let sol = pb.solve().map_err(|e| SanosError::External {
        msg: format!("LP solve failed: {e:?}"),
    })?;

    let mut values = Vec::with_capacity(model.vars.len());
    for vh in &vhandles {
        values.push(sol.value(*vh));
    }

    Ok(LpSolution { values })
}

#[cfg(not(feature = "lp-cbc"))]
fn solve_with_cbc(_model: &LpModel, _msg: bool, _time_limit_sec: Option<u64>) -> SanosResult<LpSolution> {
    Err(SanosError::NotImplemented {
        what: "CBC solver backend not enabled. Enable feature `lp-cbc`.",
    })
}

#[cfg(feature = "lp-microlp")]
fn solve_with_microlp(model: &LpModel) -> SanosResult<LpSolution> {
    use good_lp::solvers::microlp::microlp;
    use good_lp::{variable, Expression, ProblemVariables, Solution, SolverModel};
    use crate::fit::lp::model::Sense;

    let mut vars = ProblemVariables::new();
    let mut vhandles = Vec::with_capacity(model.vars.len());

    for v in &model.vars {
        let mut spec = variable();
        if v.lb.is_finite() {
            spec = spec.min(v.lb);
        }
        if v.ub.is_finite() {
            spec = spec.max(v.ub);
        }
        if let Some(initial) = v.initial {
            spec = spec.initial(initial);
        }
        vhandles.push(vars.add(spec));
    }

    let mut obj = Expression::from(0.0);
    for t in &model.objective.terms {
        let vh = vhandles[t.var];
        obj += t.coef * vh;
    }

    let mut pb = vars.minimise(obj).using(microlp);

    for c in &model.constraints {
        let mut expr = Expression::from(0.0);
        for t in &c.terms {
            let vh = vhandles[t.var];
            expr += t.coef * vh;
        }

        pb = match c.sense {
            Sense::Le => pb.with(expr.leq(c.rhs)),
            Sense::Ge => pb.with(expr.geq(c.rhs)),
            Sense::Eq => pb.with(expr.eq(c.rhs)),
        };
    }

    let sol = pb.solve().map_err(|e| SanosError::External {
        msg: format!("LP solve failed: {e:?}"),
    })?;

    let mut values = Vec::with_capacity(model.vars.len());
    for vh in &vhandles {
        values.push(sol.value(*vh));
    }

    Ok(LpSolution { values })
}

#[cfg(not(feature = "lp-microlp"))]
fn solve_with_microlp(_model: &LpModel) -> SanosResult<LpSolution> {
    Err(SanosError::NotImplemented {
        what: "Microlp solver backend not enabled. Enable feature `lp-microlp`.",
    })
}
