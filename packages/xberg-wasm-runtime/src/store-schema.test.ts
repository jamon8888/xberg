import { describe, it, expect } from "vitest";
import { sanitizeTableName, SCHEMA_SQL, vecTableName, createVecTableSql } from "./store-schema";

describe("store-schema", () => {
  it("sanitizes a collection name with hyphens into a valid identifier", () => {
    expect(sanitizeTableName("test-docs")).toBe("test_docs");
  });
  it("sanitizes a collection name with spaces and special chars", () => {
    expect(sanitizeTableName("my collection!@#")).toBe("my_collection___");
  });
  it("produces a deterministic vec table name", () => {
    expect(vecTableName("test-docs")).toBe("vec_test_docs");
  });
  it("produces a valid CREATE VIRTUAL TABLE statement with the given dimension", () => {
    const sql = createVecTableSql("test-docs", 384);
    expect(sql).toContain("CREATE VIRTUAL TABLE IF NOT EXISTS vec_test_docs");
    expect(sql).toContain("USING vec0");
    expect(sql).toContain("FLOAT[384]");
  });
  it("SCHEMA_SQL defines the collections, documents, chunks, and graph_edges tables", () => {
    expect(SCHEMA_SQL).toContain("CREATE TABLE IF NOT EXISTS collections");
    expect(SCHEMA_SQL).toContain("CREATE TABLE IF NOT EXISTS documents");
    expect(SCHEMA_SQL).toContain("CREATE TABLE IF NOT EXISTS chunks");
    expect(SCHEMA_SQL).toContain("CREATE TABLE IF NOT EXISTS graph_edges");
    expect(SCHEMA_SQL).toContain("source");
    expect(SCHEMA_SQL).toContain("target");
    expect(SCHEMA_SQL).toContain("label");
  });
});
