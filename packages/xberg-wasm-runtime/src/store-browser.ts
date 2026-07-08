import type { VectorStoreInterface, DocumentRecord, ChunkRecord, GraphEdge, CacheConfig } from "./types.js";
import type { StoreWorkerResponse, StoreWorkerRequestBase } from "./store-worker.js";

export async function createBrowserVectorStore(config?: CacheConfig): Promise<VectorStoreInterface> {
	const worker = new Worker(new URL("./store-worker.ts", import.meta.url), { type: "module" });
	const dbPath = config?.opfsPath ?? "/xberg/default.sqlite3";
	if (!dbPath.startsWith("/") || dbPath.includes("..")) {
		worker.terminate();
		throw new Error("[store-browser] opfsPath must be an absolute OPFS path without '..'");
	}

	let nextId = 1;
	const pending = new Map<
		number,
		{
			resolve: (r: StoreWorkerResponse) => void;
			reject: (error: Error) => void;
			timeout: ReturnType<typeof setTimeout>;
		}
	>();

	worker.onmessage = (event: MessageEvent<StoreWorkerResponse>) => {
		const entry = pending.get(event.data.id);
		if (entry) {
			pending.delete(event.data.id);
			clearTimeout(entry.timeout);
			entry.resolve(event.data);
		}
	};

	worker.onerror = (event) => {
		const error = new Error(`[store-browser] worker failed: ${event.message}`);
		for (const entry of pending.values()) {
			clearTimeout(entry.timeout);
			entry.reject(error);
		}
		pending.clear();
	};

	async function call<T>(req: StoreWorkerRequestBase): Promise<T> {
		const id = nextId++;
		const response = await new Promise<StoreWorkerResponse>((resolve, reject) => {
			const timeout = setTimeout(() => {
				pending.delete(id);
				reject(new Error(`[store-browser] ${req.op} timed out after 15 seconds`));
			}, 15_000);
			pending.set(id, { resolve, reject, timeout });
			worker.postMessage({ ...req, id });
		});
		if (!response.ok) {
			throw new Error(`[store-browser] ${req.op} failed: ${response.error}`);
		}
		return response.result as T;
	}

	await call<void>({ op: "init", dbPath });

	return {
		ensureCollection: (collection: string, vectorDim: number) =>
			call<void>({ op: "ensureCollection", collection, vectorDim }),
		upsertDocument: (collection: string, doc: DocumentRecord, chunks: ChunkRecord[]) =>
			call<{ documentId: string; chunksCount: number }>({ op: "upsertDocument", collection, doc, chunks }),
		query: (collection: string, queryVector: number[], k: number) =>
			call<Array<{ chunkId: string; text: string; score: number }>>({ op: "query", collection, queryVector, k }),
		delete: (collection: string, documentId: string) => call<void>({ op: "delete", collection, documentId }),
		listCollections: () => call<string[]>({ op: "listCollections" }),
		dropCollection: (collection: string) => call<void>({ op: "dropCollection", collection }),
		createEdge: (edge: GraphEdge) => call<void>({ op: "createEdge", edge }),
		traverseGraph: (startIds: string[], depth: number, edgeLabels?: string[]) =>
			call<string[]>({ op: "traverseGraph", startIds, depth, edgeLabels }),
	};
}
