use crate::error::{SanosError, SanosResult};
use crate::fit::config::LpSolverConfig;
use crate::fit::lp::model::{LpModel, Sense};

#[derive(Debug, Clone)]
pub struct LpSolution {
    pub values: Vec<f64>, // values[var_id]
}

#[cfg(feature = "lp-cbc")]
pub fn solve_lp(model: &LpModel, cfg: &LpSolverConfig) -> SanosResult<LpSolution> {
    use good_lp::{variable, Expression, ProblemVariables, Solution, SolverModel};
    use good_lp::solvers::{
        lp_solvers::{CbcSolver, LpSolver},
        WithTimeLimit,
    };

    // Currently only CBC is wired (per Cargo.toml)
    let (msg, time_limit_sec) = match cfg {
        LpSolverConfig::Cbc { msg, time_limit_sec } => (*msg, *time_limit_sec),
    };

    let mut vars = ProblemVariables::new();
    let mut vhandles = Vec::with_capacity(model.vars.len());

    // Create solver variables
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

    // Objective
    let mut obj = Expression::from(0.0);
    for t in &model.objective.terms {
        let vh = vhandles[t.var];
        obj += t.coef * vh;
    }

    // Build problem with external CBC solver binary.
    let solver = LpSolver(CbcSolver::new());
    let mut pb = vars.minimise(obj).using(solver);

    // CBC parameters (best effort).
    // The lp-solvers backend does not expose a portable verbosity toggle here.
    let _ = msg;
    if let Some(sec) = time_limit_sec {
        pb = pb.with_time_limit(sec as f64);
    }

    // Constraints
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

    // Solve
    let sol = pb.solve().map_err(|e| SanosError::External {
        msg: format!("LP solve failed: {e:?}"),
    })?;

    // Extract values
    let mut values = Vec::with_capacity(model.vars.len());
    for vh in &vhandles {
        values.push(sol.value(*vh));
    }

    Ok(LpSolution { values })
}

#[cfg(not(feature = "lp-cbc"))]
pub fn solve_lp(_model: &LpModel, _cfg: &LpSolverConfig) -> SanosResult<LpSolution> {
    Err(SanosError::NotImplemented {
        what: "LP solver backend not enabled. Enable feature `lp-cbc`.",
    })
}
