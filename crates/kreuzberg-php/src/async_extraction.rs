//! Async extraction functions for PHP.
//!
//! Provides async variants of all extraction functions that return
//! `DeferredResult` objects. The actual extraction runs on a background
//! Tokio worker thread pool.

use ext_php_rs::binary_slice::BinarySlice;
use ext_php_rs::prelude::*;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::WORKER_RUNTIME;
use crate::config::{parse_config_from_json, parse_file_config_from_json};
use crate::deferred::{DeferredInner, DeferredResult};
use crate::extraction::should_extract_tables;

/// Extract content from a file asynchronously.
///
/// Returns a `DeferredResult` immediately. The extraction runs on a
/// background Tokio worker thread.
///
/// # Parameters
///
/// - `path` (string): Path to the file to extract
/// - `mime_type` (string|null): Optional MIME type hint (auto-detected if null)
/// - `config_json` (string|null): JSON-encoded extraction configuration
///
/// # Returns
///
/// DeferredResult that can be polled or waited on.
///
/// # Example
///
/// ```php
/// $deferred = kreuzberg_extract_file_async("document.pdf");
/// $result = $deferred->getResult(); // blocks until ready
/// echo $result->content;
/// ```
#[php_function]
pub fn kreuzberg_extract_file_async(
    path: String,
    mime_type: Option<String>,
    config_json: Option<String>,
) -> PhpResult<DeferredResult> {
    let rust_config = match &config_json {
        Some(json) => parse_config_from_json(json).map_err(PhpException::from)?,
        None => Default::default(),
    };

    let extract_tables = should_extract_tables(&config_json)?;

    let slot = Arc::new(Mutex::new(DeferredInner::Single(None)));
    let slot_clone = Arc::clone(&slot);

    WORKER_RUNTIME.spawn(async move {
        let result = kreuzberg::extract_file(&path, mime_type.as_deref(), &rust_config)
            .await
            .map_err(|e| e.to_string());

        *slot_clone.lock() = DeferredInner::Single(Some(result));
    });

    Ok(DeferredResult::new_single(slot, extract_tables))
}

/// Extract content from bytes asynchronously.
///
/// # Parameters
///
/// - `data` (string): Binary data to extract
/// - `mime_type` (string): MIME type of the data
/// - `config_json` (string|null): JSON-encoded extraction configuration
///
/// # Returns
///
/// DeferredResult that can be polled or waited on.
///
/// # Example
///
/// ```php
/// $data = file_get_contents("document.pdf");
/// $deferred = kreuzberg_extract_bytes_async($data, "application/pdf");
/// $result = $deferred->getResult();
/// ```
#[php_function]
pub fn kreuzberg_extract_bytes_async(
    data: BinarySlice<u8>,
    mime_type: String,
    config_json: Option<String>,
) -> PhpResult<DeferredResult> {
    let rust_config = match &config_json {
        Some(json) => parse_config_from_json(json).map_err(PhpException::from)?,
        None => Default::default(),
    };

    let extract_tables = should_extract_tables(&config_json)?;

    // Copy the data since we need to send it to the async task
    let bytes: &[u8] = data.as_ref();
    let data_owned: Vec<u8> = bytes.to_vec();

    let slot = Arc::new(Mutex::new(DeferredInner::Single(None)));
    let slot_clone = Arc::clone(&slot);

    WORKER_RUNTIME.spawn(async move {
        let result = kreuzberg::extract_bytes(&data_owned, &mime_type, &rust_config)
            .await
            .map_err(|e| e.to_string());

        *slot_clone.lock() = DeferredInner::Single(Some(result));
    });

    Ok(DeferredResult::new_single(slot, extract_tables))
}

/// Batch extract content from multiple files asynchronously.
///
/// # Parameters
///
/// - `paths` (array): Array of file paths
/// - `config_json` (string|null): JSON-encoded extraction configuration
///
/// # Returns
///
/// DeferredResult with batch results (use getResults() to retrieve).
///
/// # Example
///
/// ```php
/// $deferred = kreuzberg_batch_extract_files_async(["doc1.pdf", "doc2.docx"]);
/// $results = $deferred->getResults();
/// ```
#[php_function]
pub fn kreuzberg_batch_extract_files_async(
    paths: Vec<String>,
    config_json: Option<String>,
) -> PhpResult<DeferredResult> {
    let rust_config = match &config_json {
        Some(json) => parse_config_from_json(json).map_err(PhpException::from)?,
        None => Default::default(),
    };

    let extract_tables = should_extract_tables(&config_json)?;

    let slot = Arc::new(Mutex::new(DeferredInner::Batch(None)));
    let slot_clone = Arc::clone(&slot);

    WORKER_RUNTIME.spawn(async move {
        let result = kreuzberg::batch_extract_file(paths, &rust_config)
            .await
            .map_err(|e| e.to_string());

        *slot_clone.lock() = DeferredInner::Batch(Some(result));
    });

    Ok(DeferredResult::new_batch(slot, extract_tables))
}

/// Batch extract content from multiple byte arrays asynchronously.
///
/// # Parameters
///
/// - `data_list` (array): Array of binary data
/// - `mime_types` (array): Array of MIME types (one per data element)
/// - `config_json` (string|null): JSON-encoded extraction configuration
///
/// # Returns
///
/// DeferredResult with batch results (use getResults() to retrieve).
///
/// # Example
///
/// ```php
/// $deferred = kreuzberg_batch_extract_bytes_async(
///     [$data1, $data2],
///     ["application/pdf", "application/pdf"],
/// );
/// $results = $deferred->getResults();
/// ```
#[php_function]
pub fn kreuzberg_batch_extract_bytes_async(
    data_list: Vec<BinarySlice<u8>>,
    mime_types: Vec<String>,
    config_json: Option<String>,
) -> PhpResult<DeferredResult> {
    if data_list.len() != mime_types.len() {
        return Err(format!(
            "data_list and mime_types must have the same length (got {} and {})",
            data_list.len(),
            mime_types.len()
        )
        .into());
    }

    let rust_config = match &config_json {
        Some(json) => parse_config_from_json(json).map_err(PhpException::from)?,
        None => Default::default(),
    };

    let extract_tables = should_extract_tables(&config_json)?;

    // Copy data since BinarySlice borrows from PHP memory
    let owned_contents: Vec<(Vec<u8>, String)> = data_list
        .into_iter()
        .zip(mime_types)
        .map(|(binary_slice, mime)| {
            let bytes: &[u8] = binary_slice.as_ref();
            (bytes.to_vec(), mime)
        })
        .collect();

    let slot = Arc::new(Mutex::new(DeferredInner::Batch(None)));
    let slot_clone = Arc::clone(&slot);

    WORKER_RUNTIME.spawn(async move {
        let result = kreuzberg::batch_extract_bytes(owned_contents, &rust_config)
            .await
            .map_err(|e| e.to_string());

        *slot_clone.lock() = DeferredInner::Batch(Some(result));
    });

    Ok(DeferredResult::new_batch(slot, extract_tables))
}

/// Batch extract content from multiple files asynchronously with per-file config overrides.
///
/// # Parameters
///
/// - `paths` (array): Array of file paths
/// - `file_configs_json` (array): Array of JSON-encoded per-file configs (string|null per element)
/// - `config_json` (string|null): JSON-encoded base extraction configuration
///
/// # Returns
///
/// DeferredResult with batch results (use getResults() to retrieve).
///
/// # Example
///
/// ```php
/// $deferred = kreuzberg_batch_extract_files_with_configs_async(
///     ["doc1.pdf", "doc2.docx"],
///     ['{"force_ocr": true}', null],
/// );
/// $results = $deferred->getResults();
/// ```
#[php_function]
pub fn kreuzberg_batch_extract_files_with_configs_async(
    paths: Vec<String>,
    file_configs_json: Vec<Option<String>>,
    config_json: Option<String>,
) -> PhpResult<DeferredResult> {
    if paths.len() != file_configs_json.len() {
        return Err(format!(
            "paths and file_configs_json must have the same length (got {} and {})",
            paths.len(),
            file_configs_json.len()
        )
        .into());
    }

    let rust_config = match &config_json {
        Some(json) => parse_config_from_json(json).map_err(PhpException::from)?,
        None => Default::default(),
    };

    let extract_tables = should_extract_tables(&config_json)?;

    let items: Vec<(std::path::PathBuf, Option<kreuzberg::FileExtractionConfig>)> = paths
        .into_iter()
        .zip(file_configs_json)
        .map(|(path, fc_json)| {
            let fc = parse_file_config_from_json(&fc_json).map_err(PhpException::from)?;
            Ok((std::path::PathBuf::from(path), fc))
        })
        .collect::<PhpResult<Vec<_>>>()?;

    let slot = Arc::new(Mutex::new(DeferredInner::Batch(None)));
    let slot_clone = Arc::clone(&slot);

    WORKER_RUNTIME.spawn(async move {
        let result = kreuzberg::batch_extract_file_with_configs(items, &rust_config)
            .await
            .map_err(|e| e.to_string());

        *slot_clone.lock() = DeferredInner::Batch(Some(result));
    });

    Ok(DeferredResult::new_batch(slot, extract_tables))
}

/// Batch extract content from multiple byte arrays asynchronously with per-file config overrides.
///
/// # Parameters
///
/// - `data_list` (array): Array of binary data
/// - `mime_types` (array): Array of MIME types (one per data element)
/// - `file_configs_json` (array): Array of JSON-encoded per-file configs (string|null per element)
/// - `config_json` (string|null): JSON-encoded base extraction configuration
///
/// # Returns
///
/// DeferredResult with batch results (use getResults() to retrieve).
///
/// # Example
///
/// ```php
/// $deferred = kreuzberg_batch_extract_bytes_with_configs_async(
///     [$data1, $data2],
///     ["application/pdf", "application/pdf"],
///     ['{"force_ocr": true}', null],
/// );
/// $results = $deferred->getResults();
/// ```
#[php_function]
pub fn kreuzberg_batch_extract_bytes_with_configs_async(
    data_list: Vec<BinarySlice<u8>>,
    mime_types: Vec<String>,
    file_configs_json: Vec<Option<String>>,
    config_json: Option<String>,
) -> PhpResult<DeferredResult> {
    if data_list.len() != mime_types.len() || data_list.len() != file_configs_json.len() {
        return Err(format!(
            "data_list, mime_types, and file_configs_json must have the same length (got {}, {}, and {})",
            data_list.len(),
            mime_types.len(),
            file_configs_json.len()
        )
        .into());
    }

    let rust_config = match &config_json {
        Some(json) => parse_config_from_json(json).map_err(PhpException::from)?,
        None => Default::default(),
    };

    let extract_tables = should_extract_tables(&config_json)?;

    let items: Vec<(Vec<u8>, String, Option<kreuzberg::FileExtractionConfig>)> = data_list
        .into_iter()
        .zip(mime_types)
        .zip(file_configs_json)
        .map(|((binary_slice, mime), fc_json)| {
            let fc = parse_file_config_from_json(&fc_json).map_err(PhpException::from)?;
            let bytes: &[u8] = binary_slice.as_ref();
            Ok((bytes.to_vec(), mime, fc))
        })
        .collect::<PhpResult<Vec<_>>>()?;

    let slot = Arc::new(Mutex::new(DeferredInner::Batch(None)));
    let slot_clone = Arc::clone(&slot);

    WORKER_RUNTIME.spawn(async move {
        let result = kreuzberg::batch_extract_bytes_with_configs(items, &rust_config)
            .await
            .map_err(|e| e.to_string());

        *slot_clone.lock() = DeferredInner::Batch(Some(result));
    });

    Ok(DeferredResult::new_batch(slot, extract_tables))
}

/// Returns all function builders for the async extraction module.
pub fn get_function_builders() -> Vec<ext_php_rs::builders::FunctionBuilder<'static>> {
    vec![
        wrap_function!(kreuzberg_extract_file_async),
        wrap_function!(kreuzberg_extract_bytes_async),
        wrap_function!(kreuzberg_batch_extract_files_async),
        wrap_function!(kreuzberg_batch_extract_bytes_async),
        wrap_function!(kreuzberg_batch_extract_files_with_configs_async),
        wrap_function!(kreuzberg_batch_extract_bytes_with_configs_async),
    ]
}
