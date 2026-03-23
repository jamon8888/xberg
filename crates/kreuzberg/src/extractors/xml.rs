//! XML extractor.

use crate::Result;
use crate::core::config::ExtractionConfig;
use crate::extraction::xml::{parse_xml, parse_xml_svg};
use crate::extractors::SyncExtractor;
use crate::plugins::{DocumentExtractor, Plugin};
use crate::types::ExtractionResult;
use async_trait::async_trait;

/// XML extractor.
///
/// Extracts text content from XML files, preserving element structure information.
pub struct XmlExtractor;

impl XmlExtractor {
    /// Create a new XML extractor.
    pub fn new() -> Self {
        Self
    }
}

impl Default for XmlExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for XmlExtractor {
    fn name(&self) -> &str {
        "xml-extractor"
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
        "Extracts text content from XML files with element metadata"
    }

    fn author(&self) -> &str {
        "Kreuzberg Team"
    }
}

impl SyncExtractor for XmlExtractor {
    fn extract_sync(&self, content: &[u8], mime_type: &str, config: &ExtractionConfig) -> Result<ExtractionResult> {
        let xml_result = if mime_type == "image/svg+xml" {
            parse_xml_svg(content, false)?
        } else {
            parse_xml(content, false)?
        };

        let document = if config.include_document_structure && !xml_result.content.trim().is_empty() {
            use crate::types::builder::DocumentStructureBuilder;

            let mut builder = DocumentStructureBuilder::new().source_format("xml");
            for line in xml_result.content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    builder.push_paragraph(trimmed, vec![], None, None);
                }
            }
            Some(builder.build())
        } else {
            None
        };

        Ok(ExtractionResult {
            content: xml_result.content,
            mime_type: mime_type.to_string().into(),
            metadata: crate::types::Metadata {
                format: Some(crate::types::FormatMetadata::Xml(crate::types::XmlMetadata {
                    element_count: xml_result.element_count,
                    unique_elements: xml_result.unique_elements,
                })),
                ..Default::default()
            },
            tables: vec![],
            detected_languages: None,
            chunks: None,
            images: None,
            pages: None,
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
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DocumentExtractor for XmlExtractor {
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
        self.extract_sync(content, mime_type, config)
    }

    fn supported_mime_types(&self) -> &[&str] {
        &[
            "application/xml",
            "text/xml",
            "image/svg+xml",
            "application/x-endnote+xml",
        ]
    }

    fn priority(&self) -> i32 {
        50
    }

    fn as_sync_extractor(&self) -> Option<&dyn crate::extractors::SyncExtractor> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_xml_extractor() {
        let extractor = XmlExtractor::new();
        let content = b"<root><item>Hello</item><item>World</item></root>";
        let config = ExtractionConfig::default();

        let result = extractor
            .extract_bytes(content, "application/xml", &config)
            .await
            .unwrap();

        assert_eq!(result.mime_type, "application/xml");
        // Now includes element names as context
        assert!(result.content.contains("item: Hello"));
        assert!(result.content.contains("item: World"));
        assert!(result.metadata.format.is_some());
        let xml_meta = match result.metadata.format.as_ref().unwrap() {
            crate::types::FormatMetadata::Xml(meta) => meta,
            _ => panic!("Expected Xml metadata"),
        };
        assert_eq!(xml_meta.element_count, 3);
        assert!(xml_meta.unique_elements.contains(&"root".to_string()));
        assert!(xml_meta.unique_elements.contains(&"item".to_string()));
    }

    #[test]
    fn test_xml_plugin_interface() {
        let extractor = XmlExtractor::new();
        assert_eq!(extractor.name(), "xml-extractor");
        assert_eq!(extractor.version(), env!("CARGO_PKG_VERSION"));
        assert_eq!(
            extractor.supported_mime_types(),
            &[
                "application/xml",
                "text/xml",
                "image/svg+xml",
                "application/x-endnote+xml"
            ]
        );
        assert_eq!(extractor.priority(), 50);
    }
}
