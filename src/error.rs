//! Custom error type for `DataFusion-ObjectStore-GCS`

use std::error::Error;
use std::fmt::{Display, Formatter};

/// Enum with all errors in this crate.
/// PartialEq is to enable testing for specific error types
#[derive(Debug, PartialEq)]
pub enum GCSError {
    /// Returned when functionaly is not yet available.
    NotImplemented(String),
    /// Wrapper for GCS errors
    GCS(String),
}

impl Display for GCSError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GCSError::NotImplemented(desc) => write!(f, "Not yet implemented: {}", desc),
            GCSError::GCS(desc) => write!(f, "AWS error: {}", desc),
        }
    }
}

impl Error for GCSError {}
