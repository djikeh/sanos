use resopt::{Matrix, TikhonovRegularization};

use crate::error::{SanosError, SanosResult};
use crate::fit::builder::QLayout;
use crate::fit::config::{LambdaScaling, RegularizationConfig, RegularizationMode, SmoothingOrder};

/// Build a `TikhonovRegularization` from the calibration config and the QLayout.
///
/// The matrix L is always **block-diagonal**: each maturity j has its own block
/// operating on q_j, so there is no cross-maturity regularization.
///
/// When `lambda_scaling` is non-uniform, the rows of block j are multiplied by
/// `sqrt(scale_j)` so that the effective regularization weight for maturity j is
/// `lambda * scale_j`.
pub fn build_tikhonov(
    cfg: &RegularizationConfig,
    layout: &QLayout,
) -> SanosResult<Option<TikhonovRegularization>> {
    match &cfg.mode {
        RegularizationMode::None => Ok(None),
        RegularizationMode::Ridge => {
            let reg = build_ridge(cfg.lambda, layout, &cfg.lambda_scaling)?;
            Ok(Some(reg))
        }
        RegularizationMode::Smoothing { order } => {
            let reg = build_smoothing(cfg.lambda, layout, *order, &cfg.lambda_scaling)?;
            Ok(Some(reg))
        }
    }
}

/// Build a (possibly scaled) ridge regularization.
///
/// When scaling is uniform, this is just `TikhonovRegularization::ridge`.
/// Otherwise, diagonal entries for maturity j are `sqrt(scale_j)`.
fn build_ridge(
    lambda: f64,
    layout: &QLayout,
    scaling: &LambdaScaling,
) -> SanosResult<TikhonovRegularization> {
    if scaling.is_uniform() {
        return TikhonovRegularization::ridge(layout.total, lambda).map_err(|e| {
            SanosError::External {
                msg: format!("resopt ridge regularization failed: {e}"),
            }
        });
    }

    let n = layout.total;
    let mut data = vec![0.0; n * n];
    for j in 0..layout.sizes.len() {
        let s = scaling.scale_at(layout.maturities[j]).sqrt();
        for i in 0..layout.sizes[j] {
            let idx = layout.offsets[j] + i;
            data[idx * n + idx] = s;
        }
    }

    let matrix = Matrix::from_row_major(n, n, data).map_err(|e| SanosError::External {
        msg: format!("resopt scaled ridge Matrix failed: {e}"),
    })?;
    let target = vec![0.0; n];
    TikhonovRegularization::new(lambda, matrix, target).map_err(|e| SanosError::External {
        msg: format!("resopt scaled ridge TikhonovRegularization failed: {e}"),
    })
}

/// Build a block-diagonal finite-difference regularization matrix.
///
/// For each maturity j with N_j model strikes, the block is:
/// - D1: (N_j - 1) × N_j first-order difference matrix
/// - D2: (N_j - 2) × N_j second-order difference matrix
///
/// When lambda_scaling is non-uniform, each row of block j is scaled by
/// `sqrt(scale_j)` so that `lambda/2 * ||L x||^2 = 1/2 * sum_j lambda_j * ||D_j q_j||^2`.
fn build_smoothing(
    lambda: f64,
    layout: &QLayout,
    order: SmoothingOrder,
    scaling: &LambdaScaling,
) -> SanosResult<TikhonovRegularization> {
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
        let s = scaling.scale_at(layout.maturities[j]).sqrt();

        match order {
            SmoothingOrder::D1 => {
                for i in 0..nj.saturating_sub(1) {
                    data[row * total_cols + col_offset + i] = -s;
                    data[row * total_cols + col_offset + i + 1] = s;
                    row += 1;
                }
            }
            SmoothingOrder::D2 => {
                for i in 0..nj.saturating_sub(2) {
                    data[row * total_cols + col_offset + i] = s;
                    data[row * total_cols + col_offset + i + 1] = -2.0 * s;
                    data[row * total_cols + col_offset + i + 2] = s;
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

    TikhonovRegularization::new(lambda, matrix, target).map_err(|e| SanosError::External {
        msg: format!("resopt smoothing TikhonovRegularization failed: {e}"),
    })
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
            maturities: vec![0.5, 1.0],
            total: 7,
        }
    }

    fn cfg_smoothing_d1(lambda: f64) -> RegularizationConfig {
        RegularizationConfig {
            mode: RegularizationMode::Smoothing {
                order: SmoothingOrder::D1,
            },
            lambda,
            ..RegularizationConfig::default()
        }
    }

    #[test]
    fn ridge_dimensions() {
        let cfg = RegularizationConfig {
            mode: RegularizationMode::Ridge,
            lambda: 0.01,
            ..RegularizationConfig::default()
        };
        let layout = layout_3_4();
        let reg = build_tikhonov(&cfg, &layout).unwrap().unwrap();

        assert_eq!(reg.rows(), 7);
        assert_eq!(reg.matrix().ncols(), 7);
    }

    #[test]
    fn smoothing_d1_dimensions() {
        let cfg = cfg_smoothing_d1(0.01);
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
            ..RegularizationConfig::default()
        };
        let layout = layout_3_4();
        let reg = build_tikhonov(&cfg, &layout).unwrap().unwrap();

        // D2 rows: (3-2) + (4-2) = 1 + 2 = 3
        assert_eq!(reg.rows(), 3);
        assert_eq!(reg.matrix().ncols(), 7);
    }

    #[test]
    fn smoothing_d1_values() {
        let cfg = cfg_smoothing_d1(1.0);
        let layout = QLayout {
            offsets: vec![0],
            sizes: vec![4],
            maturities: vec![1.0],
            total: 4,
        };
        let reg = build_tikhonov(&cfg, &layout).unwrap().unwrap();

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
            ..RegularizationConfig::default()
        };
        let layout = QLayout {
            offsets: vec![0],
            sizes: vec![4],
            maturities: vec![1.0],
            total: 4,
        };
        let reg = build_tikhonov(&cfg, &layout).unwrap().unwrap();

        let d = reg.matrix().data();
        assert_eq!(d[0], 1.0);
        assert_eq!(d[1], -2.0);
        assert_eq!(d[2], 1.0);
        assert_eq!(d[3], 0.0);
        assert_eq!(d[4], 0.0);
        assert_eq!(d[5], 1.0);
        assert_eq!(d[6], -2.0);
        assert_eq!(d[7], 1.0);
    }

    #[test]
    fn block_diagonal_structure() {
        let cfg = cfg_smoothing_d1(0.5);
        let layout = QLayout {
            offsets: vec![0, 3],
            sizes: vec![3, 3],
            maturities: vec![0.5, 1.0],
            total: 6,
        };
        let reg = build_tikhonov(&cfg, &layout).unwrap().unwrap();

        assert_eq!(reg.rows(), 4);
        let d = reg.matrix().data();

        // Block 0: cols 0-2 active
        assert_eq!(d[0], -1.0);
        assert_eq!(d[1], 1.0);
        assert_eq!(d[3], 0.0);
        assert_eq!(d[4], 0.0);

        // Block 1: cols 3-5 active
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
        let cfg = cfg_smoothing_d1(0.1);
        let layout = layout_3_4();
        let reg = build_tikhonov(&cfg, &layout).unwrap().unwrap();
        assert!(reg.target().iter().all(|&v| v == 0.0));
    }

    #[test]
    fn per_maturity_scaling_applies_sqrt() {
        // Two maturities: T=0.02 with scale 4.0, T=1.0 with scale 1.0
        let cfg = RegularizationConfig {
            mode: RegularizationMode::Smoothing {
                order: SmoothingOrder::D1,
            },
            lambda: 1.0,
            lambda_scaling: LambdaScaling {
                knots: vec![(0.02, 4.0), (1.0, 1.0)],
            },
        };
        let layout = QLayout {
            offsets: vec![0, 3],
            sizes: vec![3, 3],
            maturities: vec![0.02, 1.0],
            total: 6,
        };
        let reg = build_tikhonov(&cfg, &layout).unwrap().unwrap();
        let d = reg.matrix().data();

        // Block 0 (T=0.02, scale=4.0): rows scaled by sqrt(4) = 2.0
        assert!((d[0] - (-2.0)).abs() < 1e-12); // -1 * 2
        assert!((d[1] - 2.0).abs() < 1e-12);    // +1 * 2

        // Block 1 (T=1.0, scale=1.0): rows scaled by sqrt(1) = 1.0
        assert!((d[15] - (-1.0)).abs() < 1e-12);
        assert!((d[16] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn uniform_scaling_matches_unscaled() {
        let cfg_plain = cfg_smoothing_d1(0.5);
        let cfg_scaled = RegularizationConfig {
            mode: RegularizationMode::Smoothing {
                order: SmoothingOrder::D1,
            },
            lambda: 0.5,
            lambda_scaling: LambdaScaling { knots: vec![] },
        };
        let layout = layout_3_4();

        let reg_plain = build_tikhonov(&cfg_plain, &layout).unwrap().unwrap();
        let reg_scaled = build_tikhonov(&cfg_scaled, &layout).unwrap().unwrap();

        assert_eq!(reg_plain.matrix().data(), reg_scaled.matrix().data());
    }

    #[test]
    fn lambda_scaling_interpolates() {
        let scaling = LambdaScaling {
            knots: vec![(0.1, 10.0), (1.0, 1.0)],
        };
        // Midpoint: T=0.55 => scale = 10 + (1-10) * (0.55-0.1)/(1.0-0.1) = 10 - 4.5 = 5.5
        assert!((scaling.scale_at(0.55) - 5.5).abs() < 1e-12);
        // Below first knot: flat at 10.0
        assert!((scaling.scale_at(0.01) - 10.0).abs() < 1e-12);
        // Above last knot: flat at 1.0
        assert!((scaling.scale_at(5.0) - 1.0).abs() < 1e-12);
    }
}
