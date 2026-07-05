import { describe, it, expect } from "vitest";
import { chunkText } from "../src/chunker.js";

describe("chunkText", () => {
  it("returns empty array for blank text", () => {
    expect(chunkText("   ")).toHaveLength(0);
  });

  it("keeps a short paragraph as a single chunk", () => {
    const chunks = chunkText("Hello world.");
    expect(chunks).toEqual(["Hello world."]);
  });

  it("splits a long paragraph into overlapping chunks bounded by maxChars", () => {
    const para = "a".repeat(1000);
    const chunks = chunkText(para, { maxChars: 300, overlap: 50 });
    expect(chunks.length).toBeGreaterThan(1);
    for (const chunk of chunks) {
      expect(chunk.length).toBeLessThanOrEqual(300);
    }
  });

  it("throws when overlap equals maxChars instead of looping forever", () => {
    const para = "b".repeat(1000);
    expect(() => chunkText(para, { maxChars: 100, overlap: 100 })).toThrow(
      "overlap must be smaller than maxChars",
    );
  });

  it("throws when overlap exceeds maxChars instead of looping forever", () => {
    const para = "c".repeat(1000);
    expect(() => chunkText(para, { maxChars: 100, overlap: 150 })).toThrow(
      "overlap must be smaller than maxChars",
    );
  });

  it("throws for a bad overlap/maxChars pair even when text is short", () => {
    expect(() => chunkText("short text", { maxChars: 10, overlap: 10 })).toThrow(
      "overlap must be smaller than maxChars",
    );
  });
});
