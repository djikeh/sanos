use serde::{Deserialize, Serialize};

/// Schema identifier for implied-vol surface snapshots (v1).
pub const IV_SNAPSHOT_SCHEMA: &str = "sanos.iv_surface.v1";

/// A simple implied-vol surface snapshot in forward normalization.
///
/// Conventions (assumed unless stated otherwise in `conventions`):
/// - forward F = 1.0
/// - rates r = 0.0, q = 0.0
/// - strikes are forward-moneyness: K = k * F, so K = k
/// - maturities are year fractions (ACT/365F in the provided example)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IvSurfaceSnapshotV1 {
    /// Optional schema name, useful for validation and forward compatibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// Optional free-form metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_of: Option<String>,

    /// Optional conventions description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conventions: Option<SnapshotConventionsV1>,

    pub maturities: Vec<MaturityNodeV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SnapshotConventionsV1 {
    /// Forward level (default: 1.0)
    #[serde(default = "default_one")]
    pub forward: f64,

    /// Risk-free rate (default: 0.0)
    #[serde(default)]
    pub r: f64,

    /// Dividend yield / funding spread (default: 0.0)
    #[serde(default)]
    pub q: f64,

    /// Strike convention. In v1 we only support forward-moneyness.
    #[serde(default = "default_strike_convention")]
    pub strike_convention: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MaturityNodeV1 {
    pub t: f64,
    pub quotes: Vec<IvQuoteV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IvQuoteV1 {
    /// Strike in forward-moneyness (K = k * F).
    pub k: f64,

    pub bid_iv: f64,
    pub ask_iv: f64,
}

fn default_one() -> f64 {
    1.0
}

fn default_strike_convention() -> String {
    "forward_moneyness".to_string()
}

impl Default for SnapshotConventionsV1 {
    fn default() -> Self {
        Self {
            forward: 1.0,
            r: 0.0,
            q: 0.0,
            strike_convention: default_strike_convention(),
        }
    }
}

impl Default for IvSurfaceSnapshotV1 {
    fn default() -> Self {
        Self {
            schema: Some(IV_SNAPSHOT_SCHEMA.to_string()),
            as_of: None,
            conventions: Some(SnapshotConventionsV1::default()),
            maturities: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_default_sets_expected_schema_and_conventions() {
        let snap = IvSurfaceSnapshotV1::default();
        assert_eq!(snap.schema.as_deref(), Some(IV_SNAPSHOT_SCHEMA));
        let conv = snap.conventions.unwrap();
        assert_eq!(conv.forward, 1.0);
        assert_eq!(conv.r, 0.0);
        assert_eq!(conv.q, 0.0);
        assert_eq!(conv.strike_convention, "forward_moneyness");
    }

    #[test]
    fn snapshot_roundtrip_json_preserves_payload() {
        let snap = IvSurfaceSnapshotV1 {
            schema: Some(IV_SNAPSHOT_SCHEMA.to_string()),
            as_of: Some("2026-01-01".to_string()),
            conventions: Some(SnapshotConventionsV1::default()),
            maturities: vec![MaturityNodeV1 {
                t: 0.5,
                quotes: vec![IvQuoteV1 {
                    k: 1.0,
                    bid_iv: 0.2,
                    ask_iv: 0.21,
                }],
            }],
        };
        let raw = serde_json::to_string(&snap).unwrap();
        let parsed: IvSurfaceSnapshotV1 = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed, snap);
    }

    #[test]
    fn serde_rejects_unknown_fields() {
        let raw = r#"{
            "schema": "sanos.iv_surface.v1",
            "maturities": [],
            "unexpected": 123
        }"#;
        let err = serde_json::from_str::<IvSurfaceSnapshotV1>(raw).unwrap_err();
        assert!(err.to_string().contains("unknown field"));
    }
}
