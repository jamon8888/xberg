import type { VectorStoreInterface, DocumentRecord, ChunkRecord, GraphEdge, CacheConfig } from "./types";
import type { StoreWorkerRequest, StoreWorkerResponse, StoreWorkerRequestBase } from "./store-worker";

export async function createBrowserVectorStore(
  config?: CacheConfig
): Promise<VectorStoreInterface> {
  const worker = new Worker(new URL("./store-worker.ts", import.meta.url), { type: "module" });
  const dbPath = config?.opfsPath
    ? `opfs:${config.opfsPath}`
    : "opfs:/xberg/default.sqlite3";

  let nextId = 1;
  const pending = new Map<number, { resolve: (r: StoreWorkerResponse) => void }>();

  worker.onmessage = (event: MessageEvent<StoreWorkerResponse>) => {
    const entry = pending.get(event.data.id);
    if (entry) {
      pending.delete(event.data.id);
      entry.resolve(event.data);
    }
  };

  worker.postMessage({ dbPath });

  async function call<T>(req: StoreWorkerRequestBase): Promise<T> {
    const id = nextId++;
    const response = await new Promise<StoreWorkerResponse>((resolve) => {
      pending.set(id, { resolve });
      worker.postMessage({ ...req, id });
    });
    if (!response.ok) {
      throw new Error(`[store-browser] ${req.op} failed: ${response.error}`);
    }
    return response.result as T;
  }

  return {
    ensureCollection: (collection: string, vectorDim: number) =>
      call<void>({ op: "ensureCollection", collection, vectorDim }),
    upsertDocument: (collection: string, doc: DocumentRecord, chunks: ChunkRecord[]) =>
      call<{ documentId: string; chunksCount: number }>({ op: "upsertDocument", collection, doc, chunks }),
    query: (collection: string, queryVector: number[], k: number) =>
      call<Array<{ chunkId: string; text: string; score: number }>>({ op: "query", collection, queryVector, k }),
    delete: (collection: string, documentId: string) =>
      call<void>({ op: "delete", collection, documentId }),
    listCollections: () => call<string[]>({ op: "listCollections" }),
    dropCollection: (collection: string) => call<void>({ op: "dropCollection", collection }),
    createEdge: (edge: GraphEdge) => call<void>({ op: "createEdge", edge }),
    traverseGraph: (startIds: string[], depth: number, edgeLabels?: string[]) =>
      call<string[]>({ op: "traverseGraph", startIds, depth, edgeLabels }),
  };
}
