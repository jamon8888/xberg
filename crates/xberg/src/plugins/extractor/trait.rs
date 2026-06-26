//! Document extractor plugin trait.
//!
//! This module defines the trait for implementing custom document extractors.

use crate::Result;
use crate::core::config::{ExtractInput, ExtractInputKind, ExtractionConfig};
use crate::plugins::Plugin;
use crate::types::ExtractionResult;
use crate::types::internal::InternalDocument;
use async_trait::async_trait;
use std::path::Path;

#[cfg(not(feature = "tokio-runtime"))]
use crate::XbergError;

/// Trait for document extractor plugins.
///
/// Implement this trait to add support for new document formats or override
/// built-in extraction behavior. Foreign-language bindings expose the
/// [`DocumentExtractor::extract`] method, which accepts [`ExtractInput`] and
/// returns an [`ExtractionResult`].
///
/// Native Rust extractors can override the skipped internal byte/file methods
/// below to participate in the full pipeline representation.
///
/// # Priority System
///
/// When multiple extractors support the same MIME type, the registry selects
/// the extractor with the highest priority value. Use this to:
/// - Override built-in extractors (priority > 50)
/// - Provide fallback extractors (priority < 50)
/// - Implement specialized extractors for specific use cases
///
/// Default priority is 50.
///
/// # Thread Safety
///
/// Extractors must be thread-safe (`Send + Sync`) to support concurrent extraction.
///
/// # Example
///
/// ```rust
/// use xberg::plugins::{Plugin, DocumentExtractor};
/// use xberg::{ExtractInput, ExtractionConfig, ExtractionResult, Result};
/// use async_trait::async_trait;
///
/// struct CustomTextExtractor;
///
/// impl Plugin for CustomTextExtractor {
///     fn name(&self) -> &str { "custom-text" }
///     fn version(&self) -> String { "1.0.0".to_string() }
///     fn initialize(&self) -> Result<()> { Ok(()) }
///     fn shutdown(&self) -> Result<()> { Ok(()) }
/// }
///
/// #[async_trait]
/// impl DocumentExtractor for CustomTextExtractor {
///     async fn extract(&self, input: ExtractInput, _config: &ExtractionConfig)
///         -> Result<ExtractionResult> {
///         let bytes = input.bytes.unwrap_or_default();
///         Ok(ExtractionResult {
///             content: String::from_utf8_lossy(&bytes).to_string(),
///             mime_type: "text/plain".into(),
///             ..Default::default()
///         })
///     }
///
///     fn supported_mime_types(&self) -> &[&str] {
///         &["text/plain"]
///     }
/// }
/// ```
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait DocumentExtractor: Plugin {
    /// Binding-safe extraction entry point for foreign-language plugin bridges.
    ///
    /// This is the only document-extractor method generated into language
    /// bindings. It accepts the same unified input shape as the public
    /// extraction API and returns one extracted document result.
    async fn extract(&self, input: ExtractInput, config: &ExtractionConfig) -> Result<ExtractionResult> {
        let doc = match input.kind {
            ExtractInputKind::Bytes => {
                let bytes = input.bytes.ok_or_else(|| {
                    crate::XbergError::validation(
                        "document extractor input kind 'bytes' requires the 'bytes' field".to_string(),
                    )
                })?;
                let mime_type = input.mime_type.as_deref().unwrap_or("application/octet-stream");
                self.extract_bytes(&bytes, mime_type, config).await?
            }
            ExtractInputKind::Uri => {
                let uri = input.uri.ok_or_else(|| {
                    crate::XbergError::validation(
                        "document extractor input kind 'uri' requires the 'uri' field".to_string(),
                    )
                })?;
                let mime_type = input.mime_type.as_deref().unwrap_or("application/octet-stream");
                self.extract_file(Path::new(&uri), mime_type, config).await?
            }
        };

        Ok(crate::extraction::derive::derive_extraction_result(
            doc,
            config.include_document_structure,
            config.output_format.clone(),
        ))
    }

    /// Extract content from a byte array.
    ///
    /// This is the core extraction method that processes in-memory document data.
    ///
    /// # Arguments
    ///
    /// * `content` - Raw document bytes
    /// * `mime_type` - MIME type of the document (already validated)
    /// * `config` - Extraction configuration
    ///
    /// # Returns
    ///
    /// An `InternalDocument` containing the extracted elements, metadata, and tables.
    /// The pipeline will convert this into the public `ExtractionResult`.
    ///
    /// # Errors
    ///
    /// - `XbergError::Parsing` - Document parsing failed
    /// - `XbergError::Validation` - Invalid document structure
    /// - `XbergError::Io` - I/O errors (these always bubble up)
    /// - `XbergError::MissingDependency` - Required dependency not available
    #[cfg_attr(alef, alef(skip))]
    async fn extract_bytes(
        &self,
        content: &[u8],
        mime_type: &str,
        config: &ExtractionConfig,
    ) -> Result<InternalDocument> {
        let result = self
            .extract(
                ExtractInput::from_bytes(content.to_vec(), mime_type.to_string(), None),
                config,
            )
            .await?;
        Ok(result.into())
    }

    /// Extract content from a file.
    ///
    /// Default implementation reads the file and calls `extract_bytes`.
    /// Override for custom file handling, streaming, or memory optimizations.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the document file
    /// * `mime_type` - MIME type of the document (already validated)
    /// * `config` - Extraction configuration
    ///
    /// # Returns
    ///
    /// An `InternalDocument` containing the extracted elements, metadata, and tables.
    ///
    /// # Errors
    ///
    /// Same as `extract_bytes`, plus file I/O errors.
    #[cfg_attr(alef, alef(skip))]
    async fn extract_file(&self, path: &Path, mime_type: &str, config: &ExtractionConfig) -> Result<InternalDocument> {
        #[cfg(feature = "tokio-runtime")]
        {
            use crate::core::io;
            // Use memory-mapped I/O for large files (> 1 MiB) to avoid an extra
            // heap allocation.  `open_file_bytes` falls back to a plain read for
            // small files and on WASM where mmap is unavailable.
            let bytes = io::open_file_bytes(path)?;
            self.extract_bytes(&bytes, mime_type, config).await
        }
        #[cfg(not(feature = "tokio-runtime"))]
        {
            let _ = (path, mime_type, config);
            Err(XbergError::Other(
                "File-based extraction requires the tokio-runtime feature".to_string(),
            ))
        }
    }

    /// Get the list of MIME types supported by this extractor.
    ///
    /// Can include exact MIME types and prefix patterns:
    /// - Exact: `"application/pdf"`, `"text/plain"`
    /// - Prefix: `"image/*"` (matches any image type)
    ///
    /// # Returns
    ///
    /// A slice of MIME type strings.
    fn supported_mime_types(&self) -> &[&str];

    /// Get the priority of this extractor.
    ///
    /// Higher priority extractors are preferred when multiple extractors
    /// support the same MIME type.
    ///
    /// # Priority Guidelines
    ///
    /// - **0-25**: Fallback/low-quality extractors
    /// - **26-49**: Alternative extractors
    /// - **50**: Default priority (built-in extractors)
    /// - **51-75**: Premium/enhanced extractors
    /// - **76-100**: Specialized/high-priority extractors
    ///
    /// # Returns
    ///
    /// Priority value (default: 50)
    fn priority(&self) -> i32 {
        50
    }

    /// Optional: Check if this extractor can handle a specific file.
    ///
    /// Allows for more sophisticated detection beyond MIME types.
    /// Defaults to `true` (rely on MIME type matching).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to check
    /// * `mime_type` - Detected MIME type
    ///
    /// # Returns
    ///
    /// `true` if the extractor can handle this file, `false` otherwise.
    fn can_handle(&self, _path: &Path, _mime_type: &str) -> bool {
        true
    }

    /// Attempt to get a reference to this extractor as a SyncExtractor.
    ///
    /// Returns None if the extractor doesn't support synchronous extraction.
    /// This is used for WASM and other sync-only environments.
    #[doc(hidden)]
    fn as_sync_extractor(&self) -> Option<&dyn crate::extractors::SyncExtractor> {
        None
    }
}
