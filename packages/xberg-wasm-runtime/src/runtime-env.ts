import { env } from "@huggingface/transformers";
import type { CacheConfig } from "./types.js";

export function defaultNodeCachePath(): string {
	const home = process.env.USERPROFILE ?? process.env.HOME ?? ".";
	if (process.platform === "win32") {
		return `${process.env.LOCALAPPDATA ?? `${home}/AppData/Local`}/xberg`;
	}
	return `${home}/.cache/xberg`;
}

// Documented (onnxruntime-common Env.WebAssemblyFlags.initTimeout): timeout
// in ms for WASM *environment* initialization (module instantiation +
// worker-pool bootstrap); 0 (the default) means no timeout, which is why a
// stalled env-init previously hung forever with no error at all. This does
// NOT cover per-session InferenceSession.create() time (no such API exists
// for that phase -- that's why createPipelineWithFallback in backend.ts
// wraps pipeline() in its own Promise.race timeout instead). Generous value
// since large-model env init can legitimately take a while.
const ONNX_WASM_INIT_TIMEOUT_MS = 300_000;

export function configureTransformersEnvironment(config?: CacheConfig): void {
	const onnxWasm = (
		env.backends?.onnx as
			| {
					wasm?: {
						wasmPaths?: string | { mjs?: string; wasm?: string };
						initTimeout?: number;
						numThreads?: number;
						proxy?: boolean;
					};
			  }
			| undefined
	)?.wasm;

	if (onnxWasm) {
		onnxWasm.initTimeout = ONNX_WASM_INIT_TIMEOUT_MS;
		// CONFIRMED ROOT CAUSE (read directly from onnxruntime-web's own
		// source, not guesswork): its "wasm" backend init
		// (tryResolveAndInitializeBackend in ort.wasm.js) caches the init
		// Promise the FIRST time any session is created, keyed only by
		// backend name -- every later session-creation call just awaits that
		// SAME cached Promise. A previous version of this fix tried to
		// recover from a stalled threaded init by mutating numThreads/proxy
		// and retrying -- that retry is a no-op: the already-in-flight init
		// Promise was captured with the ORIGINAL (threaded) config and never
		// re-reads these fields, so if the first (default, multi-threaded)
		// attempt's pthread worker-pool bootstrap stalls, EVERY subsequent
		// attempt just awaits the same stuck Promise forever, regardless of
		// what this config is mutated to afterward. The only way to actually
		// avoid the stall is to never let the threaded bootstrap start in
		// the first place -- force single-threaded (no worker pool at all)
		// before the very first backend init.
		onnxWasm.numThreads = 1;
		onnxWasm.proxy = false;
	}

	// Browser: point onnxruntime-web at self-hosted runtime files when the
	// host app provides a same-origin location for them. transformers.js
	// defaults `wasmPaths` to the jsdelivr CDN, which breaks on
	// crossOriginIsolated pages: ORT's threaded runtime spawns its pthread
	// worker pool via `new Worker(new URL(import.meta.url))`, and with CDN
	// wasmPaths that URL is cross-origin -> SecurityError swallowed by the
	// Emscripten bootstrap -> pipeline() hangs forever with no console or
	// network signal. Same-origin wasmPaths is the fix, not an optimization.
	if (config?.wasmPaths && onnxWasm) {
		// Prefer the explicit { mjs, wasm } object form (absolute URLs) over
		// the string-prefix form: ORT checks the object override BEFORE its
		// `import.meta.url`-relative resolution, which is the only hook that
		// survives webpack bundling (webpack rewrites ORT's internal dynamic
		// import so the string prefix is never consulted -- observed as zero
		// requests to the self-hosted directory despite wasmPaths being set).
		const origin = (globalThis as { location?: { href: string } }).location?.href;
		if (origin) {
			const base = new URL(config.wasmPaths, origin);
			onnxWasm.wasmPaths = {
				mjs: new URL("ort-wasm-simd-threaded.jsep.mjs", base).href,
				wasm: new URL("ort-wasm-simd-threaded.jsep.wasm", base).href,
			};
		} else {
			onnxWasm.wasmPaths = config.wasmPaths;
		}
		console.debug(`[runtime-env] ORT wasmPaths ->`, onnxWasm.wasmPaths);
	}
	if (typeof process === "undefined" || !process.versions?.node) return;
	env.cacheDir = config?.nodeCachePath ?? defaultNodeCachePath();
}
