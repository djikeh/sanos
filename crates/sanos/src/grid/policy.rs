use log::{debug, info};
use statrs::distribution::{ContinuousCDF, Normal};

use crate::backbone::bs::bs_implied_atm_var_from_call;
use crate::error::{SanosError, SanosResult};
use crate::grid::config::{AtmRefineConfig, GridSizeConfig, WingsConfig};
use crate::grid::StrikeGrid;
use crate::market::{AtmMidPolicy, OptionBook};

const PROB_EPS: f64 = 1e-12;
const UNIQUE_TOL: f64 = 1e-12;
const TINY_SIGMA: f64 = 1e-10;
const TINY_WINDOW_HALF_WIDTH: f64 = 0.05;

#[derive(Debug, Clone, Copy)]
struct QuantileShape {
    left_tail_alpha: f64,
    right_tail_alpha: f64,
}

#[derive(Debug, Clone, Copy)]
struct LogMoneynessMapping {
    mean: f64,
    volatility: f64,
    min_log_moneyness: f64,
    max_log_moneyness: f64,
}

pub trait StrikeGridPolicy: Send + Sync {
    fn build(
        &self,
        book: &OptionBook,
        atm: &dyn AtmMidPolicy,
        total_variances: Option<&[f64]>,
    ) -> SanosResult<Vec<StrikeGrid>>;
}

/// Market-anchored strike grid:
/// K_j = market_strikes U {1} U wings U (optional ATM refine), then cleaned.
#[derive(Debug, Clone, Copy)]
pub struct MarketAnchored {
    pub ensure_atm: bool,
    pub wings: WingsConfig,
    pub atm_refine: AtmRefineConfig,
    pub size_control: GridSizeConfig,
    pub min_strike: f64,
    pub max_strike: f64,
    pub min_spacing_log: f64,
}

impl Default for MarketAnchored {
    fn default() -> Self {
        Self {
            ensure_atm: true,
            wings: WingsConfig::default(),
            atm_refine: AtmRefineConfig::default(),
            size_control: GridSizeConfig::default(),
            min_strike: 1e-4,
            max_strike: 1e4,
            min_spacing_log: 1e-3,
        }
    }
}

impl MarketAnchored {
    fn validate(&self) -> SanosResult<()> {
        self.wings.validate()?;
        self.atm_refine.validate()?;
        self.size_control.validate()?;

        for (field, v) in [
            ("min_strike", self.min_strike),
            ("max_strike", self.max_strike),
            ("min_spacing_log", self.min_spacing_log),
        ] {
            if !v.is_finite() {
                return Err(SanosError::NonFinite { field, value: v });
            }
        }
        if self.min_strike <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "min_strike",
                value: self.min_strike,
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }
        if self.max_strike <= self.min_strike {
            return Err(SanosError::InvalidOrdering {
                msg: "max_strike must be > min_strike",
            });
        }
        if self.min_spacing_log < 0.0 {
            return Err(SanosError::InvalidBound {
                field: "min_spacing_log",
                value: self.min_spacing_log,
                min: 0.0,
                max: f64::INFINITY,
            });
        }

        Ok(())
    }
}

impl StrikeGridPolicy for MarketAnchored {
    fn build(
        &self,
        book: &OptionBook,
        _atm: &dyn AtmMidPolicy,
        _total_variances: Option<&[f64]>,
    ) -> SanosResult<Vec<StrikeGrid>> {
        self.validate()?;

        let mut grids = Vec::with_capacity(book.len());

        for chain in book.chains() {
            let t = chain.maturity();
            let market_strikes: Vec<f64> = chain.quotes().iter().map(|q| q.k).collect();

            let k_min = *market_strikes.first().ok_or(SanosError::EmptyCollection {
                what: "market strikes",
            })?;
            let k_max = *market_strikes.last().ok_or(SanosError::EmptyCollection {
                what: "market strikes",
            })?;

            // Build candidate list.
            let mut cand: Vec<f64> = Vec::new();
            cand.extend(market_strikes.iter().copied());

            if self.ensure_atm {
                cand.push(1.0);
            }

            // Wings.
            let r = self.wings.ratio;
            for p in 1..=self.wings.n_left {
                cand.push(k_min / r.powi(p as i32));
            }
            for p in 1..=self.wings.n_right {
                cand.push(k_max * r.powi(p as i32));
            }

            // ATM refine (around 1 in log-space).
            if self.atm_refine.enabled {
                let d = self.atm_refine.delta_log;
                for s in 1..=self.atm_refine.steps {
                    let x = (s as f64) * d;
                    cand.push(x.exp());
                    cand.push((-x).exp());
                }
            }

            let strikes = clean_strikes(
                cand,
                self.min_strike,
                self.max_strike,
                self.min_spacing_log,
                self.ensure_atm,
            )?;

            let strikes = enforce_size_control(
                &strikes,
                &market_strikes,
                self.size_control.max_points,
                self.size_control.keep_all_market_strikes,
                self.min_spacing_log,
            )?;

            grids.push(StrikeGrid::new(t, strikes)?);
        }

        Ok(grids)
    }
}

/// Log-moneyness quantile strike grid with asymmetric left/right shaping.
#[derive(Debug, Clone, Copy)]
pub struct LogMoneynessQuantiles {
    pub n: usize,
    pub left_sigmas: f64,
    pub right_sigmas: f64,
    pub alpha_left: f64,
    pub alpha_right: f64,
    pub include_market_strikes: bool,
    pub min_spacing: Option<f64>,
    pub k_min: Option<f64>,
    pub k_max: Option<f64>,
}

impl Default for LogMoneynessQuantiles {
    fn default() -> Self {
        Self {
            n: 80,
            left_sigmas: 4.5,
            right_sigmas: 3.0,
            alpha_left: 1.8,
            alpha_right: 1.2,
            include_market_strikes: true,
            min_spacing: None,
            k_min: None,
            k_max: None,
        }
    }
}

impl LogMoneynessQuantiles {
    fn validate(&self) -> SanosResult<()> {
        if self.n < 3 {
            return Err(SanosError::InvalidOrdering {
                msg: "log_moneyness_quantiles.n must be >= 3",
            });
        }
        for (field, value) in [
            ("log_moneyness_quantiles.left_sigmas", self.left_sigmas),
            ("log_moneyness_quantiles.right_sigmas", self.right_sigmas),
            ("log_moneyness_quantiles.alpha_left", self.alpha_left),
            ("log_moneyness_quantiles.alpha_right", self.alpha_right),
        ] {
            if !value.is_finite() {
                return Err(SanosError::NonFinite { field, value });
            }
            if value <= 0.0 {
                return Err(SanosError::InvalidBound {
                    field,
                    value,
                    min: f64::MIN_POSITIVE,
                    max: f64::INFINITY,
                });
            }
        }

        if let Some(min_spacing) = self.min_spacing {
            if !min_spacing.is_finite() {
                return Err(SanosError::NonFinite {
                    field: "log_moneyness_quantiles.min_spacing",
                    value: min_spacing,
                });
            }
            if min_spacing < 0.0 {
                return Err(SanosError::InvalidBound {
                    field: "log_moneyness_quantiles.min_spacing",
                    value: min_spacing,
                    min: 0.0,
                    max: f64::INFINITY,
                });
            }
        }

        if let Some(k_min) = self.k_min {
            if !k_min.is_finite() {
                return Err(SanosError::NonFinite {
                    field: "log_moneyness_quantiles.k_min",
                    value: k_min,
                });
            }
            if k_min <= 0.0 {
                return Err(SanosError::InvalidBound {
                    field: "log_moneyness_quantiles.k_min",
                    value: k_min,
                    min: f64::MIN_POSITIVE,
                    max: f64::INFINITY,
                });
            }
        }

        if let Some(k_max) = self.k_max {
            if !k_max.is_finite() {
                return Err(SanosError::NonFinite {
                    field: "log_moneyness_quantiles.k_max",
                    value: k_max,
                });
            }
            if k_max <= 0.0 {
                return Err(SanosError::InvalidBound {
                    field: "log_moneyness_quantiles.k_max",
                    value: k_max,
                    min: f64::MIN_POSITIVE,
                    max: f64::INFINITY,
                });
            }
        }

        if let (Some(k_min), Some(k_max)) = (self.k_min, self.k_max) {
            if k_max <= k_min {
                return Err(SanosError::InvalidOrdering {
                    msg: "log_moneyness_quantiles.k_max must be > k_min",
                });
            }
        }

        Ok(())
    }
}

impl StrikeGridPolicy for LogMoneynessQuantiles {
    fn build(
        &self,
        book: &OptionBook,
        atm: &dyn AtmMidPolicy,
        total_variances: Option<&[f64]>,
    ) -> SanosResult<Vec<StrikeGrid>> {
        self.validate()?;

        if let Some(vars) = total_variances {
            if vars.len() != book.len() {
                return Err(SanosError::InvalidOrdering {
                    msg: "total_variances length must match number of book maturities",
                });
            }
        }

        let normal = standard_normal()?;
        let mut grids = Vec::with_capacity(book.len());

        for (j, chain) in book.chains().iter().enumerate() {
            let t = chain.maturity();
            let market_strikes: Vec<f64> = chain.quotes().iter().map(|q| q.k).collect();

            let total_variance = match total_variances.and_then(|vars| vars.get(j).copied()) {
                Some(w) => w,
                None => {
                    let atm_mid = chain.atm_mid(atm)?;
                    bs_implied_atm_var_from_call(atm_mid)?
                }
            };
            if !total_variance.is_finite() {
                return Err(SanosError::NonFinite {
                    field: "total_variance",
                    value: total_variance,
                });
            }
            if total_variance < 0.0 {
                return Err(SanosError::InvalidBound {
                    field: "total_variance",
                    value: total_variance,
                    min: 0.0,
                    max: f64::INFINITY,
                });
            }

            let sigma = total_variance.sqrt();
            let tiny_sigma_fallback = sigma <= TINY_SIGMA;
            let (x_min, x_max, sigma_for_mapping, mu) = if tiny_sigma_fallback {
                let sigma_floor = (TINY_WINDOW_HALF_WIDTH
                    / self.left_sigmas.max(self.right_sigmas).max(1.0))
                .max(1e-6);
                let mu = -0.5 * sigma * sigma;
                (
                    -TINY_WINDOW_HALF_WIDTH,
                    TINY_WINDOW_HALF_WIDTH,
                    sigma_floor,
                    mu,
                )
            } else {
                let mu = -0.5 * sigma * sigma;
                (
                    -self.left_sigmas * sigma,
                    self.right_sigmas * sigma,
                    sigma,
                    mu,
                )
            };

            let quantile_shape = QuantileShape {
                left_tail_alpha: self.alpha_left,
                right_tail_alpha: self.alpha_right,
            };
            let log_moneyness_mapping = LogMoneynessMapping {
                mean: mu,
                volatility: sigma_for_mapping,
                min_log_moneyness: x_min,
                max_log_moneyness: x_max,
            };

            let mut quantiles =
                generate_quantile_strikes(self.n, quantile_shape, log_moneyness_mapping, &normal);
            quantiles = sort_and_dedup_strikes(quantiles);
            quantiles = enforce_min_spacing(quantiles, self.min_spacing);

            let market_injected = if self.include_market_strikes {
                market_strikes.len()
            } else {
                0
            };

            let merged = if self.include_market_strikes {
                merge_market_and_quantiles(
                    &market_strikes,
                    &quantiles,
                    self.n,
                    self.min_spacing.unwrap_or(0.0).max(UNIQUE_TOL),
                )
            } else if quantiles.len() > self.n {
                pick_evenly_spaced(&quantiles, self.n)
            } else {
                quantiles
            };

            let merged = sort_and_dedup_strikes(merged);
            let points_after_dedup_downsample = merged.len();

            let mut final_strikes = apply_hard_caps(merged, self.k_min, self.k_max);
            final_strikes = sort_and_dedup_strikes(final_strikes);

            if final_strikes.len() < 3 {
                debug!(
                    "LogMoneynessQuantiles fallback activated at T={} (n_after_caps={})",
                    t,
                    final_strikes.len()
                );
                final_strikes = fallback_safe_grid(
                    &market_strikes,
                    self.include_market_strikes,
                    self.k_min,
                    self.k_max,
                );
                final_strikes = sort_and_dedup_strikes(final_strikes);
            }

            if final_strikes.len() < 3 {
                return Err(SanosError::InvalidOrdering {
                    msg: "log-moneyness quantile grid too small after fallback",
                });
            }

            info!(
                "LogMoneynessQuantiles grid: T={} sigma={} window=[{},{}] n={} market_injected={} after_dedup_downsample={}",
                t,
                sigma,
                x_min,
                x_max,
                final_strikes.len(),
                market_injected,
                points_after_dedup_downsample
            );
            debug!(
                "LogMoneynessQuantiles details: T={} target_n={} tiny_sigma_fallback={} used_backbone_variance={}",
                t,
                self.n,
                tiny_sigma_fallback,
                total_variances.is_some()
            );

            grids.push(StrikeGrid::new(t, final_strikes)?);
        }

        Ok(grids)
    }
}

fn standard_normal() -> SanosResult<Normal> {
    Normal::new(0.0, 1.0).map_err(|e| SanosError::External {
        msg: format!("failed to build Normal(0,1): {e}"),
    })
}

fn shaped_probability(sample_index: usize, sample_count: usize, shape: QuantileShape) -> f64 {
    let centered_probability = ((sample_index as f64) + 0.5) / (sample_count as f64);
    if sample_index < sample_count / 2 {
        0.5 * (2.0 * centered_probability).powf(shape.left_tail_alpha)
    } else {
        1.0 - 0.5 * (2.0 * (1.0 - centered_probability)).powf(shape.right_tail_alpha)
    }
}

fn generate_quantile_strikes(
    sample_count: usize,
    shape: QuantileShape,
    mapping: LogMoneynessMapping,
    normal: &Normal,
) -> Vec<f64> {
    let mut out = Vec::with_capacity(sample_count);
    for sample_index in 0..sample_count {
        let u =
            shaped_probability(sample_index, sample_count, shape).clamp(PROB_EPS, 1.0 - PROB_EPS);
        let z = normal.inverse_cdf(u);
        let x = (mapping.mean + mapping.volatility * z)
            .clamp(mapping.min_log_moneyness, mapping.max_log_moneyness);
        out.push(x.exp());
    }
    out
}

fn sort_and_dedup_strikes(mut strikes: Vec<f64>) -> Vec<f64> {
    strikes.retain(|k| k.is_finite() && *k > 0.0);
    strikes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    strikes.dedup_by(|a, b| (*a - *b).abs() <= UNIQUE_TOL);
    strikes
}

fn enforce_min_spacing(strikes: Vec<f64>, min_spacing: Option<f64>) -> Vec<f64> {
    let spacing = match min_spacing {
        Some(v) if v > 0.0 => v,
        _ => return strikes,
    };

    let mut out = Vec::with_capacity(strikes.len());
    for k in strikes {
        if out.is_empty() {
            out.push(k);
            continue;
        }
        let last = *out.last().unwrap();
        if k - last >= spacing {
            out.push(k);
        }
    }
    out
}

fn apply_hard_caps(strikes: Vec<f64>, k_min: Option<f64>, k_max: Option<f64>) -> Vec<f64> {
    let mut out = Vec::with_capacity(strikes.len());
    for mut k in strikes {
        if let Some(lo) = k_min {
            if k < lo {
                k = lo;
            }
        }
        if let Some(hi) = k_max {
            if k > hi {
                k = hi;
            }
        }
        out.push(k);
    }
    out
}

fn merge_market_and_quantiles(
    market_strikes: &[f64],
    quantile_strikes: &[f64],
    target_n: usize,
    distance_tol: f64,
) -> Vec<f64> {
    let market = sort_and_dedup_strikes(market_strikes.to_vec());
    if market.len() >= target_n {
        return market;
    }

    let mut quantile_candidates = Vec::with_capacity(quantile_strikes.len());
    for &k in quantile_strikes {
        if !is_too_close_to_any_sorted(&market, k, distance_tol) {
            quantile_candidates.push(k);
        }
    }

    let slots = target_n.saturating_sub(market.len());
    let extras = pick_evenly_spaced(&quantile_candidates, slots);

    let mut merged = market;
    merged.extend(extras);
    sort_and_dedup_strikes(merged)
}

fn is_too_close_to_any_sorted(values: &[f64], x: f64, tol: f64) -> bool {
    if values.is_empty() {
        return false;
    }

    let idx = values.partition_point(|v| *v < x);
    if idx < values.len() && (values[idx] - x).abs() <= tol {
        return true;
    }
    if idx > 0 && (values[idx - 1] - x).abs() <= tol {
        return true;
    }
    false
}

fn pick_evenly_spaced(values: &[f64], n: usize) -> Vec<f64> {
    if n == 0 || values.is_empty() {
        return Vec::new();
    }
    if values.len() <= n {
        return values.to_vec();
    }
    if n == 1 {
        return vec![values[values.len() / 2]];
    }

    let mut out = Vec::with_capacity(n);
    let len_minus_one = values.len() - 1;
    let n_minus_one = n - 1;

    let mut last_idx: Option<usize> = None;
    for i in 0..n {
        let idx = i * len_minus_one / n_minus_one;
        if last_idx == Some(idx) {
            continue;
        }
        out.push(values[idx]);
        last_idx = Some(idx);
    }

    if out.len() < n {
        for &k in values {
            if out.len() == n {
                break;
            }
            if !out.iter().any(|x| (*x - k).abs() <= UNIQUE_TOL) {
                out.push(k);
            }
        }
        out.sort_by(|a, b| a.partial_cmp(b).unwrap());
    }

    out
}

fn fallback_safe_grid(
    market_strikes: &[f64],
    include_market_strikes: bool,
    k_min: Option<f64>,
    k_max: Option<f64>,
) -> Vec<f64> {
    let mut out = Vec::new();
    if include_market_strikes {
        out.extend(market_strikes.iter().copied());
    }
    out.extend([(-0.1_f64).exp(), 1.0, (0.1_f64).exp()]);
    if let Some(&k0) = market_strikes.first() {
        out.push(k0);
    }
    if let Some(&k1) = market_strikes.last() {
        out.push(k1);
    }

    out = apply_hard_caps(out, k_min, k_max);
    out = sort_and_dedup_strikes(out);
    if out.len() >= 3 {
        return out;
    }

    let (lo, hi) = fallback_bounds(k_min, k_max, market_strikes);
    let mid = (lo * hi).sqrt();
    sort_and_dedup_strikes(vec![lo, mid, hi])
}

fn fallback_bounds(k_min: Option<f64>, k_max: Option<f64>, market_strikes: &[f64]) -> (f64, f64) {
    let (default_lo, default_hi) =
        if let (Some(&lo), Some(&hi)) = (market_strikes.first(), market_strikes.last()) {
            (0.95 * lo, 1.05 * hi)
        } else {
            (0.8, 1.2)
        };

    let mut lo = k_min.unwrap_or(default_lo.max(f64::MIN_POSITIVE));
    let mut hi = k_max.unwrap_or(default_hi.max(lo * 1.1));
    if hi <= lo {
        hi = lo * 1.1;
    }
    lo = lo.max(f64::MIN_POSITIVE);
    (lo, hi)
}

/// Clamp, sort, unique, enforce min log spacing, and ensure ATM if requested.
fn clean_strikes(
    mut strikes: Vec<f64>,
    min_strike: f64,
    max_strike: f64,
    min_spacing_log: f64,
    ensure_atm: bool,
) -> SanosResult<Vec<f64>> {
    let mut filtered: Vec<f64> = Vec::with_capacity(strikes.len());
    for k in strikes.drain(..) {
        if !k.is_finite() || k <= 0.0 {
            continue;
        }
        let kk = k.clamp(min_strike, max_strike);
        filtered.push(kk);
    }
    if filtered.is_empty() {
        return Err(SanosError::EmptyCollection {
            what: "cleaned strikes",
        });
    }

    filtered.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut out: Vec<f64> = Vec::with_capacity(filtered.len());
    for k in filtered {
        if out.is_empty() {
            out.push(k);
            continue;
        }
        let last = *out.last().unwrap();
        let dlog = (k.ln() - last.ln()).abs();
        if dlog >= min_spacing_log {
            out.push(k);
        }
    }

    if ensure_atm {
        let mut has_atm = false;
        let tol = min_spacing_log.max(1e-12);
        for &k in &out {
            if (k.ln()).abs() <= tol {
                has_atm = true;
                break;
            }
        }
        if !has_atm {
            out.push(1.0);
            out.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mut out2: Vec<f64> = Vec::with_capacity(out.len());
            for k in out {
                if out2.is_empty() {
                    out2.push(k);
                    continue;
                }
                let last = *out2.last().unwrap();
                let dlog = (k.ln() - last.ln()).abs();
                if dlog >= min_spacing_log {
                    out2.push(k);
                }
            }
            out = out2;
        }
    }

    if out.len() < 2 {
        return Err(SanosError::InvalidOrdering {
            msg: "strike grid too small after cleaning",
        });
    }

    Ok(out)
}

/// MVP size control:
/// - If keep_all_market_strikes = true: keep all market strikes
/// - Else: allow thinning everything.
fn enforce_size_control(
    strikes: &[f64],
    market_strikes: &[f64],
    max_points: usize,
    keep_all_market_strikes: bool,
    min_spacing_log: f64,
) -> SanosResult<Vec<f64>> {
    if strikes.len() <= max_points {
        return Ok(strikes.to_vec());
    }

    if keep_all_market_strikes && market_strikes.len() > max_points {
        return Err(SanosError::InvalidOrdering {
            msg: "market strikes exceed max_points; cannot keep_all_market_strikes",
        });
    }

    let mut protected = vec![false; strikes.len()];
    if keep_all_market_strikes {
        for (i, &k) in strikes.iter().enumerate() {
            if market_strikes
                .binary_search_by(|x| x.partial_cmp(&k).unwrap())
                .is_ok()
            {
                protected[i] = true;
            }
        }
    }

    let mut out: Vec<f64> = Vec::new();
    for (i, &k) in strikes.iter().enumerate() {
        if protected[i] {
            out.push(k);
        }
    }
    out.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if out.len() <= max_points {
        return Ok(out);
    }

    let mut must_keep = Vec::new();
    must_keep.push(strikes[0]);
    must_keep.push(strikes[strikes.len() - 1]);
    for &k in strikes {
        if (k.ln()).abs() <= min_spacing_log.max(1e-12) {
            must_keep.push(k);
            break;
        }
    }
    must_keep.sort_by(|a, b| a.partial_cmp(b).unwrap());
    must_keep.dedup_by(|a, b| (*a - *b).abs() <= 0.0);

    let mut spacing = min_spacing_log.max(1e-6);
    loop {
        let mut candidate: Vec<f64> = Vec::new();
        candidate.extend(must_keep.iter().copied());
        candidate.extend(strikes.iter().copied());
        candidate.sort_by(|a, b| a.partial_cmp(b).unwrap());
        candidate.dedup_by(|a, b| (*a - *b).abs() <= 0.0);

        let mut filtered: Vec<f64> = Vec::new();
        for k in candidate {
            if filtered.is_empty() {
                filtered.push(k);
                continue;
            }
            let last = *filtered.last().unwrap();
            if (k.ln() - last.ln()).abs() >= spacing {
                filtered.push(k);
            }
        }

        if filtered.len() <= max_points {
            return Ok(filtered);
        }

        spacing *= 1.25;
        if spacing > 1.0 {
            return Ok(filtered);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::{CallQuote, NearestOrLinearLogMoneyness, OptionChain};

    fn sample_book() -> OptionBook {
        let c1 = OptionChain::new(
            0.5,
            vec![
                CallQuote::new(0.9, 0.22, 0.24, 1.0).unwrap(),
                CallQuote::new(1.1, 0.15, 0.17, 1.0).unwrap(),
            ],
        )
        .unwrap();
        let c2 = OptionChain::new(
            1.0,
            vec![
                CallQuote::new(0.85, 0.28, 0.30, 1.0).unwrap(),
                CallQuote::new(1.15, 0.11, 0.13, 1.0).unwrap(),
            ],
        )
        .unwrap();
        OptionBook::new(vec![c2, c1]).unwrap()
    }

    fn median(mut values: Vec<f64>) -> f64 {
        assert!(!values.is_empty());
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = values.len() / 2;
        if values.len() % 2 == 1 {
            values[mid]
        } else {
            0.5 * (values[mid - 1] + values[mid])
        }
    }

    #[test]
    fn clean_strikes_filters_invalid_and_ensures_atm() {
        let out = clean_strikes(vec![f64::NAN, -2.0, 0.5, 0.5, 2.0], 0.1, 5.0, 1e-3, true).unwrap();
        assert!(out.windows(2).all(|w| w[1] > w[0]));
        assert!(out
            .iter()
            .any(|&k| k > 0.0 && (k.ln()).abs() <= 1e-3 + 1e-12));
    }

    #[test]
    fn clean_strikes_rejects_too_small_grid() {
        let err = clean_strikes(vec![0.5], 0.1, 5.0, 1e-3, false).unwrap_err();
        match err {
            SanosError::InvalidOrdering { msg } => {
                assert_eq!(msg, "strike grid too small after cleaning")
            }
            _ => panic!("unexpected error variant: {err:?}"),
        }
    }

    #[test]
    fn enforce_size_control_errors_when_market_exceeds_limit_and_is_protected() {
        let strikes = vec![0.8, 1.0, 1.2];
        let market = vec![0.8, 1.0, 1.2];
        let err = enforce_size_control(&strikes, &market, 2, true, 1e-3).unwrap_err();
        match err {
            SanosError::InvalidOrdering { msg } => {
                assert_eq!(
                    msg,
                    "market strikes exceed max_points; cannot keep_all_market_strikes"
                )
            }
            _ => panic!("unexpected error variant: {err:?}"),
        }
    }

    #[test]
    fn enforce_size_control_noop_when_within_limit() {
        let strikes = vec![0.8, 1.0, 1.2];
        let out = enforce_size_control(&strikes, &strikes, 3, true, 1e-3).unwrap();
        assert_eq!(out, strikes);
    }

    #[test]
    fn market_anchored_build_returns_grids_per_chain() {
        let book = sample_book();
        let atm = NearestOrLinearLogMoneyness::default();
        let policy = MarketAnchored::default();

        let grids = policy.build(&book, &atm, None).unwrap();
        assert_eq!(grids.len(), book.len());
        for g in grids {
            assert!(g.strikes().len() >= 2);
            assert!(g.strikes().windows(2).all(|w| w[1] > w[0]));
            assert!(g.strikes().iter().any(|&k| (k.ln()).abs() <= 1e-2));
        }
    }

    #[test]
    fn market_anchored_build_rejects_invalid_policy_config() {
        let book = sample_book();
        let atm = NearestOrLinearLogMoneyness::default();
        let policy = MarketAnchored {
            min_spacing_log: -1e-3,
            ..MarketAnchored::default()
        };

        let err = policy.build(&book, &atm, None).unwrap_err();
        match err {
            SanosError::InvalidBound { field, .. } => assert_eq!(field, "min_spacing_log"),
            _ => panic!("unexpected error variant: {err:?}"),
        }
    }

    #[test]
    fn log_moneyness_quantiles_shape_is_monotone_and_left_denser() {
        let normal = standard_normal().unwrap();
        let sigma = 0.22;
        let mu = -0.5 * sigma * sigma;
        let strikes = generate_quantile_strikes(
            151,
            QuantileShape {
                left_tail_alpha: 1.8,
                right_tail_alpha: 1.2,
            },
            LogMoneynessMapping {
                mean: mu,
                volatility: sigma,
                min_log_moneyness: -4.5 * sigma,
                max_log_moneyness: 3.0 * sigma,
            },
            &normal,
        );
        let strikes = sort_and_dedup_strikes(strikes);

        assert!(strikes.windows(2).all(|w| w[1] > w[0]));

        let left_spacings: Vec<f64> = strikes
            .windows(2)
            .filter_map(|w| if w[1] <= 1.0 { Some(w[1] - w[0]) } else { None })
            .collect();
        let right_spacings: Vec<f64> = strikes
            .windows(2)
            .filter_map(|w| if w[0] >= 1.0 { Some(w[1] - w[0]) } else { None })
            .collect();

        assert!(!left_spacings.is_empty());
        assert!(!right_spacings.is_empty());
        assert!(median(left_spacings) < median(right_spacings));
    }
}
