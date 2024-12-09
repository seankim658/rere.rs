//!
//!

use crate::parser::error::BiParserError;
use crate::validator::error::BiValidationError;
use thiserror::Error;

/// Top-level error type combining all possible bi format errors. 
#[derive(Debug, Error)]
pub enum BiError {
    /// Low level I/O error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error during parsing of bi format struture.
    #[error("Parse error: {0}")]
    ParseError(#[from] BiParserError),

    /// Error during validation of field content.
    #[error("Validation error: {0}")]
    ValidationError(#[from] BiValidationError),
}
