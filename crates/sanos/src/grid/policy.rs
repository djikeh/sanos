// src/grid/policy.rs
use crate::error::{SanosError, SanosResult};
use crate::grid::config::{AtmRefineConfig, GridSizeConfig, WingsConfig};
use crate::grid::StrikeGrid;
use crate::market::{AtmMidPolicy, OptionBook};

pub trait StrikeGridPolicy: Send + Sync {
    fn build(&self, book: &OptionBook, atm: &dyn AtmMidPolicy) -> SanosResult<Vec<StrikeGrid>>;
}

/// Market-anchored strike grid:
/// K_j = market_strikes ∪ {1} ∪ wings ∪ (optional ATM refine), then cleaned.
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
            return Err(SanosError::InvalidOrdering { msg: "max_strike must be > min_strike" });
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
    fn build(&self, book: &OptionBook, _atm: &dyn AtmMidPolicy) -> SanosResult<Vec<StrikeGrid>> {
        self.validate()?;

        let mut grids = Vec::with_capacity(book.len());

        for chain in book.chains() {
            let t = chain.maturity();
            let market_strikes: Vec<f64> = chain.quotes().iter().map(|q| q.k).collect();

            let k_min = *market_strikes.first().ok_or(SanosError::EmptyCollection { what: "market strikes" })?;
            let k_max = *market_strikes.last().ok_or(SanosError::EmptyCollection { what: "market strikes" })?;

            // Build candidate list
            let mut cand: Vec<f64> = Vec::new();
            cand.extend(market_strikes.iter().copied());

            if self.ensure_atm {
                cand.push(1.0);
            }

            // Wings
            let r = self.wings.ratio;
            for p in 1..=self.wings.n_left {
                cand.push(k_min / r.powi(p as i32));
            }
            for p in 1..=self.wings.n_right {
                cand.push(k_max * r.powi(p as i32));
            }

            // ATM refine (around 1 in log space)
            if self.atm_refine.enabled {
                let d = self.atm_refine.delta_log;
                for s in 1..=self.atm_refine.steps {
                    let x = (s as f64) * d;
                    cand.push(x.exp());
                    cand.push((-x).exp());
                }
            }

            // Clean candidates
            let strikes = clean_strikes(
                cand,
                self.min_strike,
                self.max_strike,
                self.min_spacing_log,
                self.ensure_atm,
            )?;

            // Size control (MVP):
            // If too many points, we keep market strikes first and then downsample the rest.
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

/// Clamp, sort, unique, enforce min log spacing, and ensure ATM if requested.
fn clean_strikes(
    mut strikes: Vec<f64>,
    min_strike: f64,
    max_strike: f64,
    min_spacing_log: f64,
    ensure_atm: bool,
) -> SanosResult<Vec<f64>> {
    // Filter finite and clamp
    let mut filtered: Vec<f64> = Vec::with_capacity(strikes.len());
    for k in strikes.drain(..) {
        if !k.is_finite() {
            continue;
        }
        if k <= 0.0 {
            continue;
        }
        let kk = k.clamp(min_strike, max_strike);
        filtered.push(kk);
    }
    if filtered.is_empty() {
        return Err(SanosError::EmptyCollection { what: "cleaned strikes" });
    }

    filtered.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Enforce uniqueness with log-spacing threshold
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

    // Ensure ATM
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
            // Re-unique after inserting
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
        return Err(SanosError::InvalidOrdering { msg: "strike grid too small after cleaning" });
    }

    Ok(out)
}

/// MVP size control:
/// - If keep_all_market_strikes = true: keep all market strikes
/// - Else: allow thinning everything.
/// Strategy:
/// 1) Keep market strikes (exact float equality here; ok because they come from the same source vector)
/// 2) Add extra strikes by increasing log-spacing until we meet max_points.
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
            if market_strikes.binary_search_by(|x| x.partial_cmp(&k).unwrap()).is_ok() {
                protected[i] = true;
            }
        }
    }

    // Start with protected (market) points
    let mut out: Vec<f64> = Vec::new();
    for (i, &k) in strikes.iter().enumerate() {
        if protected[i] {
            out.push(k);
        }
    }
    out.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // If already within limit, done
    if out.len() <= max_points {
        return Ok(out);
    }

    // Otherwise, thin even protected points (rare), by increasing spacing.
    // MVP: keep endpoints and ATM if present, then enforce spacing.
    let mut must_keep = Vec::new();
    must_keep.push(strikes[0]);
    must_keep.push(strikes[strikes.len() - 1]);
    // try keep ATM
    for &k in strikes {
        if (k.ln()).abs() <= min_spacing_log.max(1e-12) {
            must_keep.push(k);
            break;
        }
    }
    must_keep.sort_by(|a, b| a.partial_cmp(b).unwrap());
    must_keep.dedup_by(|a, b| (*a - *b).abs() == 0.0);

    // Increase spacing until size is <= max_points
    let mut spacing = min_spacing_log.max(1e-6);
    loop {
        let mut candidate: Vec<f64> = Vec::new();
        candidate.extend(must_keep.iter().copied());

        for &k in strikes {
            candidate.push(k);
        }
        candidate.sort_by(|a, b| a.partial_cmp(b).unwrap());
        candidate.dedup_by(|a, b| (*a - *b).abs() == 0.0);

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

        // Ensure must_keep are still present (they should be due to merging first, but keep safe)
        // If too big, increase spacing; else accept.
        if filtered.len() <= max_points {
            return Ok(filtered);
        }

        spacing *= 1.25;
        if spacing > 1.0 {
            // Extremely aggressive thinning; accept last filtered even if large to avoid infinite loop.
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

    #[test]
    fn clean_strikes_filters_invalid_and_ensures_atm() {
        let out = clean_strikes(vec![f64::NAN, -2.0, 0.5, 0.5, 2.0], 0.1, 5.0, 1e-3, true).unwrap();
        assert!(out.windows(2).all(|w| w[1] > w[0]));
        assert!(out.iter().any(|&k| k > 0.0 && (k.ln()).abs() <= 1e-3 + 1e-12));
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
                assert_eq!(msg, "market strikes exceed max_points; cannot keep_all_market_strikes")
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

        let grids = policy.build(&book, &atm).unwrap();
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

        let err = policy.build(&book, &atm).unwrap_err();
        match err {
            SanosError::InvalidBound { field, .. } => assert_eq!(field, "min_spacing_log"),
            _ => panic!("unexpected error variant: {err:?}"),
        }
    }
}
