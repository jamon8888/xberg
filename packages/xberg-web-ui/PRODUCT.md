# Product

## Register

product

## Platform

web

## Users

Analysts and knowledge workers who need to ingest documents into a RAG pipeline without sending raw data to a server. They work in browser-based workflows, often with sensitive or regulated content (contracts, financial docs, medical records). Primary job-to-be-done: **create a collection, upload files, and have them extracted, redacted, embedded, and synced — all client-side — so nothing leaves their machine until they explicitly authorize it.**

## Product Purpose

A Next.js static-export browser app that runs the full xberg extraction pipeline (text → OCR → NER → PII redaction → chunk → embed) in WebAssembly via `xberg-wasm-runtime` / `xberg-wasm`. Users create folders (mapped to MCP collections), drop documents, watch the engine process them locally, and click **Sync** to push only the redacted output + encrypted rehydration map to the MCP HTTP server. Success = zero-friction ingest flow *and* PII-safe by default (redaction happens before any network call).

## Positioning

**WASM-powered RAG in the browser.** The only document ingestion UI where the entire extract→OCR→NER→redact→embed pipeline runs locally in WebAssembly, with server sync as an explicit user action.

## Brand Personality

Bold, technical, transparent. Three words: **confident, capable, honest**. The UI should feel like a power tool — visible engine state (SyncBar), explicit user actions (no hidden background uploads), and no marketing fluff. Evoke: "this runs the real thing in your browser."

## Anti-references

Enterprise dashboard bloat: dense tables with 20 columns, mystery-meat navigation, hidden background jobs, generic cream/slate SaaS palettes, tiny gray body text, loading spinners that never explain *what* is loading.

## Design Principles

1. **Show the engine** — SyncBar surfaces pending count and errors; no invisible background work.
2. **User initiates every network call** — Create folder, Upload, Sync are explicit buttons.
3. **Redaction is visible** — Document viewer shows redacted text; the map is encrypted but its existence is surfaced.
4. **Technical honesty over polish** — If WASM init takes 2s, say "Initializing engine…" not a skeleton spinner.
5. **Keyboard-first, accessible by default** — WCAG 2.1 AA is the floor, not the ceiling.

## Accessibility & Inclusion

WCAG 2.1 AA compliance. Semantic HTML, focus management in dialogs, `role="alert"` for live errors, `prefers-reduced-motion` respected (all transitions crossfade/instant), sufficient contrast on all text (slate-900 on white, slate-100 on slate-900). No color-only state changes.