import type { VectorStoreInterface, CacheConfig } from "./types";
import { createNodeVectorStore } from "./store-node";

export async function createVectorStore(config?: CacheConfig): Promise<VectorStoreInterface> {
  if (typeof window === "undefined") {
    return createNodeVectorStore(config);
  }
  const { createBrowserVectorStore } = await import("./store-browser");
  return createBrowserVectorStore(config);
}
