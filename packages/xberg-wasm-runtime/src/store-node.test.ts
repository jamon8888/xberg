import { afterEach, describe, it, expect, beforeEach } from "vitest";
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

	afterEach(async () => store.close());

	it("creates a persistent database inside nodeCachePath", async () => {
		const cacheDirectory = mkdtempSync(join(tmpdir(), "xberg-store-"));
		const persistentStore = await createNodeVectorStore({ nodeCachePath: cacheDirectory });
		expect(existsSync(join(cacheDirectory, "store.sqlite3"))).toBe(true);
		await persistentStore.close();
	});

	it("validates collection names and dimensions", async () => {
		await expect(store.ensureCollection(" ", vectorDim)).rejects.toThrow("collection must not be empty");
		await expect(store.ensureCollection(testCollection, 0)).rejects.toThrow("positive integer");
		await store.ensureCollection(testCollection, vectorDim);
		await expect(store.ensureCollection(testCollection, vectorDim + 1)).rejects.toThrow(
			"expects vectors of dimension",
		);
	});

	it.each([0, -1, 1.5])("rejects invalid query limit %s", async (k) => {
		await expect(store.query(testCollection, [1, 0, 0, 0], k)).rejects.toThrow("positive integer");
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

	it("retrieve() in fulltext mode finds a chunk by exact text match", async () => {
		await store.ensureCollection(testCollection, vectorDim);
		const doc: DocumentRecord = { documentId: "doc-1", sourceId: "src-1", collectionId: testCollection };
		await store.upsertDocument(testCollection, doc, [
			{
				sourceId: "src-1",
				chunkIndex: 0,
				text: "the quick brown fox",
				startOffset: 0,
				endOffset: 19,
				embedding: new Float32Array([1, 0, 0, 0]),
			},
		]);
		const results = await store.retrieve(testCollection, { mode: "fulltext", queryText: "brown fox", k: 5 });
		expect(results[0]?.text).toBe("the quick brown fox");
	});

	it("retrieve() in vector mode matches query() behavior", async () => {
		await store.ensureCollection(testCollection, vectorDim);
		const doc: DocumentRecord = { documentId: "doc-1", sourceId: "src-1", collectionId: testCollection };
		await store.upsertDocument(testCollection, doc, [
			{
				sourceId: "src-1",
				chunkIndex: 0,
				text: "apple",
				startOffset: 0,
				endOffset: 5,
				embedding: new Float32Array([1, 0, 0, 0]),
			},
		]);
		const results = await store.retrieve(testCollection, { mode: "vector", queryVector: [1, 0, 0, 0], k: 5 });
		expect(results[0]?.text).toBe("apple");
	});

	it("retrieve() in hybrid mode ranks a chunk good on both signals above either extreme", async () => {
		await store.ensureCollection(testCollection, vectorDim);
		const doc: DocumentRecord = { documentId: "doc-1", sourceId: "src-1", collectionId: testCollection };
		// Fixture verified against the real better-sqlite3 + sqlite-vec + FTS5 engine (not assumed):
		// - FTS5 MATCH ANDs bareword terms by default, so the query text is kept to terms every
		//   textually-relevant chunk actually contains ("hybrid search"), otherwise a partial text
		//   match is excluded from the fulltext ranking entirely rather than ranked lower.
		// - Two vector-only filler chunks push chunk 1's vector rank down to 5th so RRF's convex
		//   1/(k+rank) sum genuinely favors chunk 2 (moderate rank 2 + rank 2) over chunk 1
		//   (best-possible text rank 1 offset by a much worse vector rank) instead of tying/losing.
		await store.upsertDocument(testCollection, doc, [
			{
				// Exact vector match, textually irrelevant to the query text.
				sourceId: "src-1",
				chunkIndex: 0,
				text: "zzz unrelated content",
				startOffset: 0,
				endOffset: 22,
				embedding: new Float32Array([1, 0, 0, 0]),
			},
			{
				// Textually exact, vector-distant.
				sourceId: "src-1",
				chunkIndex: 1,
				text: "hybrid search",
				startOffset: 23,
				endOffset: 37,
				embedding: new Float32Array([0, 0, 0, 1]),
			},
			{
				// Moderately good on both.
				sourceId: "src-1",
				chunkIndex: 2,
				text: "hybrid search related content",
				startOffset: 38,
				endOffset: 68,
				embedding: new Float32Array([0.7, 0, 0, 0.7]),
			},
			{
				// Vector-only filler, textually irrelevant: closer to the query vector than chunk 1,
				// pushing chunk 1 further down the vector ranking.
				sourceId: "src-1",
				chunkIndex: 3,
				text: "filler padding words one",
				startOffset: 69,
				endOffset: 94,
				embedding: new Float32Array([0.2, 0, 0, 0.6]),
			},
			{
				// Second vector-only filler, same purpose as chunk 3.
				sourceId: "src-1",
				chunkIndex: 4,
				text: "filler padding words two",
				startOffset: 95,
				endOffset: 120,
				embedding: new Float32Array([0.1, 0, 0, 0.8]),
			},
		]);
		const results = await store.retrieve(testCollection, {
			mode: "hybrid",
			queryVector: [1, 0, 0, 0],
			queryText: "hybrid search",
			k: 3,
		});
		expect(results[0]?.chunkId).toBe("src-1:2");
	});

	it("retrieve() throws for fulltext mode without queryText", async () => {
		await store.ensureCollection(testCollection, vectorDim);
		await expect(store.retrieve(testCollection, { mode: "fulltext", k: 5 })).rejects.toThrow(/queryText/);
	});

	it("retrieve() throws for hybrid mode missing either query input", async () => {
		await store.ensureCollection(testCollection, vectorDim);
		await expect(store.retrieve(testCollection, { mode: "hybrid", queryText: "x", k: 5 })).rejects.toThrow(
			/queryVector/,
		);
	});
});
