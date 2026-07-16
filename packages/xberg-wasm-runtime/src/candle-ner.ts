/**
 * Browser-only wiring for the in-binary Candle GLiNER2 NER backend
 * (`crates/xberg-gliner-candle`, exposed via `initCandleNer` in
 * `crates/xberg-wasm/src/bridge/ner.rs`). Unlike `ner.ts`'s transformers.js
 * `bert-base-NER` pipeline (fixed PER/ORG/LOC/MISC labels), this is real
 * GLiNER2: a schema-prompt zero-shot NER model that accepts arbitrary
 * label strings (email/phone/person/location/... — see
 * `crates/xberg/src/types/entity.rs`'s `EntityCategory`) at call time.
 *
 * `initCandleNer` is a free function on the wasm module (not a method on
 * `XbergEngine`) that populates a `thread_local!` inside the compiled
 * binary -- once called, `crates/xberg-wasm/src/bridge/ner.rs::resolve_ner`
 * uses it automatically as the fallback path whenever `XbergEngine` is NOT
 * given an injected `ner` object. That means the caller (`factory.ts`) must
 * omit `ner` from the returned `InjectionDescriptor` for this to actually
 * take effect, not just call `initCandleNer` and also inject the JS NER.
 *
 * This module has no dependency on `xberg-wasm-runtime` itself being
 * browser-only: `@xberg-io/xberg-wasm` resolves to `pkg/nodejs` by default
 * and to `pkg/web` only via the browser-side bundler alias xberg-web-ui's
 * next.config.js already sets up. `initCandleNer`/`init` exist on both
 * targets, but this module's entry point (`initCandleNerBackend`) is only
 * ever invoked when `typeof window !== "undefined"` -- Candle-in-WASM
 * exists specifically to cover the browser, where a native ONNX Runtime
 * binary (what Node/mcp-server uses instead) cannot run at all.
 */
import type { CacheConfig } from "./types.js";

// fastino/gliner2-privacy-filter-PII-multi: the pinned Candle-compatible
// GLiNER2 PII model (~1.24GB total across the three files below). Confirmed
// via the HF Hub API to ship model.safetensors (single-file, not sharded --
// required, see model.rs's `from_buffered_safetensors`), tokenizer.json, and
// encoder_config/config.json (a DeBERTa-v2-style config, deserialized
// directly as `candle_transformers::models::debertav2::Config`).
const DEFAULT_CANDLE_NER_BASE_URL = "https://huggingface.co/fastino/gliner2-privacy-filter-PII-multi/resolve/main/";
const CANDLE_NER_CACHE_NAME = "xberg-candle-ner-v1";

interface CandleNerModelBytes {
	safetensors: Uint8Array;
	tokenizerJson: Uint8Array;
	encoderConfigJson: Uint8Array;
}

/**
 * Fetch `url`, using the browser Cache Storage API to persist the response
 * across page loads when available. Falls back to a plain fetch if the
 * Cache API is unavailable or a cache write fails (matches the resilience
 * pattern transformers.js itself uses for its own model cache -- see the
 * "Unable to add response to browser cache" warning path in
 * `@huggingface/transformers`' hub.js, which never treats a cache-write
 * failure as fatal).
 */
async function fetchWithCache(url: string): Promise<Uint8Array> {
	const cacheApi = typeof caches !== "undefined" ? caches : undefined;
	if (cacheApi) {
		try {
			const cache = await cacheApi.open(CANDLE_NER_CACHE_NAME);
			const cached = await cache.match(url);
			if (cached) {
				return new Uint8Array(await cached.arrayBuffer());
			}
		} catch (err) {
			console.warn(`[candle-ner] cache read failed for ${url}, fetching directly:`, err);
		}
	}

	const response = await fetch(url);
	if (!response.ok) {
		throw new Error(`[candle-ner] fetch failed for ${url}: ${response.status} ${response.statusText}`);
	}

	if (cacheApi) {
		try {
			const cache = await cacheApi.open(CANDLE_NER_CACHE_NAME);
			await cache.put(url, response.clone());
		} catch (err) {
			console.warn(`[candle-ner] unable to cache response for ${url}:`, err);
		}
	}

	return new Uint8Array(await response.arrayBuffer());
}

/**
 * Download the three files `CandleBackend::from_bytes` requires. Fetched in
 * parallel; the safetensors file dominates total size (~1.2GB of the
 * ~1.24GB total), so this is a genuinely large, multi-minute download on a
 * cold cache -- callers should surface progress/expectations to the user
 * rather than assume this resolves quickly.
 */
async function downloadCandleNerModel(baseUrl: string): Promise<CandleNerModelBytes> {
	const [safetensors, tokenizerJson, encoderConfigJson] = await Promise.all([
		fetchWithCache(new URL("model.safetensors", baseUrl).href),
		fetchWithCache(new URL("tokenizer.json", baseUrl).href),
		fetchWithCache(new URL("encoder_config/config.json", baseUrl).href),
	]);
	return { safetensors, tokenizerJson, encoderConfigJson };
}

/**
 * Download the pinned GLiNER2 PII model and initialize the in-binary Candle
 * NER backend via `@xberg-io/xberg-wasm`'s `initCandleNer`. Browser-only --
 * throws if called outside a browser context (no Candle-in-WASM path exists
 * for Node; mcp-server uses the native ONNX Runtime `xberg-gliner` crate
 * instead).
 *
 * Idempotent-safe to call multiple times: `initCandleNer` replaces the
 * previously-loaded model rather than erroring, and the underlying wasm
 * `init()` short-circuits once the module is already instantiated.
 */
export async function initCandleNerBackend(config?: CacheConfig): Promise<void> {
	if (typeof window === "undefined") {
		throw new Error("[candle-ner] initCandleNerBackend is browser-only (no Candle-in-WASM path exists for Node)");
	}

	const baseUrl = config?.candleNerModelUrl ?? DEFAULT_CANDLE_NER_BASE_URL;
	console.debug(`[candle-ner] downloading GLiNER2 PII model from ${baseUrl}`);

	const [wasmModule, bytes] = await Promise.all([
		import("@xberg-io/xberg-wasm"),
		downloadCandleNerModel(baseUrl),
	]);

	// Idempotent (see xberg_wasm.js's __wbg_init: `if (wasm !== undefined) return wasm;`).
	// Safe even if the caller already called `init()` before constructing
	// XbergEngine -- initCandleNer operates on the same module-global wasm
	// instance either way, so ordering relative to XbergEngine construction
	// does not matter, only that the module is instantiated before this call.
	//
	// `@xberg-io/xberg-wasm` resolves to pkg/nodejs's types by default (no
	// `default` export -- the Node wasm-bindgen target auto-initializes
	// synchronously), even though the browser bundler alias substitutes
	// pkg/web at build time (which DOES need this async init). Read `default`
	// dynamically rather than statically so this typechecks against either
	// target; harmless no-op on a target that doesn't need it.
	const init = (wasmModule as unknown as { default?: () => Promise<unknown> }).default;
	if (typeof init === "function") {
		await init();
	}

	wasmModule.initCandleNer(bytes.safetensors, bytes.tokenizerJson, bytes.encoderConfigJson);
	console.debug("[candle-ner] GLiNER2 PII model loaded");
}
