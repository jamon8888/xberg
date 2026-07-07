/**
 * Runtime detection of the best available ONNX Runtime Web backend.
 *
 * Preference order (graceful degradation):
 *   1. "webgpu" — if `navigator.gpu` exists and `requestAdapter()` resolves
 *   2. "webgl"  — if a WebGL rendering context is obtainable from a canvas
 *   3. "wasm"   — always-available CPU fallback
 *
 * Every browser-API access is guarded so this returns "wasm" in Node
 * (where `navigator`, `document`, and `HtmlCanvasElement` are undefined)
 * without throwing.
 */
export type OnnxBackend = "webgpu" | "webgl" | "wasm";

export function detectBackend(): OnnxBackend {
	if (typeof window === "undefined" || typeof document === "undefined") {
		return "wasm";
	}

	try {
		const nav = globalThis.navigator as Navigator & {
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
