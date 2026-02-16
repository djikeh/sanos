//! I/O utilities for SANOS.
//!
//! This crate keeps JSON parsing and filesystem interaction out of the core `sanos` crate.
//! It provides:
//! - loading the example IV surface snapshot format (v1)
//! - loading calibration configs as JSON (using `sanos`'s config types)
//! - converting a snapshot into an `OptionBook` (call bid/ask prices)

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

use sanos::backbone::bs::bs_call_forward_norm;
use sanos::calibration::CalibrationConfig;
use sanos::market::{CallQuote, OptionBook, OptionChain};
use sanos_schema::v1::{IvSurfaceSnapshotV1, IV_SNAPSHOT_SCHEMA};

/// Load an IV surface snapshot (schema v1) from a JSON file.
pub fn load_iv_snapshot_v1<P: AsRef<Path>>(path: P) -> Result<IvSurfaceSnapshotV1> {
    let path_ref = path.as_ref();
    let raw = fs::read_to_string(path_ref)
        .with_context(|| format!("failed to read snapshot file: {}", path_ref.display()))?;

    let snap: IvSurfaceSnapshotV1 = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse snapshot JSON: {}", path_ref.display()))?;

    // Light schema check (optional field).
    if let Some(schema) = &snap.schema {
        if schema != IV_SNAPSHOT_SCHEMA {
            bail!("unexpected snapshot schema: expected {IV_SNAPSHOT_SCHEMA}, got {schema}");
        }
    }

    Ok(snap)
}

/// Load a SANOS calibration config from a JSON file.
///
/// This deserializes the runtime `sanos::calibration::CalibrationConfig` directly.
/// Ensure the core crate is compiled with the `serde` feature (enabled by the CLI by default).
pub fn load_calibration_config<P: AsRef<Path>>(path: P) -> Result<CalibrationConfig> {
    let path_ref = path.as_ref();
    let raw = fs::read_to_string(path_ref)
        .with_context(|| format!("failed to read config file: {}", path_ref.display()))?;

    let cfg: CalibrationConfig = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse config JSON: {}", path_ref.display()))?;

    Ok(cfg)
}

/// Convert a v1 IV snapshot into an `OptionBook` of call bid/ask prices.
///
/// Assumptions:
/// - Forward is constant and equal to `conventions.forward` (default 1.0).
/// - Strikes are forward-moneyness: K = k * F.
/// - Black-76 forward-normalized pricing is used:
///   - total variance w = iv^2 * T
///   - price = call_forward_norm(k, w) * F
pub fn snapshot_v1_to_option_book(snapshot: &IvSurfaceSnapshotV1) -> Result<OptionBook> {
    let conv = snapshot.conventions.clone().unwrap_or_default();

    if conv.strike_convention != "forward_moneyness" {
        bail!(
            "unsupported strike_convention: {} (expected 'forward_moneyness')",
            conv.strike_convention
        );
    }
    if !conv.forward.is_finite() || conv.forward <= 0.0 {
        bail!("invalid forward convention: {}", conv.forward);
    }
    if conv.r != 0.0 || conv.q != 0.0 {
        bail!(
            "only r=0.0 and q=0.0 are supported for now; got r={}, q={}",
            conv.r,
            conv.q
        );
    }

    let mut chains = Vec::with_capacity(snapshot.maturities.len());

    for node in &snapshot.maturities {
        if !node.t.is_finite() || node.t <= 0.0 {
            bail!("invalid maturity t: {}", node.t);
        }

        let mut quotes = Vec::with_capacity(node.quotes.len());
        for q in &node.quotes {
            if !q.k.is_finite() || q.k <= 0.0 {
                bail!("invalid strike k: {}", q.k);
            }
            if !q.bid_iv.is_finite() || q.bid_iv <= 0.0 {
                bail!("invalid bid_iv: {}", q.bid_iv);
            }
            if !q.ask_iv.is_finite() || q.ask_iv <= 0.0 {
                bail!("invalid ask_iv: {}", q.ask_iv);
            }
            if q.bid_iv > q.ask_iv {
                bail!("bid_iv > ask_iv at t={}, k={}", node.t, q.k);
            }

            let w_bid = q.bid_iv * q.bid_iv * node.t;
            let w_ask = q.ask_iv * q.ask_iv * node.t;

            let c_bid_norm = bs_call_forward_norm(q.k, w_bid).map_err(|err| {
                anyhow::anyhow!(
                    "failed pricing bid: t={}, k={}, w={}: {:?}",
                    node.t,
                    q.k,
                    w_bid,
                    err
                )
            })?;
            let c_ask_norm = bs_call_forward_norm(q.k, w_ask).map_err(|err| {
                anyhow::anyhow!(
                    "failed pricing ask: t={}, k={}, w={}: {:?}",
                    node.t,
                    q.k,
                    w_ask,
                    err
                )
            })?;

            let bid = c_bid_norm * conv.forward;
            let ask = c_ask_norm * conv.forward;
            let strike = q.k * conv.forward;

            let quote = CallQuote::new(strike, bid, ask, conv.forward).with_context(|| {
                format!("failed to build CallQuote (t={}, k={})", node.t, q.k)
            })?;
            quotes.push(quote);
        }

        let chain = OptionChain::new(node.t, quotes)
            .with_context(|| format!("failed to build OptionChain at t={}", node.t))?;
        chains.push(chain);
    }

    OptionBook::new(chains).context("failed to build OptionBook")
}
