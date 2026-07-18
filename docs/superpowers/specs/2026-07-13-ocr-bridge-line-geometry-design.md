# OCR Bridge: Real Per-Line Geometry

**Date:** 2026-07-13
**Status:** Approved for planning

## Problem

The web-ui OCR layout viewer (`packages/xberg-web-ui/src/components/LayoutBlocks.tsx`,
`src/lib/ocr-to-layout.ts`, `OcrLine` in `src/lib/types.ts`) exists to display per-line
bounding boxes and per-page layout — but that code lives only on the unmerged branch
`lot3/web-ui-advanced-viz` (tip `0432511d42`, "fix(web-ui): address CodeRabbit review
findings on PR #27"). It is not present on `claude/zealous-cohen-872fe9` today.

On that branch, `engine.worker.ts`'s `handleOcr` calls `XbergEngine.ocr()` (the
`@xberg-io/xberg-wasm` binding, `crates/xberg-wasm`), gets back a flat string, and
fakes "lines" by splitting on `\n` with `confidence` hardcoded to `1` and no bbox at
all. `OcrLine` has no `page` field. This was flagged by CodeRabbit on PR #27 and
deliberately left as a documented limitation rather than faked further.

**The data isn't actually missing.** `packages/xberg-wasm-runtime/src/ocr.ts`'s
`createOcr()` (backed by `ppu-paddle-ocr`, injected into `XbergEngine` via
`createXbergRuntimeFactory`) already computes real per-line `{text, confidence,
bbox:{x,y,w,h}}` and returns it to the Rust side. The bug is at the Rust↔JS boundary:
`crates/xberg-wasm/src/bridge/ocr.rs`'s `call_injected_ocr` reads only the `.text`
field off that result via `Reflect::get` and discards `lines` entirely; `engine.rs`'s
`ocr()` then returns a bare string to its JS caller.

## Scope

**In scope:**
1. Bring `lot3/web-ui-advanced-viz`'s web-ui OCR-layout code into this branch.
2. Fix the Rust↔JS bridge so real per-line `text`/`confidence`/`bbox` survive the
   round trip instead of being discarded.
3. Wire `engine.worker.ts`'s `handleOcr` to use the real lines instead of
   newline-splitting a flat string.
4. Add an optional `page` field to `OcrLine` and make `ocr-to-layout.ts` group by it
   when present, as forward-compatible plumbing.

**Out of scope:** multi-page PDF rasterization (splitting a multi-page PDF into
per-page images before OCR). No caller anywhere in the codebase currently splits a
document into pages before calling OCR — `DocumentPageClient.tsx` passes an entire
uploaded file's raw bytes to `ocrLayout()` in one call. Building that rasterization
step is a separate, larger project (new dependency for PDF page rendering, per-page
looping in the worker, result aggregation). This spec adds the `page` field so the
data model doesn't need to change again once that project happens, but nothing
populates it yet.

## Design

### 1. Branch integration

Merge `lot3/web-ui-advanced-viz` into `claude/zealous-cohen-872fe9`. `crates/xberg-wasm`
is byte-identical between the branches (verified via `git diff`), so the Rust bridge
work below applies cleanly regardless of merge order.

### 2. Rust bridge (`crates/xberg-wasm/src/bridge/ocr.rs`, `src/engine.rs`)

New types in `bridge/ocr.rs`, following the existing `ner()` bridge's pattern
(`bridge/ner.rs:108` already parses a whole injected JS result via
`serde_wasm_bindgen::from_value` into a typed `Vec<Entity>` rather than picking
fields with `Reflect::get`):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrBbox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrLineResult {
    pub text: String,
    #[serde(default)]
    pub confidence: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<OcrBbox>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    pub text: String,
    #[serde(default)] // missing `lines` degrades to empty vec, not an error
    pub lines: Vec<OcrLineResult>,
}
```

Field names (`x, y, w, h`) match `packages/xberg-wasm-runtime/src/types.ts`'s
existing `OcrResult` TS type exactly — no renaming/translation layer needed.

Changes:
- `resolve_ocr`, `resolve_ocr_with_timeout`, `call_injected_ocr`, `fallback_ocr`:
  return type changes from `Result<String, JsValue>` to `Result<OcrResult, JsValue>`.
- `call_injected_ocr`'s body replaces the manual `Reflect::get(&js_val, "text")` with
  `serde_wasm_bindgen::from_value(js_val)` — parse the whole result at once.
- `fallback_ocr` (Tesseract path) only ever returns `Err(...)` today regardless of
  input (its backend constructor is `pub(crate)` in `xberg`, not reachable from
  `xberg-wasm`) — signature-only change, no behavior change.
- `engine.rs::ocr()`: replaces `Ok(JsValue::from_str(&text))` with
  `Ok(serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))?)`,
  mirroring `ner()`'s return exactly.

**Docs to update** (both currently describe the old string-only contract):
- `bridge/ocr.rs` module doc — currently says the injected backend "returns a promise
  resolving to `{ text: "..." }`".
- `engine.rs::XbergEngine::new`'s doc comment — currently documents
  `` `ocr` — object with `ocr(imageBytes, opts): Promise<string>` ``.
- `engine.rs::ocr()`'s own doc comment — currently says "returning extracted text".

**No `Cargo.toml` changes needed** — `serde`, `serde_json`, `serde-wasm-bindgen`,
`js-sys` are already dependencies. (Note: `crates/xberg-wasm/Cargo.toml` is
alef-generated; this spec doesn't touch it.)

**Generated TS types:** `crates/xberg-wasm/pkg/nodejs/xberg_wasm.d.ts` is a build
output (regenerated by the crate's normal wasm build step, not hand-edited). Its
`ocr()` signature is `Promise<any>` today and stays `Promise<any>` after this change
(same as `ner()` — a `JsValue`-returning method without a dedicated `#[wasm_bindgen]`
struct or `tsify` type). Consumers assert the concrete shape via a TS type, consistent
with how `ner()` is already consumed.

### 3. Web-ui wiring

**`packages/xberg-web-ui/src/engine/engine.worker.ts`** (`handleOcr`):

```ts
async function handleOcr(msg: OcrMessage): Promise<void> {
  try {
    const xEngine = await getEngine();
    const result = (await xEngine.ocr(msg.bytes, undefined)) as {
      text: string;
      lines: Array<{
        text: string;
        confidence: number;
        bbox?: { x: number; y: number; w: number; h: number };
      }>;
    };
    post({ type: "ocrResult", requestId: msg.requestId, lines: result.lines });
  } catch (err) {
    post({ type: "error", requestId: msg.requestId, message: err instanceof Error ? err.message : String(err) });
  }
}
```

No more `text.split(/\r?\n/)`, no more hardcoded `confidence: 1`.

**`packages/xberg-web-ui/src/lib/types.ts`** (`OcrLine`): update the doc comment (the
existing one explains why bbox/confidence/page are absent — that reasoning becomes
stale) and add an optional `page` field:

```ts
export interface OcrLine {
  text: string;
  confidence: number;
  bbox?: { x: number; y: number; w: number; h: number };
  page?: { number: number; width: number; height: number };
}
```

Nothing populates `page` yet — documented in the comment as inert forward-compatible
plumbing, not a live feature, pending the out-of-scope rasterization work.

**`packages/xberg-web-ui/src/lib/ocr-to-layout.ts`** (`toParsedOcrOutput`): group
lines by `line.page?.number ?? 1` instead of hardcoding `page: 1` for every block,
using each line's own `page.width`/`page.height` when present, else the
caller-supplied `width`/`height` defaults. With no line carrying `page` today, output
is behaviorally identical to current (one chunk, page 1) — the grouping logic is real
and activates automatically once something supplies page numbers, rather than being
faked twice.

### 4. Testing

- **Rust** (`crates/xberg-wasm/tests/hybrid_dispatch.rs`): update
  `resolve_ocr_with_injected_stub` (stub must now return `lines` too) and
  `resolve_ocr_without_injected_returns_error` (return type change only); add a new
  case asserting the soft-default — a stub returning `{text: 'x'}` with no `lines`
  field produces `lines: []`, not an error.
- **`xberg-wasm-runtime`**: `ocr.test.ts`/`full-pipeline.test.ts` already assert
  `lines`/bbox shape on the injected-side `OcrInterface` output — unaffected by this
  change, no updates needed.
- **`xberg-web-ui`**: add/update a `handleOcr` unit test — mock `xEngine.ocr` to
  return `{text, lines: [{text, confidence, bbox}]}`, assert the worker posts those
  lines through unchanged (not split, not hardcoded). Add a `toParsedOcrOutput` test
  for the page-grouping branch (mix of lines with and without `page` → multiple
  chunks), alongside its existing default-page-1 test.
- **Manual verification**: run the real worker OCR path against an actual image in
  the browser to confirm bounding boxes render in `LayoutBlocks`, per this project's
  convention of testing UI changes in a live browser rather than relying on
  type-checks alone.

## Out of scope / follow-up

Multi-page PDF rasterization: splitting a multi-page PDF into per-page images before
OCR, looping the OCR call per page, and aggregating results with real page identity.
No rasterization step exists anywhere in the pipeline today (confirmed: the only
PDF-page-aware code, `packages/xberg-web-ui/src/components/ui/pdf-viewer.tsx`, is a
viewer built on `@embedpdf/core` and never rasterizes a page for OCR). This is a
separate project once needed.
