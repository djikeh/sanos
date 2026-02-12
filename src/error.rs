// src/error.rs
use std::fmt;

/// Result type alias used throughout the sanos crate.
pub type SanosResult<T> = Result<T, SanosError>;

#[derive(Debug, Clone)]
pub enum SanosError {
    /// A floating-point input is NaN or infinite.
    NonFinite {
        field: &'static str,
        value: f64,
    },

    /// A value violates inclusive bounds [min, max].
    InvalidBound {
        field: &'static str,
        value: f64,
        min: f64,
        max: f64,
    },

    /// A monotonicity or ordering constraint is violated.
    InvalidOrdering {
        msg: &'static str,
    },

    /// A required collection is empty.
    EmptyCollection {
        what: &'static str,
    },

    /// A duplicate key/value was detected where uniqueness is required.
    DuplicateKey {
        what: &'static str,
        value: f64,
    },

    /// ATM mid-price could not be computed for a given maturity.
    AtmNotComputable {
        maturity: f64,
        reason: &'static str,
    },
}

impl fmt::Display for SanosError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SanosError::NonFinite { field, value } => {
                write!(f, "Non-finite value for {field}: {value}")
            }
            SanosError::InvalidBound { field, value, min, max } => {
                write!(f, "Invalid {field}: {value} not in [{min}, {max}]")
            }
            SanosError::InvalidOrdering { msg } => {
                write!(f, "Invalid ordering: {msg}")
            }
            SanosError::EmptyCollection { what } => {
                write!(f, "Empty collection: {what}")
            }
            SanosError::DuplicateKey { what, value } => {
                write!(f, "Duplicate {what}: {value}")
            }
            SanosError::AtmNotComputable { maturity, reason } => {
                write!(f, "ATM mid not computable for maturity {maturity}: {reason}")
            }
        }
    }
}

impl std::error::Error for SanosError {}
