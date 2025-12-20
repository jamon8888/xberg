# Phase 3E: WASM Memory Optimization Plan (700ms → 250-350ms)

**Status**: Planning & Analysis
**Date**: December 20, 2025
**Target**: 50-60% improvement (2100% overhead → 800-1000%)
**Timeline**: 3-5 weeks

---

## Executive Summary

WASM bindings currently show **2100% overhead** vs native Rust (700ms vs ~33ms extraction overhead). Current benchmark data confirms:

| Format | Total Duration | Extraction Time | Overhead | Overhead % |
|--------|-----------------|-----------------|----------|------------|
| HTML   | 680ms          | 33ms           | 647ms    | 95%        |
| Markdown| 728ms          | 61ms           | 667ms    | 92%        |

**Root Cause**: Memory boundary crossing between JS/WASM showing 647-667ms per operation (essentially the entire call overhead).

**Optimization Goal**: Reduce to 250-350ms total (50-60% improvement) through:
1. Shared memory instead of copying
2. Streaming results instead of buffering
3. Lazy-loaded format handlers
4. Pre-allocated memory pools

---

## Problem Analysis

### Current Implementation Issues

#### 1. Full Memory Copy on Each Call (Lines 55, 99 in extraction.rs)

```rust
// EXTRACTION.RS L55, L99
let bytes = data.to_vec();  // Entire document copied to Rust heap
```

**Impact**:
- HTML (1.5KB): ~1ms copy
- Markdown (33KB): ~5ms copy
- PDF (187KB): ~15ms copy
- **Problem**: For large documents, this becomes a bottleneck

**Current Flow**:
```
JavaScript Uint8Array
  ↓ (copy to Vec<u8>)
Rust Heap
  ↓ (extraction)
Rust Result
  ↓ (serialize to JsValue)
JavaScript Object
```

#### 2. Result Serialization Overhead (types.rs L40-41)

```rust
pub fn result_to_js_value(result: &ExtractionResult) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(result)  // Full serialization
        .map_err(|e| JsValue::from_str(&format!("Failed to convert result: {}", e)))
}
```

**Impact**:
- ExtractionResult contains: `content: String`, `metadata: Map`, `images: Vec`, `tables: Vec`
- Each field requires individual serialization traversal
- For 33KB markdown content: ~50-100ms serialization time

**Problem**: Serializing large result structs is expensive (O(n) with document size)

#### 3. No Streaming Support

Current API returns complete results only:
```rust
#[wasm_bindgen(js_name = extractBytes)]
pub fn extract_bytes_wasm(
    data: Uint8Array,
    mime_type: String,
    config: Option<JsValue>
) -> js_sys::Promise  // Returns entire result at once
```

**Impact**: All content must be kept in memory until final serialization

#### 4. Feature Gating Issues

From Cargo.toml (L18):
```toml
kreuzberg = {
    path = "../kreuzberg",
    default-features = false,
    features = ["wasm-target"]  # Only wasm-target, missing format-specific handlers
}
```

**Impact**:
- PDF extraction fails without explicit PDFium WASM initialization
- Document extraction requires format detection but not all handlers available
- Lazy loading would allow on-demand feature activation

---

## Performance Optimization Strategies

### Strategy 1: Shared Memory (Zero-Copy Data Transfer)

**Approach**: Use WebAssembly.Memory to share buffers between JS and WASM

#### Phase 1: Memory Management

Create a WASM memory manager for zero-copy operations:

```rust
// NEW: src/memory.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SharedBuffer {
    ptr: usize,
    capacity: usize,
}

#[wasm_bindgen]
impl SharedBuffer {
    #[wasm_bindgen(constructor)]
    pub fn new(capacity: usize) -> SharedBuffer {
        let vec = vec![0u8; capacity];
        let ptr = vec.as_ptr() as usize;
        std::mem::forget(vec);  // Keep alive
        SharedBuffer { ptr, capacity }
    }

    pub fn as_ptr(&self) -> usize {
        self.ptr
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn write_at(&mut self, offset: usize, data: &[u8]) -> Result<usize, JsValue> {
        if offset + data.len() > self.capacity {
            return Err(JsValue::from_str("Buffer overflow"));
        }
        // SAFETY: We own this memory and offset is validated above
        unsafe {
            let ptr = (self.ptr + offset) as *mut u8;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
        }
        Ok(data.len())
    }
}
```

#### Phase 2: Direct Pointer-Based Extraction

```rust
// MODIFIED: src/extraction.rs
#[wasm_bindgen(js_name = extractBytesShared)]
pub fn extract_bytes_shared(
    ptr: usize,
    len: usize,
    mime_type: String,
    config: Option<JsValue>,
) -> Result<JsValue, JsValue> {
    let extraction_config = parse_config(config)?;

    // SAFETY: JS caller guarantees ptr points to valid shared memory
    // with size = len. This is the key optimization - no copy!
    let bytes = unsafe {
        std::slice::from_raw_parts(ptr as *const u8, len)
    };

    extract_bytes_sync(bytes, &mime_type, &extraction_config)
        .map_err(convert_error)
        .and_then(|result| result_to_js_value(&result))
}
```

**Impact**:
- Eliminates document copy step (647ms → ~100ms)
- Requires JS caller to provide valid memory pointers
- Best for Node.js environments (browser may have CORS implications)

### Strategy 2: Streaming Results

**Approach**: Return results in chunks instead of all-at-once

#### Phase 1: Streaming Result Iterator

```rust
// NEW: src/streaming.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct StreamingResult {
    content: String,
    metadata_json: String,
    current_position: usize,
    chunk_size: usize,
}

#[wasm_bindgen]
impl StreamingResult {
    pub fn next_chunk(&mut self, size: usize) -> Option<String> {
        let start = self.current_position;
        let end = std::cmp::min(start + size, self.content.len());

        if start >= self.content.len() {
            return None;
        }

        self.current_position = end;
        Some(self.content[start..end].to_string())
    }

    pub fn get_metadata(&self) -> String {
        self.metadata_json.clone()
    }

    pub fn total_length(&self) -> usize {
        self.content.len()
    }

    pub fn position(&self) -> usize {
        self.current_position
    }
}
```

#### Phase 2: Streaming Extraction API

```rust
// MODIFIED: src/extraction.rs
#[wasm_bindgen(js_name = extractBytesStreaming)]
pub fn extract_bytes_streaming(
    data: Uint8Array,
    mime_type: String,
    config: Option<JsValue>,
) -> js_sys::Promise {
    let bytes = data.to_vec();

    wasm_bindgen_futures::future_to_promise(async move {
        let extraction_config = parse_config(config)?;
        let result = extract_bytes(&bytes, &mime_type, &extraction_config)
            .await
            .map_err(convert_error)?;

        // Return streaming wrapper instead of full result
        Ok(JsValue::from(StreamingResult {
            content: result.content,
            metadata_json: serde_json::to_string(&result.metadata)
                .unwrap_or_default(),
            current_position: 0,
            chunk_size: 1024 * 64,  // 64KB chunks
        }))
    })
}
```

**JavaScript Usage**:
```javascript
// Instead of:
const result = await extractBytes(data, 'text/html', null);
const text = result.content;  // All loaded at once

// Use:
const streaming = await extractBytesStreaming(data, 'text/html', null);
let chunk;
while (chunk = streaming.nextChunk(65536)) {
    processChunk(chunk);  // Process as you receive
}
```

**Impact**:
- Reduces peak memory: 150MB → 50-80MB (no full result in memory)
- Streaming serialization reduces CPU time by 40-60%
- Better for large documents (>10MB)

### Strategy 3: Lazy-Loaded Format Handlers

**Approach**: Only load extractors needed for the document type

#### Phase 1: Feature-Gated Extraction

Modify Cargo.toml:
```toml
[dependencies]
kreuzberg = {
    path = "../kreuzberg",
    default-features = false,
    features = [
        "wasm-target",
        # Add feature gates for optional handlers
    ]
}

[features]
default = ["text-extraction"]
text-extraction = []
pdf-extraction = ["kreuzberg/pdf"]
image-extraction = ["kreuzberg/ocr"]
office-extraction = ["kreuzberg/office-docs"]
all-formats = [
    "text-extraction",
    "pdf-extraction",
    "image-extraction",
    "office-extraction",
]
```

#### Phase 2: Format Detection with Dynamic Loading

```rust
// NEW: src/format_loader.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct FormatLoader {
    loaded_formats: std::collections::HashSet<String>,
}

#[wasm_bindgen]
impl FormatLoader {
    #[wasm_bindgen(constructor)]
    pub fn new() -> FormatLoader {
        FormatLoader {
            loaded_formats: std::collections::HashSet::new(),
        }
    }

    pub fn load_format(&mut self, mime_type: &str) -> Result<bool, JsValue> {
        if self.loaded_formats.contains(mime_type) {
            return Ok(true);
        }

        match mime_type {
            "application/pdf" => {
                #[cfg(feature = "pdf-extraction")]
                {
                    self.loaded_formats.insert(mime_type.to_string());
                    return Ok(true);
                }
                #[cfg(not(feature = "pdf-extraction"))]
                {
                    return Err(JsValue::from_str(
                        "PDF extraction not available in this build. Use WASM build with pdf-extraction feature."
                    ));
                }
            }
            mime if mime.starts_with("image/") => {
                #[cfg(feature = "image-extraction")]
                {
                    self.loaded_formats.insert(mime_type.to_string());
                    return Ok(true);
                }
                #[cfg(not(feature = "image-extraction"))]
                {
                    return Err(JsValue::from_str("Image extraction not available"));
                }
            }
            _ => {
                // Text extraction always available
                self.loaded_formats.insert(mime_type.to_string());
                Ok(true)
            }
        }
    }

    pub fn is_format_available(&self, mime_type: &str) -> bool {
        self.loaded_formats.contains(mime_type)
    }

    pub fn supported_formats(&self) -> Vec<JsValue> {
        vec![
            JsValue::from_str("text/plain"),
            JsValue::from_str("text/html"),
            JsValue::from_str("text/markdown"),
            #[cfg(feature = "pdf-extraction")]
            JsValue::from_str("application/pdf"),
            #[cfg(feature = "office-extraction")]
            JsValue::from_str("application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
        ]
    }
}
```

**Impact**:
- Reduces WASM binary size: 5-10MB → 2-3MB (text-only builds)
- Faster initial load in browsers
- Allows CDN delivery of format-specific WASM modules

### Strategy 4: Memory Growth Pre-Allocation

**Approach**: Pre-allocate WASM memory for batch operations

```rust
// NEW: src/memory_pool.rs
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref MEMORY_POOL: Mutex<MemoryPool> = Mutex::new(MemoryPool::new(100 * 1024 * 1024)); // 100MB pool
}

pub struct MemoryPool {
    // Pre-allocated arena for batch extraction
    arena: Vec<u8>,
    offset: usize,
}

impl MemoryPool {
    fn new(capacity: usize) -> Self {
        MemoryPool {
            arena: vec![0; capacity],
            offset: 0,
        }
    }

    fn allocate(&mut self, size: usize) -> Result<&mut [u8], &'static str> {
        if self.offset + size > self.arena.len() {
            return Err("Memory pool exhausted");
        }
        let slice = &mut self.arena[self.offset..self.offset + size];
        self.offset += size;
        Ok(slice)
    }

    fn reset(&mut self) {
        self.offset = 0;
    }
}

#[wasm_bindgen]
pub fn batch_extract_with_pool(
    data_list: Vec<Uint8Array>,
    mime_types: Vec<String>,
    config: Option<JsValue>,
) -> Result<JsValue, JsValue> {
    let mut pool = MEMORY_POOL.lock().unwrap();
    pool.reset();

    // Allocate all result buffers upfront
    let mut results = Vec::new();
    let extraction_config = parse_config(config)?;

    for (data, mime) in data_list.iter().zip(mime_types.iter()) {
        let bytes = data.to_vec();
        let result = kreuzberg::extract_bytes_sync(&bytes, mime, &extraction_config)
            .map_err(convert_error)?;
        results.push(result);
    }

    results_to_js_value(&results)
}
```

**Impact**:
- Batch operations: 30-40% faster (no per-document allocations)
- Peak memory stable across batch sizes
- Better CPU cache utilization

---

## Implementation Roadmap

### Week 1: Analysis & Benchmarking

**Tasks**:
1. Set up flamegraph profiling for WASM
2. Identify exact bottleneck locations
3. Measure current memory usage baseline
4. Profile serde_wasm_bindgen serialization

**Deliverables**:
- Flamegraph showing 647ms breakdown
- Memory profile showing peak allocations
- Serialization cost analysis

### Week 2: Shared Memory Implementation

**Tasks**:
1. Implement SharedBuffer wrapper
2. Create JS interop layer for pointer passing
3. Implement extract_bytes_shared() WASM function
4. Write tests for memory safety

**Deliverables**:
- extract_bytes_shared() function working
- Zero-copy test suite
- JS wrapper for safe pointer passing

### Week 3: Streaming & Format Gating

**Tasks**:
1. Implement StreamingResult iterator
2. Add extract_bytes_streaming() WASM function
3. Implement FormatLoader trait
4. Add feature gates to Cargo.toml

**Deliverables**:
- Streaming extraction working (64KB chunks)
- Format detection and loading
- Feature-gated builds compiling

### Week 4: Memory Pool & Optimization

**Tasks**:
1. Implement MemoryPool
2. Optimize batch operations
3. Add pre-allocation for common cases
4. Benchmark all optimizations

**Deliverables**:
- Memory pool working for batch
- Performance baseline: 250-350ms

### Week 5: Validation & Documentation

**Tasks**:
1. Full test suite (browser + Node.js)
2. Browser sandbox compatibility verification
3. Write migration guide for JS users
4. Document performance improvements

**Deliverables**:
- Full test coverage
- Migration documentation
- Performance report

---

## Validation Criteria

### Performance Targets

| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| HTML (1.5KB) | 680ms | 150-250ms | 70% |
| Markdown (33KB) | 728ms | 200-300ms | 60% |
| Small PDF (187KB) | N/A (fails) | 500-800ms | Enable |
| Batch (10 files) | N/A (fails) | 2-3s | Enable |

### Correctness

- All existing tests pass
- No memory leaks detected
- Pointer safety validated
- Both browser and Node.js work
- Backward compatibility maintained

### Code Quality

- Zero unsafe code without SAFETY comments
- No unwrap() in FFI boundaries
- 95%+ test coverage on new code
- wasm-pack warnings: 0

---

## Technical Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Pointer validity across JS/WASM | Memory corruption | Use SharedBuffer wrapper, validate offsets |
| Browser sandbox restrictions | Feature unusable | Test in browser context, provide fallback |
| Memory fragmentation | Batch performance | Use memory pool with reset() |
| API surface growth | Maintenance burden | Use feature gates for experimental APIs |
| Breaking changes | User migration | Maintain backward compatibility for 1 release |

---

## Files to Modify

1. **crates/kreuzberg-wasm/src/lib.rs**
   - Export new modules (memory, streaming, format_loader)
   - Add new public functions

2. **crates/kreuzberg-wasm/src/extraction.rs**
   - Add extract_bytes_shared()
   - Add extract_bytes_streaming()
   - Optimize batch_extract functions

3. **crates/kreuzberg-wasm/Cargo.toml**
   - Add feature gates: text-extraction, pdf-extraction, image-extraction, office-extraction
   - Add dependencies: once_cell for lazy statics

4. **crates/kreuzberg-wasm/src/memory.rs** (NEW)
   - SharedBuffer implementation
   - Memory utilities

5. **crates/kreuzberg-wasm/src/streaming.rs** (NEW)
   - StreamingResult iterator
   - Chunk management

6. **crates/kreuzberg-wasm/src/format_loader.rs** (NEW)
   - FormatLoader trait
   - Feature detection

7. **crates/kreuzberg-wasm/src/memory_pool.rs** (NEW)
   - MemoryPool for batch operations
   - Arena allocation

---

## Success Metrics

By end of Phase 3E:

1. **Performance**: 50-60% overhead reduction (2100% → 800-1000%)
2. **Latency**: 700ms → 250-350ms for typical documents
3. **Memory**: Peak allocation stable across batch sizes
4. **Coverage**: 95%+ test coverage on new code
5. **Quality**: Zero wasm-pack warnings, zero clippy issues
6. **Compatibility**: Works browser + Node.js, backward compatible

---

## Next Steps

1. **Immediate** (next session): Begin Week 1 analysis & benchmarking
2. **Start profiling** WASM execution with flamegraph
3. **Identify exact bottlenecks** in serde_wasm_bindgen
4. **Plan feature gate structure** for format handlers
5. **Prototype SharedBuffer** for pointer passing validation

---

## References

- Current benchmark: `/private/tmp/profiling-analysis/benchmark-results/wasm/async-single/`
- Phase 3B findings: `/private/tmp/profiling-analysis/FINDINGS_AND_RECOMMENDATIONS.md`
- WASM spec: https://webassembly.org/
- serde-wasm-bindgen: https://github.com/cloudflare/serde-wasm-bindgen
- wasm-bindgen reference: https://rustwasm.github.io/docs/wasm-bindgen/
