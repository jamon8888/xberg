# Xberg WASM Shared JS Runtime Layer — Design Spec

**Date:** 2026-07-02
**Status:** Approved (design)
**Scope:** Sub-project **C** — the shared JavaScript/TypeScript runtime layer that supplies the host-injected implementations the wasm engine (B) consumes.
**Depends on:** [B — Shared WASM Engine](2026-07-02-xberg-wasm-engine-design.md).
**Consumed by:** D (browser UI), E (MCP server).

## Purpose

Sub-project B (`XbergEngine`) links no concrete embedder, vector store, NER, or OCR — it declares them as injected interfaces bridged via JSPI. **C is the single shared package that implements those interfaces once**, so that both the browser (D) and the MCP server (E) inject the *same* code. C is the reason "one engine, two frontends" holds: the frontends differ only in glue, not in ML/storage logic.

Package: `packages/xberg-wasm-runtime/` (TypeScript, ESM). It exports factory functions returning objects that satisfy the engine's injection descriptor `{ embedder, store, ner?, ocr? }`.

## Components

### 1. Embedder (`embedder.ts`)
- Backed by **transformers.js v3** (`@huggingface/transformers`) with the **ONNX Runtime Web** backend.
- WebGPU execution provider when available; silent WASM-CPU fallback.
- Default model: a MiniLM-class sentence-transformer (small, ~8–12 ms/inference on CPU per the WebGPU-vs-WASM benchmark; GPU not required).
- Interface: `embed(texts: string[]) -> Promise<Float32Array[]>`; vectors L2-normalized before return (matches the `rag-embeddings` unit-length rule).
- Batch size default 32 (per `rag-embeddings`).

### 2. Vector store (`store.ts`)
- **wa-sqlite** over **OPFS** (browser) and better-sqlite3-compatible path (Node, sub-project E may substitute native) behind one interface.
- Implements the engine's `VectorStore` surface: `upsert`/`query`/`delete`/`listCollections`/`dropCollection`.
- Runs in a **dedicated Web Worker** (OPFS SQLite is Worker-only). Main thread ↔ Worker via a typed message channel.
- Vector search: `sqlite-vec` compiled to wasm, or a JS cosine index over stored vectors if `sqlite-vec` wasm is unavailable — selected at init, logged.
- `upsert` idempotent on `(collectionId, sourceId, chunkIndex)`; `query` returns results sorted by score desc.

### 3. NER fast-path (`ner.ts`)
- transformers.js v3 token-classification / GLiNER2-ONNX pipeline over ONNX Runtime Web (WebGPU).
- Interface: `ner(text, opts) -> Promise<Entity[]>`.
- Optional: if not constructed/injected, the engine falls back to in-binary Candle NER.

### 4. OCR fast-path (`ocr.ts`)
- **`ppu-paddle-ocr`** (MIT) over ONNX Runtime — `onnxruntime-web` in browser, `onnxruntime-node` in Node.
- Default model **PP-OCRv6 small** (unified, 50+ languages). WebGPU auto-acceleration (2–5×), silent WASM fallback. INT8 variant available for extra WASM speedup.
- Interface: `ocr(bytes, opts) -> Promise<OcrResult>` mapped to the engine's `OcrResult` shape.
- Optional: if not injected, the engine falls back to in-binary Tesseract (`ocr-wasm`).
- Browser entry uses `ppu-paddle-ocr/web` (canvas-native, no OpenCV.js bundle); Node uses the default entry.

### 5. Model cache manager (`cache.ts`)
- Persists model weights (embeddings, NER, OCR — 5–500 MB) in **OPFS** (browser) / `~/.cache/xberg` (Node) after first fetch; never re-downloads per load.
- Exposes `warm(models[])` and `status()` — mirrors the existing MCP `WarmupManager` responsibilities.
- Sets `ort.env.wasm.wasmPaths` to self-hosted ORT binaries so no CDN dependency at runtime.

### 6. Async binding shim (`async_shim.ts`)
> **Mechanism Correction (2026-07-02 review):** The engine bridges use **standard async `wasm-bindgen`** (`JsFuture` over the injected JS Promises), not JavaScript Promise Integration. So this component does **not** need `WebAssembly.Suspending`/`promising` — the injected factories just return objects with `async` methods returning Promises, and `wasm-bindgen-futures` awaits them. This works in all modern browsers and Node.
- Provides the thin adapter that shapes each factory's `async` methods to the exact names/signatures the engine expects (`embed`, `upsertDocument`, `query`, `ner`, `ocr`, …).
- Enforces **single-flight per engine instance** — the engine holds `&self` across an `await`, so overlapping calls on one handle must be serialized by the caller. Documented and asserted here. (If a future revision adopts true JSPI for a sync-Rust path, re-entrancy limits would apply then; not now.)

## Architecture / data flow

```
frontend (D or E)
  └─ constructs C factories → { embedder, store, ner, ocr }
       └─ new XbergEngine(config, injection)   [wasm, B]
            ├─ ingest: extract(in-binary) → PII(in-binary) → embedder.embed(JSPI→ORT-Web) → store.upsert(JSPI→Worker/OPFS)
            └─ query:  embedder.embed → store.query → chunks
       └─ ocr(): engine → ocr.ocr (JSPI→ppu-paddle-ocr) or in-binary Tesseract
       └─ ner(): engine → ner.ner (JSPI→transformers.js) or in-binary Candle
```

## Error handling

- Every injected method rejects with a structured error `{ code, message }`; the engine's bridges surface it as a Rust-side engine error, never a panic.
- Model-fetch failure in `cache.ts` surfaces a typed error the frontend can render (offline + no cached model → OCR/NER degrade to in-binary fallback rather than hard-fail).

## Testing

- Vitest unit tests per component with a tiny fixture model (or a stub ORT session) — no network in CI.
- Store: round-trip upsert/query against wa-sqlite in a Worker (happy path + idempotent re-upsert).
- OCR: golden-image → expected text on one small fixture through `ppu-paddle-ocr`.
- Contract test: the factory outputs satisfy the engine's injection descriptor types (typecheck + a wasm-engine smoke ingest/query using real C impls).

## Non-goals

- The engine itself (B) and the frontends (D, E).
- React Native / mobile OCR entry (`ppu-paddle-ocr/mobile`) — recorded as a future option, out of scope (Chrome/Node only for now).
- Remote embedding providers (liter-llm `wasm-http`) — deferred; default is local ORT-Web.

## Constraints (inherited from B)

- **COOP/COEP headers** required by the browser host (D) for SharedArrayBuffer / multithreaded ORT-Web and OPFS SQLite — without them ORT silently runs single-threaded and wa-sqlite OPFS is unavailable.
- All inference + store operations run **off the main thread** (Worker).
- Chrome/Edge is the **product** target (WebGPU + OPFS SQLite + COOP/COEP), not an async-mechanism limit — see the Mechanism Correction in §6. The async engine layer is browser-portable; broadening to Safari/Firefox is gated only by WebGPU/OPFS maturity, not by the bridge design.
