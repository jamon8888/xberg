import { expect, test } from "@playwright/test";

test("persists and queries vectors through the real OPFS worker", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/tests/browser/");
  await page.waitForFunction(() => typeof (globalThis as any).createTestStore === "function");

  const path = `/browser-${Date.now()}.sqlite3`;
  const first = await page.evaluate(async (databasePath) => Promise.race([(async () => {
    const store = await (globalThis as any).createTestStore(databasePath);
    await store.ensureCollection("browser-docs", 4);
    await store.upsertDocument(
      "browser-docs",
      { documentId: "doc-1", sourceId: "source-1", collectionId: "browser-docs" },
      [{
        sourceId: "source-1",
        chunkIndex: 0,
        text: "persistent browser vector",
        startOffset: 0,
        endOffset: 25,
        embedding: new Float32Array([1, 0, 0, 0]),
      }],
    );
    const result = await store.query("browser-docs", [1, 0, 0, 0], 1);
    await store.close();
    return result;
  })(), new Promise<never>((_, reject) => setTimeout(() => reject(new Error("browser store scenario timed out")), 20_000))]), path);
  expect(first[0]?.text).toBe("persistent browser vector");

  await page.reload();
  const persisted = await page.evaluate(async (databasePath) => Promise.race([(async () => {
    const store = await (globalThis as any).createTestStore(databasePath);
    const result = await store.query("browser-docs", [1, 0, 0, 0], 1);
    await store.createEdge({ id: "edge-1", source: "source-1", target: "source-2", label: "references" });
    const graph = await store.traverseGraph(["source-1"], 1, ["references"]);
    await store.close();
    return { result, graph };
  })(), new Promise<never>((_, reject) => setTimeout(() => reject(new Error("browser store reload timed out")), 20_000))]), path);
  expect(persisted.result[0]?.text).toBe("persistent browser vector");
  expect(persisted.graph).toEqual(expect.arrayContaining(["source-1", "source-2"]));
  expect(errors).toEqual([]);
});

test("isolates colliding collection names and supports delete/drop", async ({ page }) => {
  await page.goto("/tests/browser/");
  await page.waitForFunction(() => typeof (globalThis as any).createTestStore === "function");
  const result = await page.evaluate(async (databasePath) => {
    const store = await (globalThis as any).createTestStore(databasePath);
    for (const collection of ["test-docs", "test_docs"]) {
      await store.ensureCollection(collection, 4);
      await store.upsertDocument(
        collection,
        { documentId: "same-doc", sourceId: "same-source", collectionId: collection },
        [{
          sourceId: "same-source",
          chunkIndex: 0,
          text: collection,
          startOffset: 0,
          endOffset: collection.length,
          embedding: new Float32Array([1, 0, 0, 0]),
        }],
      );
    }
    const first = await store.query("test-docs", [1, 0, 0, 0], 1);
    const second = await store.query("test_docs", [1, 0, 0, 0], 1);
    await store.delete("test-docs", "same-doc");
    const deleted = await store.query("test-docs", [1, 0, 0, 0], 1);
    await store.dropCollection("test_docs");
    const collections = await store.listCollections();
    await store.close();
    return { first, second, deleted, collections };
  }, `/isolation-${Date.now()}.sqlite3`);

  expect(result.first[0]?.text).toBe("test-docs");
  expect(result.second[0]?.text).toBe("test_docs");
  expect(result.deleted).toEqual([]);
  expect(result.collections).toEqual(["test-docs"]);
});
