//! Core extraction functions
//!
//! Provides both synchronous and asynchronous extraction functions for Python.

use crate::config::{ExtractionConfig, FileExtractionConfig};
use crate::error::to_py_err;
use crate::types::ExtractionResult;
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::path::PathBuf;

/// Extract format strings from ExtractionConfig before it's consumed.
fn extract_format_strings(config: &ExtractionConfig) -> (Option<String>, Option<String>) {
    let output_fmt = match config.inner.output_format {
        kreuzberg::core::config::formats::OutputFormat::Plain => Some("plain".to_string()),
        kreuzberg::core::config::formats::OutputFormat::Markdown => Some("markdown".to_string()),
        kreuzberg::core::config::formats::OutputFormat::Djot => Some("djot".to_string()),
        kreuzberg::core::config::formats::OutputFormat::Html => Some("html".to_string()),
        kreuzberg::core::config::formats::OutputFormat::Structured => Some("structured".to_string()),
    };
    let result_fmt = match config.inner.result_format {
        kreuzberg::types::OutputFormat::Unified => Some("unified".to_string()),
        kreuzberg::types::OutputFormat::ElementBased => Some("element_based".to_string()),
    };
    (output_fmt, result_fmt)
}

/// Extract a path string from Python input (str, pathlib.Path, or bytes).
///
/// Supports:
/// - `str`: Direct string paths
/// - `pathlib.Path`: Extracts via `__fspath__()` protocol
/// - `bytes`: UTF-8 decoded path bytes (Unix paths)
fn extract_path_string(path: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(s) = path.extract::<String>() {
        return Ok(s);
    }

    if let Ok(fspath) = path.call_method0("__fspath__")
        && let Ok(s) = fspath.extract::<String>()
    {
        return Ok(s);
    }

    if let Ok(b) = path.extract::<Vec<u8>>() {
        if let Ok(s) = String::from_utf8(b) {
            return Ok(s);
        }
        return Err(pyo3::exceptions::PyValueError::new_err(
            "Path bytes must be valid UTF-8",
        ));
    }

    Err(pyo3::exceptions::PyTypeError::new_err(
        "Path must be a string, pathlib.Path, or bytes",
    ))
}

/// Extract content from a file (synchronous).
///
/// Args:
///     path: Path to the file to extract (str or pathlib.Path)
///     mime_type: Optional MIME type hint (auto-detected if None)
///     config: Extraction configuration
///
/// Returns:
///     ExtractionResult with content, metadata, and tables
///
/// Raises:
///     ValueError: Invalid configuration or unsupported format
///     IOError: File access errors
///     RuntimeError: Extraction failures
///
/// Example:
///     >>> from kreuzberg import extract_file_sync, ExtractionConfig
///     >>> result = extract_file_sync("document.pdf", None, ExtractionConfig())
///     >>> print(result.content)
///     >>> # Also works with pathlib.Path
///     >>> from pathlib import Path
///     >>> result = extract_file_sync(Path("document.pdf"), None, ExtractionConfig())
#[pyfunction]
#[pyo3(signature = (path, mime_type=None, config=ExtractionConfig::default()))]
pub fn extract_file_sync(
    py: Python,
    path: &Bound<'_, PyAny>,
    mime_type: Option<String>,
    config: ExtractionConfig,
) -> PyResult<ExtractionResult> {
    let path_str = extract_path_string(path)?;
    let (output_fmt, result_fmt) = extract_format_strings(&config);
    let rust_config = config.into();

    // Release GIL during sync extraction - OSError/RuntimeError must bubble up ~keep
    let result = Python::detach(py, || {
        kreuzberg::extract_file_sync(&path_str, mime_type.as_deref(), &rust_config)
    })
    .map_err(to_py_err)?;

    ExtractionResult::from_rust(result, py, output_fmt, result_fmt)
}

/// Extract content from bytes (synchronous).
///
/// Args:
///     data: Bytes to extract (bytes or bytearray)
///     mime_type: MIME type of the data
///     config: Extraction configuration
///
/// Returns:
///     ExtractionResult with content, metadata, and tables
///
/// Raises:
///     ValueError: Invalid configuration or unsupported format
///     RuntimeError: Extraction failures
///
/// Example:
///     >>> from kreuzberg import extract_bytes_sync, ExtractionConfig
///     >>> with open("document.pdf", "rb") as f:
///     ...     data = f.read()
///     >>> result = extract_bytes_sync(data, "application/pdf", ExtractionConfig())
///     >>> print(result.content)
#[pyfunction]
#[pyo3(signature = (data, mime_type, config=ExtractionConfig::default()))]
pub fn extract_bytes_sync(
    py: Python,
    data: Vec<u8>,
    mime_type: String,
    config: ExtractionConfig,
) -> PyResult<ExtractionResult> {
    let (output_fmt, result_fmt) = extract_format_strings(&config);
    let rust_config = config.into();

    // Release GIL during extraction and result conversion - OSError/RuntimeError must bubble up ~keep
    let result =
        Python::detach(py, || kreuzberg::extract_bytes_sync(&data, &mime_type, &rust_config)).map_err(to_py_err)?;

    ExtractionResult::from_rust(result, py, output_fmt, result_fmt)
}

/// Batch extract content from multiple files (synchronous).
///
/// MIME types are auto-detected for each file.
///
/// Args:
///     paths: List of file paths to extract (str, pathlib.Path, or bytes)
///     config: Extraction configuration
///
/// Returns:
///     List of ExtractionResult objects (one per file)
///
/// Raises:
///     ValueError: Invalid configuration
///     IOError: File access errors
///     RuntimeError: Extraction failures
///
/// Example:
///     >>> from kreuzberg import batch_extract_files_sync, ExtractionConfig
///     >>> paths = ["doc1.pdf", "doc2.docx"]
///     >>> results = batch_extract_files_sync(paths, ExtractionConfig())
///     >>> for result in results:
///     ...     print(result.content)
///     >>> # Also works with pathlib.Path
///     >>> from pathlib import Path
///     >>> paths = [Path("doc1.pdf"), Path("doc2.docx")]
///     >>> results = batch_extract_files_sync(paths, ExtractionConfig())
#[pyfunction]
#[pyo3(signature = (paths, config=ExtractionConfig::default()))]
pub fn batch_extract_files_sync(
    py: Python,
    paths: &Bound<'_, PyList>,
    config: ExtractionConfig,
) -> PyResult<Py<PyList>> {
    let path_strings: PyResult<Vec<String>> = paths.iter().map(|p| extract_path_string(&p)).collect();
    let path_strings = path_strings?;

    let (output_fmt, result_fmt) = extract_format_strings(&config);
    let rust_config = config.into();

    // Release GIL during sync batch extraction - OSError/RuntimeError must bubble up ~keep
    let results =
        Python::detach(py, || kreuzberg::batch_extract_file_sync(path_strings, &rust_config)).map_err(to_py_err)?;

    let converted: PyResult<Vec<_>> = results
        .into_iter()
        .map(|result| {
            ExtractionResult::from_rust(result, py, output_fmt.as_ref().cloned(), result_fmt.as_ref().cloned())
        })
        .collect();
    let list = PyList::new(py, converted?)?;
    Ok(list.unbind())
}

/// Batch extract content from multiple byte arrays (synchronous).
///
/// Args:
///     data_list: List of bytes objects to extract
///     mime_types: List of MIME types (one per data object)
///     config: Extraction configuration
///
/// Returns:
///     List of ExtractionResult objects (one per data object)
///
/// Raises:
///     ValueError: Invalid configuration or list length mismatch
///     RuntimeError: Extraction failures
///
/// Example:
///     >>> from kreuzberg import batch_extract_bytes_sync, ExtractionConfig
///     >>> data_list = [open("doc1.pdf", "rb").read(), open("doc2.pdf", "rb").read()]
///     >>> mime_types = ["application/pdf", "application/pdf"]
///     >>> results = batch_extract_bytes_sync(data_list, mime_types, ExtractionConfig())
#[pyfunction]
#[pyo3(signature = (data_list, mime_types, config=ExtractionConfig::default()))]
pub fn batch_extract_bytes_sync(
    py: Python,
    data_list: Vec<Vec<u8>>,
    mime_types: Vec<String>,
    config: ExtractionConfig,
) -> PyResult<Py<PyList>> {
    if data_list.len() != mime_types.len() {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "data_list and mime_types must have the same length (got {} and {})",
            data_list.len(),
            mime_types.len()
        )));
    }

    let (output_fmt, result_fmt) = extract_format_strings(&config);
    let rust_config = config.into();

    let contents: Vec<(&[u8], &str)> = data_list
        .iter()
        .zip(mime_types.iter())
        .map(|(data, mime)| (data.as_slice(), mime.as_str()))
        .collect();

    let owned_contents: Vec<(Vec<u8>, String)> = contents
        .into_iter()
        .map(|(bytes, mime)| (bytes.to_vec(), mime.to_string()))
        .collect();

    // Release GIL during sync batch extraction - OSError/RuntimeError must bubble up ~keep
    let results =
        Python::detach(py, || kreuzberg::batch_extract_bytes_sync(owned_contents, &rust_config)).map_err(to_py_err)?;

    let converted: PyResult<Vec<_>> = results
        .into_iter()
        .map(|result| ExtractionResult::from_rust(result, py, output_fmt.clone(), result_fmt.clone()))
        .collect();
    let list = PyList::new(py, converted?)?;
    Ok(list.unbind())
}

/// Extract content from a file (asynchronous).
///
/// Args:
///     path: Path to the file to extract (str or pathlib.Path)
///     mime_type: Optional MIME type hint (auto-detected if None)
///     config: Extraction configuration
///
/// Returns:
///     ExtractionResult with content, metadata, and tables
///
/// Raises:
///     ValueError: Invalid configuration or unsupported format
///     IOError: File access errors
///     RuntimeError: Extraction failures
///
/// Example:
///     >>> import asyncio
///     >>> from kreuzberg import extract_file, ExtractionConfig
///     >>> async def main():
///     ...     result = await extract_file("document.pdf", None, ExtractionConfig())
///     ...     print(result.content)
///     >>> asyncio.run(main())
///     >>> # Also works with pathlib.Path
///     >>> from pathlib import Path
///     >>> async def main():
///     ...     result = await extract_file(Path("document.pdf"))
#[pyfunction]
#[pyo3(signature = (path, mime_type=None, config=ExtractionConfig::default()))]
pub fn extract_file<'py>(
    py: Python<'py>,
    path: &Bound<'py, PyAny>,
    mime_type: Option<String>,
    config: ExtractionConfig,
) -> PyResult<Bound<'py, PyAny>> {
    let path_str = extract_path_string(path)?;
    let (output_fmt, result_fmt) = extract_format_strings(&config);
    let rust_config: kreuzberg::ExtractionConfig = config.into();
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let result = kreuzberg::extract_file(&path_str, mime_type.as_deref(), &rust_config)
            .await
            .map_err(to_py_err)?;
        Python::attach(|py| ExtractionResult::from_rust(result, py, output_fmt, result_fmt))
    })
}

/// Extract content from bytes (asynchronous).
///
/// Args:
///     data: Bytes to extract (bytes or bytearray)
///     mime_type: MIME type of the data
///     config: Extraction configuration
///
/// Returns:
///     ExtractionResult with content, metadata, and tables
///
/// Raises:
///     ValueError: Invalid configuration or unsupported format
///     RuntimeError: Extraction failures
///
/// Example:
///     >>> import asyncio
///     >>> from kreuzberg import extract_bytes, ExtractionConfig
///     >>> async def main():
///     ...     with open("document.pdf", "rb") as f:
///     ...         data = f.read()
///     ...     result = await extract_bytes(data, "application/pdf", ExtractionConfig())
///     ...     print(result.content)
///     >>> asyncio.run(main())
#[pyfunction]
#[pyo3(signature = (data, mime_type, config=ExtractionConfig::default()))]
pub fn extract_bytes<'py>(
    py: Python<'py>,
    data: Vec<u8>,
    mime_type: String,
    config: ExtractionConfig,
) -> PyResult<Bound<'py, PyAny>> {
    let (output_fmt, result_fmt) = extract_format_strings(&config);
    let rust_config: kreuzberg::ExtractionConfig = config.into();
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let result = kreuzberg::extract_bytes(&data, &mime_type, &rust_config)
            .await
            .map_err(to_py_err)?;
        Python::attach(|py| ExtractionResult::from_rust(result, py, output_fmt, result_fmt))
    })
}

/// Batch extract content from multiple files (asynchronous).
///
/// MIME types are auto-detected for each file.
///
/// Args:
///     paths: List of file paths to extract (str, pathlib.Path, or bytes)
///     config: Extraction configuration
///
/// Returns:
///     List of ExtractionResult objects (one per file)
///
/// Raises:
///     ValueError: Invalid configuration
///     IOError: File access errors
///     RuntimeError: Extraction failures
///
/// Example:
///     >>> import asyncio
///     >>> from kreuzberg import batch_extract_files, ExtractionConfig
///     >>> async def main():
///     ...     paths = ["doc1.pdf", "doc2.docx"]
///     ...     results = await batch_extract_files(paths, ExtractionConfig())
///     ...     for result in results:
///     ...         print(result.content)
///     >>> asyncio.run(main())
///     >>> # Also works with pathlib.Path
///     >>> from pathlib import Path
///     >>> async def main():
///     ...     paths = [Path("doc1.pdf"), Path("doc2.docx")]
///     ...     results = await batch_extract_files(paths, ExtractionConfig())
#[pyfunction]
#[pyo3(signature = (paths, config=ExtractionConfig::default()))]
pub fn batch_extract_files<'py>(
    py: Python<'py>,
    paths: &Bound<'py, PyList>,
    config: ExtractionConfig,
) -> PyResult<Bound<'py, PyAny>> {
    let path_strings: PyResult<Vec<String>> = paths.iter().map(|p| extract_path_string(&p)).collect();
    let path_strings = path_strings?;

    let (output_fmt, result_fmt) = extract_format_strings(&config);
    let rust_config: kreuzberg::ExtractionConfig = config.into();
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let results = kreuzberg::batch_extract_file(path_strings, &rust_config)
            .await
            .map_err(to_py_err)?;

        Python::attach(|py| {
            let converted: PyResult<Vec<_>> = results
                .into_iter()
                .map(|result| {
                    ExtractionResult::from_rust(result, py, output_fmt.as_ref().cloned(), result_fmt.as_ref().cloned())
                })
                .collect();
            let list = PyList::new(py, converted?)?;
            Ok(list.unbind())
        })
    })
}

/// Batch extract content from multiple byte arrays (asynchronous).
///
/// Args:
///     data_list: List of bytes objects to extract
///     mime_types: List of MIME types (one per data object)
///     config: Extraction configuration
///
/// Returns:
///     List of ExtractionResult objects (one per data object)
///
/// Raises:
///     ValueError: Invalid configuration or list length mismatch
///     RuntimeError: Extraction failures
///
/// Example:
///     >>> import asyncio
///     >>> from kreuzberg import batch_extract_bytes, ExtractionConfig
///     >>> async def main():
///     ...     data_list = [open("doc1.pdf", "rb").read(), open("doc2.pdf", "rb").read()]
///     ...     mime_types = ["application/pdf", "application/pdf"]
///     ...     results = await batch_extract_bytes(data_list, mime_types, ExtractionConfig())
///     >>> asyncio.run(main())
#[pyfunction]
#[pyo3(signature = (data_list, mime_types, config=ExtractionConfig::default()))]
pub fn batch_extract_bytes<'py>(
    py: Python<'py>,
    data_list: Vec<Vec<u8>>,
    mime_types: Vec<String>,
    config: ExtractionConfig,
) -> PyResult<Bound<'py, PyAny>> {
    if data_list.len() != mime_types.len() {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "data_list and mime_types must have the same length (got {} and {})",
            data_list.len(),
            mime_types.len()
        )));
    }

    let (output_fmt, result_fmt) = extract_format_strings(&config);
    let rust_config: kreuzberg::ExtractionConfig = config.into();
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let contents: Vec<(&[u8], &str)> = data_list
            .iter()
            .zip(mime_types.iter())
            .map(|(data, mime)| (data.as_slice(), mime.as_str()))
            .collect();

        let owned_contents: Vec<(Vec<u8>, String)> = contents
            .into_iter()
            .map(|(bytes, mime)| (bytes.to_vec(), mime.to_string()))
            .collect();

        let results = kreuzberg::batch_extract_bytes(owned_contents, &rust_config)
            .await
            .map_err(to_py_err)?;

        Python::attach(|py| {
            let converted: PyResult<Vec<_>> = results
                .into_iter()
                .map(|result| {
                    ExtractionResult::from_rust(result, py, output_fmt.as_ref().cloned(), result_fmt.as_ref().cloned())
                })
                .collect();
            let list = PyList::new(py, converted?)?;
            Ok(list.unbind())
        })
    })
}

/// Batch extract content from multiple files with per-file config overrides (synchronous).
///
/// Args:
///     items: List of (path, Optional[FileExtractionConfig]) tuples
///     config: Base extraction configuration
///
/// Returns:
///     List of ExtractionResult objects (one per file)
#[pyfunction]
#[pyo3(signature = (items, config=ExtractionConfig::default()))]
pub fn batch_extract_files_with_configs_sync(
    py: Python,
    items: &Bound<'_, PyList>,
    config: ExtractionConfig,
) -> PyResult<Py<PyList>> {
    let mut rust_items: Vec<(PathBuf, Option<kreuzberg::FileExtractionConfig>)> = Vec::with_capacity(items.len());

    for item in items.iter() {
        let tuple = item.cast::<pyo3::types::PyTuple>().map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err(
                "Each item must be a tuple of (path, Optional[FileExtractionConfig])",
            )
        })?;
        if tuple.len() != 2 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Each item must be a 2-tuple of (path, Optional[FileExtractionConfig])",
            ));
        }
        let path_str = extract_path_string(&tuple.get_item(0)?)?;
        let file_config: Option<FileExtractionConfig> = tuple.get_item(1)?.extract()?;
        rust_items.push((PathBuf::from(path_str), file_config.map(Into::into)));
    }

    let (output_fmt, result_fmt) = extract_format_strings(&config);
    let rust_config = config.into();

    let results = Python::detach(py, || {
        kreuzberg::batch_extract_file_with_configs_sync(rust_items, &rust_config)
    })
    .map_err(to_py_err)?;

    let converted: PyResult<Vec<_>> = results
        .into_iter()
        .map(|result| {
            ExtractionResult::from_rust(result, py, output_fmt.as_ref().cloned(), result_fmt.as_ref().cloned())
        })
        .collect();
    let list = PyList::new(py, converted?)?;
    Ok(list.unbind())
}

/// Batch extract content from multiple byte arrays with per-item config overrides (synchronous).
///
/// Args:
///     items: List of (bytes, mime_type, Optional[FileExtractionConfig]) tuples
///     config: Base extraction configuration
///
/// Returns:
///     List of ExtractionResult objects (one per item)
#[pyfunction]
#[pyo3(signature = (items, config=ExtractionConfig::default()))]
pub fn batch_extract_bytes_with_configs_sync(
    py: Python,
    items: &Bound<'_, PyList>,
    config: ExtractionConfig,
) -> PyResult<Py<PyList>> {
    let mut rust_items: Vec<(Vec<u8>, String, Option<kreuzberg::FileExtractionConfig>)> =
        Vec::with_capacity(items.len());

    for item in items.iter() {
        let tuple = item.cast::<pyo3::types::PyTuple>().map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err(
                "Each item must be a tuple of (bytes, mime_type, Optional[FileExtractionConfig])",
            )
        })?;
        if tuple.len() != 3 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Each item must be a 3-tuple of (bytes, mime_type, Optional[FileExtractionConfig])",
            ));
        }
        let data: Vec<u8> = tuple.get_item(0)?.extract()?;
        let mime_type: String = tuple.get_item(1)?.extract()?;
        let file_config: Option<FileExtractionConfig> = tuple.get_item(2)?.extract()?;
        rust_items.push((data, mime_type, file_config.map(Into::into)));
    }

    let (output_fmt, result_fmt) = extract_format_strings(&config);
    let rust_config = config.into();

    let results = Python::detach(py, || {
        kreuzberg::batch_extract_bytes_with_configs_sync(rust_items, &rust_config)
    })
    .map_err(to_py_err)?;

    let converted: PyResult<Vec<_>> = results
        .into_iter()
        .map(|result| {
            ExtractionResult::from_rust(result, py, output_fmt.as_ref().cloned(), result_fmt.as_ref().cloned())
        })
        .collect();
    let list = PyList::new(py, converted?)?;
    Ok(list.unbind())
}

/// Batch extract content from multiple files with per-file config overrides (asynchronous).
///
/// Args:
///     items: List of (path, Optional[FileExtractionConfig]) tuples
///     config: Base extraction configuration
///
/// Returns:
///     List of ExtractionResult objects (one per file)
#[pyfunction]
#[pyo3(signature = (items, config=ExtractionConfig::default()))]
pub fn batch_extract_files_with_configs<'py>(
    py: Python<'py>,
    items: &Bound<'py, PyList>,
    config: ExtractionConfig,
) -> PyResult<Bound<'py, PyAny>> {
    let mut rust_items: Vec<(PathBuf, Option<kreuzberg::FileExtractionConfig>)> = Vec::with_capacity(items.len());

    for item in items.iter() {
        let tuple = item.cast::<pyo3::types::PyTuple>().map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err(
                "Each item must be a tuple of (path, Optional[FileExtractionConfig])",
            )
        })?;
        if tuple.len() != 2 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Each item must be a 2-tuple of (path, Optional[FileExtractionConfig])",
            ));
        }
        let path_str = extract_path_string(&tuple.get_item(0)?)?;
        let file_config: Option<FileExtractionConfig> = tuple.get_item(1)?.extract()?;
        rust_items.push((PathBuf::from(path_str), file_config.map(Into::into)));
    }

    let (output_fmt, result_fmt) = extract_format_strings(&config);
    let rust_config: kreuzberg::ExtractionConfig = config.into();

    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let results = kreuzberg::batch_extract_file_with_configs(rust_items, &rust_config)
            .await
            .map_err(to_py_err)?;

        Python::attach(|py| {
            let converted: PyResult<Vec<_>> = results
                .into_iter()
                .map(|result| {
                    ExtractionResult::from_rust(result, py, output_fmt.as_ref().cloned(), result_fmt.as_ref().cloned())
                })
                .collect();
            let list = PyList::new(py, converted?)?;
            Ok(list.unbind())
        })
    })
}

/// Batch extract content from multiple byte arrays with per-item config overrides (asynchronous).
///
/// Args:
///     items: List of (bytes, mime_type, Optional[FileExtractionConfig]) tuples
///     config: Base extraction configuration
///
/// Returns:
///     List of ExtractionResult objects (one per item)
#[pyfunction]
#[pyo3(signature = (items, config=ExtractionConfig::default()))]
pub fn batch_extract_bytes_with_configs<'py>(
    py: Python<'py>,
    items: &Bound<'py, PyList>,
    config: ExtractionConfig,
) -> PyResult<Bound<'py, PyAny>> {
    let mut rust_items: Vec<(Vec<u8>, String, Option<kreuzberg::FileExtractionConfig>)> =
        Vec::with_capacity(items.len());

    for item in items.iter() {
        let tuple = item.cast::<pyo3::types::PyTuple>().map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err(
                "Each item must be a tuple of (bytes, mime_type, Optional[FileExtractionConfig])",
            )
        })?;
        if tuple.len() != 3 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Each item must be a 3-tuple of (bytes, mime_type, Optional[FileExtractionConfig])",
            ));
        }
        let data: Vec<u8> = tuple.get_item(0)?.extract()?;
        let mime_type: String = tuple.get_item(1)?.extract()?;
        let file_config: Option<FileExtractionConfig> = tuple.get_item(2)?.extract()?;
        rust_items.push((data, mime_type, file_config.map(Into::into)));
    }

    let (output_fmt, result_fmt) = extract_format_strings(&config);
    let rust_config: kreuzberg::ExtractionConfig = config.into();

    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let results = kreuzberg::batch_extract_bytes_with_configs(rust_items, &rust_config)
            .await
            .map_err(to_py_err)?;

        Python::attach(|py| {
            let converted: PyResult<Vec<_>> = results
                .into_iter()
                .map(|result| {
                    ExtractionResult::from_rust(result, py, output_fmt.as_ref().cloned(), result_fmt.as_ref().cloned())
                })
                .collect();
            let list = PyList::new(py, converted?)?;
            Ok(list.unbind())
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::{PyBytes, PyString};
    use std::sync::Once;

    fn prepare_python() {
        static INIT: Once = Once::new();
        INIT.call_once(Python::initialize);
    }

    fn with_py<F, R>(f: F) -> R
    where
        F: FnOnce(Python<'_>) -> R,
    {
        prepare_python();
        Python::attach(f)
    }

    #[test]
    fn test_extract_path_string_from_str() {
        with_py(|py| {
            let value = PyString::new(py, "document.txt");
            let result = extract_path_string(&value.into_any()).expect("string path should extract");
            assert_eq!(result, "document.txt");
        });
    }

    #[test]
    fn test_extract_path_string_from_pathlib_path() {
        with_py(|py| -> PyResult<()> {
            let pathlib = py.import("pathlib")?;
            let path_obj = pathlib.getattr("Path")?.call1(("nested/file.md",))?;
            let extracted = extract_path_string(&path_obj)?;
            assert!(
                extracted.ends_with("nested/file.md"),
                "expected path to end with nested/file.md, got {extracted}"
            );
            Ok(())
        })
        .expect("pathlib.Path extraction should succeed");
    }

    #[test]
    fn test_extract_path_string_from_bytes() {
        with_py(|py| {
            let value = PyBytes::new(py, b"ascii.bin");
            let result = extract_path_string(&value.into_any()).expect("bytes path should extract");
            assert_eq!(result, "ascii.bin");
        });
    }

    #[test]
    fn test_extract_path_string_invalid_type() {
        with_py(|py| {
            let value = py
                .eval(pyo3::ffi::c_str!("42"), None, None)
                .expect("should evaluate literal");
            let err = extract_path_string(&value).expect_err("non-path type should fail");
            assert!(err.is_instance_of::<pyo3::exceptions::PyTypeError>(py));
        });
    }

    #[test]
    fn test_extract_bytes_sync_returns_content() {
        with_py(|py| {
            let data = b"hello kreuzberg".to_vec();
            let result = extract_bytes_sync(py, data, "text/plain".to_string(), ExtractionConfig::default())
                .expect("text/plain extraction should succeed");
            assert_eq!(result.mime_type, "text/plain");
            assert!(result.content.contains("hello"));
        });
    }

    #[test]
    fn test_batch_extract_bytes_sync_length_mismatch() {
        with_py(|py| {
            let err = batch_extract_bytes_sync(
                py,
                vec![b"a".to_vec(), b"b".to_vec()],
                vec!["text/plain".to_string()],
                ExtractionConfig::default(),
            )
            .expect_err("length mismatch should error");
            assert!(err.is_instance_of::<pyo3::exceptions::PyValueError>(py));
        });
    }

    #[test]
    fn test_batch_extract_bytes_sync_returns_list() {
        with_py(|py| {
            let data = vec![b"first".to_vec(), b"second".to_vec()];
            let mimes = vec!["text/plain".to_string(), "text/plain".to_string()];
            let list = batch_extract_bytes_sync(py, data, mimes, ExtractionConfig::default())
                .expect("batch extraction should succeed");
            assert_eq!(list.bind(py).len(), 2);
        });
    }
}
