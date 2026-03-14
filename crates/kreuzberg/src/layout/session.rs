use ort::session::{Session, builder::GraphOptimizationLevel};

use crate::layout::error::LayoutError;

/// Build an optimized ORT session from an ONNX model file.
///
/// Optimizations applied:
/// - Level3 graph optimization (most aggressive constant folding + operator fusion)
/// - intra_threads = num logical CPUs (within-operator parallelism for matrix ops)
/// - inter_threads = 1 (single graph execution — avoids contention)
/// - CoreML execution provider on macOS (Neural Engine / GPU acceleration)
/// - CUDA execution provider on Linux (GPU acceleration)
///
/// Level3 is used because the aggressive operator fusion significantly speeds up
/// per-page inference (2x faster than Level1), which amortizes the higher session
/// creation cost across multi-page documents.
pub fn build_session(path: &str) -> Result<Session, LayoutError> {
    let num_cores = num_cpus::get();

    let builder = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .with_intra_threads(num_cores)?
        .with_inter_threads(1)?;

    // Platform-specific execution providers with CPU fallback.
    // Note: if the EP is not available at runtime, ORT silently falls back to CPU.
    #[cfg(target_os = "macos")]
    let builder = builder.with_execution_providers([ort::ep::CoreML::default().build()])?;

    #[cfg(target_os = "linux")]
    let builder = builder.with_execution_providers([ort::ep::CUDA::default().build()])?;

    let session = builder.commit_from_file(path)?;

    // Log which execution providers were requested so diagnostics are easy to
    // read in traces.  ORT silently falls back to CPU if a requested EP is
    // unavailable at runtime, so this reflects intent, not guarantee.
    #[cfg(target_os = "macos")]
    tracing::debug!("ORT session built with CoreML EP requested (CPU fallback enabled)");
    #[cfg(target_os = "linux")]
    tracing::debug!("ORT session built with CUDA EP requested (CPU fallback enabled)");
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    tracing::debug!("ORT session built with CPU EP");

    Ok(session)
}
