import type { XbergEngine } from "@xberg-io/xberg-wasm";
import { createXbergRuntimeFactory } from "xberg-wasm-runtime";
import type { InjectionDescriptor } from "xberg-wasm-runtime";
import { getCacheDir } from "./paths.js";

let _engine: XbergEngine | null = null;
let _injection: InjectionDescriptor | null = null;
let _initPromise: Promise<XbergEngine> | null = null;

/**
 * Build the `XbergEngine` (B) once, wiring it to C's shared runtime factory.
 *
 * C's single public entry point `createXbergRuntimeFactory` constructs and
 * validates the whole injection descriptor (`{ embedder, store, ner?, ocr? }`)
 * that B's constructor consumes — we do not touch the per-capability factories
 * directly.
 *
 * `CacheConfig` exposes `nodeCachePath` (model/ORT cache) but no explicit
 * store location — the vector-store backend location is C's internal
 * concern. C's default Node store persists to `<nodeCachePath>/store.sqlite3`
 * (see `store-node.ts`'s `createNodeVectorStore`), so ingested documents
 * survive process restarts; pass `nodeStorePath` explicitly to relocate it.
 */
export function initializeEngine(): Promise<XbergEngine> {
  // Guard the async startup with a cached promise so concurrent callers share a
  // single initialization instead of each racing past the `_engine === null`
  // check and constructing a duplicate engine.
  if (_initPromise !== null) return _initPromise;

  _initPromise = (async () => {
    if (_engine !== null) return _engine;

    const cacheDir = getCacheDir();

    const injection = await createXbergRuntimeFactory({ nodeCachePath: cacheDir });
    _injection = injection;

    // Per Task 1 spec, engine construction uses default config.
    const { XbergEngine } = await import("@xberg-io/xberg-wasm");
    _engine = new XbergEngine({}, injection);

    return _engine;
  })();

  return _initPromise;
}

/** Return the initialized singleton engine, or throw if not yet initialized. */
export function getEngine(): XbergEngine {
  if (_engine === null) {
    throw new Error("Engine not initialized. Call initializeEngine() first.");
  }
  return _engine;
}

/** Return the injected runtime descriptor (embedder/store/ner/ocr), or throw if not yet initialized. */
export function getRuntime(): InjectionDescriptor {
  if (_injection === null) {
    throw new Error("Engine not initialized. Call initializeEngine() first.");
  }
  return _injection;
}
