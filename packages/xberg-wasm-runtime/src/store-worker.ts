import { SCHEMA_SQL, createVecTableSql, sanitizeTableName, vecTableName } from "./store-schema.js";
import type { ChunkRecord, DocumentRecord, GraphEdge } from "./types.js";

export type StoreWorkerRequest =
	| { op: "init"; dbPath: string; id: number }
	| { op: "close"; id: number }
	| { op: "ensureCollection"; collection: string; vectorDim: number; id: number }
	| { op: "upsertDocument"; collection: string; doc: DocumentRecord; chunks: ChunkRecord[]; id: number }
	| { op: "query"; collection: string; queryVector: number[]; k: number; id: number }
	| { op: "delete"; collection: string; documentId: string; id: number }
	| { op: "listCollections"; id: number }
	| { op: "dropCollection"; collection: string; id: number }
	| { op: "createEdge"; edge: GraphEdge; id: number }
	| { op: "traverseGraph"; startIds: string[]; depth: number; edgeLabels?: string[]; id: number };

export interface StoreWorkerResponse {
	id: number;
	ok: boolean;
	result?: unknown;
	error?: string;
}

export type StoreWorkerRequestBase = DistributiveOmit<StoreWorkerRequest, "id">;
type DistributiveOmit<T, K extends PropertyKey> = T extends unknown ? Omit<T, K> : never;

interface SqliteDb {
	exec(options: string | { sql: string; bind?: unknown[]; rowMode?: "object"; returnValue?: "resultRows" }): unknown;
	selectValue(sql: string, bind?: unknown[]): unknown;
	close(): void;
}

interface SqliteModule {
	oo1: {
		DB: new (filename: string, flags?: string) => SqliteDb;
		OpfsDb: new (filename: string) => SqliteDb;
	};
}

let database: SqliteDb | undefined;

async function ensureOpfsDirectory(dbPath: string): Promise<void> {
	const directoryNames = dbPath.split("/").filter(Boolean).slice(0, -1);
	let directory = await navigator.storage.getDirectory();
	for (const name of directoryNames) {
		// oxlint-disable-next-line no-await-in-loop -- each handle is relative to its parent
		directory = await directory.getDirectoryHandle(name, { create: true });
	}
}

function requireDatabase(): SqliteDb {
	if (!database) throw new Error("store is not initialized");
	return database;
}

function rows<T>(db: SqliteDb, sql: string, bind: unknown[] = []): T[] {
	return db.exec({ sql, bind, rowMode: "object", returnValue: "resultRows" }) as T[];
}

function transaction(db: SqliteDb, operation: () => void): void {
	db.exec("BEGIN IMMEDIATE");
	try {
		operation();
		db.exec("COMMIT");
	} catch (error) {
		db.exec("ROLLBACK");
		throw error;
	}
}

async function initialize(dbPath: string): Promise<void> {
	if (!globalThis.crossOriginIsolated) {
		throw new Error("OPFS SQLite requires cross-origin isolation (COOP/COEP headers)");
	}
	const module = (await import("../wasm/sqlite-vec/sqlite3.mjs")) as {
		default: (options?: { locateFile?: (filename: string) => string }) => Promise<SqliteModule>;
	};
	let sqlite3: SqliteModule;
	try {
		sqlite3 = await module.default({
			locateFile: (filename) => new URL(`../wasm/sqlite-vec/${filename}`, import.meta.url).href,
		});
	} catch (error) {
		throw new Error(`loading sqlite-vec WASM: ${error instanceof Error ? error.message : String(error)}`, {
			cause: error,
		});
	}
	if (!sqlite3.oo1.OpfsDb) throw new Error("OPFS is unavailable in this browser context");
	try {
		await ensureOpfsDirectory(dbPath);
		database = new sqlite3.oo1.OpfsDb(dbPath);
	} catch (error) {
		throw new Error(`opening OPFS database ${dbPath}: ${error instanceof Error ? error.message : String(error)}`, {
			cause: error,
		});
	}
	try {
		database.exec(SCHEMA_SQL);
	} catch (error) {
		database.close();
		database = undefined;
		throw new Error(`creating store schema: ${error instanceof Error ? error.message : String(error)}`, {
			cause: error,
		});
	}
	let version: unknown;
	try {
		version = database.selectValue("SELECT vec_version()");
	} catch (error) {
		database.close();
		database = undefined;
		throw new Error(`checking sqlite-vec: ${error instanceof Error ? error.message : String(error)}`, {
			cause: error,
		});
	}
	if (typeof version !== "string") {
		database.close();
		database = undefined;
		throw new Error("sqlite-vec failed to initialize");
	}
}

function closeDatabase(): void {
	database?.close();
	database = undefined;
}

function ensureCollection(collection: string, vectorDim: number): void {
	if (!collection.trim()) throw new Error("collection must not be empty");
	if (!Number.isInteger(vectorDim) || vectorDim <= 0) throw new Error("vectorDim must be a positive integer");
	const db = requireDatabase();
	const existing = db.selectValue("SELECT vector_dim FROM collections WHERE name = ?", [collection]);
	if (existing !== undefined) {
		if (existing !== vectorDim)
			throw new Error(`collection ${collection} expects vectors of dimension ${existing}`);
		return;
	}
	db.exec({
		sql: "INSERT INTO collections (name, sanitized_name, vector_dim) VALUES (?, ?, ?)",
		bind: [collection, sanitizeTableName(collection), vectorDim],
	});
	db.exec(createVecTableSql(collection, vectorDim));
}

function upsertDocument(
	collection: string,
	doc: DocumentRecord,
	chunks: ChunkRecord[],
): { documentId: string; chunksCount: number } {
	const db = requireDatabase();
	const table = vecTableName(collection);
	transaction(db, () => {
		db.exec({
			sql: "INSERT OR REPLACE INTO documents (document_id, source_id, collection, metadata, text) VALUES (?, ?, ?, ?, ?)",
			bind: [
				doc.documentId,
				doc.sourceId,
				collection,
				doc.metadata ? JSON.stringify(doc.metadata) : null,
				doc.text ?? null,
			],
		});
		for (const chunk of chunks) {
			const chunkId = `${chunk.sourceId}:${chunk.chunkIndex}`;
			db.exec({
				sql: "INSERT OR REPLACE INTO chunks (chunk_id, collection, source_id, chunk_index, text, start_offset, end_offset) VALUES (?, ?, ?, ?, ?, ?, ?)",
				bind: [
					chunkId,
					collection,
					chunk.sourceId,
					chunk.chunkIndex,
					chunk.text,
					chunk.startOffset,
					chunk.endOffset,
				],
			});
			db.exec({ sql: `DELETE FROM ${table} WHERE chunk_id = ?`, bind: [chunkId] });
			const embedding = new Uint8Array(
				chunk.embedding.buffer,
				chunk.embedding.byteOffset,
				chunk.embedding.byteLength,
			);
			db.exec({ sql: `INSERT INTO ${table} (chunk_id, embedding) VALUES (?, ?)`, bind: [chunkId, embedding] });
		}
	});
	return { documentId: doc.documentId, chunksCount: chunks.length };
}

function query(
	collection: string,
	queryVector: number[],
	k: number,
): Array<{ chunkId: string; text: string; score: number }> {
	if (!Number.isInteger(k) || k <= 0) throw new Error("k must be a positive integer");
	const db = requireDatabase();
	const result = rows<{ chunkId: string; text: string; distance: number }>(
		db,
		`SELECT v.chunk_id AS chunkId, c.text AS text, v.distance AS distance
     FROM ${vecTableName(collection)} v
     JOIN chunks c ON c.chunk_id = v.chunk_id AND c.collection = ?
     WHERE v.embedding MATCH ? AND k = ? ORDER BY v.distance`,
		[collection, new Uint8Array(new Float32Array(queryVector).buffer), k],
	);
	return result.map((row) => ({ chunkId: row.chunkId, text: row.text, score: -row.distance }));
}

function deleteDocument(collection: string, documentId: string): void {
	const db = requireDatabase();
	const sourceId = db.selectValue("SELECT source_id FROM documents WHERE collection = ? AND document_id = ?", [
		collection,
		documentId,
	]);
	if (typeof sourceId !== "string") return;
	transaction(db, () => {
		const chunkIds = rows<{ chunk_id: string }>(
			db,
			"SELECT chunk_id FROM chunks WHERE collection = ? AND source_id = ?",
			[collection, sourceId],
		);
		for (const { chunk_id } of chunkIds)
			db.exec({ sql: `DELETE FROM ${vecTableName(collection)} WHERE chunk_id = ?`, bind: [chunk_id] });
		db.exec({ sql: "DELETE FROM chunks WHERE collection = ? AND source_id = ?", bind: [collection, sourceId] });
		db.exec({
			sql: "DELETE FROM documents WHERE collection = ? AND document_id = ?",
			bind: [collection, documentId],
		});
	});
}

function listCollections(): string[] {
	return rows<{ name: string }>(requireDatabase(), "SELECT name FROM collections ORDER BY name").map(
		({ name }) => name,
	);
}

function dropCollection(collection: string): void {
	const db = requireDatabase();
	transaction(db, () => {
		db.exec(`DROP TABLE IF EXISTS ${vecTableName(collection)}`);
		db.exec({ sql: "DELETE FROM chunks WHERE collection = ?", bind: [collection] });
		db.exec({ sql: "DELETE FROM documents WHERE collection = ?", bind: [collection] });
		db.exec({ sql: "DELETE FROM collections WHERE name = ?", bind: [collection] });
	});
}

function createEdge(edge: GraphEdge): void {
	requireDatabase().exec({
		sql: "INSERT OR REPLACE INTO graph_edges (id, source, target, label, properties) VALUES (?, ?, ?, ?, ?)",
		bind: [
			edge.id,
			edge.source,
			edge.target,
			edge.label ?? null,
			edge.properties ? JSON.stringify(edge.properties) : null,
		],
	});
}

function traverseGraph(startIds: string[], depth: number, edgeLabels?: string[]): string[] {
	if (startIds.length === 0) return [];
	if (!Number.isInteger(depth) || depth < 0) throw new Error("depth must be a non-negative integer");
	const labels = edgeLabels ?? [];
	const labelFilter = labels.length ? `AND e.label IN (${labels.map(() => "?").join(",")})` : "";
	const result = rows<{ node_id: string }>(
		requireDatabase(),
		`
    WITH RECURSIVE traversal(node_id, depth) AS (
      SELECT value, 0 FROM json_each(?)
      UNION
      SELECT e.target, traversal.depth + 1 FROM traversal
      JOIN graph_edges e ON e.source = traversal.node_id
      WHERE traversal.depth < ? ${labelFilter}
    ) SELECT DISTINCT node_id FROM traversal`,
		[JSON.stringify(startIds), depth, ...labels],
	);
	return result.map(({ node_id }) => node_id);
}

async function dispatch(request: StoreWorkerRequest): Promise<unknown> {
	switch (request.op) {
		case "init":
			return initialize(request.dbPath);
		case "close":
			return closeDatabase();
		case "ensureCollection":
			return ensureCollection(request.collection, request.vectorDim);
		case "upsertDocument":
			return upsertDocument(request.collection, request.doc, request.chunks);
		case "query":
			return query(request.collection, request.queryVector, request.k);
		case "delete":
			return deleteDocument(request.collection, request.documentId);
		case "listCollections":
			return listCollections();
		case "dropCollection":
			return dropCollection(request.collection);
		case "createEdge":
			return createEdge(request.edge);
		case "traverseGraph":
			return traverseGraph(request.startIds, request.depth, request.edgeLabels);
	}
}

globalThis.onmessage = async (event: MessageEvent<StoreWorkerRequest>) => {
	const { id } = event.data;
	try {
		const result = await dispatch(event.data);
		globalThis.postMessage({ id, ok: true, result } satisfies StoreWorkerResponse);
	} catch (error) {
		const message = error instanceof Error ? error.message : String(error);
		globalThis.postMessage({ id, ok: false, error: message } satisfies StoreWorkerResponse);
	}
};
