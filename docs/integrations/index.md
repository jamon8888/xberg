# Integrations

Kreuzberg integrates with external databases and services to store, index, and search extracted documents.

Each integration connects Kreuzberg's extraction output to a target system — handling schema setup, content deduplication, and indexing automatically.

---

## Available Integrations

| Integration | Target | Package | Search Capabilities | Status |
|---|---|---|---|---|
| [SurrealDB](surrealdb.md) | [SurrealDB](https://surrealdb.com/) | [`kreuzberg-surrealdb`](https://pypi.org/project/kreuzberg-surrealdb/) | BM25, Vector (HNSW), Hybrid (RRF¹) | :white_check_mark: Stable |

¹ RRF = Reciprocal Rank Fusion — a method for combining results from multiple search strategies into a single ranked list.

!!! tip "Building a New Integration?"
    Follow the pattern established by the [SurrealDB integration](https://github.com/kreuzberg-dev/kreuzberg-surrealdb), which serves as the reference implementation.