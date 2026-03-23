//! RTF (Rich Text Format) extractor.
//!
//! Supports: Rich Text Format (.rtf)
//!
//! This native Rust extractor provides text extraction from RTF documents with:
//! - Character encoding support (Windows-1252 for 0x80-0x9F range)
//! - Common RTF control words (paragraph breaks, tabs, bullets, quotes, dashes)
//! - Unicode escape sequences
//! - Image metadata extraction
//! - Whitespace normalization

mod encoding;
mod formatting;
mod images;
mod metadata;
mod parser;
mod tables;

// Re-export public functions for backward compatibility
pub use encoding::{hex_digit_to_u8, parse_hex_byte, parse_rtf_control_word};
pub use formatting::normalize_whitespace;
pub use images::extract_image_metadata;
pub use metadata::{extract_rtf_metadata, parse_rtf_datetime};
pub use parser::extract_text_from_rtf;

use crate::Result;
use crate::core::config::ExtractionConfig;
use crate::plugins::{DocumentExtractor, Plugin};
use crate::types::{ExtractionResult, Metadata};
use async_trait::async_trait;

/// Native Rust RTF extractor.
///
/// Extracts text content, metadata, and structure from RTF documents
pub struct RtfExtractor;

impl RtfExtractor {
    /// Create a new RTF extractor.
    pub fn new() -> Self {
        Self
    }
}

impl Default for RtfExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for RtfExtractor {
    fn name(&self) -> &str {
        "rtf-extractor"
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
        "Extracts content from RTF (Rich Text Format) files with native Rust parsing"
    }

    fn author(&self) -> &str {
        "Kreuzberg Team"
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DocumentExtractor for RtfExtractor {
    #[cfg_attr(feature = "otel", tracing::instrument(
        skip(self, content, config),
        fields(
            extractor.name = self.name(),
            content.size_bytes = content.len(),
        )
    ))]
    async fn extract_bytes(
        &self,
        content: &[u8],
        mime_type: &str,
        config: &ExtractionConfig,
    ) -> Result<ExtractionResult> {
        let rtf_content = String::from_utf8_lossy(content);
        let plain = matches!(
            config.output_format,
            crate::core::config::OutputFormat::Plain | crate::core::config::OutputFormat::Structured
        );

        let (extracted_text, tables) = extract_text_from_rtf(&rtf_content, plain);
        let metadata_map = extract_rtf_metadata(&rtf_content, &extracted_text);

        let document = if config.include_document_structure {
            use crate::types::builder::DocumentStructureBuilder;
            let mut builder = DocumentStructureBuilder::new().source_format("rtf");

            // Build structure from extracted text paragraphs and tables.
            // Tables are emitted separately; text paragraphs come from double-newline splitting.
            let mut table_idx = 0;
            for paragraph in extracted_text.split("\n\n") {
                let trimmed = paragraph.trim();
                if trimmed.is_empty() {
                    continue;
                }
                // Check if this paragraph looks like a table (contains pipe separators on multiple lines)
                let lines: Vec<&str> = trimmed.lines().collect();
                let is_table_like = lines.len() >= 2 && lines.iter().all(|l| l.contains('|'));
                if is_table_like && table_idx < tables.len() {
                    builder.push_table_simple(&tables[table_idx].cells, None);
                    table_idx += 1;
                } else {
                    builder.push_paragraph(trimmed, vec![], None, None);
                }
            }

            // Push any remaining tables that weren't matched inline
            while table_idx < tables.len() {
                builder.push_table_simple(&tables[table_idx].cells, None);
                table_idx += 1;
            }

            Some(builder.build())
        } else {
            None
        };

        Ok(ExtractionResult {
            content: extracted_text,
            mime_type: mime_type.to_string().into(),
            metadata: Metadata {
                additional: metadata_map,
                ..Default::default()
            },
            pages: None,
            tables,
            detected_languages: None,
            chunks: None,
            images: None,
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
        &["application/rtf", "text/rtf"]
    }

    fn priority(&self) -> i32 {
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rtf_extractor_plugin_interface() {
        let extractor = RtfExtractor::new();
        assert_eq!(extractor.name(), "rtf-extractor");
        assert_eq!(extractor.version(), env!("CARGO_PKG_VERSION"));
        assert!(extractor.supported_mime_types().contains(&"application/rtf"));
        assert_eq!(extractor.priority(), 50);
    }

    #[test]
    fn test_simple_rtf_extraction() {
        let _extractor = RtfExtractor;
        let rtf_content = r#"{\rtf1 Hello World}"#;
        let (extracted, _) = extract_text_from_rtf(rtf_content, false);
        assert!(extracted.contains("Hello") || extracted.contains("World"));
    }

    #[test]
    fn test_plain_text_no_image_markdown() {
        let rtf_content = r#"{\rtf1 Before image {\pict\jpegblip\picw100\pich100 ffd8ffe0} After image}"#;
        let (plain, _) = extract_text_from_rtf(rtf_content, true);
        assert!(!plain.contains("!["), "Plain text should not contain image markdown");
        assert!(
            !plain.contains("]("),
            "Plain text should not contain image markdown links"
        );

        let (md, _) = extract_text_from_rtf(rtf_content, false);
        assert!(md.contains("![image]"), "Markdown output should contain image markers");
    }

    #[test]
    fn test_plain_text_table_uses_pipes() {
        let rtf_content = r#"{\rtf1 {\trowd Cell1\cell Cell2\cell\row}}"#;
        let (plain, tables) = extract_text_from_rtf(rtf_content, true);
        assert!(
            plain.contains('|'),
            "Plain text should use pipe delimiters for table cells"
        );
        assert!(!tables.is_empty(), "Tables should still be extracted");
    }
}
