use crate::error::{SanosError, SanosResult};
use crate::fit::config::{FitConfig, ObjectiveConfig};
use crate::fit::kernels::KernelSet;
use crate::fit::lp::model::{LinTerm, LpModel, Sense};
use crate::market::OptionBook;

#[derive(Debug, Clone)]
pub struct LpLayout {
    pub q_var_ids: Vec<Vec<usize>>, // q_var_ids[j][i]
}

#[derive(Debug, Clone)]
pub struct BuiltLp {
    pub model: LpModel,
    pub layout: LpLayout,
}

pub trait LpBuilder: Send + Sync {
    fn build(&self, book: &OptionBook, kernels: &KernelSet, cfg: &FitConfig) -> SanosResult<BuiltLp>;
}

#[derive(Debug, Default, Clone)]
pub struct SanosLpBuilder;

impl SanosLpBuilder {
    #[inline]
    fn paper_quote_weight(quote_bid: f64, quote_ask: f64, quote_weight: f64) -> f64 {
        // Eq. (26): w ~ 1 / (ask - bid). We also keep user-provided quote weight.
        // Clamp spread away from zero for numerical robustness.
        let spread = (quote_ask - quote_bid).max(1e-12);
        quote_weight / spread
    }

    fn add_time_constraints(
        &self,
        lp: &mut LpModel,
        kernels: &KernelSet,
        q_var_ids: &[Vec<usize>],
        cfg: &FitConfig,
    ) -> SanosResult<()> {
        if !cfg.lp.include_time_constraints {
            return Ok(());
        }

        if kernels.transitions.len() + 1 != q_var_ids.len() {
            return Err(SanosError::InvalidOrdering {
                msg: "time constraints require transitions.len() = q.len()-1",
            });
        }

        // For each j >= 1, enforce component-wise:
        //   U_j q_j - R_j q_{j-1} >= 0
        // and optionally a second block for omega=Both.
        for (idx, tr) in kernels.transitions.iter().enumerate() {
            let j = idx + 1;
            let q_prev = &q_var_ids[j - 1];
            let q_cur = &q_var_ids[j];

            if tr.u.nrows != q_cur.len() || tr.u.ncols != q_cur.len() {
                return Err(SanosError::InvalidOrdering {
                    msg: "U dimensions must be Nj x Nj",
                });
            }
            if tr.r.nrows != q_cur.len() || tr.r.ncols != q_prev.len() {
                return Err(SanosError::InvalidOrdering {
                    msg: "R dimensions must be Nj x N(j-1)",
                });
            }

            self.add_one_time_block(lp, j, "time", &tr.u, &tr.r, q_cur, q_prev)?;

            if let (Some(u_alt), Some(r_alt)) = (&tr.u_alt, &tr.r_alt) {
                if u_alt.nrows != q_cur.len() || u_alt.ncols != q_cur.len() {
                    return Err(SanosError::InvalidOrdering {
                        msg: "U_alt dimensions must be Nj x Nj",
                    });
                }
                if r_alt.nrows != q_cur.len() || r_alt.ncols != q_prev.len() {
                    return Err(SanosError::InvalidOrdering {
                        msg: "R_alt dimensions must be Nj x N(j-1)",
                    });
                }
                self.add_one_time_block(lp, j, "time_alt", u_alt, r_alt, q_cur, q_prev)?;
            } else if tr.u_alt.is_some() || tr.r_alt.is_some() {
                return Err(SanosError::InvalidOrdering {
                    msg: "u_alt and r_alt must both be present or both absent",
                });
            }
        }

        Ok(())
    }

    fn add_one_time_block(
        &self,
        lp: &mut LpModel,
        j: usize,
        name_prefix: &str,
        u: &crate::fit::kernels::DenseMat,
        r: &crate::fit::kernels::DenseMat,
        q_cur: &[usize],
        q_prev: &[usize],
    ) -> SanosResult<()> {
        for row in 0..q_cur.len() {
            let mut terms: Vec<LinTerm> = Vec::with_capacity(q_cur.len() + q_prev.len());

            for (col, &vid) in q_cur.iter().enumerate() {
                let coef = u.get(row, col);
                if coef != 0.0 {
                    terms.push(LinTerm { var: vid, coef });
                }
            }
            for (col, &vid) in q_prev.iter().enumerate() {
                let coef = r.get(row, col);
                if coef != 0.0 {
                    terms.push(LinTerm {
                        var: vid,
                        coef: -coef,
                    });
                }
            }

            lp.add_constraint(
                format!("{}_{}_{}", name_prefix, j, row),
                terms,
                Sense::Ge,
                0.0,
            )?;
        }
        Ok(())
    }

    fn build_hard_bid_ask(&self, book: &OptionBook, kernels: &KernelSet, cfg: &FitConfig) -> SanosResult<BuiltLp> {
        // basic alignment checks
        if kernels.c.len() != book.len() {
            return Err(SanosError::InvalidOrdering { msg: "kernels.c.len() must match book.len()" });
        }

        let mut lp = LpModel::new();
        let mut q_var_ids: Vec<Vec<usize>> = Vec::with_capacity(book.len());

        // 1) variables q_{j,i}
        for (j, kc) in kernels.c.iter().enumerate() {
            let n_mod = kc.model_strikes.len();
            if n_mod == 0 {
                return Err(SanosError::EmptyCollection { what: "model_strikes" });
            }

            let mut qj = Vec::with_capacity(n_mod);
            for i in 0..n_mod {
                let name = format!("q_{}_{}", j, i);
                let lb = if cfg.lp.enforce_nonnegativity { 0.0 } else { f64::NEG_INFINITY };
                let ub = f64::INFINITY;
                qj.push(lp.add_var(name, lb, ub)?);
            }

            // simplex constraint: sum_i q_{j,i} = 1
            if cfg.lp.enforce_simplex {
                let terms = qj.iter().map(|&vid| LinTerm { var: vid, coef: 1.0 }).collect();
                lp.add_constraint(format!("simplex_{}", j), terms, Sense::Eq, 1.0)?;
            }

            q_var_ids.push(qj);
        }

        // 2) Hard bid/ask constraints at each quote:
        //    bid <= p <= ask
        for (j, chain) in book.chains().iter().enumerate() {
            let quotes = chain.quotes();
            let kc = &kernels.c[j];
            let qj = &q_var_ids[j];

            let n_mkt = quotes.len();
            if kc.market_strikes.len() != n_mkt {
                return Err(SanosError::InvalidOrdering { msg: "kernel market_strikes must align with chain quotes" });
            }
            if kc.c.nrows != n_mkt || kc.c.ncols != qj.len() {
                return Err(SanosError::InvalidOrdering { msg: "kernel matrix dims mismatch" });
            }

            for (m, quote) in quotes.iter().enumerate() {
                let mut terms = Vec::with_capacity(qj.len());
                for (i, &qid) in qj.iter().enumerate() {
                    let c = kc.c.get(m, i);
                    if c != 0.0 {
                        terms.push(LinTerm { var: qid, coef: c });
                    }
                }

                lp.add_constraint(format!("hard_bid_{}_{}", j, m), terms.clone(), Sense::Ge, quote.bid)?;
                lp.add_constraint(format!("hard_ask_{}_{}", j, m), terms, Sense::Le, quote.ask)?;
            }

            // mean constraint: sum_i q_{j,i} * k_i = 1
            // model_strikes are the k_i atoms
            let mut mean_terms = Vec::with_capacity(qj.len());
            for (i, &vid) in qj.iter().enumerate() {
                let k_i = kernels.c[j].model_strikes[i];
                mean_terms.push(LinTerm { var: vid, coef: k_i });
            }
            lp.add_constraint(format!("mean_{}", j), mean_terms, Sense::Eq, 1.0)?;
        }

        self.add_time_constraints(&mut lp, kernels, &q_var_ids, cfg)?;

        Ok(BuiltLp { model: lp, layout: LpLayout { q_var_ids } })
    }

    fn build_hinge_bid_ask(
        &self,
        book: &OptionBook,
        kernels: &KernelSet,
        slack_penalty: f64,
        epsilon_inside: f64,
        cfg: &FitConfig,
    ) -> SanosResult<BuiltLp> {
        if !slack_penalty.is_finite() || slack_penalty <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "objective.slack_penalty",
                value: slack_penalty,
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }
        if !epsilon_inside.is_finite() || epsilon_inside < 0.0 {
            return Err(SanosError::InvalidBound {
                field: "objective.epsilon_inside",
                value: epsilon_inside,
                min: 0.0,
                max: f64::INFINITY,
            });
        }

        if kernels.c.len() != book.len() {
            return Err(SanosError::InvalidOrdering { msg: "kernels.c.len() must match book.len()" });
        }

        let mut lp = LpModel::new();
        let mut q_var_ids: Vec<Vec<usize>> = Vec::with_capacity(book.len());

        // 1) variables q_{j,i}
        for (j, kc) in kernels.c.iter().enumerate() {
            let n_mod = kc.model_strikes.len();
            if n_mod == 0 {
                return Err(SanosError::EmptyCollection { what: "model_strikes" });
            }

            let mut qj = Vec::with_capacity(n_mod);
            for i in 0..n_mod {
                let name = format!("q_{}_{}", j, i);
                let lb = if cfg.lp.enforce_nonnegativity { 0.0 } else { f64::NEG_INFINITY };
                let ub = f64::INFINITY;
                qj.push(lp.add_var(name, lb, ub)?);
            }

            if cfg.lp.enforce_simplex {
                let terms = qj.iter().map(|&vid| LinTerm { var: vid, coef: 1.0 }).collect();
                lp.add_constraint(format!("simplex_{}", j), terms, Sense::Eq, 1.0)?;
            }

            q_var_ids.push(qj);
        }

        // 2) SANOS robust objective (Remark 4.1):
        //    epsilon_inside * |mid - p|
        //  + slack_penalty * (max(0, bid - p) + max(0, p - ask))
        for (j, chain) in book.chains().iter().enumerate() {
            let quotes = chain.quotes();
            let kc = &kernels.c[j];
            let qj = &q_var_ids[j];

            let n_mkt = quotes.len();
            if kc.market_strikes.len() != n_mkt {
                return Err(SanosError::InvalidOrdering { msg: "kernel market_strikes must align with chain quotes" });
            }
            if kc.c.nrows != n_mkt || kc.c.ncols != qj.len() {
                return Err(SanosError::InvalidOrdering { msg: "kernel matrix dims mismatch" });
            }

            for (m, quote) in quotes.iter().enumerate() {
                let e_mid = lp.add_var(format!("e_mid_{}_{}", j, m), 0.0, f64::INFINITY)?;
                let s_bid = lp.add_var(format!("s_bid_{}_{}", j, m), 0.0, f64::INFINITY)?;
                let s_ask = lp.add_var(format!("s_ask_{}_{}", j, m), 0.0, f64::INFINITY)?;
                let mid = quote.mid();

                // |mid - p| <= e_mid
                // p - mid <= e_mid
                let mut mid_pos_terms: Vec<LinTerm> = Vec::with_capacity(qj.len() + 1);
                for (i, &qid) in qj.iter().enumerate() {
                    let c = kc.c.get(m, i);
                    if c != 0.0 {
                        mid_pos_terms.push(LinTerm { var: qid, coef: c });
                    }
                }
                mid_pos_terms.push(LinTerm { var: e_mid, coef: -1.0 });
                lp.add_constraint(format!("hinge_mid_pos_{}_{}", j, m), mid_pos_terms, Sense::Le, mid)?;

                // mid - p <= e_mid
                let mut mid_neg_terms: Vec<LinTerm> = Vec::with_capacity(qj.len() + 1);
                for (i, &qid) in qj.iter().enumerate() {
                    let c = kc.c.get(m, i);
                    if c != 0.0 {
                        mid_neg_terms.push(LinTerm { var: qid, coef: -c });
                    }
                }
                mid_neg_terms.push(LinTerm { var: e_mid, coef: -1.0 });
                lp.add_constraint(format!("hinge_mid_neg_{}_{}", j, m), mid_neg_terms, Sense::Le, -mid)?;

                // bid - p <= s_bid  <=>  p + s_bid >= bid
                let mut lo_terms: Vec<LinTerm> = Vec::with_capacity(qj.len() + 1);
                for (i, &qid) in qj.iter().enumerate() {
                    let c = kc.c.get(m, i);
                    if c != 0.0 {
                        lo_terms.push(LinTerm { var: qid, coef: c });
                    }
                }
                lo_terms.push(LinTerm { var: s_bid, coef: 1.0 });
                lp.add_constraint(
                    format!("hinge_bid_{}_{}", j, m),
                    lo_terms,
                    Sense::Ge,
                    quote.bid,
                )?;

                // p - ask <= s_ask  <=>  p - s_ask <= ask
                let mut hi_terms: Vec<LinTerm> = Vec::with_capacity(qj.len() + 1);
                for (i, &qid) in qj.iter().enumerate() {
                    let c = kc.c.get(m, i);
                    if c != 0.0 {
                        hi_terms.push(LinTerm { var: qid, coef: c });
                    }
                }
                hi_terms.push(LinTerm { var: s_ask, coef: -1.0 });
                lp.add_constraint(
                    format!("hinge_ask_{}_{}", j, m),
                    hi_terms,
                    Sense::Le,
                    quote.ask,
                )?;

                let wq = Self::paper_quote_weight(quote.bid, quote.ask, quote.weight);
                let w = wq * slack_penalty;
                let w_mid = wq * epsilon_inside;
                if w_mid.is_finite() && w_mid > 0.0 {
                    lp.add_obj_term(e_mid, w_mid)?;
                }
                if w.is_finite() && w > 0.0 {
                    lp.add_obj_term(s_bid, w)?;
                    lp.add_obj_term(s_ask, w)?;
                }
            }

            let mut mean_terms = Vec::with_capacity(qj.len());
            for (i, &vid) in qj.iter().enumerate() {
                let k_i = kernels.c[j].model_strikes[i];
                mean_terms.push(LinTerm { var: vid, coef: k_i });
            }
            lp.add_constraint(format!("mean_{}", j), mean_terms, Sense::Eq, 1.0)?;
        }

        self.add_time_constraints(&mut lp, kernels, &q_var_ids, cfg)?;

        Ok(BuiltLp { model: lp, layout: LpLayout { q_var_ids } })
    }

    fn build_l1_mid(&self, book: &OptionBook, kernels: &KernelSet, global_weight: f64, cfg: &FitConfig) -> SanosResult<BuiltLp> {
        if !global_weight.is_finite() || global_weight <= 0.0 {
            return Err(SanosError::InvalidBound {
                field: "objective.weight",
                value: global_weight,
                min: f64::MIN_POSITIVE,
                max: f64::INFINITY,
            });
        }

        // basic alignment checks
        if kernels.c.len() != book.len() {
            return Err(SanosError::InvalidOrdering { msg: "kernels.c.len() must match book.len()" });
        }

        let mut lp = LpModel::new();
        let mut q_var_ids: Vec<Vec<usize>> = Vec::with_capacity(book.len());

        // 1) variables q_{j,i}
        for (j, kc) in kernels.c.iter().enumerate() {
            let n_mod = kc.model_strikes.len();
            if n_mod == 0 {
                return Err(SanosError::EmptyCollection { what: "model_strikes" });
            }

            let mut qj = Vec::with_capacity(n_mod);
            for i in 0..n_mod {
                let name = format!("q_{}_{}", j, i);
                let lb = if cfg.lp.enforce_nonnegativity { 0.0 } else { f64::NEG_INFINITY };
                let ub = f64::INFINITY;
                qj.push(lp.add_var(name, lb, ub)?);
            }

            // simplex constraint: sum_i q_{j,i} = 1
            if cfg.lp.enforce_simplex {
                let terms = qj.iter().map(|&vid| LinTerm { var: vid, coef: 1.0 }).collect();
                lp.add_constraint(format!("simplex_{}", j), terms, Sense::Eq, 1.0)?;
            }

            q_var_ids.push(qj);
        }

        // 2) per-quote L1 slack constraints + objective
        // For each quote m at maturity j:
        //   p - mid <= e
        //   mid - p <= e
        // objective: sum w * e
        for (j, chain) in book.chains().iter().enumerate() {
            let quotes = chain.quotes();
            let kc = &kernels.c[j];
            let qj = &q_var_ids[j];

            let n_mkt = quotes.len();
            if kc.market_strikes.len() != n_mkt {
                return Err(SanosError::InvalidOrdering { msg: "kernel market_strikes must align with chain quotes" });
            }
            if kc.c.nrows != n_mkt || kc.c.ncols != qj.len() {
                return Err(SanosError::InvalidOrdering { msg: "kernel matrix dims mismatch" });
            }

            for m in 0..n_mkt {
                let q = quotes[m];
                let mid = q.mid();

                // slack variable e_{j,m} >= 0
                let e_id = lp.add_var(format!("e_{}_{}", j, m), 0.0, f64::INFINITY)?;

                // p = sum_i q_{j,i} * C[m,i]
                // constraint 1: p - mid <= e  => sum_i C[m,i] q_i - e <= mid
                let mut terms1: Vec<LinTerm> = Vec::with_capacity(qj.len() + 1);
                for (i, &qid) in qj.iter().enumerate() {
                    let c = kc.c.get(m, i);
                    if c != 0.0 {
                        terms1.push(LinTerm { var: qid, coef: c });
                    }
                }
                terms1.push(LinTerm { var: e_id, coef: -1.0 });
                lp.add_constraint(format!("l1_pos_{}_{}", j, m), terms1, Sense::Le, mid)?;

                // constraint 2: mid - p <= e  => -sum_i C[m,i] q_i - e <= -mid
                let mut terms2: Vec<LinTerm> = Vec::with_capacity(qj.len() + 1);
                for (i, &qid) in qj.iter().enumerate() {
                    let c = kc.c.get(m, i);
                    if c != 0.0 {
                        terms2.push(LinTerm { var: qid, coef: -c });
                    }
                }
                terms2.push(LinTerm { var: e_id, coef: -1.0 });
                lp.add_constraint(format!("l1_neg_{}_{}", j, m), terms2, Sense::Le, -mid)?;

                // objective coefficient
                let wq = Self::paper_quote_weight(q.bid, q.ask, q.weight);
                let w = wq * global_weight;
                if w.is_finite() && w > 0.0 {
                    lp.add_obj_term(e_id, w)?;
                }
            }

            // mean constraint: sum_i q_{j,i} * k_i = 1
            // model_strikes are the k_i atoms
            let mut mean_terms = Vec::with_capacity(qj.len());
            for (i, &vid) in qj.iter().enumerate() {
                let k_i = kernels.c[j].model_strikes[i];
                mean_terms.push(LinTerm { var: vid, coef: k_i });
            }
            lp.add_constraint(format!("mean_{}", j), mean_terms, Sense::Eq, 1.0)?;
        }

        self.add_time_constraints(&mut lp, kernels, &q_var_ids, cfg)?;

        Ok(BuiltLp { model: lp, layout: LpLayout { q_var_ids } })
    }
}

impl LpBuilder for SanosLpBuilder {
    fn build(&self, book: &OptionBook, kernels: &KernelSet, cfg: &FitConfig) -> SanosResult<BuiltLp> {
        cfg.validate()?;
        kernels.validate()?;

        match cfg.objective {
            ObjectiveConfig::HardBidAsk => self.build_hard_bid_ask(book, kernels, cfg),
            ObjectiveConfig::HingeBidAsk { slack_penalty, epsilon_inside } => {
                self.build_hinge_bid_ask(book, kernels, slack_penalty, epsilon_inside, cfg)
            }
            ObjectiveConfig::L1Mid { weight } => self.build_l1_mid(book, kernels, weight, cfg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::backbone::bs::bs_call_forward_norm;
    use crate::backbone::{TimeChangedLognormal, YModel};
    use crate::fit::config::{LpConfig, ObjectiveConfig, OmegaConfig};
    use crate::fit::kernel_builder::build_kernels;
    use crate::grid::StrikeGrid;
    use crate::market::{CallQuote, OptionBook, OptionChain};
    use crate::term::PiecewiseLinearCurve;

    fn sample_book_and_grids() -> (OptionBook, Vec<StrikeGrid>, Arc<TimeChangedLognormal>) {
        let t0 = 0.5;
        let t1 = 1.0;
        let w0 = 0.04;
        let w1 = 0.16;

        let q0 = vec![
            CallQuote::new(0.9, bs_call_forward_norm(0.9, w0).unwrap(), bs_call_forward_norm(0.9, w0).unwrap(), 1.0).unwrap(),
            CallQuote::new(1.0, bs_call_forward_norm(1.0, w0).unwrap(), bs_call_forward_norm(1.0, w0).unwrap(), 1.0).unwrap(),
            CallQuote::new(1.1, bs_call_forward_norm(1.1, w0).unwrap(), bs_call_forward_norm(1.1, w0).unwrap(), 1.0).unwrap(),
        ];
        let q1 = vec![
            CallQuote::new(0.9, bs_call_forward_norm(0.9, w1).unwrap(), bs_call_forward_norm(0.9, w1).unwrap(), 1.0).unwrap(),
            CallQuote::new(1.0, bs_call_forward_norm(1.0, w1).unwrap(), bs_call_forward_norm(1.0, w1).unwrap(), 1.0).unwrap(),
            CallQuote::new(1.1, bs_call_forward_norm(1.1, w1).unwrap(), bs_call_forward_norm(1.1, w1).unwrap(), 1.0).unwrap(),
        ];

        let c0 = OptionChain::new(t0, q0).unwrap();
        let c1 = OptionChain::new(t1, q1).unwrap();
        let book = OptionBook::new(vec![c0, c1]).unwrap();

        let grid0 = StrikeGrid::new(t0, vec![0.9, 1.0, 1.1]).unwrap();
        let grid1 = StrikeGrid::new(t1, vec![0.9, 1.0, 1.1]).unwrap();

        let curve = PiecewiseLinearCurve::new(vec![(t0, w0), (t1, w1)]).unwrap();
        let y = Arc::new(TimeChangedLognormal::new(curve, 1.0));

        (book, vec![grid0, grid1], y)
    }

    #[test]
    fn transition_r_uses_previous_maturity() {
        let (_book, grids, y) = sample_book_and_grids();

        let t0 = grids[0].maturity();
        let t1 = grids[1].maturity();
        let q0 = vec![CallQuote::new(1.0, y.call(t0, 1.0, 1.0).unwrap(), y.call(t0, 1.0, 1.0).unwrap(), 1.0).unwrap()];
        let q1 = vec![CallQuote::new(1.0, y.call(t1, 1.0, 1.0).unwrap(), y.call(t1, 1.0, 1.0).unwrap(), 1.0).unwrap()];
        let book = OptionBook::new(vec![OptionChain::new(t0, q0).unwrap(), OptionChain::new(t1, q1).unwrap()]).unwrap();

        let kernels = build_kernels(&book, &grids, &(y.clone() as Arc<dyn YModel>), &FitConfig::default().kernel).unwrap();
        let tr = &kernels.transitions[0];

        let r_11 = tr.r.get(1, 1);
        let expected_prev = y.call(t0, 1.0, 1.0).unwrap();
        let wrong_current = y.call(t1, 1.0, 1.0).unwrap();

        assert!((r_11 - expected_prev).abs() < 1e-12);
        assert!((r_11 - wrong_current).abs() > 1e-6);
    }

    #[test]
    fn lp_builder_adds_time_constraints_when_enabled() {
        let (book, grids, y) = sample_book_and_grids();
        let y_dyn = y as Arc<dyn YModel>;
        let fit = FitConfig {
            objective: ObjectiveConfig::HardBidAsk,
            ..FitConfig::default()
        };
        let kernels = build_kernels(&book, &grids, &y_dyn, &fit.kernel).unwrap();

        let builder = SanosLpBuilder::default();
        let built = builder.build(&book, &kernels, &fit).unwrap();

        let expected: usize = kernels.transitions.iter().map(|tr| tr.u.nrows).sum();
        let actual = built
            .model
            .constraints
            .iter()
            .filter(|c| c.name.starts_with("time_"))
            .count();

        assert_eq!(actual, expected);
    }

    #[test]
    fn lp_builder_skips_time_constraints_when_disabled() {
        let (book, grids, y) = sample_book_and_grids();
        let y_dyn = y as Arc<dyn YModel>;
        let fit = FitConfig {
            objective: ObjectiveConfig::HardBidAsk,
            lp: LpConfig {
                include_time_constraints: false,
                ..LpConfig::default()
            },
            ..FitConfig::default()
        };

        let kernels = build_kernels(&book, &grids, &y_dyn, &fit.kernel).unwrap();
        let built = SanosLpBuilder::default()
            .build(&book, &kernels, &fit)
            .unwrap();

        let actual = built
            .model
            .constraints
            .iter()
            .filter(|c| c.name.starts_with("time_"))
            .count();
        assert_eq!(actual, 0);
    }

    #[test]
    fn lp_builder_adds_both_time_constraint_blocks_for_omega_both() {
        let (book, grids, y) = sample_book_and_grids();
        let y_dyn = y as Arc<dyn YModel>;

        let mut fit = FitConfig {
            objective: ObjectiveConfig::HardBidAsk,
            ..FitConfig::default()
        };
        fit.kernel.omega = OmegaConfig::Both;

        let kernels = build_kernels(&book, &grids, &y_dyn, &fit.kernel).unwrap();
        assert!(kernels
            .transitions
            .iter()
            .all(|tr| tr.u_alt.is_some() && tr.r_alt.is_some()));

        let built = SanosLpBuilder::default().build(&book, &kernels, &fit).unwrap();
        let expected: usize = kernels.transitions.iter().map(|tr| tr.u.nrows * 2).sum();
        let actual = built
            .model
            .constraints
            .iter()
            .filter(|c| c.name.starts_with("time_"))
            .count();

        assert_eq!(actual, expected);
    }

    #[test]
    fn hinge_bid_ask_objective_builds() {
        let (book, grids, y) = sample_book_and_grids();
        let y_dyn = y as Arc<dyn YModel>;
        let fit = FitConfig {
            objective: ObjectiveConfig::HingeBidAsk {
                slack_penalty: 1000.0,
                epsilon_inside: 0.0,
            },
            ..FitConfig::default()
        };
        let kernels = build_kernels(&book, &grids, &y_dyn, &fit.kernel).unwrap();

        let built = SanosLpBuilder::default()
            .build(&book, &kernels, &fit)
            .unwrap();
        assert!(!built.model.objective.terms.is_empty());
    }

    #[test]
    fn hinge_objective_uses_inverse_spread_weights() {
        let t = 0.5;
        let q0 = CallQuote::new(0.9, 0.25, 0.26, 1.0).unwrap();
        let q1 = CallQuote::new(1.1, 0.08, 0.18, 1.0).unwrap();

        let book = OptionBook::new(vec![OptionChain::new(t, vec![q0, q1]).unwrap()]).unwrap();
        let grids = vec![StrikeGrid::new(t, vec![0.9, 1.0, 1.1]).unwrap()];

        let curve = PiecewiseLinearCurve::new(vec![(t, 0.04)]).unwrap();
        let y = Arc::new(TimeChangedLognormal::new(curve, 0.25)) as Arc<dyn YModel>;

        let fit = FitConfig {
            objective: ObjectiveConfig::HingeBidAsk {
                slack_penalty: 1.0,
                epsilon_inside: 1.0,
            },
            ..FitConfig::default()
        };
        let kernels = build_kernels(&book, &grids, &y, &fit.kernel).unwrap();
        let built = SanosLpBuilder::default().build(&book, &kernels, &fit).unwrap();

        let coef = |var_name: &str| -> f64 {
            let vid = built
                .model
                .vars
                .iter()
                .position(|v| v.name == var_name)
                .expect("variable must exist");
            built
                .model
                .objective
                .terms
                .iter()
                .find(|t| t.var == vid)
                .map(|t| t.coef)
                .unwrap_or(0.0)
        };

        let e0 = coef("e_mid_0_0");
        let e1 = coef("e_mid_0_1");
        let s0 = coef("s_bid_0_0");
        let s1 = coef("s_bid_0_1");

        assert!((e0 / e1 - 10.0).abs() < 1e-12, "e_mid ratio expected 10, got {}", e0 / e1);
        assert!((s0 / s1 - 10.0).abs() < 1e-12, "slack ratio expected 10, got {}", s0 / s1);
    }
}
