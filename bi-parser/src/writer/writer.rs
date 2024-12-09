use super::error::BiWriterError;
use crate::prelude::{BiError, BiField, MARKER_BLOB, MARKER_INT, MARKER_SYM, NEWLINE, SPACE};
use crate::validator::utils::validate_field_name;
use std::io::Write;

pub struct BiWriter<W> {
    writer: W,
}

impl<W: Write> BiWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn write_field_default(&mut self, field: &BiField) -> Result<(), BiError> {
        self.write_field(field, true)
    }

    pub fn write_field(&mut self, field: &BiField, validate: bool) -> Result<(), BiError> {
        let mut content = Vec::new();

        match field {
            BiField::Integer { name, value } => {
                if validate {
                    validate_field_name(name).map_err(|e| BiWriterError::ValidationError(e))?;
                }

                content.extend_from_slice(&[MARKER_SYM, MARKER_INT, SPACE]);
                content.extend_from_slice(name);
                content.extend_from_slice(&[SPACE]);
                content.extend_from_slice(value.to_string().as_bytes());
                content.push(NEWLINE);
            }
            BiField::Blob { name, data } => {
                if validate {
                    validate_field_name(name).map_err(|e| BiWriterError::ValidationError(e))?;
                }

                content.extend_from_slice(&[MARKER_SYM, MARKER_BLOB, SPACE]);
                content.extend_from_slice(name);
                content.extend_from_slice(&[SPACE]);
                content.extend_from_slice(data.len().to_string().as_bytes());
                content.push(NEWLINE);
                content.extend_from_slice(data);
                content.push(NEWLINE);
            }
        }

        self.writer
            .write_all(&content)
            .map_err(|e| BiWriterError::WriteError(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_and_verify(field: &BiField, expected: &[u8]) {
        let mut buf = Vec::new();
        let mut writer = BiWriter::new(&mut buf);
        writer.write_field_default(field).unwrap();
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_write_integer() {
        let field = BiField::Integer {
            name: b"count".to_vec(),
            value: 42,
        };
        write_and_verify(&field, b":i count 42\n");
    }

    #[test]
    fn test_write_blob() {
        let field = BiField::Blob {
            name: b"data".to_vec(),
            data: b"hello".to_vec(),
        };
        write_and_verify(&field, b":b data 5\nhello\n");
    }

    #[test]
    fn test_write_empty_blob() {
        let field = BiField::Blob {
            name: b"empty".to_vec(),
            data: vec![],
        };
        write_and_verify(&field, b":b empty 0\n\n");
    }

    #[test]
    fn test_invalid_field_name() {
        let mut buf = Vec::new();
        let mut writer = BiWriter::new(&mut buf);
        let field = BiField::Integer {
            name: vec![],
            value: 42,
        };
        assert!(matches!(
            writer.write_field_default(&field).unwrap_err(),
            BiError::WriteError(BiWriterError::ValidationError(_))
        ));
    }

    #[test]
    fn test_non_utf8_field_name() {
        let mut buf = Vec::new();
        let mut writer = BiWriter::new(&mut buf);
        let field = BiField::Integer {
            name: vec![0xFF, 0xFF],
            value: 42,
        };
        assert!(matches!(
            writer.write_field_default(&field).unwrap_err(),
            BiError::WriteError(BiWriterError::ValidationError(_))
        ));
    }
}
