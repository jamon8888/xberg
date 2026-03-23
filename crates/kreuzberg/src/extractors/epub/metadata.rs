//! Metadata extraction from EPUB OPF files.
//!
//! Handles parsing of OPF (Open Packaging Format) files and extraction of
//! Dublin Core metadata following EPUB2 and EPUB3 standards.

use crate::Result;
use roxmltree;
use std::collections::BTreeMap;

/// Metadata extracted from OPF (Open Packaging Format) file
#[derive(Debug, Default, Clone)]
pub(super) struct OepbMetadata {
    pub(super) title: Option<String>,
    pub(super) creator: Option<String>,
    pub(super) date: Option<String>,
    pub(super) language: Option<String>,
    pub(super) identifier: Option<String>,
    pub(super) publisher: Option<String>,
    pub(super) subject: Option<String>,
    pub(super) description: Option<String>,
    pub(super) rights: Option<String>,
    pub(super) coverage: Option<String>,
    pub(super) format: Option<String>,
    pub(super) relation: Option<String>,
    pub(super) source: Option<String>,
    pub(super) dc_type: Option<String>,
    pub(super) cover_image_href: Option<String>,
}

/// Extract metadata from EPUB OPF file
pub(super) fn extract_metadata(opf_xml: &str) -> Result<(OepbMetadata, BTreeMap<String, serde_json::Value>)> {
    let mut additional_metadata = BTreeMap::new();

    let (epub_metadata, _) = parse_opf(opf_xml)?;

    if let Some(ref identifier) = epub_metadata.identifier {
        additional_metadata.insert("identifier".to_string(), serde_json::json!(identifier.clone()));
    }

    if let Some(ref publisher) = epub_metadata.publisher {
        additional_metadata.insert("publisher".to_string(), serde_json::json!(publisher.clone()));
    }

    if let Some(ref subject) = epub_metadata.subject {
        additional_metadata.insert("subject".to_string(), serde_json::json!(subject.clone()));
    }

    if let Some(ref description) = epub_metadata.description {
        additional_metadata.insert("description".to_string(), serde_json::json!(description.clone()));
    }

    if let Some(ref rights) = epub_metadata.rights {
        additional_metadata.insert("rights".to_string(), serde_json::json!(rights.clone()));
    }

    if let Some(ref coverage) = epub_metadata.coverage {
        additional_metadata.insert("coverage".to_string(), serde_json::json!(coverage.clone()));
    }

    if let Some(ref format) = epub_metadata.format {
        additional_metadata.insert("format".to_string(), serde_json::json!(format.clone()));
    }

    if let Some(ref relation) = epub_metadata.relation {
        additional_metadata.insert("relation".to_string(), serde_json::json!(relation.clone()));
    }

    if let Some(ref source) = epub_metadata.source {
        additional_metadata.insert("source".to_string(), serde_json::json!(source.clone()));
    }

    if let Some(ref dc_type) = epub_metadata.dc_type {
        additional_metadata.insert("type".to_string(), serde_json::json!(dc_type.clone()));
    }

    if let Some(ref cover_href) = epub_metadata.cover_image_href {
        additional_metadata.insert("cover_image".to_string(), serde_json::json!(cover_href.clone()));
    }

    Ok((epub_metadata, additional_metadata))
}

/// Parse OPF file and extract metadata and spine order
pub(super) fn parse_opf(xml: &str) -> Result<(OepbMetadata, Vec<String>)> {
    match roxmltree::Document::parse(xml) {
        Ok(doc) => {
            let root = doc.root();

            let mut metadata = OepbMetadata::default();
            let mut manifest: BTreeMap<String, String> = BTreeMap::new();
            let mut spine_order: Vec<String> = Vec::new();

            for node in root.descendants() {
                match node.tag_name().name() {
                    "title" => {
                        if let Some(text) = node.text() {
                            metadata.title = Some(text.trim().to_string());
                        }
                    }
                    "creator" => {
                        if let Some(text) = node.text() {
                            metadata.creator = Some(text.trim().to_string());
                        }
                    }
                    "date" => {
                        if let Some(text) = node.text() {
                            metadata.date = Some(text.trim().to_string());
                        }
                    }
                    "language" => {
                        if let Some(text) = node.text() {
                            metadata.language = Some(text.trim().to_string());
                        }
                    }
                    "identifier" => {
                        if let Some(text) = node.text() {
                            metadata.identifier = Some(text.trim().to_string());
                        }
                    }
                    "publisher" => {
                        if let Some(text) = node.text() {
                            metadata.publisher = Some(text.trim().to_string());
                        }
                    }
                    "subject" => {
                        if let Some(text) = node.text() {
                            metadata.subject = Some(text.trim().to_string());
                        }
                    }
                    "description" => {
                        if let Some(text) = node.text() {
                            metadata.description = Some(text.trim().to_string());
                        }
                    }
                    "rights" => {
                        if let Some(text) = node.text() {
                            metadata.rights = Some(text.trim().to_string());
                        }
                    }
                    "coverage" => {
                        if let Some(text) = node.text() {
                            metadata.coverage = Some(text.trim().to_string());
                        }
                    }
                    "format" => {
                        if let Some(text) = node.text() {
                            metadata.format = Some(text.trim().to_string());
                        }
                    }
                    "relation" => {
                        if let Some(text) = node.text() {
                            metadata.relation = Some(text.trim().to_string());
                        }
                    }
                    "source" => {
                        if let Some(text) = node.text() {
                            metadata.source = Some(text.trim().to_string());
                        }
                    }
                    "type" => {
                        if let Some(text) = node.text() {
                            metadata.dc_type = Some(text.trim().to_string());
                        }
                    }
                    "item" => {
                        if let Some(id) = node.attribute("id")
                            && let Some(href) = node.attribute("href")
                        {
                            manifest.insert(id.to_string(), href.to_string());
                        }
                    }
                    _ => {}
                }
            }

            // Find cover image via <meta name="cover" content="item-id"/>
            let mut cover_item_id = None;
            for node in root.descendants() {
                if node.tag_name().name() == "meta"
                    && node.attribute("name") == Some("cover")
                    && let Some(content) = node.attribute("content")
                {
                    cover_item_id = Some(content.to_string());
                    break;
                }
            }

            if let Some(cover_id) = cover_item_id {
                if let Some(href) = manifest.get(&cover_id) {
                    metadata.cover_image_href = Some(href.clone());
                }
            }

            for node in root.descendants() {
                if node.tag_name().name() == "itemref"
                    && let Some(idref) = node.attribute("idref")
                    && let Some(href) = manifest.get(idref)
                {
                    spine_order.push(href.clone());
                }
            }

            Ok((metadata, spine_order))
        }
        Err(e) => Err(crate::KreuzbergError::Parsing {
            message: format!("Failed to parse OPF file: {}", e),
            source: None,
        }),
    }
}
