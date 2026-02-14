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

            for row in 0..q_cur.len() {
                let mut terms: Vec<LinTerm> = Vec::with_capacity(q_cur.len() + q_prev.len());

                for (col, &vid) in q_cur.iter().enumerate() {
                    let coef = tr.u.get(row, col);
                    if coef != 0.0 {
                        terms.push(LinTerm { var: vid, coef });
                    }
                }
                for (col, &vid) in q_prev.iter().enumerate() {
                    let coef = tr.r.get(row, col);
                    if coef != 0.0 {
                        terms.push(LinTerm { var: vid, coef: -coef });
                    }
                }

                lp.add_constraint(format!("time_{}_{}", j, row), terms, Sense::Ge, 0.0)?;
            }
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

        // 2) Soft bid/ask with hinge slacks.
        // s_bid >= max(0, bid+eps - p), s_ask >= max(0, p-(ask-eps))
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
                let s_bid = lp.add_var(format!("s_bid_{}_{}", j, m), 0.0, f64::INFINITY)?;
                let s_ask = lp.add_var(format!("s_ask_{}_{}", j, m), 0.0, f64::INFINITY)?;

                // p + s_bid >= bid + epsilon_inside
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
                    quote.bid + epsilon_inside,
                )?;

                // p - s_ask <= ask - epsilon_inside
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
                    quote.ask - epsilon_inside,
                )?;

                let w = quote.weight * slack_penalty;
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
                let w = q.weight * global_weight;
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
