use crate::validator::error::BiValidationError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BiWriterError {
    #[error("Error writing onput: {0}")]
    WriteError(String),

    #[error(transparent)]
    ValidationError(#[from] BiValidationError),
}
