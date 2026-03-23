//! Native PPT extractor for PowerPoint 97-2003 binary format.
//!
//! Extracts text directly from OLE/CFB compound documents without LibreOffice.

use crate::Result;
use crate::core::config::ExtractionConfig;
use crate::core::mime::LEGACY_POWERPOINT_MIME_TYPE;
use crate::extraction::ppt::extract_ppt_text;
use crate::plugins::{DocumentExtractor, Plugin};
use crate::types::{ExtractionResult, Metadata, PageInfo, PageStructure, PageUnitType};
use ahash::AHashMap;
use async_trait::async_trait;
use std::borrow::Cow;

/// Native PPT extractor using OLE/CFB parsing.
///
/// This extractor handles PowerPoint 97-2003 binary (.ppt) files without
/// requiring LibreOffice, providing ~50x faster extraction.
pub struct PptExtractor;

impl PptExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PptExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for PptExtractor {
    fn name(&self) -> &str {
        "ppt-extractor"
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
        "Native PPT text extraction via OLE/CFB parsing"
    }

    fn author(&self) -> &str {
        "Kreuzberg Team"
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DocumentExtractor for PptExtractor {
    async fn extract_bytes(
        &self,
        content: &[u8],
        mime_type: &str,
        config: &ExtractionConfig,
    ) -> Result<ExtractionResult> {
        let result = {
            #[cfg(feature = "tokio-runtime")]
            if crate::core::batch_mode::is_batch_mode() {
                let content_owned = content.to_vec();
                let span = tracing::Span::current();
                tokio::task::spawn_blocking(move || -> crate::error::Result<_> {
                    let _guard = span.entered();
                    extract_ppt_text(&content_owned)
                })
                .await
                .map_err(|e| crate::error::KreuzbergError::parsing(format!("PPT extraction task failed: {e}")))?
            } else {
                extract_ppt_text(content)
            }

            #[cfg(not(feature = "tokio-runtime"))]
            extract_ppt_text(content)
        }?;

        let mut metadata_map = AHashMap::new();

        if let Some(title) = result.metadata.title {
            metadata_map.insert(Cow::Borrowed("title"), serde_json::Value::String(title));
        }
        if let Some(author) = result.metadata.author {
            metadata_map.insert(
                Cow::Borrowed("authors"),
                serde_json::Value::Array(vec![serde_json::Value::String(author.clone())]),
            );
            metadata_map.insert(Cow::Borrowed("created_by"), serde_json::Value::String(author));
        }
        if let Some(subject) = result.metadata.subject {
            metadata_map.insert(Cow::Borrowed("subject"), serde_json::Value::String(subject));
        }
        if let Some(last_author) = result.metadata.last_author {
            metadata_map.insert(Cow::Borrowed("modified_by"), serde_json::Value::String(last_author));
        }

        metadata_map.insert(
            Cow::Borrowed("slide_count"),
            serde_json::Value::Number(result.slide_count.into()),
        );
        metadata_map.insert(
            Cow::Borrowed("extraction_method"),
            serde_json::Value::String("native_ole".to_string()),
        );

        // Store speaker notes if available
        if !result.speaker_notes.is_empty() {
            metadata_map.insert(
                Cow::Borrowed("speaker_notes"),
                serde_json::Value::Array(
                    result
                        .speaker_notes
                        .iter()
                        .map(|n| serde_json::Value::String(n.clone()))
                        .collect(),
                ),
            );
        }

        let page_structure = if result.slide_count > 0 {
            Some(PageStructure {
                total_count: result.slide_count,
                unit_type: PageUnitType::Slide,
                boundaries: None,
                pages: Some(
                    (1..=result.slide_count)
                        .map(|num| PageInfo {
                            number: num,
                            title: None,
                            dimensions: None,
                            image_count: None,
                            table_count: None,
                            hidden: None,
                            is_blank: None,
                        })
                        .collect(),
                ),
            })
        } else {
            None
        };

        let document = if config.include_document_structure {
            use crate::types::builder::DocumentStructureBuilder;
            let mut builder = DocumentStructureBuilder::new().source_format("ppt");

            // Split text by double-newlines; each block corresponds to a slide.
            let slide_blocks: Vec<&str> = result.text.split("\n\n").collect();
            for (i, block) in slide_blocks.iter().enumerate() {
                let trimmed = block.trim();
                if !trimmed.is_empty() {
                    let slide_num = (i + 1) as u32;
                    // Use first line as slide title if it's short
                    let mut lines = trimmed.lines();
                    let first_line = lines.next().unwrap_or("");
                    let title = if first_line.len() <= 80 && lines.clone().next().is_some() {
                        Some(first_line)
                    } else {
                        None
                    };
                    builder.push_slide(slide_num, title);

                    // Push remaining lines as paragraphs
                    if title.is_some() {
                        for line in lines {
                            let lt = line.trim();
                            if !lt.is_empty() {
                                builder.push_paragraph(lt, vec![], None, None);
                            }
                        }
                    } else {
                        // Push whole block as paragraph
                        builder.push_paragraph(trimmed, vec![], None, None);
                    }

                    // Add speaker notes as footnotes if available for this slide
                    if let Some(notes) = result.speaker_notes.get(i) {
                        builder.push_footnote(notes, None);
                    }

                    builder.exit_container();
                }
            }
            Some(builder.build())
        } else {
            None
        };

        Ok(ExtractionResult {
            content: result.text,
            mime_type: mime_type.to_string().into(),
            metadata: Metadata {
                pages: page_structure,
                additional: metadata_map,
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
        &[LEGACY_POWERPOINT_MIME_TYPE]
    }

    fn priority(&self) -> i32 {
        60 // Higher than default (50) to take precedence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ppt_extractor_plugin_interface() {
        let extractor = PptExtractor::new();
        assert_eq!(extractor.name(), "ppt-extractor");
        assert_eq!(extractor.version(), env!("CARGO_PKG_VERSION"));
        assert_eq!(extractor.priority(), 60);
        assert_eq!(extractor.supported_mime_types(), &["application/vnd.ms-powerpoint"]);
    }

    #[tokio::test]
    async fn test_ppt_extractor_initialize_shutdown() {
        let extractor = PptExtractor::new();
        assert!(extractor.initialize().is_ok());
        assert!(extractor.shutdown().is_ok());
    }

    #[tokio::test]
    async fn test_ppt_extractor_real_file() {
        let test_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test_documents/ppt/simple.ppt");
        if !test_file.exists() {
            return;
        }
        let content = std::fs::read(&test_file).expect("Failed to read test PPT");
        let extractor = PptExtractor::new();
        let config = ExtractionConfig::default();
        let result = extractor
            .extract_bytes(&content, "application/vnd.ms-powerpoint", &config)
            .await
            .expect("PPT extraction failed");
        assert!(!result.content.is_empty(), "Should extract text from PPT");
        assert_eq!(&*result.mime_type, "application/vnd.ms-powerpoint");
    }

    #[tokio::test]
    async fn test_ppt_document_structure_slides() {
        let test_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test_documents/ppt/simple.ppt");
        if !test_file.exists() {
            return;
        }
        let content = std::fs::read(&test_file).expect("Failed to read test PPT");
        let extractor = PptExtractor::new();
        let mut config = ExtractionConfig::default();
        config.include_document_structure = true;
        let result = extractor
            .extract_bytes(&content, "application/vnd.ms-powerpoint", &config)
            .await
            .expect("PPT extraction failed");
        assert!(result.document.is_some(), "Should produce document structure for PPT");
        let doc = result.document.unwrap();
        // Should contain Slide nodes
        let has_slide = doc
            .nodes
            .iter()
            .any(|n| matches!(n.content, crate::types::document_structure::NodeContent::Slide { .. }));
        assert!(has_slide, "PPT should produce Slide nodes in document structure");
    }

    #[tokio::test]
    async fn test_ppt_slide_count_metadata() {
        let test_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test_documents/ppt/simple.ppt");
        if !test_file.exists() {
            return;
        }
        let content = std::fs::read(&test_file).expect("Failed to read test PPT");
        let extractor = PptExtractor::new();
        let config = ExtractionConfig::default();
        let result = extractor
            .extract_bytes(&content, "application/vnd.ms-powerpoint", &config)
            .await
            .expect("PPT extraction failed");
        assert!(
            result.metadata.additional.contains_key("slide_count"),
            "Should have slide_count metadata"
        );
        let slide_count = result.metadata.additional.get("slide_count").unwrap();
        assert!(slide_count.as_u64().unwrap_or(0) > 0, "Slide count should be > 0");
    }
}
