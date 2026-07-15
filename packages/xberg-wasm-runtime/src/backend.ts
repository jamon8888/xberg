import { AutoModel } from "@huggingface/transformers";

/**
 * Runtime detection of the best available ONNX Runtime Web backend.
 *
 * Preference order (graceful degradation):
 *   1. "webgpu" — if `navigator.gpu` exists AND `requestAdapter()` actually
 *      resolves with a real adapter within `WEBGPU_ADAPTER_TIMEOUT_MS`
 *   2. "webgl"  — if a WebGL rendering context is obtainable from a canvas
 *   3. "wasm"   — always-available CPU fallback
 *
 * Every browser-API access is guarded so this returns "wasm" in Node
 * (where `navigator`, `document`, and `HtmlCanvasElement` are undefined)
 * without throwing.
 */
export type OnnxBackend = "webgpu" | "webgl" | "wasm";
export type ModelBackend = {
	device: "webgpu" | "wasm" | "cpu";
	dtype: "fp32" | "q8";
};

// `navigator.gpu`'s mere presence is not a reliable signal: sandboxed,
// headless, or otherwise constrained browser contexts can expose the API
// while `requestAdapter()` itself hangs forever rather than rejecting (no
// adapter available, but no error either). A model pipeline built on that
// promise then never resolves -- previously observed as `pipeline(...)`
// silently stalling with zero console output and zero network activity.
// Race the real adapter request against a timeout so a non-functional
// WebGPU implementation degrades to the WASM-CPU backend instead of
// hanging the caller indefinitely.
const WEBGPU_ADAPTER_TIMEOUT_MS = 3_000;

export async function selectModelBackend(config?: { forceWasmBackend?: boolean }): Promise<ModelBackend> {
	if (typeof process !== "undefined" && process.versions?.node) {
		return { device: "cpu", dtype: "q8" };
	}
	if (config?.forceWasmBackend) {
		return { device: "wasm", dtype: "q8" };
	}
	const gpu =
		typeof navigator !== "undefined" ? (navigator as Navigator & { gpu?: GPU }).gpu : undefined;
	if (!gpu) {
		return { device: "wasm", dtype: "q8" };
	}
	try {
		const adapter = await Promise.race([
			gpu.requestAdapter(),
			new Promise<null>((resolve) => {
				setTimeout(() => resolve(null), WEBGPU_ADAPTER_TIMEOUT_MS);
			}),
		]);
		if (adapter) return { device: "webgpu", dtype: "fp32" };
	} catch {
		// fall through to wasm
	}
	return { device: "wasm", dtype: "q8" };
}

// ROOT CAUSE (isolated via direct testing, not guesswork): transformers.js's
// pipeline() always loads the tokenizer and the ONNX model CONCURRENTLY via
// Promise.all (see loadItems() in its pipelines.js). In some sandboxed/
// automated browser contexts, onnxruntime-web's lazy WASM environment
// singleton-init (WASM module instantiation + pthread worker-pool bootstrap)
// races with that concurrent tokenizer load on a fresh page and never
// resolves -- confirmed by isolating each half: AutoModel.from_pretrained()
// alone succeeds, AutoTokenizer.from_pretrained() alone succeeds, but
// calling both at once (what pipeline() does internally) hangs forever with
// zero console output and zero network activity. Once ANY ORT session has
// been created successfully once on the page, onnxruntime-web's WASM env is
// already a warm singleton, and subsequent pipeline() calls (even for a
// different model) succeed quickly -- confirmed by calling pipeline() a
// second time after a standalone AutoModel.from_pretrained() succeeded.
//
// Not caused by WebGPU misdetection (separately fixed above), missing
// COOP/COEP (crossOriginIsolated verified true), CDN-origin worker spawning
// (also separately fixed -- ORT's threaded runtime is now served
// same-origin), network egress, or basic Worker creation (all independently
// verified working in this same environment).
//
// Fix: prime the WASM env by loading the real target model ONCE, alone,
// sequentially, before ever calling pipeline(). transformers.js caches
// downloaded model files, so this costs no extra network/bandwidth -- only
// a second (cheap, already-warm) session construction when pipeline() loads
// the same model again internally.
//
// graphOptimizationLevel: ORT's default ('all') runs its full graph
// optimization pass (constant folding, operator fusion, layout transforms)
// during InferenceSession creation -- documented ORT guidance is to disable
// this for large models where session-creation time matters more than the
// runtime speedup, since the pass itself scales with graph size. For a
// ~550MB single-file model (bge-m3) this pass alone was the suspected
// remaining cause of multi-minute session creation in this environment
// after WASM-env priming already eliminated the concurrent-load race.
// `env.backends.onnx.wasm.initTimeout` (default 0 = no timeout) is
// documented as covering WASM *environment* init specifically, not
// per-session InferenceSession.create() -- there is no lower-level API to
// bound the latter, so this is the actual lever available.
export const LARGE_MODEL_SESSION_OPTIONS = { graphOptimizationLevel: "disabled" as const };

let onnxRuntimeWarmPromise: Promise<void> | null = null;

async function primeOnnxRuntime(modelId: string, backend: ModelBackend): Promise<void> {
	if (backend.device !== "wasm" && backend.device !== "webgpu") return;
	onnxRuntimeWarmPromise ??= AutoModel.from_pretrained(modelId, {
		...backend,
		session_options: LARGE_MODEL_SESSION_OPTIONS,
	})
		.then(() => undefined)
		.catch((err: unknown) => {
			console.warn("[backend] ORT warm-up failed (continuing anyway, real pipeline() load may still hang):", err);
		});
	return onnxRuntimeWarmPromise;
}

// NOTE: an earlier version of this function retried on timeout by mutating
// env.backends.onnx.wasm.numThreads/proxy and calling createPipeline again.
// That retry was a no-op -- see the comment on numThreads in runtime-env.ts:
// onnxruntime-web caches its "wasm" backend init Promise on the FIRST call
// and every later call just awaits that same Promise, ignoring config
// mutated afterward. Single-threaded WASM (no pthread worker pool to
// bootstrap, the actual stall-prone step) is now forced BEFORE the first
// init in configureTransformersEnvironment, so there is nothing left for a
// runtime retry to fix -- a stall past this point is a genuinely different
// problem (e.g. WASM compile time for a very large model on constrained
// hardware) that awaiting the same Promise for longer, not retrying it,
// actually addresses. This timeout is diagnostic only: it logs so a slow
// load is visible instead of silent, without abandoning the real result.
const PIPELINE_INIT_WARN_MS = 30_000;

/**
 * Create a transformers.js pipeline, priming onnxruntime-web's WASM
 * environment first to avoid pipeline()'s internal concurrent-load race
 * (see comment above). Logs a warning if init is taking unusually long, but
 * always resolves to the real pipeline result rather than abandoning it.
 */
export async function createPipelineWithFallback<T>(
	createPipeline: (backend: ModelBackend) => Promise<T>,
	backend: ModelBackend,
	label: string,
	modelId: string,
): Promise<T> {
	if (backend.device !== "wasm" && backend.device !== "webgpu") {
		return createPipeline(backend);
	}

	await primeOnnxRuntime(modelId, backend);

	const warnTimer = setTimeout(() => {
		console.warn(
			`[backend] ${label} pipeline init exceeded ${PIPELINE_INIT_WARN_MS}ms on device=${backend.device}` +
				" despite priming and single-threaded WASM -- still waiting (large models can legitimately take a while to compile)",
		);
	}, PIPELINE_INIT_WARN_MS);

	try {
		return await createPipeline(backend);
	} finally {
		clearTimeout(warnTimer);
	}
}

export function detectBackend(): OnnxBackend {
	if (typeof window === "undefined" || typeof document === "undefined") {
		return "wasm";
	}

	try {
		const nav = globalThis.navigator as {
			gpu?: { requestAdapter?: () => Promise<unknown> };
		};
		// Synchronous check is intentional: the full WebGPU adapter request is
		// awaited lazily by callers. Its mere presence is a strong signal that
		// WebGPU is exposed, which is sufficient for backend selection here.
		if (nav?.gpu?.requestAdapter) {
			return "webgpu";
		}
	} catch {
		// fall through to webgl
	}

	try {
		if (typeof HTMLCanvasElement !== "undefined") {
			const canvas = document.createElement("canvas");
			const gl = canvas.getContext("webgl") ?? canvas.getContext("experimental-webgl");
			if (gl) return "webgl";
		}
	} catch {
		// fall through to wasm
	}

	return "wasm";
}
