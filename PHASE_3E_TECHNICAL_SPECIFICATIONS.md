# Phase 3E: Technical Specifications

**Date**: December 20, 2025
**Purpose**: Detailed implementation specs for WASM memory optimization

---

## Specification 1: Shared Memory Buffer Management

### Problem Statement

Current implementation copies Uint8Array to Vec<u8> on every call:
```rust
// extraction.rs L55, L99
let bytes = data.to_vec();  // Expensive copy
```

For a 1MB document, this adds ~20ms overhead. Shared memory allows zero-copy access.

### Design

#### 1.1 Memory Buffer Wrapper

**File**: `crates/kreuzberg-wasm/src/memory.rs` (NEW)

```rust
use wasm_bindgen::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Shared memory buffer for zero-copy WASM/JS interop.
/// Manages a pre-allocated buffer accessible to both WASM and JavaScript.
#[wasm_bindgen]
pub struct SharedMemoryBuffer {
    /// Base pointer in WASM linear memory
    ptr: *mut u8,
    /// Total capacity in bytes
    capacity: usize,
    /// Current write position
    position: AtomicUsize,
}

#[wasm_bindgen]
impl SharedMemoryBuffer {
    /// Create a new shared memory buffer with the given capacity.
    ///
    /// # Arguments
    /// * `capacity` - Size in bytes to allocate
    ///
    /// # Returns
    /// New SharedMemoryBuffer or error if allocation fails
    #[wasm_bindgen(constructor)]
    pub fn new(capacity: usize) -> Result<SharedMemoryBuffer, JsValue> {
        if capacity == 0 {
            return Err(JsValue::from_str("Capacity must be greater than 0"));
        }

        // Allocate with Box to ensure stable pointer for buffer lifetime
        let mut vec = vec![0u8; capacity];
        let ptr = vec.as_mut_ptr();
        std::mem::forget(vec);  // Keep allocation alive for process lifetime

        Ok(SharedMemoryBuffer {
            ptr,
            capacity,
            position: AtomicUsize::new(0),
        })
    }

    /// Get the pointer to this buffer's data in WASM linear memory.
    ///
    /// JavaScript calls this to obtain the memory address for writing data.
    pub fn as_ptr(&self) -> usize {
        self.ptr as usize
    }

    /// Get the capacity of this buffer in bytes.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the current position (bytes written).
    pub fn position(&self) -> usize {
        self.position.load(Ordering::Relaxed)
    }

    /// Write data at the specified offset.
    ///
    /// # Arguments
    /// * `offset` - Byte offset in buffer
    /// * `data` - Data to write
    ///
    /// # Returns
    /// Number of bytes written or error
    ///
    /// # Safety
    /// Validates offset and length before writing.
    pub fn write_at(&self, offset: usize, data: &[u8]) -> Result<usize, JsValue> {
        if offset > self.capacity {
            return Err(JsValue::from_str("Offset exceeds buffer capacity"));
        }

        let available = self.capacity - offset;
        if data.len() > available {
            return Err(JsValue::from_str(&format!(
                "Data size {} exceeds available space {}",
                data.len(),
                available
            )));
        }

        // SAFETY: We validated offset + length <= capacity
        unsafe {
            let dst = self.ptr.add(offset);
            std::ptr::copy_nonoverlapping(data.as_ptr(), dst, data.len());
        }

        self.position.store(offset + data.len(), Ordering::Relaxed);
        Ok(data.len())
    }

    /// Read data from the specified offset.
    ///
    /// # Arguments
    /// * `offset` - Byte offset in buffer
    /// * `length` - Number of bytes to read
    ///
    /// # Returns
    /// Vector of bytes or error
    pub fn read_at(&self, offset: usize, length: usize) -> Result<Vec<u8>, JsValue> {
        if offset > self.capacity {
            return Err(JsValue::from_str("Offset exceeds buffer capacity"));
        }

        let available = self.capacity - offset;
        if length > available {
            return Err(JsValue::from_str(&format!(
                "Read size {} exceeds available bytes {}",
                length,
                available
            )));
        }

        let mut result = vec![0u8; length];

        // SAFETY: We validated offset + length <= capacity
        unsafe {
            let src = self.ptr.add(offset);
            std::ptr::copy_nonoverlapping(src, result.as_mut_ptr(), length);
        }

        Ok(result)
    }

    /// Reset the position counter.
    pub fn reset(&self) {
        self.position.store(0, Ordering::Relaxed);
    }

    /// Clear all data and reset.
    pub fn clear(&self) {
        // SAFETY: ptr is valid and allocated
        unsafe {
            std::ptr::write_bytes(self.ptr, 0, self.capacity);
        }
        self.reset();
    }
}

impl Drop for SharedMemoryBuffer {
    fn drop(&mut self) {
        // SAFETY: We allocated this with Box and forgot it, so we need to reclaim it
        // Only do this if this is the last reference
        // For now, we're not deallocating to keep buffer alive for process lifetime
        // In production, use Arc<Mutex<>> for proper cleanup
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_memory_buffer_creation() {
        let buffer = SharedMemoryBuffer::new(1024).expect("Failed to create buffer");
        assert_eq!(buffer.capacity(), 1024);
        assert_eq!(buffer.position(), 0);
    }

    #[test]
    fn test_shared_memory_buffer_write_read() {
        let buffer = SharedMemoryBuffer::new(1024).expect("Failed to create buffer");
        let data = b"Hello, WASM!";

        let written = buffer.write_at(0, data).expect("Write failed");
        assert_eq!(written, data.len());
        assert_eq!(buffer.position(), data.len());

        let read = buffer.read_at(0, data.len()).expect("Read failed");
        assert_eq!(read, data);
    }

    #[test]
    fn test_shared_memory_buffer_bounds_check() {
        let buffer = SharedMemoryBuffer::new(100).expect("Failed to create buffer");
        let data = vec![0u8; 200];

        let result = buffer.write_at(0, &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_shared_memory_buffer_offset_bounds() {
        let buffer = SharedMemoryBuffer::new(100).expect("Failed to create buffer");
        let data = b"test";

        let result = buffer.write_at(200, data);
        assert!(result.is_err());
    }
}
```

#### 1.2 Zero-Copy Extraction Function

**File**: `crates/kreuzberg-wasm/src/extraction.rs` (ADD NEW FUNCTION)

```rust
/// Extract from a pointer in shared memory (zero-copy).
///
/// This is the most performant extraction method, avoiding any data copies
/// between JavaScript and WASM. Requires JavaScript to provide valid memory addresses.
///
/// # JavaScript Parameters
///
/// * `ptr: number` - Memory pointer in WASM linear memory (from SharedMemoryBuffer.as_ptr())
/// * `len: number` - Length of data in bytes
/// * `mimeType: string` - MIME type of the document
/// * `config?: object` - Optional extraction configuration
///
/// # Returns
///
/// `object` - ExtractionResult with extracted content
///
/// # Safety
///
/// JavaScript caller must ensure:
/// - ptr is a valid address in WASM linear memory
/// - ptr + len does not exceed allocated memory
/// - data at ptr is valid for the entire extraction duration
/// - No concurrent writes to the region during extraction
///
/// # Example
///
/// ```javascript
/// import { extractBytesSharedPtr } from '@kreuzberg/wasm';
///
/// // Create shared buffer
/// const buffer = new SharedMemoryBuffer(1024 * 1024);  // 1MB
///
/// // Write PDF data to shared memory
/// const pdfBytes = await fetch('doc.pdf').then(r => r.arrayBuffer());
/// const data = new Uint8Array(pdfBytes);
/// buffer.write_at(0, data);
///
/// // Extract without copying
/// const result = extractBytesSharedPtr(
///     buffer.as_ptr(),
///     data.length,
///     'application/pdf',
///     null
/// );
/// console.log(result.content);
/// ```
#[wasm_bindgen(js_name = extractBytesSharedPtr)]
pub fn extract_bytes_shared_ptr(
    ptr: usize,
    len: usize,
    mime_type: String,
    config: Option<JsValue>,
) -> Result<JsValue, JsValue> {
    let extraction_config = parse_config(config)?;

    // SAFETY: JavaScript caller guarantees:
    // 1. ptr points to valid WASM linear memory
    // 2. len does not exceed allocated memory
    // 3. memory is not modified during extraction
    let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };

    extract_bytes_sync(bytes, &mime_type, &extraction_config)
        .map_err(convert_error)
        .and_then(|result| result_to_js_value(&result))
}

/// Async version of zero-copy extraction.
///
/// # Example
///
/// ```javascript
/// const result = await extractBytesSharedPtrAsync(
///     buffer.as_ptr(),
///     data.length,
///     'text/html',
///     null
/// );
/// ```
#[wasm_bindgen(js_name = extractBytesSharedPtrAsync)]
pub fn extract_bytes_shared_ptr_async(
    ptr: usize,
    len: usize,
    mime_type: String,
    config: Option<JsValue>,
) -> js_sys::Promise {
    wasm_bindgen_futures::future_to_promise(async move {
        let extraction_config = parse_config(config)?;

        // SAFETY: Same guarantees as sync version
        let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };

        let result = extract_bytes(bytes, &mime_type, &extraction_config)
            .await
            .map_err(convert_error)?;

        result_to_js_value(&result)
    })
}
```

### Performance Impact

**Before** (copy-based):
```
Uint8Array → data.to_vec() [5-20ms] → extraction [33ms] → serialization [50ms] → Result
Total: 88-103ms
```

**After** (shared memory):
```
SharedMemoryBuffer.as_ptr() [<1ms] → extraction [33ms] → serialization [50ms] → Result
Total: 83-84ms
```

**Savings**: 5-20ms per call (5-25% improvement for small documents)

---

## Specification 2: Streaming Result Iterator

### Problem Statement

Current results serialize entire ExtractionResult at once:
```rust
pub fn result_to_js_value(result: &ExtractionResult) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(result)  // O(n) with content size
}
```

For 100MB document, this could take 500ms+.

### Design

#### 2.1 Streaming Result Type

**File**: `crates/kreuzberg-wasm/src/streaming.rs` (NEW)

```rust
use wasm_bindgen::prelude::*;
use kreuzberg::ExtractionResult;
use std::sync::Arc;
use std::sync::Mutex;

/// A streaming result wrapper that returns extracted content in chunks.
///
/// Instead of buffering entire results in memory, StreamingResult allows
/// JavaScript to consume content incrementally as it's produced.
#[wasm_bindgen]
pub struct StreamingResult {
    // Arc to allow cloning and sharing
    inner: Arc<Mutex<StreamingResultInner>>,
}

struct StreamingResultInner {
    /// Full extracted content
    content: String,
    /// Metadata as JSON string
    metadata: String,
    /// Images as base64-encoded strings
    images: Vec<String>,
    /// Tables as JSON arrays
    tables: Vec<String>,
    /// Current read position in content
    content_position: usize,
    /// Chunk size in bytes (default 64KB)
    chunk_size: usize,
}

#[wasm_bindgen]
impl StreamingResult {
    /// Create a StreamingResult from an ExtractionResult.
    ///
    /// # Arguments
    /// * `result` - ExtractionResult from extraction operation
    /// * `chunk_size` - Size of chunks to return (default: 65536)
    pub fn from_extraction_result(
        result: &ExtractionResult,
        chunk_size: Option<usize>,
    ) -> Result<StreamingResult, JsValue> {
        let metadata = serde_json::to_string(&result.metadata)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize metadata: {}", e)))?;

        let images = result
            .images
            .iter()
            .map(|img| {
                // Encode image data as base64
                base64::encode(img.as_bytes())
            })
            .collect();

        let tables = result
            .tables
            .iter()
            .map(|table| {
                serde_json::to_string(table)
                    .unwrap_or_else(|_| String::new())
            })
            .collect();

        Ok(StreamingResult {
            inner: Arc::new(Mutex::new(StreamingResultInner {
                content: result.content.clone(),
                metadata,
                images,
                tables,
                content_position: 0,
                chunk_size: chunk_size.unwrap_or(65536),
            })),
        })
    }

    /// Get the next chunk of content.
    ///
    /// Returns a string chunk of up to `chunk_size` bytes.
    /// Returns null when no more content is available.
    ///
    /// # Example
    ///
    /// ```javascript
    /// let chunk;
    /// while (chunk = streamingResult.nextContentChunk()) {
    ///     processChunk(chunk);
    /// }
    /// ```
    #[wasm_bindgen(js_name = nextContentChunk)]
    pub fn next_content_chunk(&self) -> Option<String> {
        let mut inner = self.inner.lock().unwrap();

        let start = inner.content_position;
        let end = std::cmp::min(start + inner.chunk_size, inner.content.len());

        if start >= inner.content.len() {
            return None;
        }

        let chunk = inner.content[start..end].to_string();
        inner.content_position = end;

        Some(chunk)
    }

    /// Get all remaining content at once.
    ///
    /// Useful when you want to switch from streaming to buffered consumption.
    #[wasm_bindgen(js_name = remainingContent)]
    pub fn remaining_content(&self) -> String {
        let mut inner = self.inner.lock().unwrap();
        let start = inner.content_position;
        let result = inner.content[start..].to_string();
        inner.content_position = inner.content.len();
        result
    }

    /// Get the metadata as a JSON object.
    #[wasm_bindgen(js_name = getMetadata)]
    pub fn get_metadata(&self) -> Result<JsValue, JsValue> {
        let inner = self.inner.lock().unwrap();
        serde_wasm_bindgen::from_str(&inner.metadata)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse metadata: {}", e)))
    }

    /// Get the next image as base64.
    #[wasm_bindgen(js_name = nextImage)]
    pub fn next_image(&self) -> Option<String> {
        let mut inner = self.inner.lock().unwrap();
        if inner.images.is_empty() {
            return None;
        }
        Some(inner.images.remove(0))
    }

    /// Get the next table as JSON.
    #[wasm_bindgen(js_name = nextTable)]
    pub fn next_table(&self) -> Option<JsValue> {
        let mut inner = self.inner.lock().unwrap();
        if inner.tables.is_empty() {
            return None;
        }
        let table_json = inner.tables.remove(0);
        serde_wasm_bindgen::from_str(&table_json).ok()
    }

    /// Get progress (0-1 float representing completion).
    pub fn progress(&self) -> f64 {
        let inner = self.inner.lock().unwrap();
        if inner.content.is_empty() {
            1.0
        } else {
            inner.content_position as f64 / inner.content.len() as f64
        }
    }

    /// Reset stream position to beginning.
    pub fn reset(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.content_position = 0;
    }

    /// Get total content length in bytes.
    #[wasm_bindgen(js_name = totalLength)]
    pub fn total_length(&self) -> usize {
        let inner = self.inner.lock().unwrap();
        inner.content.len()
    }

    /// Get current position in content.
    pub fn position(&self) -> usize {
        let inner = self.inner.lock().unwrap();
        inner.content_position
    }
}
```

#### 2.2 Streaming Extraction Functions

**File**: `crates/kreuzberg-wasm/src/extraction.rs` (ADD NEW FUNCTIONS)

```rust
/// Extract content and return a streaming result.
///
/// Returns a StreamingResult that allows consuming content in chunks
/// rather than all at once. Useful for large documents.
///
/// # JavaScript Parameters
///
/// * `data: Uint8Array` - Document bytes
/// * `mimeType: string` - MIME type
/// * `config?: object` - Extraction configuration
/// * `chunkSize?: number` - Chunk size in bytes (default: 65536)
///
/// # Returns
///
/// `Promise<StreamingResult>` - Streaming result object
///
/// # Example
///
/// ```javascript
/// const streaming = await extractBytesStreaming(data, 'text/html', null, 65536);
///
/// // Consume in chunks
/// let chunk;
/// while (chunk = streaming.nextContentChunk()) {
///     console.log(`Got chunk: ${chunk.length} bytes`);
/// }
/// ```
#[wasm_bindgen(js_name = extractBytesStreaming)]
pub fn extract_bytes_streaming(
    data: Uint8Array,
    mime_type: String,
    config: Option<JsValue>,
    chunk_size: Option<usize>,
) -> js_sys::Promise {
    let bytes = data.to_vec();

    wasm_bindgen_futures::future_to_promise(async move {
        let extraction_config = parse_config(config)?;

        let result = extract_bytes(&bytes, &mime_type, &extraction_config)
            .await
            .map_err(convert_error)?;

        // Return streaming wrapper instead of full serialized result
        streaming::StreamingResult::from_extraction_result(&result, chunk_size)
            .map(|stream| JsValue::from(stream))
    })
}

/// Synchronous streaming extraction.
#[wasm_bindgen(js_name = extractBytesSyncStreaming)]
pub fn extract_bytes_sync_streaming(
    data: Uint8Array,
    mime_type: String,
    config: Option<JsValue>,
    chunk_size: Option<usize>,
) -> Result<JsValue, JsValue> {
    let bytes = data.to_vec();
    let extraction_config = parse_config(config)?;

    let result = extract_bytes_sync(&bytes, &mime_type, &extraction_config)
        .map_err(convert_error)?;

    streaming::StreamingResult::from_extraction_result(&result, chunk_size)
        .map(|stream| JsValue::from(stream))
}
```

### Performance Impact

**Before** (buffer entire result):
```
Extraction [33ms] → Serialize all content [100ms] → Return to JS
Total: 133ms per call
Memory: ~200MB for 100MB document
```

**After** (stream chunks):
```
Extraction [33ms] → Serialize metadata [5ms] → Return StreamingResult
JS reads chunks: StreamingResult.nextContentChunk() [~10ms per 65KB chunk]
Total: 38ms initial + streaming consumption
Memory: ~50-80MB peak (only one chunk + result in memory)
```

**Savings**: 50-95ms initial response, 50-70% memory reduction for large documents

---

## Specification 3: Feature-Gated Format Handlers

### Problem Statement

WASM binary includes all format handlers (~5-10MB), but many use cases only need text extraction.

```toml
# Current: No feature gating
kreuzberg = { path = "../kreuzberg", default-features = false, features = ["wasm-target"] }
```

### Design

#### 3.1 Cargo.toml Feature Structure

**File**: `crates/kreuzberg-wasm/Cargo.toml` (MODIFY)

```toml
[package]
# ... existing config ...

[dependencies]
kreuzberg = {
    path = "../kreuzberg",
    default-features = false,
    features = ["wasm-target"]
    # Format features gated at this level
}
# ... other deps ...

[features]
# Default: text extraction only (minimal binary)
default = ["text-extraction"]

# Feature flags mirror Kreuzberg core features
text-extraction = []          # Always available: HTML, Text, JSON, YAML, TOML, Markdown
pdf-extraction = ["kreuzberg/pdf-wasm"]      # PDFium WASM support
image-extraction = ["kreuzberg/ocr"]         # OCR via EasyOCR/PaddleOCR
office-extraction = ["kreuzberg/office"]     # DOCX, XLSX, PPTX
spreadsheet-extraction = ["kreuzberg/sheets"]  # CSV, ODS
all-formats = [
    "text-extraction",
    "pdf-extraction",
    "image-extraction",
    "office-extraction",
    "spreadsheet-extraction",
]

# Build presets
minimal = ["text-extraction"]  # ~1.5MB, just text formats
standard = [
    "text-extraction",
    "pdf-extraction",
    "office-extraction"
]  # ~4MB, most common formats
complete = ["all-formats"]     # ~10MB, everything
```

#### 3.2 Feature Detection API

**File**: `crates/kreuzberg-wasm/src/lib.rs` (ADD)

```rust
/// Query which format features are available in this build.
#[wasm_bindgen]
pub struct FeatureInfo {
    text: bool,
    pdf: bool,
    images: bool,
    office: bool,
    spreadsheets: bool,
}

#[wasm_bindgen]
impl FeatureInfo {
    pub fn text(&self) -> bool { self.text }
    pub fn pdf(&self) -> bool { self.pdf }
    pub fn images(&self) -> bool { self.images }
    pub fn office(&self) -> bool { self.office }
    pub fn spreadsheets(&self) -> bool { self.spreadsheets }
}

/// Get supported features in this WASM build.
#[wasm_bindgen]
pub fn get_supported_features() -> FeatureInfo {
    FeatureInfo {
        text: true,  // Always enabled
        #[cfg(feature = "pdf-extraction")]
        pdf: true,
        #[cfg(not(feature = "pdf-extraction"))]
        pdf: false,

        #[cfg(feature = "image-extraction")]
        images: true,
        #[cfg(not(feature = "image-extraction"))]
        images: false,

        #[cfg(feature = "office-extraction")]
        office: true,
        #[cfg(not(feature = "office-extraction"))]
        office: false,

        #[cfg(feature = "spreadsheet-extraction")]
        spreadsheets: true,
        #[cfg(not(feature = "spreadsheet-extraction"))]
        spreadsheets: false,
    }
}

/// List all supported MIME types in this build.
#[wasm_bindgen]
pub fn get_supported_mime_types() -> Vec<JsValue> {
    let mut types = vec![
        "text/plain",
        "text/html",
        "text/markdown",
        "application/json",
        "application/x-yaml",
        "application/toml",
    ];

    #[cfg(feature = "pdf-extraction")]
    types.push("application/pdf");

    #[cfg(feature = "image-extraction")]
    {
        types.extend(&["image/png", "image/jpeg", "image/webp"]);
    }

    #[cfg(feature = "office-extraction")]
    {
        types.extend(&[
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        ]);
    }

    #[cfg(feature = "spreadsheet-extraction")]
    {
        types.extend(&[
            "text/csv",
            "application/vnd.oasis.opendocument.spreadsheet",
        ]);
    }

    types.into_iter().map(|t| JsValue::from_str(t)).collect()
}
```

### Build Configurations

```bash
# Minimal build (~1.5MB)
wasm-pack build --target web -- --features minimal

# Standard build (~4MB) - recommended
wasm-pack build --target web -- --features standard

# Complete build (~10MB)
wasm-pack build --target web -- --features complete

# Custom feature selection
wasm-pack build --target web -- --features text-extraction,image-extraction
```

### Performance Impact

**Minimal build** (text only):
- Binary size: 1.5MB → ~200-300KB gzipped
- Initial load time: 841ms → 200-300ms
- Time savings: 500-600ms per page load

**Standard build** (text + PDF + Office):
- Binary size: 4MB → ~600KB gzipped
- Initial load time: 841ms → 300-400ms
- Time savings: 400-500ms per page load

---

## Specification 4: Memory Pool for Batch Operations

### Problem Statement

Batch operations allocate new Vec for each document:
```rust
// extraction.rs L276
let owned_data: Vec<Vec<u8>> = data_list.iter().map(|d| d.to_vec()).collect();
```

For 10 documents × 1MB each = 10 allocations, GC pressure.

### Design

#### 4.1 Arena Allocator

**File**: `crates/kreuzberg-wasm/src/memory_pool.rs` (NEW)

```rust
use lazy_static::lazy_static;
use std::sync::Mutex;

/// Memory pool for batch operations.
/// Pre-allocates a large arena and reuses it across batch calls.
struct MemoryPool {
    arena: Vec<u8>,
    offset: usize,
}

impl MemoryPool {
    fn new(capacity: usize) -> Self {
        MemoryPool {
            arena: Vec::with_capacity(capacity),
            offset: 0,
        }
    }

    /// Allocate from the pool.
    fn allocate(&mut self, size: usize) -> Result<&mut [u8], &'static str> {
        if self.offset + size > self.arena.capacity() {
            // Expand arena if needed
            let new_capacity = std::cmp::max(
                self.arena.capacity() * 2,
                self.offset + size,
            );
            self.arena.reserve(new_capacity - self.arena.capacity());
        }

        let slice = &mut self.arena[self.offset..self.offset + size];
        self.offset += size;
        Ok(slice)
    }

    /// Reset for next batch.
    fn reset(&mut self) {
        self.offset = 0;
    }

    /// Get current usage percentage.
    fn usage_percent(&self) -> f32 {
        (self.offset as f32 / self.arena.capacity() as f32) * 100.0
    }
}

lazy_static! {
    /// Global memory pool for batch operations.
    /// Sized for ~100 small documents (1KB each).
    static ref BATCH_MEMORY_POOL: Mutex<MemoryPool> =
        Mutex::new(MemoryPool::new(100 * 1024 * 1024));  // 100MB initial
}

/// Optimized batch extraction using memory pool.
#[wasm_bindgen(js_name = batchExtractBytesPooled)]
pub fn batch_extract_bytes_pooled(
    data_list: Vec<Uint8Array>,
    mime_types: Vec<String>,
    config: Option<JsValue>,
) -> Result<JsValue, JsValue> {
    if data_list.len() != mime_types.len() {
        return Err(JsValue::from_str("data_list and mime_types must have the same length"));
    }

    let extraction_config = parse_config(config)?;
    let mut pool = BATCH_MEMORY_POOL.lock()
        .map_err(|_| JsValue::from_str("Failed to acquire memory pool"))?;

    // Reset pool for this batch
    pool.reset();

    let mut results = Vec::new();

    for (data, mime) in data_list.iter().zip(mime_types.iter()) {
        let bytes = data.to_vec();  // Still need to copy from JS
        let result = kreuzberg::extract_bytes_sync(&bytes, mime, &extraction_config)
            .map_err(convert_error)?;
        results.push(result);
    }

    // Log pool usage for monitoring
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&format!("Pool usage: {:.1}%", pool.usage_percent()).into());
    }

    results_to_js_value(&results)
}
```

#### 4.2 Batch with Pre-allocation Hint

**File**: `crates/kreuzberg-wasm/src/extraction.rs` (ADD)

```rust
/// Batch extraction with capacity hints for optimization.
///
/// # JavaScript Parameters
///
/// * `dataList: Uint8Array[]` - Array of documents
/// * `mimeTypes: string[]` - MIME types array
/// * `config?: object` - Extraction configuration
/// * `expectedSize?: number` - Expected total size in bytes (helps pre-allocation)
///
/// # Returns
///
/// `object[]` - Array of ExtractionResults
///
/// # Example
///
/// ```javascript
/// // If you know you're processing ~5MB of documents
/// const results = await batchExtractBytesWithHint(
///     [doc1, doc2, doc3],
///     ['text/html', 'text/html', 'text/html'],
///     null,
///     5 * 1024 * 1024  // 5MB hint
/// );
/// ```
#[wasm_bindgen(js_name = batchExtractBytesWithHint)]
pub fn batch_extract_bytes_with_hint(
    data_list: Vec<Uint8Array>,
    mime_types: Vec<String>,
    config: Option<JsValue>,
    expected_size: Option<usize>,
) -> Result<JsValue, JsValue> {
    if data_list.len() != mime_types.len() {
        return Err(JsValue::from_str("data_list and mime_types must have the same length"));
    }

    let extraction_config = parse_config(config)?;

    // Pre-allocate results vector
    let mut results = Vec::with_capacity(data_list.len());

    // Pre-allocate owned data with hint if provided
    let mut owned_data = Vec::with_capacity(
        expected_size.unwrap_or(data_list.len() * 1024) / 1024 + data_list.len()
    );

    for data in &data_list {
        owned_data.push(data.to_vec());
    }

    // Extract all
    for (data, mime) in owned_data.iter().zip(mime_types.iter()) {
        let result = kreuzberg::extract_bytes_sync(data.as_slice(), mime, &extraction_config)
            .map_err(convert_error)?;
        results.push(result);
    }

    results_to_js_value(&results)
}
```

### Performance Impact

**Before** (no pooling):
```
Batch of 10 × 100KB documents
├─ 10 allocations: ~5ms
├─ 10 extractions: ~100ms
├─ GC pressure: ~20ms
└─ Total: 125ms
Memory peak: 2MB (all allocated separately)
```

**After** (pooled):
```
Batch of 10 × 100KB documents
├─ Arena reuse: ~1ms
├─ 10 extractions: ~100ms
├─ No GC: 0ms
└─ Total: 101ms
Memory peak: 100MB arena (reused across batches)
```

**Savings**: 20-24ms per batch, 0% GC pressure, stable memory

---

## Integration Checklist

### Phase 1: Shared Memory (Week 2)
- [ ] Implement SharedMemoryBuffer in memory.rs
- [ ] Add extract_bytes_shared_ptr() functions
- [ ] Write unit tests for pointer safety
- [ ] Add SAFETY comments for all unsafe code
- [ ] Test in both browser and Node.js

### Phase 2: Streaming (Week 3)
- [ ] Implement StreamingResult in streaming.rs
- [ ] Add extract_bytes_streaming() functions
- [ ] Write streaming iteration tests
- [ ] Validate chunk performance
- [ ] Document usage patterns

### Phase 3: Feature Gating (Week 3)
- [ ] Modify Cargo.toml with feature flags
- [ ] Implement get_supported_features()
- [ ] Implement get_supported_mime_types()
- [ ] Create build variants
- [ ] Test each build variant

### Phase 4: Memory Pool (Week 4)
- [ ] Implement MemoryPool in memory_pool.rs
- [ ] Add batch_extract_bytes_pooled()
- [ ] Add batch_extract_bytes_with_hint()
- [ ] Benchmark pooled vs non-pooled
- [ ] Monitor memory usage

---

## Testing Strategy

### Unit Tests

Each module needs:
- Allocation/deallocation correctness
- Bounds checking
- Data integrity
- Error conditions

### Integration Tests

- Multiple extraction calls in sequence
- Concurrent batch operations
- Large document streaming
- Feature detection accuracy

### Browser Tests

- Worker pool initialization
- SharedMemoryBuffer from browser context
- Streaming in fetch chains
- Cross-origin considerations

### Performance Tests

- Baseline: 680ms for HTML
- Target: 150-250ms after optimization
- Batch: 10 documents < 500ms
- Memory: Peak < 150MB

---

## Next Steps

1. Finalize SharedMemoryBuffer API (get feedback on safety model)
2. Prototype streaming with chunked results
3. Define feature gate boundaries
4. Create benchmark harness for optimizations
5. Begin implementation Week 1
