use crate::validator::error::BiValidationError;
use thiserror::Error;

/// Error that occurs during parsing of bi format files.
#[derive(Debug, Error)]
pub enum BiParserError {
    /// Reader reached end of input while parsing a field.
    #[error("Unexpected end of file: {0}")]
    UnexpectedEof(String),

    /// I/O error occurred while reading from input.
    #[error("Error reading input: {0}")]
    ReadError(String),

    /// Field validation failed.
    #[error(transparent)]
    ValidationError(#[from] BiValidationError),
}
