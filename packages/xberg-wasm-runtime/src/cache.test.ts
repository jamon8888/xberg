import { describe, it, expect } from "vitest";
import { CacheManager } from "./cache";

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
});
