//! # Bi-Parser Crate
//!
//! A Rust library for parsing and writing files in the bi format, a simple structured
//! human-readable binary format.
//!
//! ## Features
//! - Parse bi format files with validation
//! - Write bi format files
//! - Support for both Integer and Blob fields
//! - Comprehensive error handling
//!
//! ## Quickstart
//! ```no_run
//! use bi_parser::prelude::*;
//! use std::fs::File;
//!
//! // Reading bi format
//! let file = File::open("test.bi").unwrap();
//! let mut reader = BiReader::new(file);
//! let field = reader.read_field_default().unwrap();
//!
//! // Writing bi format
//! let mut file = File::create("output.bi").unwrap();
//! let mut writer = BiWriter::new(file);
//! let field = BiField::Integer {
//!     name: b"count".to_vec(),
//!     value: 42,
//! };
//! writer.write_field_default(&field).unwrap();
//! ```

pub mod bi_core;
pub mod parser;
pub mod validator;
pub mod writer;

pub mod prelude {
    pub use crate::bi_core::error::BiError;
    pub use crate::bi_core::types::BiField;
    pub use crate::bi_core::{MARKER_BLOB, MARKER_INT, MARKER_SYM, NEWLINE, SPACE};
    pub use crate::parser::reader::BiReader;
    pub use crate::writer::writer::BiWriter;
}
