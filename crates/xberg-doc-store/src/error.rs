//! Error model for `xberg-doc-store`.

use thiserror::Error;

/// Result type used throughout `xberg-doc-store`.
pub type StoreResult<T> = Result<T, StoreError>;

/// Errors raised by sidecar-store operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StoreError {
    /// A backend-specific error (SQLite, I/O, join failure, …).
    #[error("doc-store backend error: {0}")]
    Backend(#[source] Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_error_displays_source_message() {
        let inner = std::io::Error::other("disk full");
        let err = StoreError::Backend(Box::new(inner));
        assert!(err.to_string().contains("disk full"));
    }
}
