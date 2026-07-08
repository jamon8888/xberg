import { describe, expect, it, vi } from "vitest";
import { validateInjectionDescriptor } from "./validation.js";

describe("validateInjectionDescriptor", () => {
	it("returns a validated descriptor", () => {
		const method = vi.fn(async () => undefined);
		const result = validateInjectionDescriptor({
			embedder: { embed: method },
			store: {
				upsertDocument: method,
				query: method,
				delete: method,
				listCollections: method,
				dropCollection: method,
				ensureCollection: method,
				createEdge: method,
				traverseGraph: method,
			},
		});

		expect(result.valid).toBe(true);
	});

	it("returns validation details for malformed injection objects", () => {
		const result = validateInjectionDescriptor({ embedder: {} });

		expect(result.valid).toBe(false);
		if (!result.valid) expect(result.error.length).toBeGreaterThan(0);
	});
});
