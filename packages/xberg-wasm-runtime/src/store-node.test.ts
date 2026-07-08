import { describe, it, expect, beforeEach } from "vitest";
import { existsSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createNodeVectorStore } from "./store-node";
import type { VectorStoreInterface, DocumentRecord, ChunkRecord } from "./types";

describe("node vector store (better-sqlite3 + sqlite-vec)", () => {
	let store: VectorStoreInterface;
	const testCollection = "test-docs";
	const vectorDim = 4;

	beforeEach(async () => {
		store = await createNodeVectorStore({ nodeStorePath: ":memory:" });
	});

	it("creates a persistent database inside nodeCachePath", async () => {
		const cacheDirectory = mkdtempSync(join(tmpdir(), "xberg-store-"));
		await createNodeVectorStore({ nodeCachePath: cacheDirectory });
		expect(existsSync(join(cacheDirectory, "store.sqlite3"))).toBe(true);
	});

	it("keeps sanitized-name collisions isolated", async () => {
		await store.ensureCollection("test-docs", vectorDim);
		await store.ensureCollection("test_docs", vectorDim);
		const first: DocumentRecord = { documentId: "doc-1", sourceId: "same", collectionId: "test-docs" };
		const second: DocumentRecord = { documentId: "doc-1", sourceId: "same", collectionId: "test_docs" };
		await store.upsertDocument("test-docs", first, [
			{
				sourceId: "same",
				chunkIndex: 0,
				text: "first",
				startOffset: 0,
				endOffset: 5,
				embedding: new Float32Array([1, 0, 0, 0]),
			},
		]);
		await store.upsertDocument("test_docs", second, [
			{
				sourceId: "same",
				chunkIndex: 0,
				text: "second",
				startOffset: 0,
				endOffset: 6,
				embedding: new Float32Array([1, 0, 0, 0]),
			},
		]);
		expect((await store.query("test-docs", [1, 0, 0, 0], 1))[0]?.text).toBe("first");
		expect((await store.query("test_docs", [1, 0, 0, 0], 1))[0]?.text).toBe("second");
	});

	it("ensures a collection and creates its vec0 table", async () => {
		await store.ensureCollection(testCollection, vectorDim);
		const collections = await store.listCollections();
		expect(collections).toContain(testCollection);
	});

	it("upserts a document with chunks and queries by real vec0 similarity", async () => {
		await store.ensureCollection(testCollection, vectorDim);
		const doc: DocumentRecord = { documentId: "doc-1", sourceId: "src-1", collectionId: testCollection };
		const chunks: ChunkRecord[] = [
			{
				sourceId: "src-1",
				chunkIndex: 0,
				text: "apple fruit",
				startOffset: 0,
				endOffset: 11,
				embedding: new Float32Array([1, 0, 0, 0]),
			},
			{
				sourceId: "src-1",
				chunkIndex: 1,
				text: "apple tree",
				startOffset: 12,
				endOffset: 22,
				embedding: new Float32Array([0, 1, 0, 0]),
			},
		];
		const result = await store.upsertDocument(testCollection, doc, chunks);
		expect(result.chunksCount).toBe(2);
		const results = await store.query(testCollection, [1, 0, 0, 0], 2);
		expect(results.length).toBe(2);
		expect(results[0]?.text).toBe("apple fruit");
		expect(results[0]?.score).toBeGreaterThan(results[1]?.score ?? Infinity);
	});

	it("deletes a document and its chunks are no longer queryable", async () => {
		await store.ensureCollection(testCollection, vectorDim);
		const doc: DocumentRecord = { documentId: "doc-1", sourceId: "src-1", collectionId: testCollection };
		const chunk: ChunkRecord = {
			sourceId: "src-1",
			chunkIndex: 0,
			text: "hello",
			startOffset: 0,
			endOffset: 5,
			embedding: new Float32Array([1, 0, 0, 0]),
		};
		await store.upsertDocument(testCollection, doc, [chunk]);
		await store.delete(testCollection, "doc-1");
		const results = await store.query(testCollection, [1, 0, 0, 0], 10);
		expect(results.some((r) => r.chunkId.startsWith("src-1"))).toBe(false);
	});

	it("drops a collection including its vec0 table", async () => {
		await store.ensureCollection(testCollection, vectorDim);
		await store.dropCollection(testCollection);
		expect(await store.listCollections()).not.toContain(testCollection);
	});

	it("creates a graph edge and traverses it via recursive CTE", async () => {
		await store.createEdge({ id: "e1", source: "a", target: "b", label: "relates_to" });
		await store.createEdge({ id: "e2", source: "b", target: "c", label: "relates_to" });
		await store.createEdge({ id: "e3", source: "a", target: "z", label: "unrelated" });
		const reached = await store.traverseGraph(["a"], 2, ["relates_to"]);
		expect(reached).toContain("a");
		expect(reached).toContain("b");
		expect(reached).toContain("c");
		expect(reached).not.toContain("z");
	});

	it("traverseGraph respects depth limit", async () => {
		await store.createEdge({ id: "e1", source: "a", target: "b" });
		await store.createEdge({ id: "e2", source: "b", target: "c" });
		const reached = await store.traverseGraph(["a"], 1);
		expect(reached).toContain("b");
		expect(reached).not.toContain("c");
	});
});
