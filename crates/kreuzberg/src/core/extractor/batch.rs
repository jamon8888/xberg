//! Batch extraction operations for concurrent processing.
//!
//! This module provides parallel extraction capabilities for processing
//! multiple files or byte arrays concurrently with automatic resource management.

use crate::core::config::ExtractionConfig;
use crate::core::config::extraction::FileExtractionConfig;
use crate::types::ExtractionResult;
use crate::{KreuzbergError, Result};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use super::bytes::extract_bytes;
use super::file::extract_file;
use super::helpers::error_extraction_result;

/// Shared batch result collection: spawns tasks via callback, collects ordered results.
#[cfg(feature = "tokio-runtime")]
async fn collect_batch<F, Fut>(count: usize, config: &ExtractionConfig, spawn_task: F) -> Result<Vec<ExtractionResult>>
where
    F: Fn(usize, Arc<tokio::sync::Semaphore>) -> Fut,
    Fut: Future<Output = (usize, Result<ExtractionResult>, u64)> + Send + 'static,
{
    use tokio::sync::Semaphore;
    use tokio::task::JoinSet;

    if count == 0 {
        return Ok(vec![]);
    }

    let max_concurrent = config
        .max_concurrent_extractions
        .unwrap_or_else(|| (num_cpus::get() as f64 * 1.5).ceil() as usize);
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    let mut tasks = JoinSet::new();

    for index in 0..count {
        let sem = Arc::clone(&semaphore);
        tasks.spawn(spawn_task(index, sem));
    }

    let mut results: Vec<Option<ExtractionResult>> = vec![None; count];

    while let Some(task_result) = tasks.join_next().await {
        match task_result {
            Ok((index, Ok(result), _elapsed_ms)) => {
                results[index] = Some(result);
            }
            Ok((index, Err(e), elapsed_ms)) => {
                results[index] = Some(error_extraction_result(&e, Some(elapsed_ms)));
            }
            Err(join_err) => {
                return Err(KreuzbergError::Other(format!("Task panicked: {}", join_err)));
            }
        }
    }

    #[allow(clippy::unwrap_used)]
    Ok(results.into_iter().map(|r| r.unwrap()).collect())
}

/// Run a single extraction task with semaphore gating, timing, and batch mode.
#[cfg(feature = "tokio-runtime")]
async fn run_timed_extraction<F, Fut>(
    index: usize,
    semaphore: Arc<tokio::sync::Semaphore>,
    extract_fn: F,
) -> (usize, Result<ExtractionResult>, u64)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<ExtractionResult>>,
{
    let _permit = semaphore.acquire().await.unwrap();
    let start = Instant::now();
    let mut result = crate::core::batch_mode::with_batch_mode(extract_fn()).await;
    let elapsed_ms = start.elapsed().as_millis() as u64;

    if let Ok(ref mut r) = result {
        r.metadata.extraction_duration_ms = Some(elapsed_ms);
    }

    (index, result, elapsed_ms)
}

/// Resolve a per-file config against a base config. Returns owned config.
fn resolve_config(base: &ExtractionConfig, file_config: &Option<FileExtractionConfig>) -> ExtractionConfig {
    match file_config {
        Some(fc) => base.with_file_overrides(fc),
        None => base.clone(),
    }
}

/// Extract content from multiple files concurrently.
///
/// This function processes multiple files in parallel, automatically managing
/// concurrency to prevent resource exhaustion. The concurrency limit can be
/// configured via `ExtractionConfig::max_concurrent_extractions` or defaults
/// to `(num_cpus * 1.5).ceil()`.
///
/// # Arguments
///
/// * `paths` - Vector of file paths to extract
/// * `config` - Extraction configuration
///
/// # Returns
///
/// A vector of `ExtractionResult` in the same order as the input paths.
///
/// # Errors
///
/// Individual file errors are captured in the result metadata. System errors
/// (IO, RuntimeError equivalents) will bubble up and fail the entire batch.
///
/// # Example
///
/// ```rust,no_run
/// use kreuzberg::core::extractor::batch_extract_file;
/// use kreuzberg::core::config::ExtractionConfig;
///
/// # async fn example() -> kreuzberg::Result<()> {
/// let config = ExtractionConfig::default();
/// let paths = vec!["doc1.pdf", "doc2.pdf"];
/// let results = batch_extract_file(paths, &config).await?;
/// println!("Processed {} files", results.len());
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "tokio-runtime")]
#[cfg_attr(feature = "otel", tracing::instrument(
    skip(config, paths),
    fields(
        extraction.batch_size = paths.len(),
    )
))]
pub async fn batch_extract_file(
    paths: Vec<impl AsRef<Path>>,
    config: &ExtractionConfig,
) -> Result<Vec<ExtractionResult>> {
    let items: Vec<(PathBuf, Option<FileExtractionConfig>)> =
        paths.into_iter().map(|p| (p.as_ref().to_path_buf(), None)).collect();
    batch_extract_file_with_configs(items, config).await
}

/// Extract content from multiple byte arrays concurrently.
///
/// This function processes multiple byte arrays in parallel, automatically managing
/// concurrency to prevent resource exhaustion. The concurrency limit can be
/// configured via `ExtractionConfig::max_concurrent_extractions` or defaults
/// to `(num_cpus * 1.5).ceil()`.
///
/// # Arguments
///
/// * `contents` - Vector of (bytes, mime_type) tuples
/// * `config` - Extraction configuration
///
/// # Returns
///
/// A vector of `ExtractionResult` in the same order as the input.
///
/// # Example
///
/// ```rust,no_run
/// use kreuzberg::core::extractor::batch_extract_bytes;
/// use kreuzberg::core::config::ExtractionConfig;
///
/// # async fn example() -> kreuzberg::Result<()> {
/// let config = ExtractionConfig::default();
/// let contents = vec![
///     (b"content 1".to_vec(), "text/plain".to_string()),
///     (b"content 2".to_vec(), "text/plain".to_string()),
/// ];
/// let results = batch_extract_bytes(contents, &config).await?;
/// println!("Processed {} items", results.len());
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "tokio-runtime")]
#[cfg_attr(feature = "otel", tracing::instrument(
    skip(config, contents),
    fields(
        extraction.batch_size = contents.len(),
    )
))]
pub async fn batch_extract_bytes(
    contents: Vec<(Vec<u8>, String)>,
    config: &ExtractionConfig,
) -> Result<Vec<ExtractionResult>> {
    let items: Vec<(Vec<u8>, String, Option<FileExtractionConfig>)> =
        contents.into_iter().map(|(bytes, mime)| (bytes, mime, None)).collect();
    batch_extract_bytes_with_configs(items, config).await
}

/// Extract content from multiple files concurrently with per-file configuration overrides.
///
/// Each file can optionally specify a [`FileExtractionConfig`] that overrides specific
/// fields from the batch-level `config`. Fields set to `None` in the file config use
/// the batch default. Batch-level settings like `max_concurrent_extractions` and
/// `use_cache` are always taken from the batch-level `config`.
///
/// # Arguments
///
/// * `items` - Vector of `(path, optional_file_config)` tuples
/// * `config` - Batch-level extraction configuration (provides defaults and batch settings)
///
/// # Returns
///
/// A vector of `ExtractionResult` in the same order as the input items.
///
/// # Example
///
/// ```rust,no_run
/// use kreuzberg::core::extractor::batch_extract_file_with_configs;
/// use kreuzberg::core::config::ExtractionConfig;
/// use kreuzberg::FileExtractionConfig;
/// use std::path::PathBuf;
///
/// # async fn example() -> kreuzberg::Result<()> {
/// let config = ExtractionConfig::default();
/// let items: Vec<(PathBuf, Option<FileExtractionConfig>)> = vec![
///     ("scan.pdf".into(), Some(FileExtractionConfig { force_ocr: Some(true), ..Default::default() })),
///     ("notes.txt".into(), None),
/// ];
/// let results = batch_extract_file_with_configs(items, &config).await?;
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "tokio-runtime")]
#[cfg_attr(feature = "otel", tracing::instrument(
    skip(config, items),
    fields(
        extraction.batch_size = items.len(),
    )
))]
pub async fn batch_extract_file_with_configs(
    items: Vec<(PathBuf, Option<FileExtractionConfig>)>,
    config: &ExtractionConfig,
) -> Result<Vec<ExtractionResult>> {
    let config_arc = Arc::new(config.clone());
    // Use Arc<Vec> for file items — paths are small, so keeping them all alive is fine.
    let items_arc = Arc::new(items);
    let count = items_arc.len();

    collect_batch(count, config, |index, sem| {
        let cfg = Arc::clone(&config_arc);
        let items = Arc::clone(&items_arc);
        async move {
            let (ref path, ref file_config) = items[index];
            let resolved = resolve_config(&cfg, file_config);
            run_timed_extraction(index, sem, || {
                let path = path.clone();
                async move { extract_file(&path, None, &resolved).await }
            })
            .await
        }
    })
    .await
}

/// Extract content from multiple byte arrays concurrently with per-file configuration overrides.
///
/// Each item can optionally specify a [`FileExtractionConfig`] that overrides specific
/// fields from the batch-level `config`.
///
/// # Arguments
///
/// * `items` - Vector of `(bytes, mime_type, optional_file_config)` tuples
/// * `config` - Batch-level extraction configuration
///
/// # Returns
///
/// A vector of `ExtractionResult` in the same order as the input items.
///
/// # Example
///
/// ```rust,no_run
/// use kreuzberg::core::extractor::batch_extract_bytes_with_configs;
/// use kreuzberg::core::config::ExtractionConfig;
/// use kreuzberg::FileExtractionConfig;
///
/// # async fn example() -> kreuzberg::Result<()> {
/// let config = ExtractionConfig::default();
/// let items = vec![
///     (b"content".to_vec(), "text/plain".to_string(), None),
///     (b"<html>test</html>".to_vec(), "text/html".to_string(),
///      Some(FileExtractionConfig { force_ocr: Some(true), ..Default::default() })),
/// ];
/// let results = batch_extract_bytes_with_configs(items, &config).await?;
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "tokio-runtime")]
#[cfg_attr(feature = "otel", tracing::instrument(
    skip(config, items),
    fields(
        extraction.batch_size = items.len(),
    )
))]
pub async fn batch_extract_bytes_with_configs(
    items: Vec<(Vec<u8>, String, Option<FileExtractionConfig>)>,
    config: &ExtractionConfig,
) -> Result<Vec<ExtractionResult>> {
    let config_arc = Arc::new(config.clone());
    let count = items.len();

    // Move items into individually-indexed slots so each task can take ownership
    // of its bytes without cloning. This avoids the memory regression of
    // Arc<Vec<(Vec<u8>, ...)>> which would keep all byte arrays alive for the
    // entire batch duration.
    type BytesSlot = std::sync::Mutex<Option<(Vec<u8>, String, Option<FileExtractionConfig>)>>;
    let slots: Arc<Vec<BytesSlot>> = Arc::new(
        items
            .into_iter()
            .map(|item| std::sync::Mutex::new(Some(item)))
            .collect(),
    );

    collect_batch(count, config, |index, sem| {
        let cfg = Arc::clone(&config_arc);
        let slots = Arc::clone(&slots);
        async move {
            let (bytes, mime_type, file_config) = slots[index]
                .lock()
                .unwrap()
                .take()
                .expect("batch item already consumed");
            let resolved = resolve_config(&cfg, &file_config);
            run_timed_extraction(index, sem, || async move {
                extract_bytes(&bytes, &mime_type, &resolved).await
            })
            .await
        }
    })
    .await
}
