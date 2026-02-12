// src/calibration/lp/model.rs
use crate::error::{SanosError, SanosResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarType {
    Continuous,
}

#[derive(Debug, Clone)]
pub struct LpVar {
    pub name: String,
    pub lb: f64,
    pub ub: f64,
    pub ty: VarType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sense {
    Le,
    Ge,
    Eq,
}

#[derive(Debug, Clone)]
pub struct LinTerm {
    pub var: usize,
    pub coef: f64,
}

#[derive(Debug, Clone)]
pub struct LpConstraint {
    pub name: String,
    pub terms: Vec<LinTerm>,
    pub sense: Sense,
    pub rhs: f64,
}

#[derive(Debug, Clone)]
pub struct LpObjective {
    pub terms: Vec<LinTerm>,
}

#[derive(Debug, Clone)]
pub struct LpModel {
    pub vars: Vec<LpVar>,
    pub constraints: Vec<LpConstraint>,
    pub objective: LpObjective,
}

impl LpModel {
    pub fn new() -> Self {
        Self {
            vars: Vec::new(),
            constraints: Vec::new(),
            objective: LpObjective { terms: Vec::new() },
        }
    }

    pub fn add_var(&mut self, name: impl Into<String>, lb: f64, ub: f64) -> SanosResult<usize> {
        if !lb.is_finite() || !ub.is_finite() {
            return Err(SanosError::InvalidOrdering { msg: "Variable bounds must be finite" });
        }
        if lb > ub {
            return Err(SanosError::InvalidOrdering { msg: "Variable lb must be <= ub" });
        }
        let id = self.vars.len();
        self.vars.push(LpVar { name: name.into(), lb, ub, ty: VarType::Continuous });
        Ok(id)
    }

    pub fn add_constraint(
        &mut self,
        name: impl Into<String>,
        terms: Vec<LinTerm>,
        sense: Sense,
        rhs: f64,
    ) -> SanosResult<()> {
        if !rhs.is_finite() {
            return Err(SanosError::NonFinite { field: "constraint.rhs", value: rhs });
        }
        self.constraints.push(LpConstraint { name: name.into(), terms, sense, rhs });
        Ok(())
    }

    pub fn add_obj_term(&mut self, var: usize, coef: f64) -> SanosResult<()> {
        if !coef.is_finite() {
            return Err(SanosError::NonFinite { field: "objective.coef", value: coef });
        }
        self.objective.terms.push(LinTerm { var, coef });
        Ok(())
    }
}
