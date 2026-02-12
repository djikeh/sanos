// src/calibration/lp/builder.rs
use crate::calibration::config::CalibrationConfig;
use crate::calibration::kernels::KernelSet;
use crate::calibration::lp::model::LpModel;
use crate::error::SanosResult;
use crate::market::OptionBook;

#[derive(Debug, Clone)]
pub struct LpLayout {
    /// q_var_ids[j][i] = variable id for q_j(i)
    pub q_var_ids: Vec<Vec<usize>>,
}

#[derive(Debug, Clone)]
pub struct BuiltLp {
    pub model: LpModel,
    pub layout: LpLayout,
}

pub trait LpBuilder: Send + Sync {
    fn build(&self, book: &OptionBook, kernels: &KernelSet, cfg: &CalibrationConfig) -> SanosResult<BuiltLp>;
}

/// Placeholder: next step will implement the real SANOS LP.
#[derive(Debug, Default, Clone)]
pub struct SanosLpBuilder;

impl LpBuilder for SanosLpBuilder {
    fn build(&self, _book: &OptionBook, _kernels: &KernelSet, _cfg: &CalibrationConfig) -> SanosResult<BuiltLp> {
        Ok(BuiltLp { model: LpModel::new(), layout: LpLayout { q_var_ids: Vec::new() } })
    }
}
