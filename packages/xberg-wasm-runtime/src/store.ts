import type { VectorStoreInterface, CacheConfig } from "./types.js";

export async function createVectorStore(config?: CacheConfig): Promise<VectorStoreInterface> {
	if (typeof window === "undefined") {
		const { createNodeVectorStore } = await import("./store-node.js");
		return createNodeVectorStore(config);
	}
	const { createBrowserVectorStore } = await import("./store-browser.js");
	return createBrowserVectorStore(config);
}
