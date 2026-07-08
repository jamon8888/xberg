export function sanitizeTableName(collection: string): string {
	return collection.replace(/[^a-zA-Z0-9_]/g, "_").slice(0, 48);
}

function collectionHash(collection: string): string {
	let hash = 0x811c9dc5;
	for (let index = 0; index < collection.length; index += 1) {
		hash ^= collection.charCodeAt(index);
		hash = Math.imul(hash, 0x01000193);
	}
	return (hash >>> 0).toString(16).padStart(8, "0");
}
export function vecTableName(collection: string): string {
	return `vec_${sanitizeTableName(collection)}_${collectionHash(collection)}`;
}
const MAX_VECTOR_DIMENSION = 65_536;

export function createVecTableSql(collection: string, vectorDim: number): string {
	if (!Number.isSafeInteger(vectorDim) || vectorDim < 1 || vectorDim > MAX_VECTOR_DIMENSION) {
		throw new RangeError(`vector dimension must be an integer from 1 to ${MAX_VECTOR_DIMENSION}`);
	}
	const table = vecTableName(collection);
	return `CREATE VIRTUAL TABLE IF NOT EXISTS ${table} USING vec0(chunk_id TEXT PRIMARY KEY, embedding FLOAT[${vectorDim}])`;
}
export const SCHEMA_SQL = `
CREATE TABLE IF NOT EXISTS collections (
  name TEXT PRIMARY KEY,
  sanitized_name TEXT NOT NULL,
  vector_dim INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS documents (
  document_id TEXT NOT NULL,
  source_id TEXT NOT NULL,
  collection TEXT NOT NULL,
  metadata TEXT,
  text TEXT,
  PRIMARY KEY (collection, document_id)
);
CREATE INDEX IF NOT EXISTS idx_documents_collection ON documents(collection);
CREATE INDEX IF NOT EXISTS idx_documents_source ON documents(source_id);
CREATE TABLE IF NOT EXISTS chunks (
  chunk_id TEXT NOT NULL,
  collection TEXT NOT NULL,
  source_id TEXT NOT NULL,
  chunk_index INTEGER NOT NULL,
  text TEXT NOT NULL,
  start_offset INTEGER NOT NULL,
  end_offset INTEGER NOT NULL,
  PRIMARY KEY (collection, chunk_id)
);
CREATE INDEX IF NOT EXISTS idx_chunks_collection ON chunks(collection);
CREATE INDEX IF NOT EXISTS idx_chunks_source ON chunks(source_id);
CREATE TABLE IF NOT EXISTS graph_edges (
  id TEXT PRIMARY KEY,
  source TEXT NOT NULL,
  target TEXT NOT NULL,
  label TEXT,
  properties TEXT
);
CREATE INDEX IF NOT EXISTS idx_edges_source ON graph_edges(source);
CREATE INDEX IF NOT EXISTS idx_edges_target ON graph_edges(target);
CREATE INDEX IF NOT EXISTS idx_edges_label ON graph_edges(label);
`;
