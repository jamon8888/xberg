#!/usr/bin/env node
// Ensure crates/xberg-wasm/src/lib.rs declares the hand-written engine/bridge
// modules.
//
// Why this exists: lib.rs is fully regenerated (not merged) by
// `alef generate --lang wasm`, and alef has no mechanism to preserve
// hand-written module declarations across regeneration (checked alef.toml and
// `alef --help` as of alef 0.36.2). But `src/engine.rs` (the XbergEngine
// OCR/NER/RAG bridge consumed by packages/xberg-wasm-runtime and mcp-server)
// and `src/bridge/*.rs` are hand-written and MUST be reachable from lib.rs or
// Rust silently omits them from the crate — the binding compiles and ships
// without XbergEngine at all, which is exactly the bug this prevents from
// recurring (see PR #1253).
//
// Idempotent: safe to run any number of times. Wired into every wasm-pack
// build script in crates/xberg-wasm/package.json and the alef.toml
// [crates.test.wasm] before-hook, so any build after an alef regen self-heals.

import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const libPath = fileURLToPath(new URL("../crates/xberg-wasm/src/lib.rs", import.meta.url));

const MARKER = "pub mod bridge;";
// A Rust panic inside a `#[wasm_bindgen] pub async fn` has no default JS
// visibility on wasm32 -- worse, a known wasm-bindgen limitation means
// `future_to_promise` can leave the JS Promise permanently pending (neither
// resolved nor rejected) if the panic unwinds through its executor, instead
// of rejecting it (https://github.com/rustwasm/wasm-bindgen/issues/2392).
// That reproduces as a silent, zero-CPU, zero-error hang from the caller's
// side -- exactly the "extract()/ingest() sometimes never resolves" symptom
// this hook exists to make debuggable. Routes every panic message (and
// wasm-bindgen's own backtrace-ish info) to `console.error` instead.
const PANIC_MARKER = "console_error_panic_hook::set_once();";
const MODULES_BLOCK = [
  "",
  "// Hand-written modules (NOT alef-generated). Re-inserted by",
  "// scripts/ensure-wasm-mods.mjs after every alef regeneration — do not",
  "// remove. See that script's header for why alef cannot preserve these.",
  "pub mod bridge;",
  "pub mod engine;",
  "pub use engine::XbergEngine;",
  "",
].join("\n");
const PANIC_BLOCK = [
  "",
  "// Panics inside async wasm-bindgen exports otherwise vanish silently",
  "// (see scripts/ensure-wasm-mods.mjs). Re-inserted by that script.",
  "#[cfg(target_arch = \"wasm32\")]",
  "#[wasm_bindgen(start)]",
  "pub fn init_panic_hook() {",
  "    console_error_panic_hook::set_once();",
  "}",
  "",
].join("\n");

let src;
try {
  src = readFileSync(libPath, "utf8");
} catch (err) {
  console.error(`ensure-wasm-mods: cannot read ${libPath}: ${err.message}`);
  process.exit(1);
}

const hasModules = src.includes(MARKER);
const hasPanicHook = src.includes(PANIC_MARKER);
if (hasModules && hasPanicHook) {
  console.log("ensure-wasm-mods: lib.rs already wired, nothing to do");
  process.exit(0);
}

// Insert after the last top-level `use ...;` line (the generated file opens
// with a header, allow-attributes, then a block of use statements).
const lines = src.split("\n");
let lastUse = -1;
for (let i = 0; i < lines.length; i++) {
  if (/^use\s.*;\s*$/.test(lines[i])) lastUse = i;
  // Stop scanning at the first item definition — use statements past this
  // point are inside modules, not top-level.
  if (/^(pub\s+)?(struct|enum|fn|impl|mod|trait)\b/.test(lines[i]) && lastUse !== -1) break;
}

if (lastUse === -1) {
  console.error("ensure-wasm-mods: found no top-level `use` statement in lib.rs; refusing to guess an insertion point");
  process.exit(1);
}

const block = (hasModules ? "" : MODULES_BLOCK) + (hasPanicHook ? "" : PANIC_BLOCK);
lines.splice(lastUse + 1, 0, block);
writeFileSync(libPath, lines.join("\n"));
console.log(`ensure-wasm-mods: inserted wiring (modules=${!hasModules} panicHook=${!hasPanicHook}) after line ${lastUse + 1} of lib.rs`);
