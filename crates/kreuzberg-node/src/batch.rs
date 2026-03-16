use napi::bindgen_prelude::*;
use napi_derive::napi;

use std::path::PathBuf;

use crate::WORKER_POOL;
use crate::config::{JsExtractionConfig, JsFileExtractionConfig};
use crate::error_handling::convert_error;
use crate::result::{JsExtractionResult, resolve_config, resolve_file_config};

#[napi]
pub fn batch_extract_files_sync(
    paths: Vec<String>,
    config: Option<JsExtractionConfig>,
) -> Result<Vec<JsExtractionResult>> {
    let rust_config = resolve_config(config)?;

    kreuzberg::batch_extract_file_sync(paths, &rust_config)
        .map_err(convert_error)
        .and_then(|results| results.into_iter().map(JsExtractionResult::try_from).collect())
}

/// Batch extract from multiple files (asynchronous).
///
/// Asynchronously processes multiple files in parallel. Non-blocking alternative
/// to `batchExtractFilesSync` with same performance benefits.
///
/// # Parameters
///
/// * `paths` - Array of file paths to extract
/// * `config` - Optional extraction configuration (applied to all files)
///
/// # Returns
///
/// Promise resolving to array of `ExtractionResult`.
///
/// # Example
///
/// ```typescript
/// import { batchExtractFiles } from '@kreuzberg/node';
///
/// const files = ['report1.pdf', 'report2.pdf', 'report3.pdf'];
/// const results = await batchExtractFiles(files, null);
/// console.log(`Processed ${results.length} files`);
/// ```
#[napi]
pub async fn batch_extract_files(
    paths: Vec<String>,
    config: Option<JsExtractionConfig>,
) -> Result<Vec<JsExtractionResult>> {
    let rust_config = resolve_config(config)?;

    let results = WORKER_POOL
        .spawn_blocking(move || kreuzberg::batch_extract_file_sync(paths, &rust_config))
        .await
        .map_err(|e| Error::from_reason(format!("Worker thread error: {}", e)))?
        .map_err(convert_error)?;

    results.into_iter().map(JsExtractionResult::try_from).collect()
}

/// Batch extract from multiple byte arrays (synchronous).
///
/// Synchronously processes multiple in-memory buffers in parallel. Requires
/// corresponding MIME types for each buffer.
///
/// # Parameters
///
/// * `data_list` - Array of buffers to extract
/// * `mime_types` - Array of MIME types (must match data_list length)
/// * `config` - Optional extraction configuration
///
/// # Returns
///
/// Array of `ExtractionResult` in the same order as inputs.
///
/// # Errors
///
/// Throws if data_list and mime_types lengths don't match.
///
/// # Example
///
/// ```typescript
/// import { batchExtractBytesSync } from '@kreuzberg/node';
///
/// const buffers = [buffer1, buffer2, buffer3];
/// const mimeTypes = ['application/pdf', 'image/png', 'text/plain'];
/// const results = batchExtractBytesSync(buffers, mimeTypes, null);
/// ```
#[napi]
pub fn batch_extract_bytes_sync(
    data_list: Vec<Buffer>,
    mime_types: Vec<String>,
    config: Option<JsExtractionConfig>,
) -> Result<Vec<JsExtractionResult>> {
    if data_list.len() != mime_types.len() {
        return Err(Error::new(
            Status::InvalidArg,
            format!(
                "data_list length ({}) must match mime_types length ({})",
                data_list.len(),
                mime_types.len()
            ),
        ));
    }

    let rust_config = resolve_config(config)?;

    let contents: Vec<(&[u8], &str)> = data_list
        .iter()
        .zip(mime_types.iter())
        .map(|(data, mime)| (data.as_ref(), mime.as_str()))
        .collect();

    let owned_contents: Vec<(Vec<u8>, String)> = contents
        .into_iter()
        .map(|(bytes, mime)| (bytes.to_vec(), mime.to_string()))
        .collect();

    kreuzberg::batch_extract_bytes_sync(owned_contents, &rust_config)
        .map_err(convert_error)
        .and_then(|results| results.into_iter().map(JsExtractionResult::try_from).collect())
}

/// Batch extract from multiple byte arrays (asynchronous).
///
/// Asynchronously processes multiple in-memory buffers in parallel. Non-blocking
/// alternative to `batchExtractBytesSync`.
///
/// # Parameters
///
/// * `data_list` - Array of buffers to extract
/// * `mime_types` - Array of MIME types (must match data_list length)
/// * `config` - Optional extraction configuration
///
/// # Returns
///
/// Promise resolving to array of `ExtractionResult`.
///
/// # Example
///
/// ```typescript
/// import { batchExtractBytes } from '@kreuzberg/node';
///
/// const responses = await Promise.all([
///   fetch('https://example.com/doc1.pdf'),
///   fetch('https://example.com/doc2.pdf')
/// ]);
/// const buffers = await Promise.all(
///   responses.map(r => r.arrayBuffer().then(b => Buffer.from(b)))
/// );
/// const results = await batchExtractBytes(
///   buffers,
///   ['application/pdf', 'application/pdf'],
///   null
/// );
/// ```
#[napi]
pub async fn batch_extract_bytes(
    data_list: Vec<Buffer>,
    mime_types: Vec<String>,
    config: Option<JsExtractionConfig>,
) -> Result<Vec<JsExtractionResult>> {
    if data_list.len() != mime_types.len() {
        return Err(Error::new(
            Status::InvalidArg,
            format!(
                "data_list length ({}) must match mime_types length ({})",
                data_list.len(),
                mime_types.len()
            ),
        ));
    }

    let rust_config = resolve_config(config)?;

    let contents: Vec<(Vec<u8>, String)> = data_list
        .iter()
        .zip(mime_types.iter())
        .map(|(data, mime)| (data.to_vec(), mime.clone()))
        .collect();

    let results = WORKER_POOL
        .spawn_blocking(move || {
            let contents_refs: Vec<(&[u8], &str)> = contents
                .iter()
                .map(|(data, mime)| (data.as_slice(), mime.as_str()))
                .collect();
            let owned_contents: Vec<(Vec<u8>, String)> = contents_refs
                .into_iter()
                .map(|(bytes, mime)| (bytes.to_vec(), mime.to_string()))
                .collect();
            kreuzberg::batch_extract_bytes_sync(owned_contents, &rust_config)
        })
        .await
        .map_err(|e| Error::from_reason(format!("Worker thread error: {}", e)))?
        .map_err(convert_error)?;

    results.into_iter().map(JsExtractionResult::try_from).collect()
}

#[napi]
pub fn batch_extract_files_with_configs_sync(
    paths: Vec<String>,
    file_configs: Vec<Option<JsFileExtractionConfig>>,
    config: Option<JsExtractionConfig>,
) -> Result<Vec<JsExtractionResult>> {
    if paths.len() != file_configs.len() {
        return Err(Error::new(
            Status::InvalidArg,
            format!(
                "paths length ({}) must match fileConfigs length ({})",
                paths.len(),
                file_configs.len()
            ),
        ));
    }

    let rust_config = resolve_config(config)?;

    let items: Vec<(PathBuf, Option<kreuzberg::FileExtractionConfig>)> = paths
        .into_iter()
        .zip(file_configs)
        .map(|(path, fc)| Ok((PathBuf::from(path), resolve_file_config(fc)?)))
        .collect::<Result<Vec<_>>>()?;

    kreuzberg::batch_extract_file_with_configs_sync(items, &rust_config)
        .map_err(convert_error)
        .and_then(|results| results.into_iter().map(JsExtractionResult::try_from).collect())
}

#[napi]
pub async fn batch_extract_files_with_configs(
    paths: Vec<String>,
    file_configs: Vec<Option<JsFileExtractionConfig>>,
    config: Option<JsExtractionConfig>,
) -> Result<Vec<JsExtractionResult>> {
    if paths.len() != file_configs.len() {
        return Err(Error::new(
            Status::InvalidArg,
            format!(
                "paths length ({}) must match fileConfigs length ({})",
                paths.len(),
                file_configs.len()
            ),
        ));
    }

    let rust_config = resolve_config(config)?;

    let items: Vec<(PathBuf, Option<kreuzberg::FileExtractionConfig>)> = paths
        .into_iter()
        .zip(file_configs)
        .map(|(path, fc)| Ok((PathBuf::from(path), resolve_file_config(fc)?)))
        .collect::<Result<Vec<_>>>()?;

    let results = WORKER_POOL
        .spawn_blocking(move || kreuzberg::batch_extract_file_with_configs_sync(items, &rust_config))
        .await
        .map_err(|e| Error::from_reason(format!("Worker thread error: {}", e)))?
        .map_err(convert_error)?;

    results.into_iter().map(JsExtractionResult::try_from).collect()
}

#[napi]
pub fn batch_extract_bytes_with_configs_sync(
    data_list: Vec<Buffer>,
    mime_types: Vec<String>,
    file_configs: Vec<Option<JsFileExtractionConfig>>,
    config: Option<JsExtractionConfig>,
) -> Result<Vec<JsExtractionResult>> {
    if data_list.len() != mime_types.len() || data_list.len() != file_configs.len() {
        return Err(Error::new(
            Status::InvalidArg,
            format!(
                "data_list length ({}), mime_types length ({}), and fileConfigs length ({}) must all match",
                data_list.len(),
                mime_types.len(),
                file_configs.len()
            ),
        ));
    }

    let rust_config = resolve_config(config)?;

    let items: Vec<(Vec<u8>, String, Option<kreuzberg::FileExtractionConfig>)> = data_list
        .iter()
        .zip(mime_types.into_iter())
        .zip(file_configs)
        .map(|((data, mime), fc)| Ok((data.to_vec(), mime, resolve_file_config(fc)?)))
        .collect::<Result<Vec<_>>>()?;

    kreuzberg::batch_extract_bytes_with_configs_sync(items, &rust_config)
        .map_err(convert_error)
        .and_then(|results| results.into_iter().map(JsExtractionResult::try_from).collect())
}

#[napi]
pub async fn batch_extract_bytes_with_configs(
    data_list: Vec<Buffer>,
    mime_types: Vec<String>,
    file_configs: Vec<Option<JsFileExtractionConfig>>,
    config: Option<JsExtractionConfig>,
) -> Result<Vec<JsExtractionResult>> {
    if data_list.len() != mime_types.len() || data_list.len() != file_configs.len() {
        return Err(Error::new(
            Status::InvalidArg,
            format!(
                "data_list length ({}), mime_types length ({}), and fileConfigs length ({}) must all match",
                data_list.len(),
                mime_types.len(),
                file_configs.len()
            ),
        ));
    }

    let rust_config = resolve_config(config)?;

    let items: Vec<(Vec<u8>, String, Option<kreuzberg::FileExtractionConfig>)> = data_list
        .iter()
        .zip(mime_types.into_iter())
        .zip(file_configs)
        .map(|((data, mime), fc)| Ok((data.to_vec(), mime, resolve_file_config(fc)?)))
        .collect::<Result<Vec<_>>>()?;

    let results = WORKER_POOL
        .spawn_blocking(move || kreuzberg::batch_extract_bytes_with_configs_sync(items, &rust_config))
        .await
        .map_err(|e| Error::from_reason(format!("Worker thread error: {}", e)))?
        .map_err(convert_error)?;

    results.into_iter().map(JsExtractionResult::try_from).collect()
}
