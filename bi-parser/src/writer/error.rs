//! # Writer Error Module
//!
//! This module provides the writer error type.

use crate::validator::error::BiValidationError;
use thiserror::Error;

/// Error that occurs during writing of bi format data.
#[derive(Debug, Error)]
pub enum BiWriterError {
    /// Unexpected error in writing the data.
    #[error("Error writing onput: {0}")]
    WriteError(String),

    /// Field validation failed.
    #[error(transparent)]
    ValidationError(#[from] BiValidationError),
}
