// e2e/ingest.spec.ts
import { test, expect } from "@playwright/test";
import { createServer } from "node:http";
import { EMBEDDING_DIM } from "../src/lib/constants.js";

test("uploading a document with PII syncs to the MCP store via /collection, /ingest, /map", async ({ page }) => {
  const received: { collection?: unknown; ingest?: unknown; mapDocumentId?: string } = {};
  const server = createServer(async (req, res) => {
    const url = new URL(req.url ?? "/", "http://localhost");
    const send = (status: number, body: unknown) => {
      res.writeHead(status, { "Content-Type": "application/json" });
      res.end(JSON.stringify(body));
    };
    if (req.method === "POST" && url.pathname === "/collection") {
      let body = "";
      for await (const chunk of req) body += chunk;
      received.collection = JSON.parse(body);
      send(200, { created: true });
      return;
    }
    if (req.method === "POST" && url.pathname === "/ingest") {
      let body = "";
      for await (const chunk of req) body += chunk;
      received.ingest = JSON.parse(body);
      send(200, { document_id: "doc-e2e-1" });
      return;
    }
    if (req.method === "POST" && url.pathname === "/map") {
      received.mapDocumentId = url.searchParams.get("document_id") ?? undefined;
      for await (const _chunk of req) {
        // drain the body; nothing to inspect for this happy-path test
      }
      send(200, { status: "stored" });
      return;
    }
    send(404, {});
  });
  await new Promise<void>((resolve) => server.listen(8081, "127.0.0.1", resolve));

  try {
    await page.goto("http://127.0.0.1:8081/ui/?token=test");
    await page.getByText("New folder").click();
    await page.getByLabel("Folder name").fill("contrats");
    await page.getByText("Create").click();
    await page.getByText("contrats").click();

    await page.getByLabel(/passphrase/i).fill("correct-horse-battery");
    await page.setInputFiles("input[type=file]", {
      name: "contrat.pdf",
      mimeType: "application/pdf",
      buffer: Buffer.from("Contact alice@example.com about the contract"),
    });

    await expect.poll(() => received.ingest !== undefined, { timeout: 30_000 }).toBe(true);
    expect(received.collection).toEqual({ name: "contrats", embedding_dim: EMBEDDING_DIM });
    expect(received.mapDocumentId).toBe("contrat.pdf");
    expect((received.ingest as { external_id: string }).external_id).toBe("contrat.pdf");
    expect((received.ingest as { full_text: string }).full_text).not.toContain("alice@example.com");
  } finally {
    server.close();
  }
});
