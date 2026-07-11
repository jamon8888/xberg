// src/engine/worker-client.ts
import type { IngestHistoryEntry } from "../lib/types.js";

type ProgressMessage = { type: "progress"; requestId: string; stage: string };
type ResultMessage = { type: "result"; requestId: string; entry: IngestHistoryEntry };
type ErrorMessage = { type: "error"; requestId: string; message: string };
type WorkerOutMessage = ProgressMessage | ResultMessage | ErrorMessage;

function randomRequestId(): string {
  return `req-${Math.random().toString(36).slice(2)}-${Math.random().toString(36).slice(2)}`;
}

/**
 * Wraps a `postMessage` RPC protocol around the engine worker. One
 * `WorkerClient` per `Worker` instance; `ingestFile` calls are queued by
 * the worker itself (it processes one file at a time — `XbergEngine` is
 * not proven reentrant), so callers may fire multiple concurrent
 * `ingestFile` calls without waiting, but each one only resolves once its
 * own `requestId` gets a `result`/`error` message.
 */
export class WorkerClient {
  constructor(
    private readonly worker: Worker,
    private readonly baseUrl: string
  ) {}

  ingestFile(
    file: File,
    collection: string,
    passphrase: string,
    onProgress?: (stage: string) => void
  ): Promise<IngestHistoryEntry> {
    return new Promise((resolve, reject) => {
      const requestId = randomRequestId();

      const onMessage = (ev: MessageEvent<WorkerOutMessage>): void => {
        const msg = ev.data;
        if (msg.requestId !== requestId) return;
        if (msg.type === "progress") {
          onProgress?.(msg.stage);
          return;
        }
        this.worker.removeEventListener("message", onMessage as EventListener);
        if (msg.type === "error") {
          reject(new Error(msg.message));
        } else {
          resolve(msg.entry);
        }
      };

      this.worker.addEventListener("message", onMessage as EventListener);

      // `File` is structured-cloneable, so it can be posted directly —
      // this keeps the postMessage call synchronous (no intervening
      // `arrayBuffer()` microtask) and lets the worker do the byte
      // conversion off the main thread instead.
      this.worker.postMessage({
        type: "ingest",
        requestId,
        file,
        filename: file.name,
        mime: file.type,
        collection,
        passphrase,
        mcpBaseUrl: this.baseUrl,
      });
    });
  }
}
