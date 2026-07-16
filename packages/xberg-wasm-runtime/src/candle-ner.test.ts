import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

// initCandleNer/candle-ner.ts's real behavior needs a browser (window,
// fetch, caches) and the compiled wasm binary -- none of which exist in this
// Node-based vitest run. Mock the wasm module and the network so these tests
// exercise this module's own orchestration logic (download-then-init
// sequencing, cache-first fetch, Node-guard, error propagation) rather than
// real WASM/network behavior, which is covered by the live browser
// verification in wasm-self-test instead.
const initCandleNer = vi.fn();
const wasmDefault = vi.fn().mockResolvedValue(undefined);

vi.mock("@xberg-io/xberg-wasm", () => ({
	default: wasmDefault,
	initCandleNer,
}));

describe("candle-ner", () => {
	const originalWindow = globalThis.window;
	const originalFetch = globalThis.fetch;
	const originalCaches = globalThis.caches;

	beforeEach(() => {
		vi.resetModules();
		initCandleNer.mockClear();
		wasmDefault.mockClear();
		// Present a minimal browser-like global; candle-ner.ts only checks
		// `typeof window !== "undefined"`, it never touches window's contents.
		(globalThis as { window?: unknown }).window = {};
	});

	afterEach(() => {
		(globalThis as { window?: unknown }).window = originalWindow;
		globalThis.fetch = originalFetch;
		globalThis.caches = originalCaches as typeof globalThis.caches;
	});

	it("throws when called outside a browser context", async () => {
		(globalThis as { window?: unknown }).window = undefined;
		const { initCandleNerBackend } = await import("./candle-ner.js");
		await expect(initCandleNerBackend()).rejects.toThrow("browser-only");
	});

	it("downloads the three model files and calls initCandleNer with their bytes", async () => {
		const fetchMock = vi.fn().mockImplementation((url: string) =>
			Promise.resolve({
				ok: true,
				status: 200,
				statusText: "OK",
				arrayBuffer: () => Promise.resolve(new TextEncoder().encode(`bytes:${url}`).buffer),
				clone() {
					return this;
				},
			}),
		);
		globalThis.fetch = fetchMock as unknown as typeof fetch;
		globalThis.caches = undefined as unknown as typeof globalThis.caches;

		const { initCandleNerBackend } = await import("./candle-ner.js");
		await initCandleNerBackend({ candleNerModelUrl: "https://example.test/model/" });

		expect(fetchMock).toHaveBeenCalledWith("https://example.test/model/model.safetensors");
		expect(fetchMock).toHaveBeenCalledWith("https://example.test/model/tokenizer.json");
		expect(fetchMock).toHaveBeenCalledWith("https://example.test/model/encoder_config/config.json");

		expect(wasmDefault).toHaveBeenCalledTimes(1);
		expect(initCandleNer).toHaveBeenCalledTimes(1);
		const [safetensors, tokenizerJson, encoderConfigJson] = initCandleNer.mock.calls[0] as Uint8Array[];
		expect(new TextDecoder().decode(safetensors)).toBe("bytes:https://example.test/model/model.safetensors");
		expect(new TextDecoder().decode(tokenizerJson)).toBe("bytes:https://example.test/model/tokenizer.json");
		expect(new TextDecoder().decode(encoderConfigJson)).toBe(
			"bytes:https://example.test/model/encoder_config/config.json",
		);
	});

	it("propagates a fetch failure instead of calling initCandleNer with partial data", async () => {
		globalThis.fetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 404,
			statusText: "Not Found",
		}) as unknown as typeof fetch;
		globalThis.caches = undefined as unknown as typeof globalThis.caches;

		const { initCandleNerBackend } = await import("./candle-ner.js");
		await expect(initCandleNerBackend({ candleNerModelUrl: "https://example.test/model/" })).rejects.toThrow(
			"fetch failed",
		);
		expect(initCandleNer).not.toHaveBeenCalled();
	});

	it("reuses a cached response instead of re-fetching", async () => {
		const cachedBytes = new TextEncoder().encode("cached-safetensors").buffer;
		const cacheMatch = vi.fn().mockImplementation((url: string) =>
			url.endsWith("model.safetensors")
				? Promise.resolve({ arrayBuffer: () => Promise.resolve(cachedBytes) })
				: Promise.resolve(undefined),
		);
		const cachePut = vi.fn().mockResolvedValue(undefined);
		globalThis.caches = {
			open: vi.fn().mockResolvedValue({ match: cacheMatch, put: cachePut }),
		} as unknown as typeof globalThis.caches;

		const fetchMock = vi.fn().mockImplementation(() =>
			Promise.resolve({
				ok: true,
				status: 200,
				statusText: "OK",
				arrayBuffer: () => Promise.resolve(new ArrayBuffer(0)),
				clone() {
					return this;
				},
			}),
		);
		globalThis.fetch = fetchMock as unknown as typeof fetch;

		const { initCandleNerBackend } = await import("./candle-ner.js");
		await initCandleNerBackend({ candleNerModelUrl: "https://example.test/model/" });

		// The cached file must not have triggered a network fetch.
		const fetchedUrls = fetchMock.mock.calls.map((call) => call[0]);
		expect(fetchedUrls).not.toContain("https://example.test/model/model.safetensors");

		const [safetensors] = initCandleNer.mock.calls[0] as Uint8Array[];
		expect(new TextDecoder().decode(safetensors)).toBe("cached-safetensors");
	});
});
