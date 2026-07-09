# WASM MCP Server — performance baseline

**Sub-project:** WASM-backed MCP server (`mcp-server/`) on `@xberg-io/xberg-wasm` + `xberg-wasm-runtime`.
**Bench file:** [`mcp-server/benchmarks/engine_vs_native.bench.ts`](../../../mcp-server/benchmarks/engine_vs_native.bench.ts)

## What this measures

Steady-state latency of the wasm engine's three hot paths, with the engine and the
384-dim embedder model **pre-warmed once** in `beforeAll` so per-iteration timings
exclude the one-off model download/load:

- `engine.extract` — extraction of a 284-byte `text/plain` document.
- `engine.ingest` — extract + embed + store one document.
- `engine.query` — embed a query + vector-retrieve (`top_k = 5`).

## No native comparison arm (by design)

The plan's optional "vs native" arm (`@xberg-io/xberg` + `xberg-rag-node`) is **omitted**:
the native NAPI binding is not built in this worktree, so there is no in-process native
path to time against. The whole point of the migration is that the MCP server no longer
depends on that binding; a like-for-like native number would require checking out the
pre-migration server and building the `.node` artifact, which is out of scope here.

## Numbers

**Not captured in this session.** The measurement run could not complete in this
environment for reasons unrelated to the migrated code:

- The embedder model cache lives on an external SSD image mounted at
  `/Volumes/xberg-build` (the repo's `node_modules/.pnpm/@huggingface+transformers@*/
  .../.cache` is a symlink into it). That volume **detached mid-session**, leaving the
  symlink dangling — so `initializeEngine()` can neither load the cached model
  (offline: `ENOTDIR` on the dangling cache path) nor re-fetch it (online:
  `TypeError: fetch failed` — Hugging Face is unreachable over this env's constrained,
  IPv6/NAT64-restricted network).
- Note also that Vitest 1.6.1's **experimental** `bench()` under-reports `async`
  benchmark functions here (0 samples / `NaN` hz). The reliable way to capture numbers
  is a `performance.now()` harness (see below), not `vitest bench`.

The functional correctness of all three paths is already proven by the test suite
(`tests/ingest.test.ts`, `tests/query.test.ts`, `tests/e2e.test.ts`) — this document is
strictly about latency.

## How to capture numbers (stable environment)

1. Remount the model-cache volume if it uses the external SSD image
   (`hdiutil attach "/Volumes/Extreme SSD/xberg-build.sparsebundle"`), or point the
   HF/transformers cache at a local path with a warm copy of the 384-dim embedder model.
2. `source /Volumes/xberg-build/env.sh` (sets `NODE_OPTIONS=--dns-result-order=ipv4first`;
   needed only for the first, online model fetch).
3. Reliable harness (drop-in Vitest test, `performance.now()` — median of 20 iterations
   after 3 warmup iterations, per path):

   ```ts
   // pre-warm: await initializeEngine(); ensure a 384-dim collection; ingest one doc.
   async function time(n, f) { for (let i=0;i<3;i++) await f();
     const t=[]; for (let i=0;i<n;i++){const a=performance.now(); await f(); t.push(performance.now()-a);}
     const s=t.sort((x,y)=>x-y); return { median: s[n>>1], mean: t.reduce((x,y)=>x+y)/n }; }
   // time(20, () => engine.extract(input, { extraction_timeout_secs: null }))
   // time(20, () => engine.ingest(doc, "bench_col"))
   // time(20, () => engine.query("…", "bench_col", 5))
   ```

Record the median/mean per path in the table below and commit.

| Operation | median (ms) | mean (ms) | notes |
|---|---|---|---|
| `engine.extract` (284 B text) | _tbd_ | _tbd_ | wasm |
| `engine.ingest` (extract+embed+store) | _tbd_ | _tbd_ | wasm |
| `engine.query` (embed+retrieve, k=5) | _tbd_ | _tbd_ | wasm |

## Expectations

No strict target is defined; this baseline is for regression tracking. `extract` is pure
wasm CPU work; `ingest`/`query` are dominated by embedding inference (transformers.js on
the same ORT/CPU path regardless of the wasm vs native store), so those two should be
governed by embedder latency, not the store backend.
