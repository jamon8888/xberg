// STUB: real Worker message handler is implemented in Task 5 (browser OPFS + sqlite-vec WASM). Types only here so the package type-checks in Node.

import type { DocumentRecord, ChunkRecord, GraphEdge } from "./types";

export type StoreWorkerRequest =
  | {
      op: "ensureCollection";
      collection: string;
      vectorDim: number;
      dbPath?: string;
      id: number;
    }
  | {
      op: "upsertDocument";
      collection: string;
      doc: DocumentRecord;
      chunks: ChunkRecord[];
      dbPath?: string;
      id: number;
    }
  | {
      op: "query";
      collection: string;
      queryVector: number[];
      k: number;
      dbPath?: string;
      id: number;
    }
  | {
      op: "delete";
      collection: string;
      documentId: string;
      dbPath?: string;
      id: number;
    }
  | { op: "listCollections"; dbPath?: string; id: number }
  | { op: "dropCollection"; collection: string; dbPath?: string; id: number }
  | { op: "createEdge"; edge: GraphEdge; dbPath?: string; id: number }
  | {
      op: "traverseGraph";
      startIds: string[];
      depth: number;
      edgeLabels?: string[];
      dbPath?: string;
      id: number;
    };

export interface StoreWorkerResponse {
  id: number;
  ok: boolean;
  result?: unknown;
  error?: string;
}

export type StoreWorkerRequestBase = DistributiveOmit<StoreWorkerRequest, "id">;

type DistributiveOmit<T, K extends keyof T> = T extends unknown
  ? Omit<T, K>
  : never;
