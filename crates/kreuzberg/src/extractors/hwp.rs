//! Hangul Word Processor (.hwp) extractor.
//!
//! Extracts text content from HWP 5.0 documents using the `hwpers` crate.

use crate::Result;
use crate::core::config::ExtractionConfig;
use crate::plugins::{DocumentExtractor, Plugin};
use crate::types::{ExtractionResult, Metadata};
use async_trait::async_trait;

/// Extractor for Hangul Word Processor (.hwp) files.
///
/// Supports HWP 5.0 format, the standard document format in South Korea.
pub struct HwpExtractor;

impl HwpExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HwpExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for HwpExtractor {
    fn name(&self) -> &str {
        "hwp-extractor"
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
        "Hangul Word Processor (.hwp) text extraction"
    }

    fn author(&self) -> &str {
        "Kreuzberg Team"
    }
}

fn extract_hwp_content(content: &[u8]) -> Result<String> {
    let document = hwpers::HwpReader::from_bytes(content)
        .map_err(|e| crate::KreuzbergError::parsing(format!("Failed to read HWP file: {e}")))?;

    Ok(document.extract_text())
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DocumentExtractor for HwpExtractor {
    async fn extract_bytes(
        &self,
        content: &[u8],
        mime_type: &str,
        _config: &ExtractionConfig,
    ) -> Result<ExtractionResult> {
        let text = extract_hwp_content(content)?;

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
        &["application/x-hwp", "application/haansofthwpx"]
    }

    fn priority(&self) -> i32 {
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hwp_extractor_plugin_interface() {
        let extractor = HwpExtractor::new();
        assert_eq!(extractor.name(), "hwp-extractor");
        assert_eq!(extractor.version(), env!("CARGO_PKG_VERSION"));
        assert_eq!(extractor.priority(), 50);
        assert_eq!(
            extractor.supported_mime_types(),
            &["application/x-hwp", "application/haansofthwpx"]
        );
    }

    #[test]
    fn test_hwp_extractor_initialize_shutdown() {
        let extractor = HwpExtractor::new();
        assert!(extractor.initialize().is_ok());
        assert!(extractor.shutdown().is_ok());
    }
}
