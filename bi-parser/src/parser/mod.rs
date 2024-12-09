//! # Parser Module
//!
//! This module provides functionality for parsing bi format files. The bi format is a simple,
//! structured human-readable binary format consisting of Integer and Blob fields (bi format documentation can be
//! found [here](https://github.com/tsoding/bi-format/tree/main?tab=readme-ov-file)).
//!
//! ## Examples
//! ```no_run
//! use bi_parser::parser::reader::BiReader;
//! use std::fs::File;
//!
//! let file = File::open("text.bi").unwrap();
//! let mut reader = BiReader::new(file);
//! // Reads a single field from `test.bi`
//! let field = reader.read_field_default().unwrap();
//! ```

pub mod reader;
pub mod error;
