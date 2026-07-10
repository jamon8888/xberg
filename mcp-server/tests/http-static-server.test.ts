import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { createServer, type Server } from "node:http";
import { mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { resolveSafePath, serveStaticFile } from "../src/http/static-server.js";

describe("http/static-server", () => {
  let server: Server;
  let baseUrl: string;
  let dir: string;

  beforeAll(async () => {
    dir = mkdtempSync(join(tmpdir(), "xberg-ui-test-"));
    writeFileSync(join(dir, "index.html"), "<html><body>hi</body></html>");
    writeFileSync(join(dir, "app.js"), "console.log('hi');");

    server = createServer((req, res) => {
      serveStaticFile(dir, req.url ?? "/", res);
    });
    await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
    const address = server.address();
    if (address === null || typeof address === "string") throw new Error("expected an AddressInfo");
    baseUrl = `http://127.0.0.1:${address.port}`;
  });

  afterAll(async () => {
    await new Promise<void>((resolve, reject) => server.close((err) => (err ? reject(err) : resolve())));
    rmSync(dir, { recursive: true, force: true });
  });

  it("serves index.html at the root with COOP/COEP headers", async () => {
    const res = await fetch(`${baseUrl}/`);
    expect(res.status).toBe(200);
    expect(await res.text()).toContain("hi");
    expect(res.headers.get("cross-origin-opener-policy")).toBe("same-origin");
    expect(res.headers.get("cross-origin-embedder-policy")).toBe("require-corp");
  });

  it("serves a JS asset with the correct content-type", async () => {
    const res = await fetch(`${baseUrl}/app.js`);
    expect(res.status).toBe(200);
    expect(res.headers.get("content-type")).toContain("text/javascript");
  });

  it("returns 404 for a missing file", async () => {
    const res = await fetch(`${baseUrl}/missing.html`);
    expect(res.status).toBe(404);
  });

  it("resolveSafePath resolves a normal path inside the root", () => {
    expect(resolveSafePath(dir, "/app.js")).toBe(join(dir, "app.js"));
  });

  it("resolveSafePath rejects a traversal attempt above the root", () => {
    expect(resolveSafePath(dir, "/../../../etc/passwd")).toBeNull();
    expect(resolveSafePath(dir, "/..%2f..%2f..%2fetc%2fpasswd")).toBeNull();
  });
});
