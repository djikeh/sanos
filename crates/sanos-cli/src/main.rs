use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

use sanos::backbone::{
    bs_implied_vol_from_call, build_time_changed_lognormal_from_book, BackboneConfig,
    BsTimeChangedConfig,
};
use sanos::calibration::{calibrate, CalibrationConfig};
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
    per_maturity: Vec<DiagnosticsPerMaturity>,
    quotes: Vec<QuoteDiagnostics>,
}

#[derive(Debug, serde::Serialize)]
struct DiagnosticsSummary {
    n_quotes: usize,
    inside_bid_ask_ratio: f64,
    mae_mid: f64,
    rmse_mid: f64,
    mae_spread_norm: f64,
    mae_iv: Option<f64>,
    rmse_iv: Option<f64>,
}

#[derive(Debug, serde::Serialize)]
struct DiagnosticsPerMaturity {
    maturity: f64,
    n_quotes: usize,
    inside_bid_ask_ratio: f64,
    mae_mid: f64,
    rmse_mid: f64,
    mae_spread_norm: f64,
    mae_iv: Option<f64>,
    rmse_iv: Option<f64>,
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

            let surface =
                calibrate(&book, &cfg).map_err(|e| anyhow::anyhow!("calibration failed: {e:?}"))?;

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
                let diagnostics_json = build_diagnostics_json(&book, &surface);
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

fn build_diagnostics_json(book: &OptionBook, surface: &SanosSurface) -> DiagnosticsJson {
    #[derive(Default)]
    struct Agg {
        n: usize,
        inside: usize,
        sum_abs_mid: f64,
        sum_sq_mid: f64,
        sum_abs_spread_norm: f64,
        n_iv: usize,
        sum_abs_iv: f64,
        sum_sq_iv: f64,
    }

    let mut quotes = Vec::new();
    let mut per_maturity = Vec::new();
    let mut global = Agg::default();

    for chain in book.chains() {
        let t = chain.maturity();
        let mut agg = Agg::default();

        for q in chain.quotes() {
            let model = surface.call(t, q.k).unwrap_or(f64::NAN);
            let spread = (q.ask - q.bid).max(1e-12);
            let mid = q.mid();
            let residual_mid = model - mid;
            let residual_spread_norm = residual_mid / (0.5 * spread);
            let inside_bid_ask = model >= q.bid - 1e-12 && model <= q.ask + 1e-12;
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

            agg.n += 1;
            global.n += 1;
            if inside_bid_ask {
                agg.inside += 1;
                global.inside += 1;
            }
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

        per_maturity.push(DiagnosticsPerMaturity {
            maturity: t,
            n_quotes: agg.n,
            inside_bid_ask_ratio: ratio(agg.inside, agg.n),
            mae_mid: safe_div(agg.sum_abs_mid, agg.n as f64),
            rmse_mid: safe_div(agg.sum_sq_mid, agg.n as f64).sqrt(),
            mae_spread_norm: safe_div(agg.sum_abs_spread_norm, agg.n as f64),
            mae_iv: safe_metric(agg.sum_abs_iv, agg.n_iv),
            rmse_iv: safe_metric(agg.sum_sq_iv, agg.n_iv).map(f64::sqrt),
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

    DiagnosticsJson {
        schema: "sanos.calibration_diagnostics.v1".to_string(),
        summary: DiagnosticsSummary {
            n_quotes: global.n,
            inside_bid_ask_ratio: ratio(global.inside, global.n),
            mae_mid: safe_div(global.sum_abs_mid, global.n as f64),
            rmse_mid: safe_div(global.sum_sq_mid, global.n as f64).sqrt(),
            mae_spread_norm: safe_div(global.sum_abs_spread_norm, global.n as f64),
            mae_iv: safe_metric(global.sum_abs_iv, global.n_iv),
            rmse_iv: safe_metric(global.sum_sq_iv, global.n_iv).map(f64::sqrt),
        },
        per_maturity,
        quotes,
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
