use log::{debug, info};

use crate::backbone::bs::bs_implied_atm_var_from_call;
use crate::backbone::config::BsTimeChangedConfig;
use crate::backbone::lognormal_tc::TimeChangedLognormal;
use crate::error::{SanosError, SanosResult};
use crate::market::OptionBook;
use crate::term::PiecewiseLinearCurve;

pub fn build_time_changed_lognormal_from_book(
    book: &OptionBook,
    cfg: &BsTimeChangedConfig,
) -> SanosResult<TimeChangedLognormal> {
    cfg.validate()?;

    let atm_policy = cfg.atm_policy.build()?;
    let atm_calls = book.atm_mids(atm_policy.as_ref())?;

    if atm_calls.is_empty() {
        return Err(SanosError::EmptyCollection {
            what: "OptionBook.atm_mids",
        });
    }

    info!(
        "Building BS time-changed backbone from {} maturities (var_floor={}, enforce_non_decreasing={})",
        atm_calls.len(),
        cfg.var_floor,
        cfg.enforce_non_decreasing
    );

    let mut knots: Vec<(f64, f64)> = Vec::with_capacity(atm_calls.len());
    for (t, c_atm) in atm_calls {
        if !t.is_finite() {
            return Err(SanosError::NonFinite {
                field: "maturity",
                value: t,
            });
        }
        if t <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "maturity",
                value: t,
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }

        let w = bs_implied_atm_var_from_call(c_atm)?;
        let w = w.max(cfg.var_floor);
        debug!("ATM var knot: T={} => W(T)={} (C_atm={})", t, w, c_atm);
        knots.push((t, w));
    }

    if cfg.enforce_non_decreasing {
        let mut prev = knots[0].1;
        for knot in knots.iter_mut().skip(1) {
            if knot.1 < prev {
                debug!(
                    "Clamping ATM total variance: T={} W_old={} W_new={}",
                    knot.0, knot.1, prev
                );
                knot.1 = prev;
            } else {
                prev = knot.1;
            }
        }
    }

    let curve = PiecewiseLinearCurve::new(knots)?;
    Ok(TimeChangedLognormal::new(curve, cfg.eta)
        .with_effective_var_floor(cfg.effective_var_floor))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backbone::bs::bs_call_forward_norm;
    use crate::backbone::config::AtmMidPolicyConfig;
    use crate::market::{CallQuote, OptionChain};

    fn assert_close(actual: f64, expected: f64, tol: f64) {
        assert!(
            (actual - expected).abs() <= tol,
            "expected {expected}, got {actual}, tol={tol}"
        );
    }

    fn chain_from_atm_var(maturity: f64, atm_var: f64) -> OptionChain {
        let atm_call = bs_call_forward_norm(1.0, atm_var).unwrap();
        let q = CallQuote::new(1.0, atm_call, atm_call, 1.0).unwrap();
        OptionChain::new(maturity, vec![q]).unwrap()
    }

    fn book_from_pairs(pairs: &[(f64, f64)]) -> OptionBook {
        let chains: Vec<OptionChain> = pairs
            .iter()
            .map(|&(t, w)| chain_from_atm_var(t, w))
            .collect();
        OptionBook::new(chains).unwrap()
    }

    #[test]
    fn builder_recovers_input_atm_variances() {
        let book = book_from_pairs(&[(0.5, 0.04), (1.0, 0.09)]);
        let cfg = BsTimeChangedConfig {
            atm_policy: AtmMidPolicyConfig::default(),
            var_floor: 0.0,
            enforce_non_decreasing: false,
            eta: 0.25,
            ..BsTimeChangedConfig::default()
        };

        let model = build_time_changed_lognormal_from_book(&book, &cfg).unwrap();
        assert_close(model.var(0.5).unwrap(), 0.04, 1e-10);
        assert_close(model.var(1.0).unwrap(), 0.09, 1e-10);
    }

    #[test]
    fn builder_applies_var_floor() {
        let book = book_from_pairs(&[(0.5, 0.0)]);
        let cfg = BsTimeChangedConfig {
            atm_policy: AtmMidPolicyConfig::default(),
            var_floor: 0.02,
            enforce_non_decreasing: true,
            eta: 0.25,
            ..BsTimeChangedConfig::default()
        };

        let model = build_time_changed_lognormal_from_book(&book, &cfg).unwrap();
        assert_close(model.var(0.5).unwrap(), 0.02, 1e-14);
    }

    #[test]
    fn builder_clamps_decreasing_variance_when_enabled() {
        let book = book_from_pairs(&[(0.5, 0.09), (1.0, 0.04)]);
        let cfg = BsTimeChangedConfig {
            atm_policy: AtmMidPolicyConfig::default(),
            var_floor: 0.0,
            enforce_non_decreasing: true,
            eta: 0.25,
            ..BsTimeChangedConfig::default()
        };

        let model = build_time_changed_lognormal_from_book(&book, &cfg).unwrap();
        assert_close(model.var(0.5).unwrap(), 0.09, 1e-10);
        assert_close(model.var(1.0).unwrap(), 0.09, 1e-10);
    }

    #[test]
    fn builder_keeps_decreasing_variance_when_disabled() {
        let book = book_from_pairs(&[(0.5, 0.09), (1.0, 0.04)]);
        let cfg = BsTimeChangedConfig {
            atm_policy: AtmMidPolicyConfig::default(),
            var_floor: 0.0,
            enforce_non_decreasing: false,
            eta: 0.25,
            ..BsTimeChangedConfig::default()
        };

        let model = build_time_changed_lognormal_from_book(&book, &cfg).unwrap();
        assert_close(model.var(0.5).unwrap(), 0.09, 1e-10);
        assert_close(model.var(1.0).unwrap(), 0.04, 1e-10);
    }
}
