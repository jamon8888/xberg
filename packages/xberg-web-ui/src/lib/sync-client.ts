import { authedUrl, authHeaders } from "./auth-client.js";
import type { CollectionPayload, IngestPayload } from "./types.js";

const MAX_RETRIES = 3;
const BACKOFF_MS = 400;

/**
 * POST with retry+backoff on 5xx only (mirrors Lot 3's `admin-client.ts`
 * pattern for consistency). 4xx is a client error (bad payload, unknown
 * collection) and is never retried.
 */
async function postWithRetry(url: string, init: RequestInit): Promise<Response> {
  let last: Response | undefined;
  for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
    const res = await fetch(url, init);
    if (res.status < 500) return res;
    last = res;
    if (attempt < MAX_RETRIES) {
      await new Promise((resolve) => setTimeout(resolve, BACKOFF_MS * 2 ** attempt));
    }
  }
  return last!;
}

async function throwOnError(res: Response, label: string): Promise<Response> {
  if (!res.ok) {
    let detail = "";
    try {
      const body = (await res.json()) as { error?: string };
      detail = body.error ?? "";
    } catch {
      // response body wasn't JSON; fall through with the empty detail
    }
    throw new Error(`${label} failed (${res.status})${detail ? `: ${detail}` : ""}`);
  }
  return res;
}

export async function postCollection(baseUrl: string, payload: CollectionPayload): Promise<void> {
  const res = await postWithRetry(authedUrl(baseUrl, "/collection"), {
    method: "POST",
    headers: { "Content-Type": "application/json", ...authHeaders() },
    body: JSON.stringify(payload),
  });
  await throwOnError(res, "postCollection");
}

export async function postIngest(baseUrl: string, payload: IngestPayload): Promise<{ document_id: string }> {
  const res = await postWithRetry(authedUrl(baseUrl, "/ingest"), {
    method: "POST",
    headers: { "Content-Type": "application/json", ...authHeaders() },
    body: JSON.stringify(payload),
  });
  await throwOnError(res, "postIngest");
  return (await res.json()) as { document_id: string };
}

export async function postMap(baseUrl: string, documentId: string, blob: Uint8Array): Promise<void> {
  const res = await postWithRetry(authedUrl(baseUrl, `/map?document_id=${encodeURIComponent(documentId)}`), {
    method: "POST",
    headers: authHeaders(),
    body: blob,
  });
  await throwOnError(res, "postMap");
}
