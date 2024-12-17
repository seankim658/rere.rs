# Bi-Parser

A Rust library for parsing, writing, and validating files in the bi format. The original bi format specification can be seen [here](https://github.com/tsoding/bi-format).

- [Quick Start](#quick-start)
- [Field Types](#field-types)
- [Validation](#validation)
- [Error Handling](#error-handling)

---

## Quick Start

```rust 
use bi_parser::prelude::*;
use std::fs::File;

// Reading bi format
let file = File::open("test.bi")?;
let mut reader = BiReader::new(file);
let field = reader.read_field_default()?;

// Writing bi format
let file = File::create("output.bi")?;
let mut writer = BiWriter::new(file);
let field = BiField::Integer {
    name: b"count".to_vec(),
    value: 42,
};
writer.write_field_default(&field)?;
```

## Field Types

The bi format supports three types of fields:

- Integer (`:i`): Unsigned 64-bit integers
- SignedInteger (`:s`): Signed 64-bit integers (this field is not in the original bi format specification)
- Blob (`:b`): Variable-length binary data

Each field has a name and follows this format:

```
Integer:       :i name value\n
SignedInteger: :s name value\n
Blob:          :b name size\ndata\n
```

## Validation

By default, the parser performs thorough validation of:

- Field markers and format
- Field names
- Integer values
- Blob sizes and content

Validation can be disabled for performance:

```rust 
reader.read_field(false)?;  // Skip validation
writer.write_field(&field, false)?;  // Skip validation
```

## Error Handling

The crate provides detailed error types for different failure scenarios:

- `BiError`: Top-level error type
- `BiParserError`: Parsing-specific errors
- `BiWriterError`: Writing-specific errors
- `BiValidationError`: Validation-specific errors
