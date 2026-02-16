//! I/O utilities for SANOS.
//!
//! This crate keeps JSON parsing and filesystem interaction out of the core `sanos` crate.
//! It provides:
//! - loading the example IV surface snapshot format (v1)
//! - loading calibration configs as JSON (using `sanos`'s config types)
//! - converting a snapshot into an `OptionBook` (call bid/ask prices)

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

use sanos::backbone::bs::bs_call_forward_norm;
use sanos::calibration::CalibrationConfig;
use sanos::market::{CallQuote, OptionBook, OptionChain};
use sanos_schema::v1::{
    IvQuoteV1, IvSurfaceSnapshotV1, MaturityNodeV1, SnapshotConventionsV1, IV_SNAPSHOT_SCHEMA,
};

/// Load an IV surface snapshot (schema v1) from a JSON file.
pub fn load_iv_snapshot_v1<P: AsRef<Path>>(path: P) -> Result<IvSurfaceSnapshotV1> {
    let path_ref = path.as_ref();
    let raw = fs::read_to_string(path_ref)
        .with_context(|| format!("failed to read snapshot file: {}", path_ref.display()))?;

    let compat: SnapshotCompat = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse snapshot JSON: {}", path_ref.display()))?;
    let snap = compat.into_strict()?;

    // Light schema check (optional field).
    if let Some(schema) = &snap.schema {
        if schema != IV_SNAPSHOT_SCHEMA {
            bail!("unexpected snapshot schema: expected {IV_SNAPSHOT_SCHEMA}, got {schema}");
        }
    }

    Ok(snap)
}

#[derive(Debug, Clone, Deserialize)]
struct SnapshotCompat {
    #[serde(default)]
    schema: Option<String>,
    #[serde(default)]
    schema_name: Option<String>,
    #[serde(default)]
    as_of: Option<String>,
    #[serde(default)]
    conventions: Option<SnapshotConventionsCompat>,
    maturities: Vec<MaturityNodeCompat>,
}

#[derive(Debug, Clone, Deserialize)]
struct SnapshotConventionsCompat {
    #[serde(default = "default_one")]
    forward: f64,
    #[serde(default)]
    r: f64,
    #[serde(default)]
    q: f64,
    #[serde(default)]
    strike_convention: Option<String>,
    #[serde(default)]
    strike_unit: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct MaturityNodeCompat {
    t: f64,
    quotes: Vec<IvQuoteCompat>,
}

#[derive(Debug, Clone, Deserialize)]
struct IvQuoteCompat {
    k: f64,
    bid_iv: f64,
    ask_iv: f64,
}

impl SnapshotCompat {
    fn into_strict(self) -> Result<IvSurfaceSnapshotV1> {
        let schema = match (self.schema, self.schema_name) {
            (Some(schema), _) => Some(schema),
            (None, Some(schema_name)) if schema_name == "iv_surface_snapshot" => {
                Some(IV_SNAPSHOT_SCHEMA.to_string())
            }
            (None, Some(other)) => bail!("unexpected snapshot schema_name: {other}"),
            (None, None) => None,
        };

        let conventions = self
            .conventions
            .unwrap_or_else(|| SnapshotConventionsCompat {
                forward: default_one(),
                r: 0.0,
                q: 0.0,
                strike_convention: None,
                strike_unit: None,
            });
        let strike_convention = conventions
            .strike_convention
            .or(conventions.strike_unit)
            .unwrap_or_else(|| "forward_moneyness".to_string());

        let maturities = self
            .maturities
            .into_iter()
            .map(|m| MaturityNodeV1 {
                t: m.t,
                quotes: m
                    .quotes
                    .into_iter()
                    .map(|q| IvQuoteV1 {
                        k: q.k,
                        bid_iv: q.bid_iv,
                        ask_iv: q.ask_iv,
                    })
                    .collect(),
            })
            .collect();

        Ok(IvSurfaceSnapshotV1 {
            schema,
            as_of: self.as_of,
            conventions: Some(SnapshotConventionsV1 {
                forward: conventions.forward,
                r: conventions.r,
                q: conventions.q,
                strike_convention,
            }),
            maturities,
        })
    }
}

fn default_one() -> f64 {
    1.0
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

            let quote = CallQuote::new(strike, bid, ask, conv.forward)
                .with_context(|| format!("failed to build CallQuote (t={}, k={})", node.t, q.k))?;
            quotes.push(quote);
        }

        let chain = OptionChain::new(node.t, quotes)
            .with_context(|| format!("failed to build OptionChain at t={}", node.t))?;
        chains.push(chain);
    }

    OptionBook::new(chains).context("failed to build OptionBook")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compat_loader_accepts_legacy_schema_name_and_strike_unit() {
        let raw = r#"
        {
          "schema_name": "iv_surface_snapshot",
          "schema_version": "1.0.0",
          "scenario_id": "legacy_case",
          "as_of": "2026-01-01T00:00:00Z",
          "conventions": {
            "time_unit": "year_fraction_act365f",
            "strike_unit": "forward_moneyness",
            "quote_type": "implied_vol"
          },
          "maturities": [
            {
              "t": 0.25,
              "quotes": [
                {"k": 1.0, "bid_iv": 0.20, "ask_iv": 0.21, "mid_iv": 0.205}
              ]
            }
          ],
          "generator": {"name": "x"}
        }
        "#;

        let parsed: SnapshotCompat = serde_json::from_str(raw).expect("compat parse");
        let snap = parsed.into_strict().expect("strict snapshot");
        assert_eq!(snap.schema.as_deref(), Some(IV_SNAPSHOT_SCHEMA));
        assert_eq!(
            snap.conventions
                .as_ref()
                .map(|c| c.strike_convention.as_str()),
            Some("forward_moneyness")
        );
        assert_eq!(snap.maturities.len(), 1);
        assert_eq!(snap.maturities[0].quotes.len(), 1);
    }
}
