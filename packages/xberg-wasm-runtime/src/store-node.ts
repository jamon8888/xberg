import Database from "better-sqlite3";
import * as sqliteVec from "sqlite-vec";
import { mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { SCHEMA_SQL, createVecTableSql, vecTableName, sanitizeTableName } from "./store-schema.js";
import { reciprocalRankFusion } from "./retrieve-fusion.js";
import type {
	VectorStoreInterface,
	DocumentRecord,
	ChunkRecord,
	GraphEdge,
	CacheConfig,
	RetrieveOptions,
} from "./types.js";

const HYBRID_CANDIDATE_MULTIPLIER = 4;

export async function createNodeVectorStore(config?: CacheConfig): Promise<VectorStoreInterface> {
	const dbPath =
		config?.nodeStorePath ?? (config?.nodeCachePath ? join(config.nodeCachePath, "store.sqlite3") : ":memory:");
	if (dbPath !== ":memory:") {
		mkdirSync(dirname(dbPath), { recursive: true });
	}
	const db = new Database(dbPath);
	try {
		sqliteVec.load(db);
		db.pragma("journal_mode = WAL");
		db.exec(SCHEMA_SQL);
	} catch (error) {
		db.close();
		throw error;
	}

	const vectorDims = new Map<string, number>();
	let closed = false;

	async function ensureCollection(collection: string, vectorDim: number): Promise<void> {
		if (!collection.trim()) throw new Error("collection must not be empty");
		if (!Number.isInteger(vectorDim) || vectorDim <= 0) {
			throw new Error("vectorDim must be a positive integer");
		}
		const existing = db.prepare("SELECT vector_dim FROM collections WHERE name = ?").get(collection) as
			| { vector_dim: number }
			| undefined;
		if (existing) {
			if (existing.vector_dim !== vectorDim) {
				throw new Error(`collection ${collection} expects vectors of dimension ${existing.vector_dim}`);
			}
			vectorDims.set(collection, existing.vector_dim);
			return;
		}
		db.prepare("INSERT INTO collections (name, sanitized_name, vector_dim) VALUES (?, ?, ?)").run(
			collection,
			sanitizeTableName(collection),
			vectorDim,
		);
		db.exec(createVecTableSql(collection, vectorDim));
		vectorDims.set(collection, vectorDim);
	}

	async function upsertDocument(
		collection: string,
		doc: DocumentRecord,
		chunkRecords: ChunkRecord[],
	): Promise<{ documentId: string; chunksCount: number }> {
		const table = vecTableName(collection);
		const insertDoc = db.prepare(
			`INSERT OR REPLACE INTO documents (document_id, source_id, collection, metadata, text) VALUES (?, ?, ?, ?, ?)`,
		);
		const insertChunk = db.prepare(
			`INSERT OR REPLACE INTO chunks (chunk_id, collection, source_id, chunk_index, text, start_offset, end_offset) VALUES (?, ?, ?, ?, ?, ?, ?)`,
		);
		const deleteVec = db.prepare(`DELETE FROM ${table} WHERE chunk_id = ?`);
		const insertVec = db.prepare(`INSERT INTO ${table} (chunk_id, embedding) VALUES (?, ?)`);
		const tx = db.transaction(() => {
			insertDoc.run(
				doc.documentId,
				doc.sourceId,
				collection,
				doc.metadata ? JSON.stringify(doc.metadata) : null,
				doc.text ?? null,
			);
			for (const chunk of chunkRecords) {
				const chunkId = `${chunk.sourceId}:${chunk.chunkIndex}`;
				insertChunk.run(
					chunkId,
					collection,
					chunk.sourceId,
					chunk.chunkIndex,
					chunk.text,
					chunk.startOffset,
					chunk.endOffset,
				);
				deleteVec.run(chunkId);
				insertVec.run(
					chunkId,
					Buffer.from(chunk.embedding.buffer, chunk.embedding.byteOffset, chunk.embedding.byteLength),
				);
			}
		});
		tx();
		return { documentId: doc.documentId, chunksCount: chunkRecords.length };
	}

	async function query(
		collection: string,
		queryVector: number[],
		k: number,
	): Promise<Array<{ chunkId: string; text: string; score: number }>> {
		if (!Number.isInteger(k) || k <= 0) throw new Error("k must be a positive integer");
		const table = vecTableName(collection);
		const queryBuf = Buffer.from(new Float32Array(queryVector).buffer);
		const rows = db
			.prepare(
				`SELECT v.chunk_id AS chunkId, c.text AS text, v.distance AS distance
       FROM ${table} v JOIN chunks c ON c.chunk_id = v.chunk_id AND c.collection = ?
       WHERE v.embedding MATCH ? AND k = ? ORDER BY v.distance`,
			)
			.all(collection, queryBuf, k) as Array<{ chunkId: string; text: string; distance: number }>;
		return rows.map((r) => ({ chunkId: r.chunkId, text: r.text, score: -r.distance }));
	}

	async function fullTextQuery(
		collection: string,
		queryText: string,
		k: number,
	): Promise<Array<{ chunkId: string; text: string; score: number }>> {
		const rows = db
			.prepare(
				`SELECT f.chunk_id AS chunkId, c.text AS text, bm25(chunks_fts) AS rank
       FROM chunks_fts f JOIN chunks c ON c.chunk_id = f.chunk_id AND c.collection = f.collection
       WHERE chunks_fts MATCH ? AND f.collection = ? ORDER BY rank LIMIT ?`,
			)
			.all(queryText, collection, k) as Array<{ chunkId: string; text: string; rank: number }>;
		// bm25() is smaller-is-better; negate for the larger-is-better score convention query() already uses.
		return rows.map((r) => ({ chunkId: r.chunkId, text: r.text, score: -r.rank }));
	}

	async function retrieve(
		collection: string,
		opts: RetrieveOptions,
	): Promise<Array<{ chunkId: string; text: string; score: number }>> {
		if (opts.mode === "vector") {
			if (!opts.queryVector) throw new Error("retrieve: queryVector is required for mode 'vector'");
			return query(collection, opts.queryVector, opts.k);
		}
		if (opts.mode === "fulltext") {
			if (!opts.queryText) throw new Error("retrieve: queryText is required for mode 'fulltext'");
			return fullTextQuery(collection, opts.queryText, opts.k);
		}
		if (!opts.queryVector) throw new Error("retrieve: queryVector is required for mode 'hybrid'");
		if (!opts.queryText) throw new Error("retrieve: queryText is required for mode 'hybrid'");
		const candidateK = opts.k * HYBRID_CANDIDATE_MULTIPLIER;
		const [vectorResults, textResults] = await Promise.all([
			query(collection, opts.queryVector, candidateK),
			fullTextQuery(collection, opts.queryText, candidateK),
		]);
		return reciprocalRankFusion([vectorResults, textResults]).slice(0, opts.k);
	}

	async function deleteDocument(collection: string, documentId: string): Promise<void> {
		const table = vecTableName(collection);
		const doc = db
			.prepare("SELECT source_id FROM documents WHERE collection = ? AND document_id = ?")
			.get(collection, documentId) as { source_id: string } | undefined;
		if (!doc) return;
		const tx = db.transaction(() => {
			const chunkIds = db
				.prepare("SELECT chunk_id FROM chunks WHERE collection = ? AND source_id = ?")
				.all(collection, doc.source_id) as Array<{ chunk_id: string }>;
			const deleteVec = db.prepare(`DELETE FROM ${table} WHERE chunk_id = ?`);
			for (const { chunk_id } of chunkIds) deleteVec.run(chunk_id);
			db.prepare("DELETE FROM chunks WHERE collection = ? AND source_id = ?").run(collection, doc.source_id);
			db.prepare("DELETE FROM documents WHERE collection = ? AND document_id = ?").run(collection, documentId);
		});
		tx();
	}

	async function listCollections(): Promise<string[]> {
		return (db.prepare("SELECT name FROM collections").all() as Array<{ name: string }>).map((r) => r.name);
	}

	async function dropCollection(collection: string): Promise<void> {
		const table = vecTableName(collection);
		const tx = db.transaction(() => {
			db.exec(`DROP TABLE IF EXISTS ${table}`);
			db.prepare("DELETE FROM chunks WHERE collection = ?").run(collection);
			db.prepare("DELETE FROM documents WHERE collection = ?").run(collection);
			db.prepare("DELETE FROM collections WHERE name = ?").run(collection);
		});
		tx();
		vectorDims.delete(collection);
	}

	async function createEdge(edge: GraphEdge): Promise<void> {
		db.prepare(
			`INSERT OR REPLACE INTO graph_edges (id, source, target, label, properties) VALUES (?, ?, ?, ?, ?)`,
		).run(
			edge.id,
			edge.source,
			edge.target,
			edge.label ?? null,
			edge.properties ? JSON.stringify(edge.properties) : null,
		);
	}

	async function traverseGraph(startIds: string[], depth: number, edgeLabels?: string[]): Promise<string[]> {
		if (startIds.length === 0) return [];
		const labelFilter =
			edgeLabels && edgeLabels.length > 0 ? `AND e.label IN (${edgeLabels.map(() => "?").join(",")})` : "";
		const sql = `
      WITH RECURSIVE traversal(node_id, depth) AS (
        SELECT value, 0 FROM json_each(?)
        UNION
        SELECT e.target, traversal.depth + 1
        FROM traversal JOIN graph_edges e ON e.source = traversal.node_id
        WHERE traversal.depth < ? ${labelFilter}
      )
      SELECT DISTINCT node_id FROM traversal`;
		const params: unknown[] = [JSON.stringify(startIds), depth, ...(edgeLabels ?? [])];
		return (db.prepare(sql).all(...params) as Array<{ node_id: string }>).map((r) => r.node_id);
	}

	return {
		close: async () => {
			if (!closed) {
				db.close();
				closed = true;
			}
		},
		ensureCollection,
		upsertDocument,
		query,
		retrieve,
		delete: deleteDocument,
		listCollections,
		dropCollection,
		createEdge,
		traverseGraph,
	};
}
