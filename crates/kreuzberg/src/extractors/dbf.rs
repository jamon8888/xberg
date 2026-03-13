//! dBASE (.dbf) extractor.
//!
//! Reads records from dBASE files and formats them as a markdown table.

use crate::Result;
use crate::core::config::ExtractionConfig;
use crate::plugins::{DocumentExtractor, Plugin};
use crate::types::{ExtractionResult, Metadata};
use async_trait::async_trait;
use std::io::Cursor;

/// Extractor for dBASE (.dbf) files.
///
/// Reads all records and formats them as a markdown table with
/// column headers derived from field names.
pub struct DbfExtractor;

impl DbfExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DbfExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for DbfExtractor {
    fn name(&self) -> &str {
        "dbf-extractor"
    }

    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn initialize(&self) -> Result<()> {
        Ok(())
    }

    fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    fn description(&self) -> &str {
        "dBASE (.dbf) table extraction"
    }

    fn author(&self) -> &str {
        "Kreuzberg Team"
    }
}

fn field_value_to_string(value: &dbase::FieldValue) -> String {
    match value {
        dbase::FieldValue::Character(Some(s)) => s.trim().to_string(),
        dbase::FieldValue::Numeric(Some(n)) => n.to_string(),
        dbase::FieldValue::Logical(Some(b)) => b.to_string(),
        dbase::FieldValue::Date(Some(d)) => format!("{}-{:02}-{:02}", d.year(), d.month(), d.day()),
        dbase::FieldValue::Float(Some(f)) => f.to_string(),
        dbase::FieldValue::Integer(i) => i.to_string(),
        dbase::FieldValue::Currency(c) => format!("{c:.2}"),
        dbase::FieldValue::Double(d) => d.to_string(),
        dbase::FieldValue::Memo(s) => s.trim().to_string(),
        _ => String::new(),
    }
}

fn extract_dbf_content(content: &[u8]) -> Result<String> {
    let cursor = Cursor::new(content);
    let mut reader = dbase::Reader::new(cursor)
        .map_err(|e| crate::KreuzbergError::parsing(format!("Failed to open dBASE file: {e}")))?;

    let field_names: Vec<String> = reader.fields().iter().map(|f| f.name().to_string()).collect();

    if field_names.is_empty() {
        return Ok(String::new());
    }

    let mut output = String::new();

    // Header row
    output.push('|');
    for name in &field_names {
        output.push_str(&format!(" {name} |"));
    }
    output.push('\n');

    // Separator row
    output.push('|');
    for _ in &field_names {
        output.push_str(" --- |");
    }
    output.push('\n');

    // Data rows
    let records = reader
        .iter_records()
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| crate::KreuzbergError::parsing(format!("Failed to read dBASE records: {e}")))?;

    for record in records {
        output.push('|');
        for (_, value) in record {
            let s = field_value_to_string(&value);
            output.push_str(&format!(" {s} |"));
        }
        output.push('\n');
    }

    Ok(output)
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DocumentExtractor for DbfExtractor {
    async fn extract_bytes(
        &self,
        content: &[u8],
        mime_type: &str,
        _config: &ExtractionConfig,
    ) -> Result<ExtractionResult> {
        let text = extract_dbf_content(content)?;

        Ok(ExtractionResult {
            content: text,
            mime_type: mime_type.to_string().into(),
            metadata: Metadata::default(),
            pages: None,
            tables: vec![],
            detected_languages: None,
            chunks: None,
            images: Some(vec![]),
            djot_content: None,
            elements: None,
            ocr_elements: None,
            document: None,
            #[cfg(any(feature = "keywords-yake", feature = "keywords-rake"))]
            extracted_keywords: None,
            quality_score: None,
            processing_warnings: Vec::new(),
            annotations: None,
        })
    }

    fn supported_mime_types(&self) -> &[&str] {
        &["application/x-dbf", "application/dbase"]
    }

    fn priority(&self) -> i32 {
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dbf_extractor_plugin_interface() {
        let extractor = DbfExtractor::new();
        assert_eq!(extractor.name(), "dbf-extractor");
        assert_eq!(extractor.version(), env!("CARGO_PKG_VERSION"));
        assert_eq!(extractor.priority(), 50);
        assert_eq!(
            extractor.supported_mime_types(),
            &["application/x-dbf", "application/dbase"]
        );
    }

    #[test]
    fn test_dbf_extractor_initialize_shutdown() {
        let extractor = DbfExtractor::new();
        assert!(extractor.initialize().is_ok());
        assert!(extractor.shutdown().is_ok());
    }
}
