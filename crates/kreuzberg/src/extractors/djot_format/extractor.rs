//! Djot document extractor with plugin integration.
//!
//! Implements the DocumentExtractor and Plugin traits for Djot markup files.

use super::parsing::{extract_complete_djot_content, extract_tables_from_events};
use crate::Result;
use crate::core::config::ExtractionConfig;
use crate::plugins::{DocumentExtractor, Plugin};
use crate::types::builder::DocumentStructureBuilder;
use crate::types::document_structure::DocumentStructure;
use crate::types::{ExtractionResult, Metadata};
use async_trait::async_trait;
use jotdown::{Container, Event, Parser};
use std::borrow::Cow;

/// Djot markup extractor with metadata and table support.
///
/// Parses Djot documents with YAML frontmatter, extracting:
/// - Metadata from YAML frontmatter
/// - Plain text content
/// - Tables as structured data
/// - Document structure (headings, links, code blocks)
#[derive(Debug, Clone)]
pub struct DjotExtractor;

impl DjotExtractor {
    /// Create a new Djot extractor.
    pub fn new() -> Self {
        Self
    }
}

impl DjotExtractor {
    /// Build a `DocumentStructure` from jotdown events.
    fn build_document_structure(events: &[Event]) -> DocumentStructure {
        use crate::types::builder;
        use crate::types::document_structure::TextAnnotation;

        let mut b = DocumentStructureBuilder::new().source_format("djot");

        let mut paragraph_text = String::new();
        let mut paragraph_annotations: Vec<TextAnnotation> = Vec::new();
        let mut in_paragraph = false;
        let mut heading_text = String::new();
        let mut heading_level: u8 = 0;
        let mut in_heading = false;
        let mut code_text = String::new();
        let mut code_lang: Option<String> = None;
        let mut in_code_block = false;
        let mut blockquote_depth: u32 = 0;
        let mut in_math = false;
        let mut math_text = String::new();
        let mut list_stack: Vec<(crate::types::document_structure::NodeIndex, bool)> = Vec::new();
        let mut list_item_text = String::new();
        let mut in_list_item = false;
        let mut in_raw_block = false;
        let mut raw_format: Option<String> = None;
        let mut raw_text = String::new();
        let mut in_verbatim = false;
        let mut verbatim_start: u32 = 0;

        // Annotation tracking: stack of (kind_tag, byte_start, optional link url).
        // kind_tag: 0=bold/strong, 1=italic/emphasis, 2=delete/strikethrough, 4=link
        let mut annotation_starts: Vec<(u8, u32, Option<String>)> = Vec::new();

        for event in events {
            match event {
                Event::Start(Container::Heading { level, .. }, _) => {
                    heading_text.clear();
                    heading_level = *level as u8;
                    in_heading = true;
                }
                Event::End(Container::Heading { .. }) => {
                    in_heading = false;
                    let text = heading_text.trim().to_string();
                    if !text.is_empty() {
                        b.push_heading(heading_level, &text, None, None);
                    }
                    heading_text.clear();
                }
                Event::Start(Container::Paragraph, _) => {
                    if !in_heading && !in_list_item {
                        paragraph_text.clear();
                        paragraph_annotations.clear();
                        in_paragraph = true;
                    }
                }
                Event::End(Container::Paragraph) => {
                    if in_paragraph {
                        in_paragraph = false;
                        let text = paragraph_text.trim().to_string();
                        if !text.is_empty() {
                            let trim_offset = paragraph_text.len() - paragraph_text.trim_start().len();
                            let trimmed_len = text.len() as u32;
                            let annotations = if trim_offset > 0 {
                                paragraph_annotations
                                    .drain(..)
                                    .map(|mut a| {
                                        a.start = a.start.saturating_sub(trim_offset as u32);
                                        a.end = a.end.saturating_sub(trim_offset as u32);
                                        a
                                    })
                                    .filter(|a| a.start < a.end && a.end <= trimmed_len)
                                    .collect()
                            } else {
                                paragraph_annotations
                                    .drain(..)
                                    .filter(|a| a.start < a.end && a.end <= trimmed_len)
                                    .collect()
                            };
                            b.push_paragraph(&text, annotations, None, None);
                        }
                        paragraph_text.clear();
                        paragraph_annotations.clear();
                    } else if in_list_item {
                        // paragraph inside list item — text already accumulated
                    }
                }
                // Inline formatting — annotation tracking
                Event::Start(Container::Strong, _) => {
                    if in_paragraph {
                        annotation_starts.push((0, paragraph_text.len() as u32, None));
                    }
                }
                Event::End(Container::Strong) => {
                    if in_paragraph && let Some(pos) = annotation_starts.iter().rposition(|(k, _, _)| *k == 0) {
                        let (_, start, _) = annotation_starts.remove(pos);
                        let end = paragraph_text.len() as u32;
                        if start < end {
                            paragraph_annotations.push(builder::bold(start, end));
                        }
                    }
                }
                Event::Start(Container::Emphasis, _) => {
                    if in_paragraph {
                        annotation_starts.push((1, paragraph_text.len() as u32, None));
                    }
                }
                Event::End(Container::Emphasis) => {
                    if in_paragraph && let Some(pos) = annotation_starts.iter().rposition(|(k, _, _)| *k == 1) {
                        let (_, start, _) = annotation_starts.remove(pos);
                        let end = paragraph_text.len() as u32;
                        if start < end {
                            paragraph_annotations.push(builder::italic(start, end));
                        }
                    }
                }
                Event::Start(Container::Delete, _) => {
                    if in_paragraph {
                        annotation_starts.push((2, paragraph_text.len() as u32, None));
                    }
                }
                Event::End(Container::Delete) => {
                    if in_paragraph && let Some(pos) = annotation_starts.iter().rposition(|(k, _, _)| *k == 2) {
                        let (_, start, _) = annotation_starts.remove(pos);
                        let end = paragraph_text.len() as u32;
                        if start < end {
                            paragraph_annotations.push(builder::strikethrough(start, end));
                        }
                    }
                }
                Event::Start(Container::Verbatim, _) => {
                    if in_paragraph {
                        in_verbatim = true;
                        verbatim_start = paragraph_text.len() as u32;
                    }
                }
                Event::End(Container::Verbatim) => {
                    if in_paragraph && in_verbatim {
                        in_verbatim = false;
                        let end = paragraph_text.len() as u32;
                        if verbatim_start < end {
                            paragraph_annotations.push(builder::code(verbatim_start, end));
                        }
                    }
                }
                Event::Start(Container::Link(url, _), _) => {
                    if in_paragraph {
                        annotation_starts.push((4, paragraph_text.len() as u32, Some(url.to_string())));
                    }
                }
                Event::End(Container::Link(..)) => {
                    if in_paragraph && let Some(pos) = annotation_starts.iter().rposition(|(k, _, _)| *k == 4) {
                        let (_, start, url_opt) = annotation_starts.remove(pos);
                        let end = paragraph_text.len() as u32;
                        if start < end
                            && let Some(url) = url_opt
                        {
                            paragraph_annotations.push(builder::link(start, end, &url, None));
                        }
                    }
                }
                Event::Start(Container::CodeBlock { language }, _) => {
                    code_text.clear();
                    code_lang = if language.is_empty() {
                        None
                    } else {
                        Some(language.to_string())
                    };
                    in_code_block = true;
                }
                Event::End(Container::CodeBlock { .. }) => {
                    in_code_block = false;
                    let text = code_text.trim_end().to_string();
                    if !text.is_empty() {
                        b.push_code(&text, code_lang.as_deref(), None);
                    }
                    code_text.clear();
                    code_lang = None;
                }
                Event::Start(Container::RawBlock { format }, _) => {
                    in_raw_block = true;
                    raw_format = Some(format.to_string());
                    raw_text.clear();
                }
                Event::End(Container::RawBlock { .. }) => {
                    in_raw_block = false;
                    let text = raw_text.trim().to_string();
                    if !text.is_empty() {
                        b.push_raw_block(raw_format.as_deref().unwrap_or("unknown"), &text, None);
                    }
                    raw_text.clear();
                    raw_format = None;
                }
                Event::Start(Container::Blockquote, _) => {
                    b.push_quote(None);
                    blockquote_depth += 1;
                }
                Event::End(Container::Blockquote) => {
                    if blockquote_depth > 0 {
                        b.exit_container();
                        blockquote_depth -= 1;
                    }
                }
                Event::Start(Container::List { kind, .. }, _) => {
                    let ordered = matches!(kind, jotdown::ListKind::Ordered { .. });
                    let list_idx = b.push_list(ordered, None);
                    list_stack.push((list_idx, ordered));
                }
                Event::End(Container::List { .. }) => {
                    list_stack.pop();
                }
                Event::Start(Container::ListItem | Container::TaskListItem { .. }, _) => {
                    list_item_text.clear();
                    in_list_item = true;
                }
                Event::End(Container::ListItem | Container::TaskListItem { .. }) => {
                    in_list_item = false;
                    let text = list_item_text.trim().to_string();
                    if let Some((list_idx, _)) = list_stack.last()
                        && !text.is_empty()
                    {
                        b.push_list_item(*list_idx, &text, None);
                    }
                    list_item_text.clear();
                }
                Event::Start(Container::Math { display }, _) => {
                    if *display {
                        in_math = true;
                        math_text.clear();
                    }
                    // Inline math (display: false): text will be accumulated into
                    // the surrounding context (paragraph, heading, list item) via Str events.
                }
                Event::End(Container::Math { display }) => {
                    if *display {
                        in_math = false;
                        let text = math_text.trim().to_string();
                        if !text.is_empty() {
                            b.push_formula(&text, None);
                        }
                        math_text.clear();
                    }
                }
                Event::Start(Container::Image(..), _) => {
                    // Images in djot — push an image node
                }
                Event::End(Container::Image(..)) => {
                    b.push_image(None, None, None, None);
                }
                Event::FootnoteReference(name) => {
                    b.push_footnote(name, None);
                }
                Event::Str(s) => {
                    if in_code_block {
                        code_text.push_str(s);
                    } else if in_raw_block {
                        raw_text.push_str(s);
                    } else if in_math {
                        math_text.push_str(s);
                    } else if in_heading {
                        heading_text.push_str(s);
                    } else if in_list_item {
                        list_item_text.push_str(s);
                    } else if in_paragraph {
                        paragraph_text.push_str(s);
                    }
                }
                Event::Softbreak => {
                    if in_code_block {
                        code_text.push('\n');
                    } else if in_heading {
                        heading_text.push(' ');
                    } else if in_list_item {
                        list_item_text.push(' ');
                    } else if in_paragraph {
                        paragraph_text.push(' ');
                    }
                }
                Event::Hardbreak => {
                    if in_code_block {
                        code_text.push('\n');
                    } else if in_paragraph {
                        paragraph_text.push('\n');
                    }
                }
                _ => {}
            }
        }

        b.build()
    }
}

impl Default for DjotExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for DjotExtractor {
    fn name(&self) -> &str {
        "djot-extractor"
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
        "Extracts content from Djot markup files with YAML frontmatter and table support"
    }

    fn author(&self) -> &str {
        "Kreuzberg Team"
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DocumentExtractor for DjotExtractor {
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
        let text = String::from_utf8_lossy(content).into_owned();

        let (yaml, remaining_content) = crate::extractors::frontmatter_utils::extract_frontmatter(&text);

        let mut metadata = if let Some(ref yaml_value) = yaml {
            crate::extractors::frontmatter_utils::extract_metadata_from_yaml(yaml_value)
        } else {
            Metadata::default()
        };

        if metadata.title.is_none()
            && !metadata.additional.contains_key("title")
            && let Some(title) = crate::extractors::frontmatter_utils::extract_title_from_content(&remaining_content)
        {
            metadata.title = Some(title.clone());
            // DEPRECATED: kept for backward compatibility; will be removed in next major version.
            metadata.additional.insert(Cow::Borrowed("title"), title.into());
        }

        // Parse with jotdown and collect events once for extraction
        let parser = Parser::new(&remaining_content);
        let events: Vec<Event> = parser.collect();

        let tables = extract_tables_from_events(&events);

        // Extract complete djot content with all features
        let djot_content = extract_complete_djot_content(&events, metadata.clone(), tables.clone());

        let document = if config.include_document_structure {
            Some(Self::build_document_structure(&events))
        } else {
            None
        };

        // Use the raw source (after frontmatter stripping) as content to preserve
        // table structures, formatting, and all original text verbatim.
        // Structured extraction goes into djot_content.
        Ok(ExtractionResult {
            content: remaining_content.to_string(),
            mime_type: mime_type.to_string().into(),
            metadata,
            tables,
            detected_languages: None,
            chunks: None,
            images: None,
            pages: None,
            djot_content: Some(djot_content),
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
        &["text/djot", "text/x-djot"]
    }

    fn priority(&self) -> i32 {
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_djot_extractor_creation() {
        let extractor = DjotExtractor::new();
        assert_eq!(extractor.name(), "djot-extractor");
    }

    #[test]
    fn test_can_extract_djot_mime_types() {
        let extractor = DjotExtractor::new();
        let mime_types = extractor.supported_mime_types();

        assert!(mime_types.contains(&"text/djot"));
        assert!(mime_types.contains(&"text/x-djot"));
    }

    #[test]
    fn test_plugin_interface() {
        let extractor = DjotExtractor::new();
        assert_eq!(extractor.author(), "Kreuzberg Team");
        assert!(!extractor.version().is_empty());
        assert!(!extractor.description().is_empty());
    }

    #[tokio::test]
    async fn test_extract_simple_djot() {
        let content =
            b"# Header\n\nThis is a paragraph with *bold* and _italic_ text.\n\n## Subheading\n\nMore content here.";
        let extractor = DjotExtractor::new();
        let config = ExtractionConfig::default();

        let result = extractor.extract_bytes(content, "text/djot", &config).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.content.contains("Header"));
        assert!(result.content.contains("This is a paragraph"));
        assert!(result.content.contains("bold"));
        assert!(result.content.contains("italic"));
    }
}
