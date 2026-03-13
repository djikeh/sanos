use resopt::{Matrix, TikhonovRegularization};

use crate::error::{SanosError, SanosResult};
use crate::fit::builder::QLayout;
use crate::fit::config::{RegularizationConfig, RegularizationMode, SmoothingOrder};

/// Build a `TikhonovRegularization` from the calibration config and the QLayout.
///
/// The matrix L is always **block-diagonal**: each maturity j has its own block
/// operating on q_j, so there is no cross-maturity regularization.
pub fn build_tikhonov(
    cfg: &RegularizationConfig,
    layout: &QLayout,
) -> SanosResult<Option<TikhonovRegularization>> {
    match &cfg.mode {
        RegularizationMode::None => Ok(None),
        RegularizationMode::Ridge => {
            let reg = TikhonovRegularization::ridge(layout.total, cfg.lambda).map_err(|e| {
                SanosError::External {
                    msg: format!("resopt ridge regularization failed: {e}"),
                }
            })?;
            Ok(Some(reg))
        }
        RegularizationMode::Smoothing { order } => {
            let reg = build_smoothing(cfg.lambda, layout, *order)?;
            Ok(Some(reg))
        }
    }
}

/// Build a block-diagonal finite-difference regularization matrix.
///
/// For each maturity j with N_j model strikes, the block is:
/// - D1: (N_j - 1) × N_j first-order difference matrix
/// - D2: (N_j - 2) × N_j second-order difference matrix
///
/// The full matrix L is block-diagonal over all maturities.
fn build_smoothing(
    lambda: f64,
    layout: &QLayout,
    order: SmoothingOrder,
) -> SanosResult<TikhonovRegularization> {
    // Compute total rows
    let total_rows: usize = layout
        .sizes
        .iter()
        .map(|&nj| diff_rows(nj, order))
        .sum();

    if total_rows == 0 {
        return Err(SanosError::External {
            msg: "smoothing regularization requires at least 2 strikes per maturity (D1) or 3 (D2)"
                .to_string(),
        });
    }

    let total_cols = layout.total;
    let mut data = vec![0.0; total_rows * total_cols];
    let mut row = 0;

    for j in 0..layout.sizes.len() {
        let nj = layout.sizes[j];
        let col_offset = layout.offsets[j];

        match order {
            SmoothingOrder::D1 => {
                // D1[i, :] = [0 ... 0  -1  +1  0 ... 0]
                for i in 0..nj.saturating_sub(1) {
                    data[row * total_cols + col_offset + i] = -1.0;
                    data[row * total_cols + col_offset + i + 1] = 1.0;
                    row += 1;
                }
            }
            SmoothingOrder::D2 => {
                // D2[i, :] = [0 ... 0  +1  -2  +1  0 ... 0]
                for i in 0..nj.saturating_sub(2) {
                    data[row * total_cols + col_offset + i] = 1.0;
                    data[row * total_cols + col_offset + i + 1] = -2.0;
                    data[row * total_cols + col_offset + i + 2] = 1.0;
                    row += 1;
                }
            }
        }
    }

    let matrix = Matrix::from_row_major(total_rows, total_cols, data).map_err(|e| {
        SanosError::External {
            msg: format!("resopt smoothing Matrix failed: {e}"),
        }
    })?;

    let target = vec![0.0; total_rows];

    let reg = TikhonovRegularization::new(lambda, matrix, target).map_err(|e| {
        SanosError::External {
            msg: format!("resopt smoothing TikhonovRegularization failed: {e}"),
        }
    })?;

    Ok(reg)
}

/// Number of rows produced by finite-difference matrix for a block of size nj.
fn diff_rows(nj: usize, order: SmoothingOrder) -> usize {
    match order {
        SmoothingOrder::D1 => nj.saturating_sub(1),
        SmoothingOrder::D2 => nj.saturating_sub(2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout_3_4() -> QLayout {
        QLayout {
            offsets: vec![0, 3],
            sizes: vec![3, 4],
            total: 7,
        }
    }

    #[test]
    fn ridge_dimensions() {
        let cfg = RegularizationConfig {
            mode: RegularizationMode::Ridge,
            lambda: 0.01,
        };
        let layout = layout_3_4();
        let reg = build_tikhonov(&cfg, &layout).unwrap().unwrap();

        assert_eq!(reg.rows(), 7);
        assert_eq!(reg.matrix().ncols(), 7);
    }

    #[test]
    fn smoothing_d1_dimensions() {
        let cfg = RegularizationConfig {
            mode: RegularizationMode::Smoothing {
                order: SmoothingOrder::D1,
            },
            lambda: 0.01,
        };
        let layout = layout_3_4();
        let reg = build_tikhonov(&cfg, &layout).unwrap().unwrap();

        // D1 rows: (3-1) + (4-1) = 2 + 3 = 5
        assert_eq!(reg.rows(), 5);
        assert_eq!(reg.matrix().ncols(), 7);
    }

    #[test]
    fn smoothing_d2_dimensions() {
        let cfg = RegularizationConfig {
            mode: RegularizationMode::Smoothing {
                order: SmoothingOrder::D2,
            },
            lambda: 0.01,
        };
        let layout = layout_3_4();
        let reg = build_tikhonov(&cfg, &layout).unwrap().unwrap();

        // D2 rows: (3-2) + (4-2) = 1 + 2 = 3
        assert_eq!(reg.rows(), 3);
        assert_eq!(reg.matrix().ncols(), 7);
    }

    #[test]
    fn smoothing_d1_values() {
        let cfg = RegularizationConfig {
            mode: RegularizationMode::Smoothing {
                order: SmoothingOrder::D1,
            },
            lambda: 1.0,
        };
        // Single maturity with 4 strikes
        let layout = QLayout {
            offsets: vec![0],
            sizes: vec![4],
            total: 4,
        };
        let reg = build_tikhonov(&cfg, &layout).unwrap().unwrap();

        // 3 rows, 4 cols
        let d = reg.matrix().data();
        // Row 0: [-1, 1, 0, 0]
        assert_eq!(d[0], -1.0);
        assert_eq!(d[1], 1.0);
        assert_eq!(d[2], 0.0);
        assert_eq!(d[3], 0.0);
        // Row 1: [0, -1, 1, 0]
        assert_eq!(d[4], 0.0);
        assert_eq!(d[5], -1.0);
        assert_eq!(d[6], 1.0);
        assert_eq!(d[7], 0.0);
        // Row 2: [0, 0, -1, 1]
        assert_eq!(d[8], 0.0);
        assert_eq!(d[9], 0.0);
        assert_eq!(d[10], -1.0);
        assert_eq!(d[11], 1.0);
    }

    #[test]
    fn smoothing_d2_values() {
        let cfg = RegularizationConfig {
            mode: RegularizationMode::Smoothing {
                order: SmoothingOrder::D2,
            },
            lambda: 1.0,
        };
        let layout = QLayout {
            offsets: vec![0],
            sizes: vec![4],
            total: 4,
        };
        let reg = build_tikhonov(&cfg, &layout).unwrap().unwrap();

        // 2 rows, 4 cols
        let d = reg.matrix().data();
        // Row 0: [1, -2, 1, 0]
        assert_eq!(d[0], 1.0);
        assert_eq!(d[1], -2.0);
        assert_eq!(d[2], 1.0);
        assert_eq!(d[3], 0.0);
        // Row 1: [0, 1, -2, 1]
        assert_eq!(d[4], 0.0);
        assert_eq!(d[5], 1.0);
        assert_eq!(d[6], -2.0);
        assert_eq!(d[7], 1.0);
    }

    #[test]
    fn block_diagonal_structure() {
        let cfg = RegularizationConfig {
            mode: RegularizationMode::Smoothing {
                order: SmoothingOrder::D1,
            },
            lambda: 0.5,
        };
        // Two maturities: 3 and 3 strikes
        let layout = QLayout {
            offsets: vec![0, 3],
            sizes: vec![3, 3],
            total: 6,
        };
        let reg = build_tikhonov(&cfg, &layout).unwrap().unwrap();

        // 4 rows (2+2), 6 cols
        assert_eq!(reg.rows(), 4);
        let d = reg.matrix().data();

        // Block 0 rows: cols 0-2 active, cols 3-5 zero
        // Row 0: [-1, 1, 0, 0, 0, 0]
        assert_eq!(d[0], -1.0);
        assert_eq!(d[1], 1.0);
        assert_eq!(d[3], 0.0);
        assert_eq!(d[4], 0.0);

        // Block 1 rows: cols 0-2 zero, cols 3-5 active
        // Row 2: [0, 0, 0, -1, 1, 0]
        assert_eq!(d[12], 0.0);
        assert_eq!(d[13], 0.0);
        assert_eq!(d[14], 0.0);
        assert_eq!(d[15], -1.0);
        assert_eq!(d[16], 1.0);
        assert_eq!(d[17], 0.0);
    }

    #[test]
    fn none_mode_returns_none() {
        let cfg = RegularizationConfig::default();
        let layout = layout_3_4();
        assert!(build_tikhonov(&cfg, &layout).unwrap().is_none());
    }

    #[test]
    fn target_is_zero() {
        let cfg = RegularizationConfig {
            mode: RegularizationMode::Smoothing {
                order: SmoothingOrder::D1,
            },
            lambda: 0.1,
        };
        let layout = layout_3_4();
        let reg = build_tikhonov(&cfg, &layout).unwrap().unwrap();
        assert!(reg.target().iter().all(|&v| v == 0.0));
    }
}
