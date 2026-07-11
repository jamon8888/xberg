// src/engine/engine.worker.ts
/// <reference lib="webworker" />
import { createXbergRuntimeFactory } from "xberg-wasm-runtime";
import type { VectorStoreInterface, DocumentRecord, ChunkRecord, CollectionSpec } from "xberg-wasm-runtime";
import { XbergEngine } from "@xberg-io/xberg-wasm";
import { postIngest, postMap } from "../lib/sync-client.js";
import { sanitizeExternalId } from "../lib/sanitize-id.js";
import { EMBEDDING_DIM } from "../lib/constants.js";
import type { IngestHistoryEntry } from "../lib/types.js";

declare const self: DedicatedWorkerGlobalScope;

interface IngestMessage {
  type: "ingest";
  requestId: string;
  file: File;
  filename: string;
  mime: string;
  collection: string;
  passphrase: string;
  mcpBaseUrl: string;
}

let mcpBaseUrl = "";
let engine: XbergEngine | null = null;

/**
 * HTTP-backed `VectorStoreInterface`. Only `upsertDocument` matters for
 * `engine.ingest()` — everything else throws, since this shim exists
 * solely to redirect the WASM engine's internal store write to `POST
 * /ingest` instead of a local OPFS/SQLite write.
 */
function createHttpStore(): VectorStoreInterface {
  const notSupported = (name: string) => async () => {
    throw new Error(`${name} is not supported by the browser HTTP-backed store`);
  };
  return {
    close: async () => undefined,
    ensureCollection: notSupported("ensureCollection") as (spec: CollectionSpec) => Promise<string | void>,
    dropCollection: notSupported("dropCollection"),
    getCollection: notSupported("getCollection"),
    deleteDocuments: notSupported("deleteDocuments"),
    deleteByFilter: notSupported("deleteByFilter"),
    retrieve: notSupported("retrieve"),
    collectionStats: notSupported("collectionStats"),
    async upsertDocument(collection: string, doc: DocumentRecord, chunks: ChunkRecord[]): Promise<string> {
      if (chunks.length > 0 && chunks[0] && chunks[0].embedding.length !== EMBEDDING_DIM) {
        throw new Error(
          `embedder produced ${chunks[0].embedding.length}-dim vectors, expected ${EMBEDDING_DIM} (EMBEDDING_DIM constant is stale — update it and the /collection embedding_dim together)`
        );
      }
      const { document_id } = await postIngest(mcpBaseUrl, {
        collection,
        external_id: doc.external_id ?? "",
        title: doc.title,
        mime: doc.mime,
        source_uri: doc.source_uri,
        full_text: doc.full_text,
        keywords: doc.keywords,
        metadata: doc.metadata as Record<string, unknown> | undefined,
        chunks: chunks.map((c) => ({ ordinal: c.ordinal, content: c.content, embedding: c.embedding, chunk_metadata: c.chunk_metadata })),
      });
      return document_id;
    },
  };
}

async function getEngine(): Promise<XbergEngine> {
  if (engine) return engine;
  const injection = await createXbergRuntimeFactory();
  injection.store = createHttpStore();
  engine = new XbergEngine({}, injection);
  return engine;
}

function post(msg: unknown, transfer: Transferable[] = []): void {
  self.postMessage(msg, transfer);
}

async function handleIngest(msg: IngestMessage): Promise<void> {
  const { requestId, file, filename, mime, collection, passphrase } = msg;
  try {
    const xEngine = await getEngine();
    const externalId = sanitizeExternalId(filename);
    const bytes = new Uint8Array(await file.arrayBuffer());

    post({ type: "progress", requestId, stage: "extract" });
    const extracted = await xEngine.extract({ kind: "bytes", bytes: Array.from(bytes), filename }, undefined);
    const first = (extracted as { results?: Array<{ content: string; mimeType: string }> }).results?.[0];
    if (!first) throw new Error(`extraction produced no result for ${filename}`);

    post({ type: "progress", requestId, stage: "ingest" });
    const outcome = (await xEngine.ingest(
      { full_text: first.content, title: filename, mime: first.mimeType || mime, source_uri: filename, external_id: externalId },
      collection
    )) as { document_id: string; rehydration_map: Record<string, string>; pii_category_counts: Record<string, number> };

    post({ type: "progress", requestId, stage: "encrypt" });
    const blob = xEngine.encrypt_map(outcome.rehydration_map, passphrase);

    post({ type: "progress", requestId, stage: "map" });
    await postMap(mcpBaseUrl, outcome.document_id, blob);

    const entry: IngestHistoryEntry = {
      collection,
      externalId,
      filename,
      mime: first.mimeType || mime,
      redactedText: first.content,
      piiCategoryCounts: outcome.pii_category_counts,
      documentId: outcome.document_id,
      status: "synced",
      ingestedAt: Date.now(),
    };
    post({ type: "result", requestId, entry });
  } catch (err) {
    post({ type: "error", requestId, message: err instanceof Error ? err.message : String(err) });
  }
}

self.addEventListener("message", (ev: MessageEvent) => {
  const msg = ev.data as IngestMessage;
  if (msg.type === "ingest") {
    mcpBaseUrl = msg.mcpBaseUrl;
    void handleIngest(msg);
  }
});
