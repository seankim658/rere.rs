use super::error::BiParserError;
use crate::bi_core::types::FieldMarker;
use crate::bi_core::{NEWLINE, SPACE};
use crate::prelude::{BiError, BiField};
use crate::validator::error::BiValidationError;
use crate::validator::utils::{
    validate_blob, validate_field_name, validate_integer, validate_marker,
};
use std::io::{BufRead, BufReader, Read};

/// A buffered reader for parsing bi format files.
pub struct BiReader<R> {
    reader: BufReader<R>,
}

impl<R: Read> BiReader<R> {
    /// Constructor.
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
        }
    }

    /// Read a field with validation enabled (default behavior).
    ///
    /// Equivalent to `read_field(true)`.
    pub fn read_field_default(&mut self) -> Result<BiField, BiError> {
        self.read_field(true)
    }

    /// Read a single field from the underlying reader.
    ///
    /// ### Parameters
    /// - `validate`: Whether or not to perform validation that the parsed data conforms to the bi
    /// format specification.
    ///
    /// ### Returns
    /// `Result<BiField, BiError>`
    pub fn read_field(&mut self, validate: bool) -> Result<BiField, BiError> {
        // Read the 3-byte marker consisting of `:`, the marker type, and a space.
        let mut marker = [0u8; 3];
        self.reader
            .read_exact(&mut marker)
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::UnexpectedEof => {
                    BiParserError::UnexpectedEof("while reading marker".to_string())
                }
                _ => BiParserError::ReadError(e.to_string()).into(),
            })?;
        if validate {
            validate_marker(marker, false).map_err(|e| BiParserError::ValidationError(e))?;
        }
        // Extract and validate the marker type.
        let marker_type = FieldMarker::from_byte(marker[1]).ok_or_else(|| {
            BiParserError::ValidationError(BiValidationError::InvalidMarkerType(marker[1] as char))
        })?;

        // Read the field name up to the next space.
        let mut name_bytes = Vec::new();
        self.reader
            .read_until(SPACE, &mut name_bytes)
            .map_err(|e| {
                BiParserError::ReadError(format!(
                    "error when reading field name: {}",
                    e.to_string()
                ))
            })?;
        // Remove trailing space.
        name_bytes.pop();
        if validate {
            validate_field_name(&name_bytes)
                .map_err(|e| BiParserError::ValidationError(e))?;
        }

        match marker_type {
            FieldMarker::Integer => {
                let mut value_bytes = Vec::new();
                self.reader
                    .read_until(NEWLINE, &mut value_bytes)
                    .map_err(|e| {
                        BiParserError::ReadError(format!(
                            "error reading integer field value: {}",
                            e.to_string()
                        ))
                    })?;
                value_bytes.pop();
                if validate {
                    validate_integer(&value_bytes)
                        .map_err(|e| BiParserError::ValidationError(e))?;
                }

                let value_str = String::from_utf8(value_bytes).map_err(|e| {
                    BiParserError::ValidationError(BiValidationError::Utf8Error(e.utf8_error()))
                })?;
                let value = value_str
                    .parse::<u64>()
                    .map_err(|_| BiValidationError::InvalidInteger(value_str))?;

                Ok(BiField::Integer { name: name_bytes, value })
            }
            FieldMarker::Blob => {
                let mut size_bytes = Vec::new();
                self.reader
                    .read_until(NEWLINE, &mut size_bytes)
                    .map_err(|e| {
                        BiParserError::ReadError(format!(
                            "error reading blob size: {}",
                            e.to_string()
                        ))
                    })?;
                size_bytes.pop();
                if validate {
                    validate_integer(&size_bytes).map_err(|e| BiParserError::ValidationError(e))?;
                }

                let size_str = String::from_utf8(size_bytes)
                    .map_err(|e| BiValidationError::Utf8Error(e.utf8_error()))?;
                let size = size_str
                    .parse::<usize>()
                    .map_err(|_| BiValidationError::InvalidInteger(size_str))?;

                let mut data = vec![0; size + 1];
                self.reader.read_exact(&mut data).map_err(|e| {
                    BiParserError::ReadError(format!(
                        "error reading blob content: {}",
                        e.to_string()
                    ))
                })?;

                if validate {
                    validate_blob(&data, size).map_err(|e| BiParserError::ValidationError(e))?;
                }
                data.pop();

                Ok(BiField::Blob { name: name_bytes, data })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn create_reader(content: &str) -> BiReader<Cursor<Vec<u8>>> {
        BiReader::new(Cursor::new(content.as_bytes().to_vec()))
    }

    #[test]
    fn test_read_integer_field() {
        let mut reader = create_reader(":i count 42\n");
        let field = reader.read_field_default().unwrap();

        match field {
            BiField::Integer { name, value } => {
                assert_eq!(name, b"count");
                assert_eq!(value, 42);
            }
            _ => panic!("Expected integer field"),
        }
    }

    #[test]
    fn test_read_blob_field() {
        let mut reader = create_reader(":b data 5\nhello\n");
        let field = reader.read_field_default().unwrap();

        match field {
            BiField::Blob { name, data } => {
                assert_eq!(name, b"data");
                assert_eq!(data, b"hello");
            }
            _ => panic!("Expected blob field"),
        }
    }

    #[test]
    fn test_invalid_marker() {
        let mut reader = create_reader("#i count 42\n");
        assert!(matches!(
            reader.read_field_default().unwrap_err(),
            BiError::ParseError(BiParserError::ValidationError(
                BiValidationError::InvalidMarkerSymbol('#')
            ))
        ));
    }

    #[test]
    fn test_invalid_marker_type() {
        let mut reader = create_reader(":x count 42\n");
        assert!(matches!(
            reader.read_field_default().unwrap_err(),
            BiError::ParseError(BiParserError::ValidationError(
                BiValidationError::InvalidMarkerType('x')
            ))
        ));
    }

    #[test]
    fn test_invalid_integer_value() {
        let mut reader = create_reader(":i count abc\n");
        assert!(matches!(
            reader.read_field_default().unwrap_err(),
            BiError::ParseError(BiParserError::ValidationError(
                BiValidationError::InvalidInteger(_)
            ))
        ));
    }

    #[test]
    fn test_blob_wrong_size() {
        let mut reader = create_reader(":b data 3\nhello\n");
        assert!(matches!(
            reader.read_field_default().unwrap_err(),
            BiError::ParseError(BiParserError::ValidationError(
                BiValidationError::InvalidBlob(_)
            ))
        ));

        let mut reader2 = create_reader(":b data 10\nhello\n");
        assert!(matches!(
            reader2.read_field_default().unwrap_err(),
            BiError::ParseError(BiParserError::ReadError(_))
        ));
    }

    #[test]
    fn test_blob_missing_newline() {
        let mut reader = create_reader(":b data 5\nhello");
        assert!(matches!(
            reader.read_field_default().unwrap_err(),
            BiError::ParseError(BiParserError::ReadError(_))
        ));
    }

    #[test]
    fn test_unexpected_eof() {
        let mut reader = create_reader(":b");
        assert!(matches!(
            reader.read_field_default().unwrap_err(),
            BiError::ParseError(BiParserError::UnexpectedEof(_))
        ));
    }

    #[test]
    fn test_no_validation() {
        // This would fail with validation but should pass without it
        let mut reader = create_reader(":i count abc\n");
        assert!(reader.read_field(false).is_err());
    }

    #[test]
    fn test_empty_field_name() {
        let mut reader = create_reader(":i  42\n");
        assert!(matches!(
            reader.read_field_default().unwrap_err(),
            BiError::ParseError(BiParserError::ValidationError(
                BiValidationError::InvalidFieldName(_)
            ))
        ));
    }
}
