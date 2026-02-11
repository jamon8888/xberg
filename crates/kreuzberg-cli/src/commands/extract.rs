//! Extract command - Extract text and data from documents
//!
//! This module provides the extract and batch extract commands for processing single
//! or multiple documents with customizable extraction configurations.

use anyhow::{Context, Result};
use kreuzberg::{
    ChunkingConfig, ExtractionConfig, LanguageDetectionConfig, OcrConfig, batch_extract_file_sync, extract_file_sync,
};
use std::path::PathBuf;

use crate::{ContentOutputFormatArg, OutputFormat};

/// Execute single document extraction command
pub fn extract_command(
    path: PathBuf,
    config: ExtractionConfig,
    mime_type: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let path_str = path.to_string_lossy().to_string();

    let result = extract_file_sync(&path_str, mime_type.as_deref(), &config).with_context(|| {
        format!(
            "Failed to extract file '{}'. Ensure the file is readable and the format is supported.",
            path.display()
        )
    })?;

    match format {
        OutputFormat::Text => {
            println!("{}", result.content);
        }
        OutputFormat::Json => {
            // Serialize the full ExtractionResult including chunks, images, elements, etc.
            println!(
                "{}",
                serde_json::to_string_pretty(&result).context("Failed to serialize extraction result to JSON")?
            );
        }
    }

    Ok(())
}

/// Execute batch extraction command
pub fn batch_command(paths: Vec<PathBuf>, config: ExtractionConfig, format: OutputFormat) -> Result<()> {
    let path_strs: Vec<String> = paths.iter().map(|p| p.to_string_lossy().to_string()).collect();

    let results = batch_extract_file_sync(path_strs, &config).with_context(|| {
        format!(
            "Failed to batch extract {} documents. Check that all files are readable and formats are supported.",
            paths.len()
        )
    })?;

    match format {
        OutputFormat::Text => {
            for (i, result) in results.iter().enumerate() {
                println!("=== Document {} ===", i + 1);
                println!("MIME Type: {}", result.mime_type);
                println!("Content:\n{}", result.content);
                println!();
            }
        }
        OutputFormat::Json => {
            // Serialize the full ExtractionResult for each document
            println!(
                "{}",
                serde_json::to_string_pretty(&results)
                    .context("Failed to serialize batch extraction results to JSON")?
            );
        }
    }

    Ok(())
}

/// Apply extraction CLI overrides to config
///
/// # Deprecation Notices
///
/// - `output_format` (via `--output-format`): Recommended for all new code
/// - `content_format` (via `--content-format`): Deprecated since 4.2.0, use `--output-format` instead
#[allow(clippy::too_many_arguments)]
pub fn apply_extraction_overrides(
    config: &mut ExtractionConfig,
    ocr: Option<bool>,
    ocr_backend: Option<&str>,
    force_ocr: Option<bool>,
    no_cache: Option<bool>,
    chunk: Option<bool>,
    chunk_size: Option<usize>,
    chunk_overlap: Option<usize>,
    quality: Option<bool>,
    detect_language: Option<bool>,
    output_format: Option<ContentOutputFormatArg>,
    content_format: Option<ContentOutputFormatArg>,
) {
    if let Some(ocr_flag) = ocr {
        if ocr_flag {
            let (backend, language) = match ocr_backend {
                Some("paddle-ocr") => ("paddle-ocr", "en"),
                Some("easyocr") => ("easyocr", "en"),
                _ => ("tesseract", "eng"),
            };
            // Preserve existing paddle_ocr_config and element_config from config file/inline JSON
            let existing_paddle_config = config.ocr.as_ref().and_then(|o| o.paddle_ocr_config.clone());
            let existing_element_config = config.ocr.as_ref().and_then(|o| o.element_config.clone());
            config.ocr = Some(OcrConfig {
                backend: backend.to_string(),
                language: language.to_string(),
                tesseract_config: None,
                output_format: None,
                paddle_ocr_config: existing_paddle_config,
                element_config: existing_element_config,
            });
        } else {
            config.ocr = None;
        }
    }
    if let Some(force_ocr_flag) = force_ocr {
        config.force_ocr = force_ocr_flag;
    }
    if let Some(no_cache_flag) = no_cache {
        config.use_cache = !no_cache_flag;
    }
    if let Some(chunk_flag) = chunk {
        if chunk_flag {
            let max_characters = chunk_size.unwrap_or(1000);
            let overlap = chunk_overlap.unwrap_or(200);
            config.chunking = Some(ChunkingConfig {
                max_characters,
                overlap,
                trim: true,
                chunker_type: kreuzberg::chunking::ChunkerType::Text,
                embedding: None,
                preset: None,
            });
        } else {
            config.chunking = None;
        }
    } else if let Some(ref mut chunking) = config.chunking {
        if let Some(max_characters) = chunk_size {
            chunking.max_characters = max_characters;
        }
        if let Some(overlap) = chunk_overlap {
            chunking.overlap = overlap;
        }
    }
    if let Some(quality_flag) = quality {
        config.enable_quality_processing = quality_flag;
    }
    if let Some(detect_language_flag) = detect_language {
        if detect_language_flag {
            config.language_detection = Some(LanguageDetectionConfig {
                enabled: true,
                min_confidence: 0.8,
                detect_multiple: false,
            });
        } else {
            config.language_detection = None;
        }
    }

    // Handle output format with deprecation warning for --content-format
    let final_output_format = output_format.or_else(|| {
        if content_format.is_some() {
            eprintln!("warning: '--content-format' is deprecated since 4.2.0, use '--output-format' instead");
        }
        content_format
    });

    if let Some(content_fmt) = final_output_format {
        config.output_format = content_fmt.into();
    }
}
