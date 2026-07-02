# Xberg WASM Browser UI — Design Spec

**Date:** 2026-07-02
**Status:** Approved (design)
**Scope:** Sub-project **D** — a minimal browser UI for document intelligence, running entirely client-side on the shared wasm engine.
**Depends on:** [B — Shared WASM Engine](2026-07-02-xberg-wasm-engine-design.md), [C — Shared JS Runtime Layer](2026-07-02-xberg-wasm-runtime-layer-design.md).

## Purpose

A small, self-contained web app (Chrome/Edge) that demonstrates and exercises the full stack — extract, OCR, NER, anonymization, RAG — with **no server**: all computation runs in the browser via the wasm engine (B) driven by the injected runtime (C). Data never leaves the device.

Package: `apps/xberg-web/` (Vite + TypeScript, minimal framework or none). Ships static; served with the required isolation headers.

## Scope of the UI (v1, deliberately minimal)

1. **Drop / pick a file** → `extract` → render text + metadata.
2. **OCR toggle** for image/scanned PDF → injected PaddleOCR (or Tesseract fallback), show recognized text + confidence.
3. **Anonymize panel** → `detectPii` (list detections by category) → `redact` (choose mask/hash/token_replace) → download redacted output; for `token_replace`, download the encrypted map + a `rehydrate` box (passphrase → original).
4. **NER panel** → `ner` → entities highlighted inline.
5. **RAG panel** → create/select a collection → `ingest` dropped docs → `query` box → ranked chunks. Store persists in OPFS across reloads.

No auth, no multi-user, no theming system beyond a clean single-page layout. YAGNI enforced.

## Architecture

```
index.html  (COOP/COEP via server headers)
  main.ts        — UI wiring, calls engine facade
  engine.ts      — constructs C factories, builds XbergEngine (B), single-flight queue
  worker.ts      — hosts wa-sqlite/OPFS store + heavy inference (from C), postMessage bridge
  ui/            — small view modules per panel (extract, ocr, anon, ner, rag)
```

- The wasm engine + ORT-Web inference + SQLite all live in a **Worker**; the main thread only does DOM + message passing. This satisfies C's Worker requirement and keeps the UI responsive.
- Model download progress (from C's `cache.ts`) surfaced as a first-run banner.

## Deployment (hard requirement)

Served with:
```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```
Without these, SharedArrayBuffer is unavailable → ORT-Web falls back to single-threaded (3–4× slower) and OPFS SQLite from the Worker fails. The dev server (Vite) and any static host must set them. A startup self-check calls `crossOriginIsolated` and shows a hard warning if false.

## Data flow

Identical to C's flow; the UI is a thin driver. Example — RAG ingest:
`file → engine.extract → engine.ingest(collection)` where `ingest` internally does PII (in-binary) → embed (Worker/ORT-Web) → upsert (Worker/OPFS). Query: `engine.query(text, collection, k) → render chunks`.

## Error handling

- All engine calls wrapped; errors rendered inline per panel (never a blank failure).
- Offline + no cached model → OCR/NER panels show "using offline fallback" (Tesseract/Candle) rather than erroring.
- `rehydrate` with wrong passphrase → clear "decryption failed" message (AES-GCM auth tag mismatch), no crash.

## Testing

- Component tests (Vitest + jsdom) for UI view modules with a mocked engine facade.
- One Playwright end-to-end smoke test in headless Chrome with isolation headers: load app → drop a fixture PDF → assert extracted text appears; ingest → query → assert a chunk returns.
- Verify `crossOriginIsolated === true` in the E2E environment (guards the header regression).

## Non-goals

- Server-side rendering, accounts, collaboration.
- Safari/Firefox support — deferred due to **WebGPU + OPFS SQLite maturity**, not the async mechanism (the engine's async layer is portable; see the engine spec's Mechanism Correction).
- Mobile layout beyond responsive basics.
- Packaging as a browser extension (possible later reuse of C, out of scope).
