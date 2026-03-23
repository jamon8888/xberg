//! Native EPUB extractor using permissive-licensed dependencies.
//!
//! This extractor provides native Rust-based EPUB extraction without GPL-licensed
//! dependencies, extracting:
//! - Metadata from OPF (Open Packaging Format) using Dublin Core standards
//! - Content from XHTML files in spine order
//! - Proper handling of EPUB2 and EPUB3 formats
//!
//! Uses only permissive-licensed crates:
//! - `zip` (MIT/Apache) - for reading EPUB container
//! - `roxmltree` (MIT) - for parsing XML
//! - `html-to-markdown-rs` (MIT) - for converting XHTML to Markdown/Djot when requested

mod content;
mod metadata;
mod parsing;

use crate::Result;
use crate::core::config::ExtractionConfig;
use crate::plugins::{DocumentExtractor, Plugin};
use crate::types::ExtractionResult;
use crate::types::Metadata;
use ahash::AHashMap;
use async_trait::async_trait;
use std::borrow::Cow;
use std::io::Cursor;
use zip::ZipArchive;

use content::extract_content;
use content::extract_text_from_xhtml;
use metadata::extract_metadata;
use parsing::{parse_container_xml, read_file_from_zip, resolve_path};

/// EPUB format extractor using permissive-licensed dependencies.
///
/// Extracts content and metadata from EPUB files (both EPUB2 and EPUB3)
/// using native Rust parsing without GPL-licensed dependencies.
pub struct EpubExtractor;

impl EpubExtractor {
    /// Create a new EPUB extractor.
    pub fn new() -> Self {
        Self
    }
}

impl Default for EpubExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "office")]
impl EpubExtractor {
    /// Build a `DocumentStructure` from the EPUB spine.
    ///
    /// Uses the full HTML structure walker for each chapter's XHTML, preserving
    /// inline formatting annotations (bold, italic, links, etc.) and proper
    /// element types (lists, tables, code blocks).
    ///
    /// Accepts the already-parsed `spine_hrefs` from content extraction to avoid
    /// redundantly re-reading and re-parsing the OPF file from the ZIP archive.
    fn build_document_structure(
        archive: &mut ZipArchive<Cursor<Vec<u8>>>,
        spine_hrefs: &[String],
        manifest_dir: &str,
    ) -> Option<crate::types::document_structure::DocumentStructure> {
        use crate::types::builder::DocumentStructureBuilder;

        let mut builder = DocumentStructureBuilder::new().source_format("epub");

        for (index, href) in spine_hrefs.iter().enumerate() {
            let file_path = resolve_path(manifest_dir, href);
            let xhtml_content = match read_file_from_zip(archive, &file_path) {
                Ok(content) => content,
                Err(_) => continue,
            };

            // Use the HTML structure walker for rich extraction (inline formatting,
            // links, lists, etc.) from each chapter's XHTML content.
            let sanitized = content::strip_doctype_for_title(&xhtml_content);
            let chapter_structure = crate::extraction::html::structure::build_document_structure(&sanitized);

            if chapter_structure.nodes.is_empty() {
                // Fallback: extract plain text if structure walker produces nothing
                let chapter_title =
                    extract_title_from_xhtml(&xhtml_content).unwrap_or_else(|| format!("Chapter {}", index + 1));
                builder.push_heading(1, &chapter_title, None, None);

                let text = extract_text_from_xhtml(&xhtml_content);
                for paragraph in text.split("\n\n") {
                    let trimmed = paragraph.trim();
                    if !trimmed.is_empty() {
                        builder.push_paragraph(trimmed, vec![], None, None);
                    }
                }
            } else {
                // Merge the chapter's nodes into our combined structure.
                // Use push_raw with the node's content layer to preserve structure.
                for node in &chapter_structure.nodes {
                    builder.push_raw(
                        node.content.clone(),
                        None,
                        None,
                        node.content_layer,
                        node.annotations.clone(),
                    );
                }
            }
        }

        Some(builder.build())
    }
}

/// Extract the first heading text from XHTML content.
#[cfg(feature = "office")]
fn extract_title_from_xhtml(xhtml: &str) -> Option<String> {
    // Strip DOCTYPE for roxmltree
    let sanitized = content::strip_doctype_for_title(xhtml);
    let doc = roxmltree::Document::parse(&sanitized).ok()?;

    for node in doc.root().descendants() {
        if node.is_element() {
            let tag = node.tag_name().name().to_ascii_lowercase();
            if matches!(tag.as_str(), "h1" | "h2" | "h3") {
                let text: String = node
                    .descendants()
                    .filter(|n| n.is_text())
                    .filter_map(|n| n.text())
                    .collect::<Vec<_>>()
                    .join("");
                let trimmed = text.trim().to_string();
                if !trimmed.is_empty() {
                    return Some(trimmed);
                }
            }
        }
    }
    None
}

impl Plugin for EpubExtractor {
    fn name(&self) -> &str {
        "epub-extractor"
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
        "Extracts content and metadata from EPUB documents (native Rust implementation with permissive licenses)"
    }

    fn author(&self) -> &str {
        "Kreuzberg Team"
    }
}

#[cfg(feature = "office")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DocumentExtractor for EpubExtractor {
    #[cfg_attr(
        feature = "otel",
        tracing::instrument(
            skip(self, content, config),
            fields(
                extractor.name = self.name(),
                content.size_bytes = content.len(),
            )
        )
    )]
    async fn extract_bytes(
        &self,
        content: &[u8],
        mime_type: &str,
        config: &ExtractionConfig,
    ) -> Result<ExtractionResult> {
        let cursor = Cursor::new(content.to_vec());

        let mut archive = ZipArchive::new(cursor).map_err(|e| crate::KreuzbergError::Parsing {
            message: format!("Failed to open EPUB as ZIP: {}", e),
            source: None,
        })?;

        let container_xml = read_file_from_zip(&mut archive, "META-INF/container.xml")?;
        let opf_path = parse_container_xml(&container_xml)?;

        let manifest_dir = if let Some(last_slash) = opf_path.rfind('/') {
            opf_path[..last_slash].to_string()
        } else {
            String::new()
        };

        let opf_xml = read_file_from_zip(&mut archive, &opf_path)?;

        let (extracted_content, fully_converted, processing_warnings, spine_hrefs) =
            extract_content(&mut archive, &opf_path, &manifest_dir, config)?;

        let (epub_metadata, additional_metadata) = extract_metadata(&opf_xml)?;
        let metadata_map: AHashMap<Cow<'static, str>, serde_json::Value> = additional_metadata
            .into_iter()
            .map(|(k, v)| (Cow::Owned(k), v))
            .collect();

        // Signal that the extractor already formatted the output so the pipeline
        // does not double-convert (mirrors HtmlExtractor behavior).
        let pre_formatted = if fully_converted {
            match config.output_format {
                crate::core::config::OutputFormat::Markdown => Some("markdown".to_string()),
                crate::core::config::OutputFormat::Djot => Some("djot".to_string()),
                _ => None,
            }
        } else {
            None
        };

        // Build document structure from spine chapters (only when requested)
        let document = if config.include_document_structure {
            Self::build_document_structure(&mut archive, &spine_hrefs, &manifest_dir)
        } else {
            None
        };

        Ok(ExtractionResult {
            content: extracted_content,
            mime_type: mime_type.to_string().into(),
            metadata: Metadata {
                title: epub_metadata.title,
                authors: epub_metadata.creator.map(|c| vec![c]),
                language: epub_metadata.language,
                created_at: epub_metadata.date,
                additional: metadata_map,
                output_format: pre_formatted,
                ..Default::default()
            },
            pages: None,
            tables: vec![],
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
            processing_warnings,
            annotations: None,
        })
    }

    fn supported_mime_types(&self) -> &[&str] {
        &[
            "application/epub+zip",
            "application/x-epub+zip",
            "application/vnd.epub+zip",
        ]
    }

    fn priority(&self) -> i32 {
        60
    }
}

#[cfg(all(test, feature = "office"))]
mod tests {
    use super::*;

    #[test]
    fn test_epub_extractor_plugin_interface() {
        let extractor = EpubExtractor::new();
        assert_eq!(extractor.name(), "epub-extractor");
        assert_eq!(extractor.version(), env!("CARGO_PKG_VERSION"));
        assert_eq!(extractor.priority(), 60);
        assert!(!extractor.supported_mime_types().is_empty());
    }

    #[test]
    fn test_epub_extractor_default() {
        let extractor = EpubExtractor;
        assert_eq!(extractor.name(), "epub-extractor");
    }

    #[tokio::test]
    async fn test_epub_extractor_initialize_shutdown() {
        let extractor = EpubExtractor::new();
        assert!(extractor.initialize().is_ok());
        assert!(extractor.shutdown().is_ok());
    }

    #[test]
    fn test_epub_extractor_supported_mime_types() {
        let extractor = EpubExtractor::new();
        let supported = extractor.supported_mime_types();
        assert!(supported.contains(&"application/epub+zip"));
        assert!(supported.contains(&"application/x-epub+zip"));
        assert!(supported.contains(&"application/vnd.epub+zip"));
    }

    #[test]
    fn test_epub_full_dublin_core_metadata() {
        let opf = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:creator>Test Author</dc:creator>
    <dc:language>en</dc:language>
    <dc:coverage>Worldwide</dc:coverage>
    <dc:format>application/epub+zip</dc:format>
    <dc:relation>http://example.com/related</dc:relation>
    <dc:source>Original Manuscript</dc:source>
    <dc:type>Text</dc:type>
    <dc:publisher>Test Publisher</dc:publisher>
    <dc:description>A test book</dc:description>
    <dc:rights>CC BY 4.0</dc:rights>
    <meta name="cover" content="cover-img"/>
  </metadata>
  <manifest>
    <item id="cover-img" href="images/cover.jpg" media-type="image/jpeg"/>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
  </spine>
</package>"#;

        let (epub_meta, additional) = metadata::extract_metadata(opf).expect("Metadata parse failed");
        assert_eq!(epub_meta.title, Some("Test Book".to_string()));
        assert_eq!(epub_meta.coverage, Some("Worldwide".to_string()));
        assert_eq!(epub_meta.format, Some("application/epub+zip".to_string()));
        assert_eq!(epub_meta.relation, Some("http://example.com/related".to_string()));
        assert_eq!(epub_meta.source, Some("Original Manuscript".to_string()));
        assert_eq!(epub_meta.dc_type, Some("Text".to_string()));
        assert_eq!(epub_meta.cover_image_href, Some("images/cover.jpg".to_string()));

        assert!(additional.contains_key("coverage"));
        assert!(additional.contains_key("format"));
        assert!(additional.contains_key("relation"));
        assert!(additional.contains_key("source"));
        assert!(additional.contains_key("type"));
        assert!(additional.contains_key("cover_image"));
    }
}
