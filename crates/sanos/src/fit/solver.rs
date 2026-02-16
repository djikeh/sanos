use crate::error::{SanosError, SanosResult};
use crate::fit::config::LpSolverConfig;
use crate::fit::lp::model::{LpModel, Sense};

#[derive(Debug, Clone)]
pub struct LpSolution {
    pub values: Vec<f64>, // values[var_id]
}

pub fn solve_lp(model: &LpModel, cfg: &LpSolverConfig) -> SanosResult<LpSolution> {
    match cfg {
        LpSolverConfig::Microlp => solve_with_microlp(model),
        LpSolverConfig::Cbc {
            msg,
            time_limit_sec,
        } => solve_with_cbc(model, *msg, *time_limit_sec),
    }
}

#[cfg(feature = "lp-cbc")]
fn solve_with_cbc(model: &LpModel, msg: bool, time_limit_sec: Option<u64>) -> SanosResult<LpSolution> {
    use good_lp::solvers::{
        lp_solvers::{CbcSolver, LpSolver},
        WithTimeLimit,
    };
    use good_lp::{variable, Expression, ProblemVariables, Solution, SolverModel};

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
