// src/prelude.rs
pub use crate::error::SanosError;
pub use crate::market::{
    complete_slice_remark_2_8, AtmMidPolicy, CallQuote, CompletedSlice, CompletionConfig,
    CompletionDiagnostics, CompletionHardCaps, NearestOrLinearLogMoneyness, OptionBook,
    OptionChain,
};
pub use crate::density::{DensityTolerances, MarginalDensity, MartingaleDensity};
pub use crate::interp::{LinearTime, TimeInterpolator};
pub use crate::surface::SanosSurface;
pub use crate::grid::{LogMoneynessQuantiles, MarketAnchored, StrikeGrid, StrikeGridPolicy};
pub use crate::fit::{ConstraintConfig, DenseMat, FitConfig, KernelC, KernelConfig, KernelSet, KernelTransition};
