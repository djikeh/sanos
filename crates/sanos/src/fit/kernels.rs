// src/fit/kernels.rs
use crate::error::{SanosError, SanosResult};

/// Simple dense matrix in row-major layout.
#[derive(Debug, Clone)]
pub struct DenseMat {
    pub nrows: usize,
    pub ncols: usize,
    pub data: Vec<f64>,
}

impl DenseMat {
    pub fn new(nrows: usize, ncols: usize, data: Vec<f64>) -> SanosResult<Self> {
        if nrows == 0 || ncols == 0 {
            return Err(SanosError::InvalidOrdering { msg: "DenseMat dims must be > 0" });
        }
        if data.len() != nrows * ncols {
            return Err(SanosError::InvalidOrdering { msg: "DenseMat data length mismatch" });
        }
        Ok(Self { nrows, ncols, data })
    }

    #[inline]
    pub fn idx(&self, r: usize, c: usize) -> usize {
        r * self.ncols + c
    }

    #[inline]
    pub fn get(&self, r: usize, c: usize) -> f64 {
        self.data[self.idx(r, c)]
    }
}

/// Kernel matrix for one expiry:
/// C_j[market_k_index, model_k_index] = kernel price
#[derive(Debug, Clone)]
pub struct KernelC {
    pub maturity: f64,
    pub market_strikes: Vec<f64>,
    pub model_strikes: Vec<f64>,
    pub c: DenseMat,
}

/// Transition/constraint kernels between j-1 and j
#[derive(Debug, Clone)]
pub struct KernelTransition {
    pub maturity: f64,
    pub u: DenseMat, // Nj x Nj
    pub r: DenseMat, // Nj x N(j-1)
    /// Optional secondary transition block (used for omega=Both).
    pub u_alt: Option<DenseMat>, // Nj x Nj
    pub r_alt: Option<DenseMat>, // Nj x N(j-1)
}

#[derive(Debug, Clone)]
pub struct KernelSet {
    pub c: Vec<KernelC>,
    pub transitions: Vec<KernelTransition>, // len = c.len() - 1
}

impl KernelSet {
    pub fn validate(&self) -> SanosResult<()> {
        if self.c.is_empty() {
            return Err(SanosError::EmptyCollection { what: "KernelSet.c" });
        }
        if self.transitions.len() + 1 != self.c.len() {
            return Err(SanosError::InvalidOrdering { msg: "KernelSet.transitions must have len = c.len()-1" });
        }
        Ok(())
    }
}
