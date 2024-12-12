//! # Validator Utils Module
//!
//! Provides the utility functions for validating input against the bi format spec.

use super::error::BiValidationError;
use crate::bi_core::types::FieldMarker;
use crate::bi_core::{MARKER_SYM, NEWLINE, SPACE};

/// Validates a bi format field marker. Checks:
/// - First byte is the correct symbol
/// - Third byte is not a space
/// - Second byte is a marker type
///
/// ### Parameters
/// - `marker`: 3-byte array containing the marker bytes.
/// - `full_validation`: Whether to perform full validation, prevents duplicate operations when
/// also parsing the field.
pub fn validate_marker(marker: [u8; 3], full_validation: bool) -> Result<(), BiValidationError> {
    // Check marker symbol.
    if marker[0] != MARKER_SYM {
        return Err(BiValidationError::InvalidMarkerSymbol(marker[0] as char));
    }

    // Check space after marker type.
    if marker[2] != SPACE {
        return Err(BiValidationError::InvalidMarkerFormat(
            "expected single space after marker".to_owned(),
        ));
    }

    if full_validation {
        // Check marker type.
        if FieldMarker::from_byte(marker[1]).is_none() {
            return Err(BiValidationError::InvalidMarkerType(marker[1] as char));
        }
    }

    Ok(())
}

/// Validates a bi format field name. Checks:
/// - Name is not empty.
/// - Name contains valid UTF-8 characters.
///
/// ### Parameters
/// - `name_bytes`: Bytes containing the field names.
/// also parsing the field.
pub fn validate_field_name(name_bytes: &[u8]) -> Result<(), BiValidationError> {
    if name_bytes.is_empty() {
        return Err(BiValidationError::InvalidFieldName(
            "empty field name".to_owned(),
        ));
    }
    std::str::from_utf8(name_bytes)?;

    Ok(())
}

/// Validates a byte sequence contains only ASCII digits, doesn't overflow, and is not empty.
///
/// ### Parameters
/// - `bytes`: Bytes to validate.
pub fn validate_integer(bytes: &[u8]) -> Result<(), BiValidationError> {
    if bytes.is_empty() {
        return Err(BiValidationError::InvalidInteger(
            "empty integer".to_owned(),
        ));
    }

    let s = std::str::from_utf8(bytes)?;
    if s.parse::<u64>().is_err() {
        return Err(BiValidationError::InvalidInteger(s.to_owned()));
    }

    Ok(())
}

/// Validates a byte sequence contains only ASCII digits preceeded by a negative sign, doesn't
/// overflow, and is not emtpy.
///
/// ### Parameters
/// - `bytes`: Bytes to validate.
pub fn validate_signed_integer(bytes: &[u8]) -> Result<(), BiValidationError> {
    if bytes.is_empty() {
        return Err(BiValidationError::InvalidInteger(
            "empty signed integer".to_owned(),
        ));
    }

    let s = std::str::from_utf8(bytes)?;
    if s.parse::<i64>().is_err() {
        return Err(BiValidationError::InvalidInteger(s.to_owned()));
    }

    Ok(())
}

/// Validates blob content size and format. Checks:
/// - Blob content includes trailing newline.
/// - Content size matches expected size without trailing newline.
///
/// ### Parameters
/// - `content`: The blob content to validate.
/// - `expected_size`: The blob content size.
pub fn validate_blob(content: &[u8], expected_size: usize) -> Result<(), BiValidationError> {
    let expected_size_plus_newline = expected_size + 1;
    if content.len() != expected_size_plus_newline {
        return Err(BiValidationError::InvalidBlob(format!(
            "actual size ({}) doesn't match expected size ({})",
            content.len(),
            expected_size_plus_newline
        )));
    }

    if content[content.len() - 1] != NEWLINE {
        return Err(BiValidationError::InvalidBlob(
            "missing trailing newline".to_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_marker() {
        // Valid markers
        assert!(validate_marker([b':', b'i', b' '], true).is_ok());
        assert!(validate_marker([b':', b'b', b' '], true).is_ok());

        // Invalid marker symbol
        assert!(matches!(
            validate_marker([b'#', b'i', b' '], true),
            Err(BiValidationError::InvalidMarkerSymbol('#'))
        ));

        // Invalid marker type
        assert!(matches!(
            validate_marker([b':', b'n', b' '], true),
            Err(BiValidationError::InvalidMarkerType('n'))
        ));

        // Missing space
        assert!(matches!(
            validate_marker([b':', b'i', b'x'], true),
            Err(BiValidationError::InvalidMarkerFormat(_))
        ));
    }

    #[test]
    fn test_validate_field_name() {
        // Valid names
        assert!(validate_field_name(b"test").is_ok());
        assert!(validate_field_name(b"test_123").is_ok());

        // Empty name
        assert!(matches!(
            validate_field_name(b""),
            Err(BiValidationError::InvalidFieldName(_))
        ));

        // Invalid UTF-8
        let invalid_utf8 = &[0xFF, 0xFF];
        assert!(validate_field_name(invalid_utf8).is_err());
    }

    #[test]
    fn test_validate_integer() {
        // Valid integers
        assert!(validate_integer(b"123").is_ok());
        assert!(validate_integer(b"0").is_ok());

        // Invalid integers
        assert!(matches!(
            validate_integer(b"12a3"),
            Err(BiValidationError::InvalidInteger(_))
        ));
        assert!(matches!(
            validate_integer(b"-123"),
            Err(BiValidationError::InvalidInteger(_))
        ));
    }

    #[test]
    fn test_validate_signed_integer() {
        // Valid signed integers
        assert!(validate_signed_integer(b"123").is_ok());
        assert!(validate_signed_integer(b"-123").is_ok());
        assert!(validate_signed_integer(b"0").is_ok());
        assert!(validate_signed_integer(b"-0").is_ok());

        // Invalid signed integers
        assert!(matches!(
            validate_signed_integer(b""),
            Err(BiValidationError::InvalidInteger(_))
        ));
        assert!(matches!(
            validate_signed_integer(b"12a3"),
            Err(BiValidationError::InvalidInteger(_))
        ));
        assert!(matches!(
            validate_signed_integer(b"--123"),
            Err(BiValidationError::InvalidInteger(_))
        ));
    }

    #[test]
    fn test_validate_blob() {
        // Valid blob
        let content = b"test\n";
        assert!(validate_blob(content, 4).is_ok());

        // Wrong size
    }
}
