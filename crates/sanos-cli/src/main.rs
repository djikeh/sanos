use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

use sanos::backbone::{
    bs_implied_vol_from_call, build_time_changed_lognormal_from_book, BackboneConfig,
    BsTimeChangedConfig,
};
use sanos::calibration::{calibrate_with_stats, CalibrationConfig, CalibrationRunStats};
use sanos::density::DensityTolerances;
use sanos::fit::{FitConfig, LpSolverConfig, ObjectiveConfig};
use sanos::grid::StrikeGridPolicyConfig;
use sanos::interp::TimeInterpConfig;
use sanos::market::OptionBook;
use sanos::surface::SanosSurface;

#[derive(Debug, Parser)]
#[command(name = "sanos")]
#[command(about = "SANOS CLI - run calibrations on JSON snapshots", long_about = None)]
struct Cli {
    /// Verbosity (-v, -vv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Calibrate a SANOS surface from an IV snapshot and a calibration config.
    #[command(alias = "cal")]
    Calibrate {
        /// Path to IV snapshot JSON (schema v1)
        #[arg(long)]
        snapshot: PathBuf,

        /// Path to calibration config JSON
        #[arg(long)]
        config: PathBuf,

        /// Optional output directory (writes q.json + report.json)
        #[arg(long)]
        out: Option<PathBuf>,

        /// Number of maturities for dense surface export.
        #[arg(long, default_value_t = 41)]
        n_maturities: usize,

        /// Number of strikes for dense surface export.
        #[arg(long, default_value_t = 81)]
        n_strikes: usize,
    },

    /// Validate an IV snapshot (schema + basic numeric checks)
    #[command(alias = "validate")]
    ValidateSnapshot {
        #[arg(long)]
        snapshot: PathBuf,
    },

    /// Validate and reconstruct a SANOS surface from surface.json.
    ValidateSurface {
        #[arg(long)]
        surface: PathBuf,
    },

    /// Write a default calibration config JSON template.
    InitConfig {
        /// Output path for the generated JSON template.
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Debug, serde::Serialize)]
struct CalibrationReport {
    snapshot: String,
    config: String,
    n_maturities: usize,
    marginals: Vec<MarginalReport>,
}

#[derive(Debug, serde::Serialize)]
struct MarginalReport {
    maturity: f64,
    n_atoms: usize,
    mass: f64,
    mean: f64,
}

#[derive(Debug, serde::Serialize)]
struct SurfaceJson {
    schema: String,
    snapshot: String,
    config: String,
    maturities: Vec<f64>,
    strikes: Vec<f64>,
    calls: Vec<Vec<f64>>,
    marginals: Vec<QJsonMarginal>,
    reconstruction: SurfaceReconstructionJson,
}

#[derive(Debug, serde::Serialize)]
struct SurfaceReconstructionJson {
    backbone: ReconstructionBackboneJson,
    time_interp: TimeInterpConfig,
    marginals: Vec<QJsonMarginal>,
}

#[derive(Debug, serde::Serialize)]
struct ReconstructionBackboneJson {
    model: String,
    eta: f64,
    var_curve_knots: Vec<VarianceKnotJson>,
}

#[derive(Debug, serde::Serialize)]
struct VarianceKnotJson {
    maturity: f64,
    total_variance: f64,
}

#[derive(Debug, serde::Serialize)]
struct DiagnosticsJson {
    schema: String,
    summary: DiagnosticsSummary,
    no_arbitrage: NoArbitrageDiagnostics,
    smoothness_comparison: Option<SmoothnessComparison>,
    per_maturity: Vec<DiagnosticsPerMaturity>,
    quotes: Vec<QuoteDiagnostics>,
}

#[derive(Debug, serde::Serialize)]
struct DiagnosticsSummary {
    n_quotes: usize,
    objective_value: f64,
    inside_bid_ask_ratio: f64,
    max_bid_ask_violation: f64,
    mean_bid_ask_violation: f64,
    mae_mid: f64,
    rmse_mid: f64,
    mae_spread_norm: f64,
    iv_total_variation: Option<f64>,
    iv_max_second_diff: Option<f64>,
    mae_iv: Option<f64>,
    rmse_iv: Option<f64>,
}

#[derive(Debug, serde::Serialize)]
struct DiagnosticsPerMaturity {
    maturity: f64,
    n_quotes: usize,
    inside_bid_ask_ratio: f64,
    max_bid_ask_violation: f64,
    mean_bid_ask_violation: f64,
    mae_mid: f64,
    rmse_mid: f64,
    mae_spread_norm: f64,
    iv_total_variation: Option<f64>,
    iv_max_second_diff: Option<f64>,
    iv_max_second_diff_strike: Option<f64>,
    mae_iv: Option<f64>,
    rmse_iv: Option<f64>,
    monotonicity_max_violation: f64,
    convexity_max_violation: f64,
    density_mass: f64,
    density_mean: f64,
    density_min: f64,
    density_max: f64,
    density_near_zero_atoms: usize,
    linear_raw_mass: Option<f64>,
    linear_raw_mean: Option<f64>,
    linear_raw_min: Option<f64>,
    linear_raw_max: Option<f64>,
    linear_projection_needed: Option<bool>,
    linear_projection_l1: Option<f64>,
    linear_projection_l2: Option<f64>,
}

#[derive(Debug, serde::Serialize)]
struct QuoteDiagnostics {
    maturity: f64,
    strike: f64,
    bid: f64,
    ask: f64,
    mid: f64,
    model: f64,
    spread: f64,
    residual_mid: f64,
    residual_spread_norm: f64,
    inside_bid_ask: bool,
    bid_iv: Option<f64>,
    ask_iv: Option<f64>,
    mid_iv: Option<f64>,
    model_iv: Option<f64>,
    residual_iv: Option<f64>,
}

#[derive(Debug, serde::Serialize)]
struct NoArbitrageDiagnostics {
    monotonicity_max_violation: f64,
    monotonicity_mean_violation: f64,
    monotonicity_violations: usize,
    convexity_max_violation: f64,
    convexity_mean_violation: f64,
    convexity_violations: usize,
    calendar_max_violation: f64,
    calendar_mean_violation: f64,
    calendar_violations: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
struct IvSmoothness {
    total_variation: Option<f64>,
    max_second_diff: Option<f64>,
    strike_at_max_second_diff: Option<f64>,
}

#[derive(Debug, serde::Serialize)]
struct SmoothnessComparison {
    baseline_total_variation: f64,
    current_total_variation: f64,
    delta_total_variation: f64,
    baseline_max_second_diff: f64,
    current_max_second_diff: f64,
    delta_max_second_diff: f64,
    per_maturity: Vec<SmoothnessComparisonPerMaturity>,
}

#[derive(Debug, serde::Serialize)]
struct SmoothnessComparisonPerMaturity {
    maturity: f64,
    baseline_total_variation: Option<f64>,
    current_total_variation: Option<f64>,
    delta_total_variation: Option<f64>,
    baseline_max_second_diff: Option<f64>,
    current_max_second_diff: Option<f64>,
    delta_max_second_diff: Option<f64>,
    baseline_strike_max_second_diff: Option<f64>,
    current_strike_max_second_diff: Option<f64>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let filter = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    match cli.command {
        Commands::ValidateSnapshot { snapshot } => {
            let snap = sanos_io::load_iv_snapshot_v1(&snapshot)?;
            // Conversion also performs numeric checks.
            let _book = sanos_io::snapshot_v1_to_option_book(&snap)?;
            info!("snapshot OK: {} maturities", snap.maturities.len());
            Ok(())
        }

        Commands::ValidateSurface { surface } => {
            let reconstructed = sanos_io::load_sanos_surface_v1(&surface)?;
            info!(
                "surface OK: {} marginals",
                reconstructed.martingale_density().marginals().len()
            );
            Ok(())
        }

        Commands::InitConfig { out } => {
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create dir: {}", parent.display()))?;
            }
            let cfg = default_calibration_config();
            fs::write(&out, serde_json::to_vec_pretty(&cfg)?)
                .with_context(|| format!("failed to write {}", out.display()))?;
            info!("wrote default config to {}", out.display());
            Ok(())
        }

        Commands::Calibrate {
            snapshot,
            config,
            out,
            n_maturities,
            n_strikes,
        } => {
            let snap = sanos_io::load_iv_snapshot_v1(&snapshot)?;
            let book = sanos_io::snapshot_v1_to_option_book(&snap)?;
            let cfg = sanos_io::load_calibration_config(&config)?;

            let run = calibrate_with_stats(&book, &cfg)
                .map_err(|e| anyhow::anyhow!("calibration failed: {e:?}"))?;
            let surface = run.surface;
            let run_stats = run.stats;

            let baseline_surface = if cfg.fit.initialization.enabled {
                let mut baseline_cfg = cfg.clone();
                baseline_cfg.fit.initialization.enabled = false;
                match calibrate_with_stats(&book, &baseline_cfg) {
                    Ok(baseline_run) => Some(baseline_run.surface),
                    Err(e) => {
                        info!("baseline (no initialization) calibration failed: {e:?}");
                        None
                    }
                }
            } else {
                None
            };

            // Basic density validation
            let tol = DensityTolerances::from_tol(1e-10)
                .map_err(|e| anyhow::anyhow!("invalid tolerance: {e:?}"))?;
            surface
                .martingale_density()
                .validate_marginals(tol)
                .map_err(|e| anyhow::anyhow!("marginal validation failed: {e:?}"))?;
            surface
                .martingale_density()
                .validate_convex_order(tol)
                .map_err(|e| anyhow::anyhow!("convex-order validation failed: {e:?}"))?;

            info!("calibration OK");

            if let Some(out_dir) = out {
                std::fs::create_dir_all(&out_dir).with_context(|| {
                    format!("failed to create output dir: {}", out_dir.display())
                })?;

                let report = build_report(&snapshot, &config, &surface)?;

                let report_path = out_dir.join("report.json");
                std::fs::write(&report_path, serde_json::to_vec_pretty(&report)?)
                    .with_context(|| format!("failed to write {}", report_path.display()))?;

                // Export q as a simple JSON (maturities + atoms)
                let q_path = out_dir.join("q.json");
                let q_json = export_q_json(surface.martingale_density());
                std::fs::write(&q_path, serde_json::to_vec_pretty(&q_json)?)
                    .with_context(|| format!("failed to write {}", q_path.display()))?;

                let surface_path = out_dir.join("surface.json");
                let surface_json = build_surface_json(
                    &snapshot,
                    &config,
                    &book,
                    &cfg,
                    &surface,
                    n_maturities,
                    n_strikes,
                )?;
                fs::write(&surface_path, serde_json::to_vec_pretty(&surface_json)?)
                    .with_context(|| format!("failed to write {}", surface_path.display()))?;

                let diagnostics_path = out_dir.join("diagnostics.json");
                let diagnostics_json =
                    build_diagnostics_json(&book, &surface, &run_stats, baseline_surface.as_ref());
                fs::write(
                    &diagnostics_path,
                    serde_json::to_vec_pretty(&diagnostics_json)?,
                )
                .with_context(|| format!("failed to write {}", diagnostics_path.display()))?;

                info!("wrote outputs to {}", out_dir.display());
            }

            Ok(())
        }
    }
}

fn default_calibration_config() -> CalibrationConfig {
    let mut fit = FitConfig::default();
    fit.objective = ObjectiveConfig::HardBidAsk;
    fit.solver = LpSolverConfig::Microlp;

    CalibrationConfig {
        backbone: BackboneConfig::BsTimeChanged(BsTimeChangedConfig::default()),
        grid: StrikeGridPolicyConfig::default(),
        fit,
        time_interp: TimeInterpConfig::default(),
    }
}

fn build_report(
    snapshot: &PathBuf,
    config: &PathBuf,
    surface: &sanos::surface::SanosSurface,
) -> Result<CalibrationReport> {
    let marginals: Vec<MarginalReport> = surface
        .martingale_density()
        .marginals()
        .iter()
        .map(|m| {
            let mass: f64 = m.atoms().iter().map(|(_, q)| *q).sum();
            let mean: f64 = m.atoms().iter().map(|(k, q)| k * q).sum();
            MarginalReport {
                maturity: m.maturity(),
                n_atoms: m.atoms().len(),
                mass,
                mean,
            }
        })
        .collect();

    Ok(CalibrationReport {
        snapshot: snapshot.display().to_string(),
        config: config.display().to_string(),
        n_maturities: marginals.len(),
        marginals,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
struct QJson {
    marginals: Vec<QJsonMarginal>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct QJsonMarginal {
    maturity: f64,
    atoms: Vec<QJsonAtom>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct QJsonAtom {
    strike: f64,
    weight: f64,
}

fn export_q_json(q: &sanos::density::MartingaleDensity) -> QJson {
    let marginals = q
        .marginals()
        .iter()
        .map(|m| QJsonMarginal {
            maturity: m.maturity(),
            atoms: m
                .atoms()
                .iter()
                .map(|(k, w)| QJsonAtom {
                    strike: *k,
                    weight: *w,
                })
                .collect(),
        })
        .collect();

    QJson { marginals }
}

fn build_surface_json(
    snapshot: &Path,
    config: &Path,
    book: &OptionBook,
    cfg: &CalibrationConfig,
    surface: &SanosSurface,
    n_maturities: usize,
    n_strikes: usize,
) -> Result<SurfaceJson> {
    let n_maturities = n_maturities.max(2);
    let n_strikes = n_strikes.max(3);

    let t_min = book
        .chains()
        .first()
        .map(|c| c.maturity())
        .context("empty OptionBook")?;
    let t_max = book
        .chains()
        .last()
        .map(|c| c.maturity())
        .context("empty OptionBook")?;

    let mut k_min = f64::INFINITY;
    let mut k_max = 0.0_f64;
    for chain in book.chains() {
        for q in chain.quotes() {
            k_min = k_min.min(q.k);
            k_max = k_max.max(q.k);
        }
    }
    if !k_min.is_finite() || !k_max.is_finite() || k_min <= 0.0 || k_max <= k_min {
        anyhow::bail!("invalid strike range for surface export: k_min={k_min}, k_max={k_max}");
    }

    let strikes = logspace(k_min * 0.95, k_max * 1.05, n_strikes);
    let maturities = linspace(t_min, t_max, n_maturities);
    let q_json = export_q_json(surface.martingale_density());

    let mut calls = Vec::with_capacity(maturities.len());
    for &t in &maturities {
        let mut row = Vec::with_capacity(strikes.len());
        for &k in &strikes {
            let c = surface
                .call(t, k)
                .map_err(|e| anyhow::anyhow!("surface call failed at T={t}, K={k}: {e:?}"))?;
            row.push(c);
        }
        calls.push(row);
    }

    let reconstruction = build_reconstruction_json(book, cfg, &q_json)?;

    Ok(SurfaceJson {
        schema: "sanos.surface.v1".to_string(),
        snapshot: snapshot.display().to_string(),
        config: config.display().to_string(),
        maturities,
        strikes,
        calls,
        marginals: q_json.marginals.clone(),
        reconstruction,
    })
}

fn build_diagnostics_json(
    book: &OptionBook,
    surface: &SanosSurface,
    run_stats: &CalibrationRunStats,
    baseline_surface: Option<&SanosSurface>,
) -> DiagnosticsJson {
    #[derive(Default)]
    struct Agg {
        n: usize,
        inside: usize,
        sum_abs_mid: f64,
        sum_sq_mid: f64,
        sum_abs_spread_norm: f64,
        sum_bid_ask_violation: f64,
        max_bid_ask_violation: f64,
        n_iv: usize,
        sum_abs_iv: f64,
        sum_sq_iv: f64,
    }

    #[derive(Debug, Clone)]
    struct MarginalDiag {
        maturity: f64,
        mass: f64,
        mean: f64,
        min: f64,
        max: f64,
        near_zero_atoms: usize,
    }

    let initialization_diags = run_stats
        .initialization
        .as_ref()
        .map(|x| x.diagnostics.as_slice())
        .unwrap_or(&[]);

    let marginal_diags: Vec<MarginalDiag> = surface
        .martingale_density()
        .marginals()
        .iter()
        .map(|m| {
            let mut mass = 0.0;
            let mut min = f64::INFINITY;
            let mut max = f64::NEG_INFINITY;
            let mut near_zero_atoms = 0usize;
            for &(_, q) in m.atoms() {
                mass += q;
                min = min.min(q);
                max = max.max(q);
                if q.abs() <= 1e-10 {
                    near_zero_atoms += 1;
                }
            }
            let mean = m.atoms().iter().map(|(k, q)| k * q).sum::<f64>();
            MarginalDiag {
                maturity: m.maturity(),
                mass,
                mean,
                min,
                max,
                near_zero_atoms,
            }
        })
        .collect();

    let find_marginal_diag = |maturity: f64| -> Option<&MarginalDiag> {
        marginal_diags
            .iter()
            .find(|d| (d.maturity - maturity).abs() <= 1e-10)
    };
    let find_init_diag = |maturity: f64| {
        initialization_diags
            .iter()
            .find(|d| (d.maturity - maturity).abs() <= 1e-10)
    };

    let mut quotes = Vec::new();
    let mut per_maturity = Vec::new();
    let mut global = Agg::default();
    let no_arb = compute_no_arbitrage_diagnostics(book, surface);

    for chain in book.chains() {
        let t = chain.maturity();
        let mut agg = Agg::default();
        let mut k_chain = Vec::with_capacity(chain.quotes().len());
        let mut model_iv_chain = Vec::with_capacity(chain.quotes().len());
        let mut model_price_chain = Vec::with_capacity(chain.quotes().len());

        for q in chain.quotes() {
            let model = surface.call(t, q.k).unwrap_or(f64::NAN);
            let spread = (q.ask - q.bid).max(1e-12);
            let mid = q.mid();
            let residual_mid = model - mid;
            let residual_spread_norm = residual_mid / (0.5 * spread);
            let lo_viol = (q.bid - model).max(0.0);
            let hi_viol = (model - q.ask).max(0.0);
            let bid_ask_violation = lo_viol.max(hi_viol);
            let inside_bid_ask = bid_ask_violation <= 1e-12;
            let bid_iv = implied_iv_or_none(q.bid, q.k, t);
            let ask_iv = implied_iv_or_none(q.ask, q.k, t);
            let mid_iv = implied_iv_or_none(mid, q.k, t);
            let model_iv = implied_iv_or_none(model, q.k, t);
            let residual_iv = match (model_iv, mid_iv) {
                (Some(m), Some(v)) => Some(m - v),
                _ => None,
            };

            quotes.push(QuoteDiagnostics {
                maturity: t,
                strike: q.k,
                bid: q.bid,
                ask: q.ask,
                mid,
                model,
                spread,
                residual_mid,
                residual_spread_norm,
                inside_bid_ask,
                bid_iv,
                ask_iv,
                mid_iv,
                model_iv,
                residual_iv,
            });

            k_chain.push(q.k);
            model_iv_chain.push(model_iv);
            model_price_chain.push(model);

            agg.n += 1;
            global.n += 1;
            if inside_bid_ask {
                agg.inside += 1;
                global.inside += 1;
            }
            agg.sum_bid_ask_violation += bid_ask_violation;
            global.sum_bid_ask_violation += bid_ask_violation;
            agg.max_bid_ask_violation = agg.max_bid_ask_violation.max(bid_ask_violation);
            global.max_bid_ask_violation = global.max_bid_ask_violation.max(bid_ask_violation);

            let abs_mid = residual_mid.abs();
            agg.sum_abs_mid += abs_mid;
            global.sum_abs_mid += abs_mid;
            agg.sum_sq_mid += residual_mid * residual_mid;
            global.sum_sq_mid += residual_mid * residual_mid;
            let abs_spread = residual_spread_norm.abs();
            agg.sum_abs_spread_norm += abs_spread;
            global.sum_abs_spread_norm += abs_spread;
            if let Some(riv) = residual_iv {
                agg.n_iv += 1;
                global.n_iv += 1;
                agg.sum_abs_iv += riv.abs();
                global.sum_abs_iv += riv.abs();
                agg.sum_sq_iv += riv * riv;
                global.sum_sq_iv += riv * riv;
            }
        }

        let smooth = iv_smoothness(&k_chain, &model_iv_chain);
        let (mono_max, conv_max) = chain_strike_no_arb(&k_chain, &model_price_chain);

        let density = find_marginal_diag(t);
        let init_diag = find_init_diag(t);

        per_maturity.push(DiagnosticsPerMaturity {
            maturity: t,
            n_quotes: agg.n,
            inside_bid_ask_ratio: ratio(agg.inside, agg.n),
            max_bid_ask_violation: agg.max_bid_ask_violation,
            mean_bid_ask_violation: safe_div(agg.sum_bid_ask_violation, agg.n as f64),
            mae_mid: safe_div(agg.sum_abs_mid, agg.n as f64),
            rmse_mid: safe_div(agg.sum_sq_mid, agg.n as f64).sqrt(),
            mae_spread_norm: safe_div(agg.sum_abs_spread_norm, agg.n as f64),
            iv_total_variation: smooth.total_variation,
            iv_max_second_diff: smooth.max_second_diff,
            iv_max_second_diff_strike: smooth.strike_at_max_second_diff,
            mae_iv: safe_metric(agg.sum_abs_iv, agg.n_iv),
            rmse_iv: safe_metric(agg.sum_sq_iv, agg.n_iv).map(f64::sqrt),
            monotonicity_max_violation: mono_max,
            convexity_max_violation: conv_max,
            density_mass: density.map(|d| d.mass).unwrap_or(0.0),
            density_mean: density.map(|d| d.mean).unwrap_or(0.0),
            density_min: density.map(|d| d.min).unwrap_or(0.0),
            density_max: density.map(|d| d.max).unwrap_or(0.0),
            density_near_zero_atoms: density.map(|d| d.near_zero_atoms).unwrap_or(0),
            linear_raw_mass: init_diag.map(|d| d.raw_mass),
            linear_raw_mean: init_diag.map(|d| d.raw_mean),
            linear_raw_min: init_diag.map(|d| d.raw_min),
            linear_raw_max: init_diag.map(|d| d.raw_max),
            linear_projection_needed: init_diag.map(|d| d.projection_needed),
            linear_projection_l1: init_diag.map(|d| d.l1_distance),
            linear_projection_l2: init_diag.map(|d| d.l2_distance),
        });
    }

    quotes.sort_by(|a, b| {
        a.maturity
            .partial_cmp(&b.maturity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.strike
                    .partial_cmp(&b.strike)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    per_maturity.sort_by(|a, b| {
        a.maturity
            .partial_cmp(&b.maturity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let iv_total_variation = per_maturity
        .iter()
        .filter_map(|x| x.iv_total_variation)
        .sum::<f64>();
    let iv_total_variation = if per_maturity.iter().any(|x| x.iv_total_variation.is_some()) {
        Some(iv_total_variation)
    } else {
        None
    };
    let iv_max_second_diff = per_maturity
        .iter()
        .filter_map(|x| x.iv_max_second_diff)
        .reduce(f64::max);

    let smoothness_comparison = build_smoothness_comparison(book, &per_maturity, baseline_surface);

    DiagnosticsJson {
        schema: "sanos.calibration_diagnostics.v2".to_string(),
        summary: DiagnosticsSummary {
            n_quotes: global.n,
            objective_value: run_stats.objective_value,
            inside_bid_ask_ratio: ratio(global.inside, global.n),
            max_bid_ask_violation: global.max_bid_ask_violation,
            mean_bid_ask_violation: safe_div(global.sum_bid_ask_violation, global.n as f64),
            mae_mid: safe_div(global.sum_abs_mid, global.n as f64),
            rmse_mid: safe_div(global.sum_sq_mid, global.n as f64).sqrt(),
            mae_spread_norm: safe_div(global.sum_abs_spread_norm, global.n as f64),
            iv_total_variation,
            iv_max_second_diff,
            mae_iv: safe_metric(global.sum_abs_iv, global.n_iv),
            rmse_iv: safe_metric(global.sum_sq_iv, global.n_iv).map(f64::sqrt),
        },
        no_arbitrage: no_arb,
        smoothness_comparison,
        per_maturity,
        quotes,
    }
}

fn build_smoothness_comparison(
    book: &OptionBook,
    per_maturity: &[DiagnosticsPerMaturity],
    baseline_surface: Option<&SanosSurface>,
) -> Option<SmoothnessComparison> {
    let baseline_surface = baseline_surface?;
    let baseline = iv_smoothness_by_maturity(book, baseline_surface);

    let baseline_total_variation = baseline
        .iter()
        .filter_map(|(_, s)| s.total_variation)
        .sum::<f64>();
    let current_total_variation = per_maturity
        .iter()
        .filter_map(|s| s.iv_total_variation)
        .sum::<f64>();
    let baseline_max_second_diff = baseline
        .iter()
        .filter_map(|(_, s)| s.max_second_diff)
        .reduce(f64::max)
        .unwrap_or(0.0);
    let current_max_second_diff = per_maturity
        .iter()
        .filter_map(|s| s.iv_max_second_diff)
        .reduce(f64::max)
        .unwrap_or(0.0);

    let mut per_maturity_cmp = Vec::with_capacity(per_maturity.len());
    for cur in per_maturity {
        let b = baseline
            .iter()
            .find(|(t, _)| (*t - cur.maturity).abs() <= 1e-10)
            .map(|(_, s)| s);

        let delta_total_variation =
            match (cur.iv_total_variation, b.and_then(|x| x.total_variation)) {
                (Some(c), Some(v)) => Some(c - v),
                _ => None,
            };
        let delta_max_second_diff =
            match (cur.iv_max_second_diff, b.and_then(|x| x.max_second_diff)) {
                (Some(c), Some(v)) => Some(c - v),
                _ => None,
            };

        per_maturity_cmp.push(SmoothnessComparisonPerMaturity {
            maturity: cur.maturity,
            baseline_total_variation: b.and_then(|x| x.total_variation),
            current_total_variation: cur.iv_total_variation,
            delta_total_variation,
            baseline_max_second_diff: b.and_then(|x| x.max_second_diff),
            current_max_second_diff: cur.iv_max_second_diff,
            delta_max_second_diff,
            baseline_strike_max_second_diff: b.and_then(|x| x.strike_at_max_second_diff),
            current_strike_max_second_diff: cur.iv_max_second_diff_strike,
        });
    }

    Some(SmoothnessComparison {
        baseline_total_variation,
        current_total_variation,
        delta_total_variation: current_total_variation - baseline_total_variation,
        baseline_max_second_diff,
        current_max_second_diff,
        delta_max_second_diff: current_max_second_diff - baseline_max_second_diff,
        per_maturity: per_maturity_cmp,
    })
}

fn iv_smoothness_by_maturity(
    book: &OptionBook,
    surface: &SanosSurface,
) -> Vec<(f64, IvSmoothness)> {
    let mut out = Vec::with_capacity(book.len());
    for chain in book.chains() {
        let t = chain.maturity();
        let strikes: Vec<f64> = chain.quotes().iter().map(|q| q.k).collect();
        let model_ivs: Vec<Option<f64>> = chain
            .quotes()
            .iter()
            .map(|q| {
                surface
                    .call(t, q.k)
                    .ok()
                    .and_then(|c| implied_iv_or_none(c, q.k, t))
            })
            .collect();
        out.push((t, iv_smoothness(&strikes, &model_ivs)));
    }
    out
}

fn iv_smoothness(strikes: &[f64], iv: &[Option<f64>]) -> IvSmoothness {
    if strikes.len() != iv.len() {
        return IvSmoothness {
            total_variation: None,
            max_second_diff: None,
            strike_at_max_second_diff: None,
        };
    }

    let mut kv = Vec::with_capacity(strikes.len());
    for i in 0..strikes.len() {
        if let Some(v) = iv[i] {
            if v.is_finite() && strikes[i].is_finite() {
                kv.push((strikes[i], v));
            }
        }
    }
    if kv.len() < 2 {
        return IvSmoothness {
            total_variation: None,
            max_second_diff: None,
            strike_at_max_second_diff: None,
        };
    }
    kv.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut total_variation = 0.0;
    for w in kv.windows(2) {
        total_variation += (w[1].1 - w[0].1).abs();
    }

    if kv.len() < 3 {
        return IvSmoothness {
            total_variation: Some(total_variation),
            max_second_diff: None,
            strike_at_max_second_diff: None,
        };
    }

    let mut max_second = 0.0;
    let mut strike_at_max = None;
    for i in 1..(kv.len() - 1) {
        let (k0, v0) = kv[i - 1];
        let (k1, v1) = kv[i];
        let (k2, v2) = kv[i + 1];
        let dk0 = k1 - k0;
        let dk1 = k2 - k1;
        if dk0 <= 0.0 || dk1 <= 0.0 {
            continue;
        }
        let s0 = (v1 - v0) / dk0;
        let s1 = (v2 - v1) / dk1;
        let second = (2.0 * (s1 - s0) / (k2 - k0)).abs();
        if second > max_second {
            max_second = second;
            strike_at_max = Some(k1);
        }
    }

    IvSmoothness {
        total_variation: Some(total_variation),
        max_second_diff: Some(max_second),
        strike_at_max_second_diff: strike_at_max,
    }
}

fn chain_strike_no_arb(strikes: &[f64], calls: &[f64]) -> (f64, f64) {
    if strikes.len() != calls.len() || strikes.len() < 2 {
        return (0.0, 0.0);
    }

    let mut mono_max = 0.0_f64;
    for i in 0..(calls.len() - 1) {
        if calls[i].is_finite() && calls[i + 1].is_finite() {
            mono_max = mono_max.max((calls[i + 1] - calls[i]).max(0.0));
        }
    }

    let mut conv_max = 0.0_f64;
    if calls.len() >= 3 {
        for i in 1..(calls.len() - 1) {
            if !calls[i - 1].is_finite() || !calls[i].is_finite() || !calls[i + 1].is_finite() {
                continue;
            }
            let dk0 = strikes[i] - strikes[i - 1];
            let dk1 = strikes[i + 1] - strikes[i];
            if dk0 <= 0.0 || dk1 <= 0.0 {
                continue;
            }
            let s0 = (calls[i] - calls[i - 1]) / dk0;
            let s1 = (calls[i + 1] - calls[i]) / dk1;
            conv_max = conv_max.max((s0 - s1).max(0.0));
        }
    }

    (mono_max, conv_max)
}

fn compute_no_arbitrage_diagnostics(
    book: &OptionBook,
    surface: &SanosSurface,
) -> NoArbitrageDiagnostics {
    let mut mono_sum = 0.0_f64;
    let mut mono_max = 0.0_f64;
    let mut mono_n = 0usize;
    let mut mono_violations = 0usize;

    let mut conv_sum = 0.0_f64;
    let mut conv_max = 0.0_f64;
    let mut conv_n = 0usize;
    let mut conv_violations = 0usize;

    for chain in book.chains() {
        let t = chain.maturity();
        let strikes: Vec<f64> = chain.quotes().iter().map(|q| q.k).collect();
        let calls: Vec<f64> = chain
            .quotes()
            .iter()
            .map(|q| surface.call(t, q.k).unwrap_or(f64::NAN))
            .collect();

        for i in 0..calls.len().saturating_sub(1) {
            if calls[i].is_finite() && calls[i + 1].is_finite() {
                let v = (calls[i + 1] - calls[i]).max(0.0);
                mono_sum += v;
                mono_max = mono_max.max(v);
                mono_n += 1;
                if v > 1e-12 {
                    mono_violations += 1;
                }
            }
        }

        for i in 1..calls.len().saturating_sub(1) {
            if !calls[i - 1].is_finite() || !calls[i].is_finite() || !calls[i + 1].is_finite() {
                continue;
            }
            let dk0 = strikes[i] - strikes[i - 1];
            let dk1 = strikes[i + 1] - strikes[i];
            if dk0 <= 0.0 || dk1 <= 0.0 {
                continue;
            }
            let s0 = (calls[i] - calls[i - 1]) / dk0;
            let s1 = (calls[i + 1] - calls[i]) / dk1;
            let v = (s0 - s1).max(0.0);
            conv_sum += v;
            conv_max = conv_max.max(v);
            conv_n += 1;
            if v > 1e-12 {
                conv_violations += 1;
            }
        }
    }

    let mut strikes_all: Vec<f64> = book
        .chains()
        .iter()
        .flat_map(|c| c.quotes().iter().map(|q| q.k))
        .collect();
    strikes_all.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    strikes_all.dedup_by(|a, b| (*a - *b).abs() <= 1e-12);

    let mut cal_sum = 0.0_f64;
    let mut cal_max = 0.0_f64;
    let mut cal_n = 0usize;
    let mut cal_violations = 0usize;

    for pair in book.chains().windows(2) {
        let t0 = pair[0].maturity();
        let t1 = pair[1].maturity();
        for &k in &strikes_all {
            let c0 = surface.call(t0, k).unwrap_or(f64::NAN);
            let c1 = surface.call(t1, k).unwrap_or(f64::NAN);
            if c0.is_finite() && c1.is_finite() {
                let v = (c0 - c1).max(0.0);
                cal_sum += v;
                cal_max = cal_max.max(v);
                cal_n += 1;
                if v > 1e-12 {
                    cal_violations += 1;
                }
            }
        }
    }

    NoArbitrageDiagnostics {
        monotonicity_max_violation: mono_max,
        monotonicity_mean_violation: safe_div(mono_sum, mono_n as f64),
        monotonicity_violations: mono_violations,
        convexity_max_violation: conv_max,
        convexity_mean_violation: safe_div(conv_sum, conv_n as f64),
        convexity_violations: conv_violations,
        calendar_max_violation: cal_max,
        calendar_mean_violation: safe_div(cal_sum, cal_n as f64),
        calendar_violations: cal_violations,
    }
}

fn build_reconstruction_json(
    book: &OptionBook,
    cfg: &CalibrationConfig,
    q_json: &QJson,
) -> Result<SurfaceReconstructionJson> {
    let backbone = match &cfg.backbone {
        BackboneConfig::BsTimeChanged(bs_cfg) => {
            let model = build_time_changed_lognormal_from_book(book, bs_cfg)
                .map_err(|e| anyhow::anyhow!("failed to reconstruct backbone knots: {e:?}"))?;
            let var_curve_knots = model
                .var_curve_knots()
                .iter()
                .map(|(t, w)| VarianceKnotJson {
                    maturity: *t,
                    total_variance: *w,
                })
                .collect();
            ReconstructionBackboneJson {
                model: "bs_time_changed_lognormal".to_string(),
                eta: model.var_scale(),
                var_curve_knots,
            }
        }
    };

    Ok(SurfaceReconstructionJson {
        backbone,
        time_interp: cfg.time_interp,
        marginals: q_json.marginals.clone(),
    })
}

fn implied_iv_or_none(call: f64, strike: f64, maturity: f64) -> Option<f64> {
    bs_implied_vol_from_call(call, 1.0, strike, maturity)
        .ok()
        .filter(|v| v.is_finite())
}

#[inline]
fn ratio(num: usize, den: usize) -> f64 {
    if den == 0 {
        0.0
    } else {
        num as f64 / den as f64
    }
}

#[inline]
fn safe_div(num: f64, den: f64) -> f64 {
    if den == 0.0 {
        0.0
    } else {
        num / den
    }
}

#[inline]
fn safe_metric(num: f64, den: usize) -> Option<f64> {
    if den == 0 {
        None
    } else {
        Some(num / den as f64)
    }
}

fn linspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    if n <= 1 {
        return vec![start];
    }
    let step = (end - start) / (n as f64 - 1.0);
    (0..n).map(|i| start + step * i as f64).collect()
}

fn logspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    let l0 = start.ln();
    let l1 = end.ln();
    linspace(l0, l1, n).into_iter().map(f64::exp).collect()
}
