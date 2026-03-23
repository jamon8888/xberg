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

/// Parsed dBASE data: field names, field types, and rows of string values.
struct DbfParsed {
    field_names: Vec<String>,
    field_types: Vec<String>,
    rows: Vec<Vec<String>>,
    record_count: usize,
}

/// Map a dbase FieldType to a descriptive string.
fn field_type_name(value: &dbase::FieldValue) -> &'static str {
    match value {
        dbase::FieldValue::Character(_) => "Character",
        dbase::FieldValue::Numeric(_) => "Numeric",
        dbase::FieldValue::Logical(_) => "Logical",
        dbase::FieldValue::Date(_) => "Date",
        dbase::FieldValue::Float(_) => "Float",
        dbase::FieldValue::Integer(_) => "Integer",
        dbase::FieldValue::Currency(_) => "Currency",
        dbase::FieldValue::Double(_) => "Double",
        dbase::FieldValue::Memo(_) => "Memo",
        _ => "Unknown",
    }
}

/// Parse a dBASE file once, returning field names, types, and row data.
fn parse_dbf(content: &[u8]) -> Result<DbfParsed> {
    let cursor = Cursor::new(content);
    let mut reader = dbase::Reader::new(cursor)
        .map_err(|e| crate::KreuzbergError::parsing(format!("Failed to open dBASE file: {e}")))?;

    let field_names: Vec<String> = reader.fields().iter().map(|f| f.name().to_string()).collect();

    let records = reader
        .iter_records()
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| crate::KreuzbergError::parsing(format!("Failed to read dBASE records: {e}")))?;

    let record_count = records.len();

    // Detect field types from the first record and build rows simultaneously
    let mut field_types: Vec<String> = vec!["Unknown".to_string(); field_names.len()];
    let mut rows: Vec<Vec<String>> = Vec::with_capacity(records.len());
    let mut first_row = true;

    for record in records {
        let mut row = Vec::with_capacity(field_names.len());
        for (col_idx, (_, v)) in record.into_iter().enumerate() {
            if first_row && col_idx < field_types.len() {
                field_types[col_idx] = field_type_name(&v).to_string();
            }
            row.push(field_value_to_string(&v));
        }
        rows.push(row);
        first_row = false;
    }

    Ok(DbfParsed {
        field_names,
        field_types,
        rows,
        record_count,
    })
}

fn build_dbf_document_structure(parsed: &DbfParsed) -> crate::types::document_structure::DocumentStructure {
    use crate::types::builder::DocumentStructureBuilder;

    let mut builder = DocumentStructureBuilder::new().source_format("dbf");

    if parsed.field_names.is_empty() {
        return builder.build();
    }

    let mut table_rows: Vec<Vec<String>> = Vec::with_capacity(parsed.rows.len() + 1);
    table_rows.push(parsed.field_names.clone());
    table_rows.extend(parsed.rows.iter().cloned());

    builder.push_table_simple(&table_rows, None);
    builder.build()
}

fn format_dbf_content(parsed: &DbfParsed) -> String {
    if parsed.field_names.is_empty() {
        return String::new();
    }

    let mut output = String::new();

    // Header row
    output.push('|');
    for name in &parsed.field_names {
        output.push_str(&format!(" {name} |"));
    }
    output.push('\n');

    // Separator row
    output.push('|');
    for _ in &parsed.field_names {
        output.push_str(" --- |");
    }
    output.push('\n');

    // Data rows
    for row in &parsed.rows {
        output.push('|');
        for s in row {
            output.push_str(&format!(" {s} |"));
        }
        output.push('\n');
    }

    output
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DocumentExtractor for DbfExtractor {
    async fn extract_bytes(
        &self,
        content: &[u8],
        mime_type: &str,
        config: &ExtractionConfig,
    ) -> Result<ExtractionResult> {
        let parsed = parse_dbf(content)?;
        let text = format_dbf_content(&parsed);

        let document = if config.include_document_structure {
            Some(build_dbf_document_structure(&parsed))
        } else {
            None
        };

        let mut additional = ahash::AHashMap::new();
        additional.insert(
            std::borrow::Cow::Borrowed("record_count"),
            serde_json::json!(parsed.record_count),
        );
        additional.insert(
            std::borrow::Cow::Borrowed("field_count"),
            serde_json::json!(parsed.field_names.len()),
        );
        // Build field info with name and type
        let field_info: Vec<serde_json::Value> = parsed
            .field_names
            .iter()
            .zip(parsed.field_types.iter())
            .map(|(name, ftype)| serde_json::json!({"name": name, "type": ftype}))
            .collect();
        additional.insert(std::borrow::Cow::Borrowed("fields"), serde_json::json!(field_info));

        Ok(ExtractionResult {
            content: text,
            mime_type: mime_type.to_string().into(),
            metadata: Metadata {
                additional,
                ..Default::default()
            },
            pages: None,
            tables: vec![],
            detected_languages: None,
            chunks: None,
            images: Some(vec![]),
            djot_content: None,
            elements: None,
            ocr_elements: None,
            document,
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
