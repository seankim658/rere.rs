//! # Core Types Module
//!
//! This module provides the funamental types used throughout the library, including field types
//! and markers.

use std::fmt;

/// Represents a field in the bi format. A field can be either an Integer or a Blob, each with an
/// associated name.
#[derive(Debug, Clone, PartialEq)]
pub enum BiField {
    /// An integer field with format `:i name value\n`
    Integer { name: Vec<u8>, value: u64 },
    /// A blob field with format `:b name size\ndata\n`
    Blob { name: Vec<u8>, data: Vec<u8> },
}

impl fmt::Display for BiField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BiField::Integer { name, value } => {
                write!(f, ":i {} {}", String::from_utf8_lossy(name), value)
            }
            BiField::Blob { name, data } => {
                write!(f, ":b {} {}", String::from_utf8_lossy(name), data.len())
            }
        }
    }
}

/// Type of field marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldMarker {
    Integer,
    Blob,
}

impl FieldMarker {
    /// Converts a byte following the `:` symbol into the corresponding marker type.
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            crate::bi_core::MARKER_INT => Some(FieldMarker::Integer),
            crate::bi_core::MARKER_BLOB => Some(FieldMarker::Blob),
            _ => None,
        }
    }
}
