# Phase 11: LanceDB Backend Implementation

## Overview
Add LanceDB as an alternative backend for both BM25 (full-text search) and vector search, alongside the existing SQLite-based backends. The architecture already has `BM25Backend` and `VectorBackend` enums and CLI args (`--fts-backend`, `--vector-backend`) defined — we need to wire up the actual LanceDB implementation.

## Status: IN PROGRESS

### Completed:
1. ✅ Added LanceDB dependencies to Cargo.toml (behind `lancedb` feature flag)
2. ✅ Created LanceDB backend module (`src/store/lance_backend.rs`)
3. ✅ Created LanceDB backend struct with stub implementation
4. ✅ Wired LanceDB into Store dispatch for `bm25_search` and `vector_search_with_embedding`
5. ✅ All 169 tests pass both with and without `lancedb` feature

### Current Implementation:
The LanceDB backend (`lance_backend.rs`) provides the following interface:
- `LanceDbBackend::new(db_path, embedding_dim)` - Create new backend instance
- `connect()` - Establish connection (stub)
- `get_fts_table(collection)` - Get FTS table (stub)
- `get_vector_table(collection)` - Get vector table (stub)
- `fts_search(table, query, limit)` - Full-text search (stub returns empty)
- `vector_search(table, query_vector, limit)` - Vector search (stub returns empty)
- `insert_fts_documents(table, documents)` - Insert documents (stub)
- `insert_vectors(table, vectors)` - Insert vectors (stub)

### Known Limitations:
The current LanceDB implementation is a **stub** that returns empty results due to LanceDB v0.23 API changes. Full implementation requires:
- Understanding the current LanceDB Rust async API
- Proper RecordBatch handling for data insertion
- Async stream processing for query results

## LanceDB Rust SDK Key Facts
- Crate: `lancedb` (v0.23)
- Fully async API (requires tokio)
- Uses Arrow RecordBatch for data ingestion
- Vector search: `table.query().nearest_to(&[f32]).execute().await`
- FTS: `table.create_index(&["text_col"], Index::Fts(...)).execute().await`
- Connect: `lancedb::connect("path").execute().await`
- Embedded mode (like SQLite), no server needed

## Files Changed:
1. `Cargo.toml` — add lancedb + arrow deps behind feature flag
2. `src/store/lance_backend.rs` — module definition
3. `src/store/lance_backend/lance_backend.rs` — LanceDB backend (stub with API ready)
4. `src/store/mod.rs` — wire LanceDB dispatch into existing search methods

## Next Steps (Future Work):
1. Implement full LanceDB FTS search using the async API
2. Implement full LanceDB vector search using `table.query().nearest_to(vector).limit(n)`
3. Add document insertion methods for LanceDB tables
4. Add integration tests for LanceDB backend
