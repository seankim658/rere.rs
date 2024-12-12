//! # Core Module
//!
//! Core functionality and constants for the bi format.

pub mod error;
pub mod types;

pub const MARKER_SYM: u8 = b':';
pub const MARKER_INT: u8 = b'i';
pub const MARKER_SINT: u8 = b's';
pub const MARKER_BLOB: u8 = b'b';
pub const SPACE: u8 = b' ';
pub const NEWLINE: u8 = b'\n';
pub const BIDOCS: &str = "https://github.com/tsoding/bi-format/blob/main/README.md";
