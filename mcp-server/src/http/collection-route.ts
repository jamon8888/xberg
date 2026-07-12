import { z } from "zod";
import type { IncomingMessage, ServerResponse } from "node:http";
import type { VectorStoreInterface } from "xberg-wasm-runtime";

const MAX_BODY_BYTES = 64 * 1024;

/**
 * `POST /collection` payload. `embedding_dim` is caller-supplied (not
 * inferred server-side) because the browser's embedder model — and
 * therefore its output dimension — is chosen client-side.
 */
const CollectionPayloadSchema = z.object({
  name: z.string().min(1),
  embedding_dim: z.number().int().positive(),
  distance_metric: z.enum(["cosine", "l2", "innerproduct"]).optional(),
  index_method: z.enum(["flat", "hnsw", "diskann"]).optional(),
});

function sanitizeError(message: string): { status: number; clientMsg: string } {
  const lower = message.toLowerCase();
  if (lower.includes("not found")) return { status: 404, clientMsg: "collection not found" };
  if (lower.includes("dimension")) return { status: 400, clientMsg: "invalid embedding dimension" };
  if (lower.includes("distance metric") || lower.includes("metric")) return { status: 400, clientMsg: "invalid distance metric" };
  if (lower.includes("index method")) return { status: 400, clientMsg: "invalid index method" };
  if (lower.includes("already exists")) return { status: 409, clientMsg: "collection already exists" };
  return { status: 400, clientMsg: "invalid request" };
}

/**
 * Build the `POST /collection` handler. Idempotent: calling it again with
 * the same `name` is a no-op on the store side (`ensureCollection`'s own
 * contract), so the browser can call this unconditionally on folder open,
 * not just folder creation.
 */
export function createCollectionHandler(
  getStore: () => VectorStoreInterface
): (req: IncomingMessage, res: ServerResponse) => Promise<void> {
  return async function handleCollection(req: IncomingMessage, res: ServerResponse): Promise<void> {
    try {
      const chunks: Buffer[] = [];
      let totalBytes = 0;
      for await (const chunk of req) {
        totalBytes += (chunk as Buffer).length;
        if (totalBytes > MAX_BODY_BYTES) {
          res.writeHead(413, { "Content-Type": "application/json" }).end(JSON.stringify({ error: "payload too large" }));
          return;
        }
        chunks.push(chunk as Buffer);
      }

      let json: unknown;
      try {
        json = JSON.parse(Buffer.concat(chunks).toString("utf-8"));
      } catch {
        res.writeHead(400, { "Content-Type": "application/json" }).end(JSON.stringify({ error: "invalid JSON body" }));
        return;
      }

      const parsed = CollectionPayloadSchema.safeParse(json);
      if (!parsed.success) {
        res
          .writeHead(400, { "Content-Type": "application/json" })
          .end(JSON.stringify({ error: "invalid payload", issues: parsed.error.issues }));
        return;
      }

      const result = await getStore().ensureCollection(parsed.data);
      if (typeof result === "string") {
        console.error(`[collection-route] ensureCollection failed: ${result}`);
        const { status, clientMsg } = sanitizeError(result);
        res.writeHead(status, { "Content-Type": "application/json" }).end(JSON.stringify({ error: clientMsg }));
        return;
      }
      res.writeHead(200, { "Content-Type": "application/json" }).end(JSON.stringify({ created: true }));
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      console.error(`[collection-route] unhandled error: ${msg}`);
      if (!res.headersSent) {
        const { status, clientMsg } = sanitizeError(msg);
        res.writeHead(status, { "Content-Type": "application/json" }).end(JSON.stringify({ error: clientMsg }));
      } else {
        res.end();
      }
    }
  };
}
