use super::error::PdfError;
use once_cell::sync::Lazy;
use pdfium_render::prelude::*;
use std::path::PathBuf;
use std::sync::Mutex;

/// Cached state for lazy Pdfium initialization.
enum InitializationState {
    Uninitialized,
    Initialized { lib_dir: Option<PathBuf> },
    Failed(String),
}

static PDFIUM_STATE: Lazy<Mutex<InitializationState>> = Lazy::new(|| Mutex::new(InitializationState::Uninitialized));

fn extract_and_get_lib_dir() -> Result<Option<PathBuf>, String> {
    #[cfg(all(feature = "pdf", feature = "bundled-pdfium", not(target_arch = "wasm32")))]
    {
        let lib_path =
            crate::pdf::extract_bundled_pdfium().map_err(|e| format!("Failed to extract bundled Pdfium: {}", e))?;

        let lib_dir = lib_path.parent().ok_or_else(|| {
            format!(
                "Failed to determine Pdfium extraction directory for '{}'",
                lib_path.display()
            )
        })?;

        Ok(Some(lib_dir.to_path_buf()))
    }

    #[cfg(any(not(feature = "bundled-pdfium"), target_arch = "wasm32"))]
    {
        Ok(None)
    }
}

fn bind_to_pdfium(lib_dir: &Option<PathBuf>) -> Result<Box<dyn PdfiumLibraryBindings>, String> {
    let _ = lib_dir;
    #[cfg(all(feature = "pdf", feature = "bundled-pdfium", not(target_arch = "wasm32")))]
    {
        if let Some(dir) = lib_dir {
            return Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(dir))
                .map_err(|e| format!("Failed to bind to Pdfium library: {}", e));
        }
    }

    // For system library or WASM
    Pdfium::bind_to_system_library().map_err(|e| format!("Failed to bind to system Pdfium library: {}", e))
}

/// Get Pdfium bindings with lazy initialization.
///
/// The first call to this function triggers initialization. On that first call,
/// if using `bundled-pdfium`, the library is extracted to a temporary directory.
/// Subsequent calls quickly create fresh bindings from the cached state without
/// re-extraction.
///
/// # Arguments
///
/// * `map_err` - Function to map error strings to `PdfError` variants
/// * `context` - Context string for error reporting
///
/// # Returns
///
/// Freshly-created PDFium bindings, or an error if initialization failed.
///
/// # Performance Impact
///
/// - **First call**: Performs initialization (8-12ms for bundled extraction) plus binding.
/// - **Subsequent calls**: Creates binding from cached state (< 0.1ms).
///
/// This defers Pdfium initialization until first PDF is processed, improving cold start
/// for non-PDF workloads by 8-12ms. See Phase 3A Optimization #4 in profiling plan.
///
/// # Lock Poisoning Recovery
///
/// If a previous holder panicked while holding `PDFIUM_STATE`, the lock becomes poisoned.
/// Instead of failing permanently, we recover by extracting the inner value from the
/// poisoned lock and proceeding. This ensures PDF extraction can continue even if an
/// earlier panic occurred, as long as the state is consistent.
pub(crate) fn bind_pdfium(map_err: fn(String) -> PdfError, context: &'static str) -> Result<Pdfium, PdfError> {
    let mut state = PDFIUM_STATE.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

    // Check current state and get lib_dir if already initialized
    let lib_dir = match &*state {
        InitializationState::Uninitialized => {
            // Extract bundled library (only happens once)
            match extract_and_get_lib_dir() {
                Ok(lib_dir) => {
                    // Update state to mark as initialized
                    let lib_dir_clone = lib_dir.clone();
                    *state = InitializationState::Initialized { lib_dir };
                    lib_dir_clone
                }
                Err(err) => {
                    *state = InitializationState::Failed(err.clone());
                    return Err(map_err(format!("Pdfium extraction failed ({}): {}", context, err)));
                }
            }
        }
        InitializationState::Failed(err) => {
            return Err(map_err(format!(
                "Pdfium initialization previously failed ({}): {}",
                context,
                err.clone()
            )));
        }
        InitializationState::Initialized { lib_dir } => lib_dir.clone(),
    };

    // Create bindings from the library path (works for both first time and subsequent calls)
    let bindings =
        bind_to_pdfium(&lib_dir).map_err(|e| map_err(format!("Pdfium binding failed ({}): {}", context, e)))?;
    Ok(Pdfium::new(bindings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::error::PdfError;

    #[test]
    fn test_bind_pdfium_lazy_initialization() {
        let result = bind_pdfium(PdfError::TextExtractionFailed, "test context");
        assert!(result.is_ok(), "First bind_pdfium call should succeed");
    }

    #[test]
    fn test_bind_pdfium_multiple_calls() {
        let result1 = bind_pdfium(PdfError::TextExtractionFailed, "test 1");
        let result2 = bind_pdfium(PdfError::TextExtractionFailed, "test 2");

        assert!(result1.is_ok(), "First call should succeed");
        assert!(result2.is_ok(), "Second call should also succeed");
    }

    #[test]
    fn test_bind_pdfium_error_mapping() {
        let map_err = |msg: String| PdfError::TextExtractionFailed(msg);

        let test_error = map_err("test".to_string());
        match test_error {
            PdfError::TextExtractionFailed(msg) => {
                assert_eq!(msg, "test");
            }
            _ => panic!("Error mapping failed"),
        }
    }
}
