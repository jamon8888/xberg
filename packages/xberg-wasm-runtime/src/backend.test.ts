import { describe, it, expect } from "vitest";
import { detectBackend } from "./backend.js";

describe("detectBackend", () => {
	it("falls back to wasm in node (no browser gpu/webgl)", () => {
		expect(detectBackend()).toBe("wasm");
	});
	it("returns a valid backend union", () => {
		expect(["webgpu", "webgl", "wasm"]).toContain(detectBackend());
	});
});
