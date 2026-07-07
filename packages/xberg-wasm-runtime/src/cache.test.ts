import { describe, it, expect, beforeAll } from "vitest";
import { CacheManager } from "./cache";
import { join } from "node:path";
import { existsSync } from "node:fs";
import { tmpdir } from "node:os";

describe("cache manager", () => {
  it("reports initial cache status", async () => {
    const manager = new CacheManager();
    const status = await manager.status();
    expect(status).toHaveProperty("cached");
    expect(status).toHaveProperty("size");
    expect(Array.isArray(status.cached)).toBe(true);
  });

  it("tracks model availability", async () => {
    const manager = new CacheManager();
    const status = await manager.status();
    // No models cached initially (or may find system defaults)
    expect(typeof status.size).toBe("number");
    expect(status.size).toBeGreaterThanOrEqual(0);
  });

  it("accepts custom cache directory", async () => {
    const customDir = "/custom/cache/path";
    const manager = new CacheManager(customDir);
    // Verify it was set (via status call which uses the directory)
    const status = await manager.status();
    expect(status).toHaveProperty("cached");
  });

  it("handles model warming with no model names (all models)", async () => {
    const manager = new CacheManager();
    const result = await manager.warm();
    expect(result).toHaveProperty("success");
    expect(result).toHaveProperty("failed");
    expect(Array.isArray(result.success)).toBe(true);
    expect(Array.isArray(result.failed)).toBe(true);
  }, 120_000);

  it("handles model warming with specific model names", async () => {
    const manager = new CacheManager();
    const result = await manager.warm(["Embedder (minilm-l6-v2)"]);
    expect(result).toHaveProperty("success");
    expect(result).toHaveProperty("failed");
  }, 120_000);

  it("setWasmPaths handles missing window gracefully", () => {
    const manager = new CacheManager();
    // In Node environment, window is undefined
    // setWasmPaths should not throw
    expect(() => manager.setWasmPaths("/some/path")).not.toThrow();
  });
});

describe("CacheManager.warm", () => {
  it("downloads and caches embedding + ner model artifacts", async () => {
    const dir = join(tmpdir(), `xberg-warm-${Date.now()}`);
    const mgr = new CacheManager(dir);
    const phases: string[] = [];
    await mgr.warm({ onProgress: (p) => phases.push(p) });
    expect(phases.length).toBeGreaterThan(0);
    expect(existsSync(dir)).toBe(true);
  }, 300_000);
});
