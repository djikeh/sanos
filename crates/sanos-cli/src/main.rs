use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

use sanos::density::DensityTolerances;
use sanos::calibration::calibrate;

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
    },

    /// Validate an IV snapshot (schema + basic numeric checks)
    ValidateSnapshot {
        #[arg(long)]
        snapshot: PathBuf,
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

        Commands::Calibrate { snapshot, config, out } => {
            let snap = sanos_io::load_iv_snapshot_v1(&snapshot)?;
            let book = sanos_io::snapshot_v1_to_option_book(&snap)?;
            let cfg = sanos_io::load_calibration_config(&config)?;

            let surface = calibrate(&book, &cfg)
                .map_err(|e| anyhow::anyhow!("calibration failed: {e:?}"))?;

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
                std::fs::create_dir_all(&out_dir)
                    .with_context(|| format!("failed to create output dir: {}", out_dir.display()))?;

                let report = build_report(&snapshot, &config, &surface)?;

                let report_path = out_dir.join("report.json");
                std::fs::write(&report_path, serde_json::to_vec_pretty(&report)?)
                    .with_context(|| format!("failed to write {}", report_path.display()))?;

                // Export q as a simple JSON (maturities + atoms)
                let q_path = out_dir.join("q.json");
                let q_json = export_q_json(surface.martingale_density());
                std::fs::write(&q_path, serde_json::to_vec_pretty(&q_json)?)
                    .with_context(|| format!("failed to write {}", q_path.display()))?;

                info!("wrote outputs to {}", out_dir.display());
            }

            Ok(())
        }
    }
}

fn build_report(snapshot: &PathBuf, config: &PathBuf, surface: &sanos::surface::SanosSurface) -> Result<CalibrationReport> {
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

#[derive(Debug, serde::Serialize)]
struct QJson {
    marginals: Vec<QJsonMarginal>,
}

#[derive(Debug, serde::Serialize)]
struct QJsonMarginal {
    maturity: f64,
    atoms: Vec<QJsonAtom>,
}

#[derive(Debug, serde::Serialize)]
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
