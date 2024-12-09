//! # Validator Error Module
//!
//! This module provides the validator error type.

use crate::bi_core::{BIDOCS, MARKER_BLOB, MARKER_INT, MARKER_SYM};
use std::fmt;
use thiserror::Error;

/// Error that occurs during validation of bi format fields.
#[derive(Debug, Error)]
pub enum BiValidationError {
    /// Field marker does not start with the expected `:` symbol.
    InvalidMarkerSymbol(char),
    /// Field marker not followed by a space or has other format issues.
    InvalidMarkerFormat(String),
    /// Field marker type is unrecognized or invalid.
    InvalidMarkerType(char),
    /// Field names is empty.
    InvalidFieldName(String),
    /// Integer value contains non-digit characters or is otherwise malformed.
    InvalidInteger(String),
    /// Blob content does not match declared size or is missing trailing newline.
    InvalidBlob(String),
    /// String content contains invalid UTF-8 encoding.
    Utf8Error(#[from] std::str::Utf8Error),
}

impl fmt::Display for BiValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMarkerSymbol(found) => write!(
                f,
                "Invalid marker symbol: expected `{s}`, found `{f}`",
                s = MARKER_SYM,
                f = found
            ),
            Self::InvalidMarkerFormat(msg) => write!(
                f,
                "Invalid marker format: {m}\nRefer to {d}",
                m = msg,
                d = BIDOCS
            ),
            Self::InvalidMarkerType(found) => write!(
                f,
                "Invalid field marker: expected `{s}{i}` or `{s}{b}`, found `{f}`",
                s = MARKER_SYM,
                i = MARKER_INT,
                b = MARKER_BLOB,
                f = found
            ),
            Self::InvalidFieldName(name) => write!(f, "Invalid field name: {}", name),
            Self::InvalidInteger(found) => write!(f, "Invalid integer: {}", found),
            Self::InvalidBlob(msg) => write!(f, "Invalid blob: {}", msg),
            Self::Utf8Error(err) => write!(f, "UTF-8 decoding error: {}", err),
        }
    }
}
