import type { VectorStoreInterface, CacheConfig } from "./types.js";
import { createNodeVectorStore } from "./store-node.js";

export async function createVectorStore(config?: CacheConfig): Promise<VectorStoreInterface> {
	if (typeof window === "undefined") {
		return createNodeVectorStore(config);
	}
	const { createBrowserVectorStore } = await import("./store-browser.js");
	return createBrowserVectorStore(config);
}
