use crate::error::{SanosError, SanosResult};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct CompletionHardCaps {
    pub k1_min: f64,
    pub k1_max: f64,
    #[cfg_attr(feature = "serde", serde(rename = "kN_min"))]
    pub k_n_min: f64,
    #[cfg_attr(feature = "serde", serde(rename = "kN_max"))]
    pub k_n_max: f64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct CompletionConfig {
    pub k1_ratio: f64,
    #[cfg_attr(feature = "serde", serde(rename = "kN_ratio"))]
    pub k_n_ratio: f64,
    pub max_iters: usize,
    pub slope_margin: f64,
    pub hard_caps: Option<CompletionHardCaps>,
    pub tol: f64,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            k1_ratio: 0.2,
            k_n_ratio: 2.0,
            max_iters: 50,
            slope_margin: 1e-10,
            hard_caps: None,
            tol: 1e-10,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct CompletionDiagnostics {
    pub k1: f64,
    #[cfg_attr(feature = "serde", serde(rename = "kN"))]
    pub k_n: f64,
    #[cfg_attr(feature = "serde", serde(rename = "dC0"))]
    pub d_c0: f64,
    #[cfg_attr(feature = "serde", serde(rename = "dC1"))]
    pub d_c1: f64,
    #[cfg_attr(feature = "serde", serde(rename = "dC2"))]
    pub d_c2: f64,
    #[cfg_attr(feature = "serde", serde(rename = "dC_last2"))]
    pub d_c_last2: f64,
    #[cfg_attr(feature = "serde", serde(rename = "dC_last1"))]
    pub d_c_last1: f64,
    pub sum_p: f64,
    pub mean_p: f64,
    pub min_p: f64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct CompletedSlice {
    pub k: Vec<f64>,
    pub calls: Vec<f64>,
    pub slopes: Vec<f64>,
    pub density: Vec<f64>,
    pub diagnostics: CompletionDiagnostics,
}

impl CompletionConfig {
    pub fn validate(&self) -> SanosResult<()> {
        if !self.k1_ratio.is_finite() || self.k1_ratio <= 0.0 {
            return Err(SanosError::External {
                msg: format!(
                    "invalid CompletionConfig.k1_ratio={}, must be finite and > 0",
                    self.k1_ratio
                ),
            });
        }
        if !self.k_n_ratio.is_finite() || self.k_n_ratio <= 1.0 {
            return Err(SanosError::External {
                msg: format!(
                    "invalid CompletionConfig.kN_ratio={}, must be finite and > 1",
                    self.k_n_ratio
                ),
            });
        }
        if self.max_iters == 0 {
            return Err(SanosError::External {
                msg: "invalid CompletionConfig.max_iters=0, must be >= 1".to_string(),
            });
        }
        if !self.slope_margin.is_finite() || self.slope_margin < 0.0 {
            return Err(SanosError::External {
                msg: format!(
                    "invalid CompletionConfig.slope_margin={}, must be finite and >= 0",
                    self.slope_margin
                ),
            });
        }
        if !self.tol.is_finite() || self.tol < 0.0 {
            return Err(SanosError::External {
                msg: format!(
                    "invalid CompletionConfig.tol={}, must be finite and >= 0",
                    self.tol
                ),
            });
        }
        if let Some(caps) = &self.hard_caps {
            if !caps.k1_min.is_finite()
                || !caps.k1_max.is_finite()
                || !caps.k_n_min.is_finite()
                || !caps.k_n_max.is_finite()
            {
                return Err(SanosError::External {
                    msg: "CompletionConfig.hard_caps contains non-finite value".to_string(),
                });
            }
            if !(caps.k1_min > 0.0 && caps.k1_min < caps.k1_max) {
                return Err(SanosError::External {
                    msg: format!(
                        "invalid CompletionConfig.hard_caps on left wing: k1_min={}, k1_max={}",
                        caps.k1_min, caps.k1_max
                    ),
                });
            }
            if !(caps.k_n_min > 0.0 && caps.k_n_min < caps.k_n_max) {
                return Err(SanosError::External {
                    msg: format!(
                        "invalid CompletionConfig.hard_caps on right wing: kN_min={}, kN_max={}",
                        caps.k_n_min, caps.k_n_max
                    ),
                });
            }
        }
        Ok(())
    }
}

pub fn complete_slice_remark_2_8(
    k_internal: &[f64],
    calls_internal: &[f64],
    cfg: &CompletionConfig,
) -> SanosResult<CompletedSlice> {
    cfg.validate()?;
    validate_internal_slice(k_internal, calls_internal, cfg.tol)?;

    let k2 = k_internal[0];
    let c2 = calls_internal[0];
    let k3 = k_internal[1];
    let c3 = calls_internal[1];
    let d_c2 = (c3 - c2) / (k3 - k2);

    let k_last = *k_internal.last().unwrap_or(&f64::NAN);
    let c_last = *calls_internal.last().unwrap_or(&f64::NAN);
    let k_last2 = k_internal[k_internal.len() - 2];
    let c_last2 = calls_internal[calls_internal.len() - 2];
    let d_c_last2 = (c_last - c_last2) / (k_last - k_last2);

    let k1 = choose_k1(k2, c2, d_c2, cfg)?;
    let k_n = choose_k_n(k_last, c_last, d_c_last2, cfg)?;

    let mut k = Vec::with_capacity(k_internal.len() + 3);
    let mut calls = Vec::with_capacity(calls_internal.len() + 3);
    k.push(0.0);
    calls.push(1.0);
    k.push(k1);
    calls.push(1.0 - k1);
    k.extend_from_slice(k_internal);
    calls.extend_from_slice(calls_internal);
    k.push(k_n);
    calls.push(0.0);

    let mut slopes = Vec::with_capacity(k.len());
    for i in 0..(k.len() - 1) {
        slopes.push((calls[i + 1] - calls[i]) / (k[i + 1] - k[i]));
    }
    slopes.push(0.0); // paper convention: dC^N = 0

    let n = k.len() - 1;
    let d_c0 = slopes[0];
    let d_c1 = slopes[1];
    let d_c2_final = slopes[2];
    let d_c_last2_final = slopes[n - 2];
    let d_c_last1 = slopes[n - 1];
    if !(d_c0 + cfg.slope_margin < d_c1 && d_c1 + cfg.slope_margin < d_c2_final) {
        return Err(SanosError::External {
            msg: format!(
                "left completion inequalities violated after assembly: dC0={:+.6e}, dC1={:+.6e}, dC2={:+.6e}, margin={:.3e}",
                d_c0, d_c1, d_c2_final, cfg.slope_margin
            ),
        });
    }
    if !(d_c_last2_final + cfg.slope_margin < d_c_last1 && d_c_last1 + cfg.slope_margin < 0.0) {
        return Err(SanosError::External {
            msg: format!(
                "right completion inequalities violated after assembly: dC_last2={:+.6e}, dC_last1={:+.6e}, dC_N=0, margin={:.3e}",
                d_c_last2_final, d_c_last1, cfg.slope_margin
            ),
        });
    }

    let mut density = Vec::with_capacity(n);
    for i in 1..=n {
        let p = slopes[i] - slopes[i - 1];
        density.push(p);
    }

    for (idx, p) in density.iter_mut().enumerate() {
        if *p < -cfg.tol {
            return Err(SanosError::External {
                msg: format!(
                    "density negativity beyond tolerance at i={}: p={:+.6e}, tol={:.3e}",
                    idx + 1,
                    *p,
                    cfg.tol
                ),
            });
        }
        if *p < 0.0 {
            *p = 0.0;
        }
    }

    let sum_raw: f64 = density.iter().sum();
    if !sum_raw.is_finite() || sum_raw <= 0.0 {
        return Err(SanosError::External {
            msg: format!(
                "invalid density sum after clamping: sum_p={:+.6e}",
                sum_raw
            ),
        });
    }
    for p in &mut density {
        *p /= sum_raw;
    }

    let sum_p: f64 = density.iter().sum();
    let mean_p: f64 = density
        .iter()
        .enumerate()
        .map(|(i, &p)| p * k[i + 1])
        .sum();
    if (sum_p - 1.0).abs() > cfg.tol {
        return Err(SanosError::External {
            msg: format!(
                "density mass check failed: sum_p={:+.6e}, tol={:.3e}",
                sum_p, cfg.tol
            ),
        });
    }
    if (mean_p - 1.0).abs() > cfg.tol {
        return Err(SanosError::External {
            msg: format!(
                "density mean check failed: mean_p={:+.6e}, tol={:.3e}",
                mean_p, cfg.tol
            ),
        });
    }
    let min_p = density
        .iter()
        .fold(f64::INFINITY, |acc, &v| if v < acc { v } else { acc });

    Ok(CompletedSlice {
        k,
        calls,
        slopes,
        density,
        diagnostics: CompletionDiagnostics {
            k1,
            k_n,
            d_c0,
            d_c1,
            d_c2: d_c2_final,
            d_c_last2: d_c_last2_final,
            d_c_last1,
            sum_p,
            mean_p,
            min_p,
        },
    })
}

fn choose_k1(k2: f64, c2: f64, d_c2: f64, cfg: &CompletionConfig) -> SanosResult<f64> {
    let mut k1 = cfg.k1_ratio * k2;
    if let Some(caps) = &cfg.hard_caps {
        k1 = k1.max(caps.k1_min).min(caps.k1_max);
    }
    if !k1.is_finite() || k1 <= 0.0 || k1 >= k2 {
        return Err(SanosError::External {
            msg: format!(
                "invalid initial K1 candidate: K1={:.6e}, K2={:.6e}",
                k1, k2
            ),
        });
    }

    for _ in 0..cfg.max_iters {
        let c1 = 1.0 - k1;
        let d_c1 = (c2 - c1) / (k2 - k1);
        let left_ok = -1.0 + cfg.slope_margin < d_c1;
        let right_ok = d_c1 + cfg.slope_margin < d_c2;
        if left_ok && right_ok {
            return Ok(k1);
        }

        let mut next = k1 * 0.5;
        if let Some(caps) = &cfg.hard_caps {
            next = next.max(caps.k1_min).min(caps.k1_max);
        }
        if !next.is_finite() || next <= 0.0 || next >= k2 || (next - k1).abs() <= f64::EPSILON {
            break;
        }
        k1 = next;
    }

    let c1 = 1.0 - k1;
    let d_c1 = (c2 - c1) / (k2 - k1);
    Err(SanosError::External {
        msg: format!(
            "failed to find K1 satisfying -1 < dC1 < dC2 with margin: K1={:.6e}, K2={:.6e}, C2={:.6e}, dC1={:+.6e}, dC2={:+.6e}, margin={:.3e}",
            k1, k2, c2, d_c1, d_c2, cfg.slope_margin
        ),
    })
}

fn choose_k_n(k_last: f64, c_last: f64, d_c_last2: f64, cfg: &CompletionConfig) -> SanosResult<f64> {
    let mut k_n = cfg.k_n_ratio * k_last;
    if let Some(caps) = &cfg.hard_caps {
        k_n = k_n.max(caps.k_n_min).min(caps.k_n_max);
    }
    if !k_n.is_finite() || k_n <= k_last {
        return Err(SanosError::External {
            msg: format!(
                "invalid initial KN candidate: KN={:.6e}, K_last={:.6e}",
                k_n, k_last
            ),
        });
    }

    for _ in 0..cfg.max_iters {
        let d_c_last1 = -c_last / (k_n - k_last);
        let left_ok = d_c_last2 + cfg.slope_margin < d_c_last1;
        let right_ok = d_c_last1 + cfg.slope_margin < 0.0;
        if left_ok && right_ok {
            return Ok(k_n);
        }

        let mut next = k_n * 1.5;
        if let Some(caps) = &cfg.hard_caps {
            next = next.max(caps.k_n_min).min(caps.k_n_max);
        }
        if !next.is_finite()
            || next <= k_last
            || (next - k_n).abs() <= f64::EPSILON
        {
            break;
        }
        k_n = next;
    }

    let d_c_last1 = -c_last / (k_n - k_last);
    Err(SanosError::External {
        msg: format!(
            "failed to find KN satisfying dC_last2 < dC_last1 < 0 with margin: KN={:.6e}, K_last={:.6e}, C_last={:.6e}, dC_last2={:+.6e}, dC_last1={:+.6e}, margin={:.3e}",
            k_n, k_last, c_last, d_c_last2, d_c_last1, cfg.slope_margin
        ),
    })
}

fn validate_internal_slice(k_internal: &[f64], calls_internal: &[f64], tol: f64) -> SanosResult<()> {
    if k_internal.len() != calls_internal.len() {
        return Err(SanosError::External {
            msg: format!(
                "internal slice length mismatch: strikes={}, calls={}",
                k_internal.len(),
                calls_internal.len()
            ),
        });
    }
    if k_internal.len() < 3 {
        return Err(SanosError::External {
            msg: format!(
                "Remark 2.8 strict completion requires at least 3 internal strikes, got {}",
                k_internal.len()
            ),
        });
    }

    for i in 0..k_internal.len() {
        let k = k_internal[i];
        let c = calls_internal[i];
        if !k.is_finite() || !c.is_finite() {
            return Err(SanosError::External {
                msg: format!("non-finite input at i={}: K={}, C={}", i, k, c),
            });
        }
        if k <= 0.0 {
            return Err(SanosError::External {
                msg: format!("invalid strike at i={}: K={} must be > 0", i, k),
            });
        }
        if !(0.0 < c && c < 1.0) {
            return Err(SanosError::External {
                msg: format!("invalid call at i={}: C={} must satisfy 0 < C < 1", i, c),
            });
        }
        if i > 0 {
            let k_prev = k_internal[i - 1];
            let c_prev = calls_internal[i - 1];
            if k <= k_prev {
                return Err(SanosError::External {
                    msg: format!(
                        "strikes not strictly increasing at i={}: K_prev={}, K={}",
                        i, k_prev, k
                    ),
                });
            }
            if c > c_prev + tol {
                return Err(SanosError::External {
                    msg: format!(
                        "calls not monotone decreasing at i={}: C_prev={}, C={}, tol={}",
                        i, c_prev, c, tol
                    ),
                });
            }
        }
    }

    if !(k_internal[0] < 1.0 && *k_internal.last().unwrap_or(&f64::NAN) > 1.0) {
        return Err(SanosError::External {
            msg: format!(
                "internal strikes must straddle 1.0: K_min={}, K_max={}",
                k_internal[0],
                k_internal[k_internal.len() - 1]
            ),
        });
    }

    let mut prev_slope = (calls_internal[1] - calls_internal[0]) / (k_internal[1] - k_internal[0]);
    for i in 1..(k_internal.len() - 1) {
        let slope = (calls_internal[i + 1] - calls_internal[i]) / (k_internal[i + 1] - k_internal[i]);
        if slope + tol < prev_slope {
            return Err(SanosError::External {
                msg: format!(
                    "internal convexity violated between intervals {} and {}: slope_prev={:+.6e}, slope={:+.6e}, tol={:.3e}",
                    i - 1,
                    i,
                    prev_slope,
                    slope,
                    tol
                ),
            });
        }
        prev_slope = slope;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_succeeds_on_clean_slice_and_density_is_martingale() {
        let k = vec![0.7, 0.9, 1.0, 1.1, 1.3];
        let c = vec![0.30001, 0.14, 0.08, 0.045, 0.015];
        let cfg = CompletionConfig::default();

        let out = complete_slice_remark_2_8(&k, &c, &cfg).unwrap();
        assert_eq!(out.k[0], 0.0);
        assert!((out.calls[0] - 1.0).abs() < 1e-14);
        assert_eq!(out.slopes.len(), out.k.len());
        assert_eq!(out.density.len() + 1, out.k.len());
        assert!(out.density.iter().all(|&p| p >= -cfg.tol));

        let sum_p: f64 = out.density.iter().sum();
        let mean_p: f64 = out
            .density
            .iter()
            .enumerate()
            .map(|(i, &p)| p * out.k[i + 1])
            .sum();
        assert!((sum_p - 1.0).abs() <= cfg.tol);
        assert!((mean_p - 1.0).abs() <= cfg.tol);
    }

    #[test]
    fn completion_fails_when_internal_calls_are_not_monotone_or_convex() {
        let k = vec![0.8, 0.95, 1.05, 1.2];
        let c_bad_monotone = vec![0.22, 0.18, 0.19, 0.10];
        let err = complete_slice_remark_2_8(&k, &c_bad_monotone, &CompletionConfig::default())
            .unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("monotone decreasing"));

        let c_bad_convex = vec![0.22, 0.12, 0.115, 0.05];
        let err = complete_slice_remark_2_8(&k, &c_bad_convex, &CompletionConfig::default())
            .unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("convexity"));
    }
}
