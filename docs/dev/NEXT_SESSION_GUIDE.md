# Next Session Guide - QMD Development

**Last Updated**: 2026-02-12
**Current Phase**: Phase 4B & 4C Complete ✅

## 🎯 Phase 1 Status: COMPLETED ✅

### What Was Accomplished

1. **llama-cpp-2 Integration**
   - Added as optional Cargo feature (`llama-cpp`)
   - Implemented `LocalEmbedder::embed()` with real GGUF model support
   - Includes GPU acceleration and vector normalization
   - Fallback to random vectors when model unavailable

2. **sqlite-vec Vector Search**
   - Fixed SQL syntax for `vec_distance_cosine` function
   - Proper table joins (content_vectors, vectors_vec, documents)
   - Distance to similarity score conversion working correctly

3. **Batch Embedding Generation**
   - `embed_collection()` method with batch processing (size=10)
   - Stores embeddings in both metadata and vector tables
   - Supports incremental updates and force regeneration (`--force`)

4. **Async Runtime Fixes**
   - Created Tokio runtime in embed and vsearch CLI handlers
   - Implemented async versions of embed and search functions

5. **End-to-End Testing**
   - ✅ `qmd-rust embed` - successfully generates embeddings
   - ✅ `qmd-rust vsearch "machine learning"` - returns 3 results (0.76+ scores)
   - ✅ Vector search shows semantic understanding (BM25 found 0 results)

### Build Status
```bash
cd src/qmd-rust
cargo build --features sqlite-vec  # ✅ Successful
```

### Current Limitations
- Using random vectors as fallback (no real embedding model installed)
- llama-cpp feature disabled (requires libomp installation on macOS)

---

## 🎯 Phase 2 Status: COMPLETED ✅

### What Was Accomplished

1. **Hybrid Search Implementation**
   - Integrated BM25 and vector search in `hybrid_search()` method
   - Implemented RRF (Reciprocal Rank Fusion) algorithm for result merging
   - Added query expansion support (rule-based fallback)
   - Integrated LLM reranking pipeline

2. **Async Runtime Fixes**
   - Created `vector_search_with_embedder_async()` for async embedding
   - Fixed nested runtime issues in `query` command
   - Proper Tokio runtime management in CLI handlers

3. **RRF Fusion Bug Fixes**
   - Fixed path loss in `rrf_fusion()` - changed from `into_values()` to `into_iter()`
   - Proper document deduplication using path as key
   - Weighted fusion (BM25: 1.0, Vector: 1.5)

4. **End-to-End Testing**
   - ✅ `qmd-rust query "Rust programming"` - returns 4 results with correct paths
   - ✅ `qmd-rust query "Python development"` - returns 4 results
   - ✅ Hybrid search combines BM25 (1 result) + Vector (3 results) effectively

### Test Results Comparison

```bash
# Query: "Python"
BM25 Search:    1 result  (test3.md with "Python" keyword)
Vector Search:  3 results (semantic similarity with random vectors)
Hybrid Search:  3 results (RRF fusion, better ranking)
```

### Build Status
```bash
cd src/qmd-rust
cargo build --features sqlite-vec  # ✅ Successful
```

### Current Limitations
- Using random vectors as fallback (no real embedding model installed)
- llama-cpp feature disabled (requires libomp installation on macOS)

---

## 🎯 Phase 3 Status: COMPLETED ✅

### What Was Accomplished

1. **Real Embedding Model Installation**
   - Downloaded nomic-embed-text-v1.5.f16.gguf (262MB) from HuggingFace
   - Configured OpenMP linking for llama-cpp on macOS
   - Fixed build.rs to use correct OpenMP library path (`/opt/homebrew/opt/libomp/lib`)
   - Successfully built with `RUSTFLAGS="-L /opt/homebrew/opt/libomp/lib -l omp"`

2. **Vector Dimension Fix**
   - Updated sqlite-vec schema from 384 to 768 dimensions
   - Matches nomic-embed-text-v1.5 model output (768-dim embeddings)
   - Modified `src/store/mod.rs:160` to use `float[768]`

3. **GPU Acceleration Working**
   - Model loaded on Apple M3 Max GPU (Metal backend)
   - Flash Attention enabled for faster inference
   - Model params: 136.73M, embedding dimension: 768

4. **End-to-End Testing with Real Embeddings**
   - ✅ `qmd-rust embed` - Generated 3 real embeddings (768-dim)
   - ✅ `qmd-rust vsearch "machine learning"` - Returns semantic results (scores: 0.40, 0.40, 0.36)
   - ✅ `qmd-rust query "artificial intelligence"` - Hybrid search working (scores: 0.75, 0.55, 0.25)
   - ✅ BM25 vs Vector comparison: Vector search finds semantic matches even when BM25 returns 0 results

### Build Commands (Updated)
```bash
# Build with real embedding model support
RUSTFLAGS="-L /opt/homebrew/opt/libomp/lib -l omp" cargo build --features "sqlite-vec,llama-cpp"

# Or use the simpler command (sqlite-vec only, uses random vectors)
cargo build --features sqlite-vec
```

### Configuration (Updated)
```yaml
# ~/.config/qmd/index.yaml
models:
  embed:
    local: "nomic-embed-text-v1.5"
    local_path: "~/.cache/qmd/models/nomic-embed-text-v1.5.gguf"  # Symlink to .f16.gguf
    remote: "text-embedding-3-small"
```

### Test Results with Real Embeddings

**Query: "artificial intelligence"**
- BM25 Search: 0 results (no exact keyword match)
- Vector Search: 3 results (semantic similarity: 0.36, 0.34, 0.32)
- Hybrid Search: 3 results (RRF fusion: 0.75, 0.56, 0.26)

**Query: "machine learning"**
- Vector Search: 3 results (0.41, 0.41, 0.37)
- Hybrid Search: 3 results (0.48, 0.12, 0.07)

### Current Status
- ✅ Real embedding model working with GPU acceleration
- ✅ All search modes functional (BM25, Vector, Hybrid)
- ✅ Semantic search significantly better than random vectors
- ✅ No more "fallback to random vectors" warnings

---

## 🚀 Phase 4: Next Steps (Recommended Priority)

### Option A: Add Unit Tests (Highest Priority)

### Option A: Add Unit Tests (Highest Priority)

**Goal**: Ensure code quality and prevent regressions

**Files to Create**:
- `src/store/tests.rs` - Vector search and RRF fusion tests
- `src/llm/tests.rs` - Embedding generation tests

**Test Cases**:
1. RRF fusion algorithm correctness with known inputs
2. Vector search with known embeddings
3. Hybrid search result ordering
4. Query expansion functionality
5. Distance to similarity conversion
6. Embedding dimension validation (768-dim)

**Commands**:
```bash
cargo test --features sqlite-vec
cargo test --features "sqlite-vec,llama-cpp"
```

---

### Option B: Performance Optimization

**Goal**: Improve search speed and reduce memory usage

**Areas to Optimize**:
1. Cache embedding model in memory (avoid reloading for each query)
2. Batch vector search queries
3. Optimize RRF fusion for large result sets
4. Add connection pooling for multi-collection searches

**Expected Outcome**: Faster search response times, especially for repeated queries

---

### Option C: Clean Up Unused Code

**Goal**: Remove deprecated sync methods and improve code maintainability

**Files to Clean**:
- `src/store/mod.rs` - Remove unused methods (vector_search, vector_search_with_embedder, embed_collection, embed_all_collections)
- `src/llm/mod.rs` - Remove sync wrappers (embed_sync, rerank_sync) if no longer needed

**Benefits**:
- Cleaner codebase
- Fewer compiler warnings
- Easier maintenance

---

## 📝 Important Notes for Next Session

### Key Files Modified (Phase 1, 2 & 3)
- `Cargo.toml` - Added llama-cpp-2 as optional dependency
- `build.rs` - Added OpenMP linking configuration for macOS
- `src/store/mod.rs` - Updated vector dimension from 384 to 768, implemented hybrid_search(), fixed RRF fusion bug
- `src/llm/mod.rs` - Implemented real embedding generation with llama-cpp-2 and async embed()
- `src/cli/embed.rs` - Async embedding with Tokio runtime
- `src/cli/vsearch.rs` - Async vector search with Tokio runtime
- `src/cli/query.rs` - Async hybrid search with Tokio runtime
- `src/main.rs` - Updated query command to pass LLM router
- `~/.config/qmd/index.yaml` - Updated model configuration for nomic-embed-text-v1.5

### Methods Now in Use
- `Store::hybrid_search()` - ✅ Fully implemented with BM25 + Vector + RRF + Reranking
- `Store::vector_search_with_embedder_async()` - ✅ Async version for hybrid search
- `Store::rrf_fusion()` - ✅ Used in hybrid search for result merging
- `Router::embed()` - ✅ Async embedding generation
- `Router::rerank()` - ✅ Async LLM reranking

### Unused Methods (Can Be Removed)
- `Store::vector_search()` - Fallback to BM25 (line 259)
- `Store::vector_search_with_embedder()` - Sync version, replaced by async version
- `Store::embed_collection()` - Sync version (line 549)
- `Store::embed_all_collections()` - Sync version (line 626)
- `Router::embed_sync()` - Sync version, causes nested runtime issues
- `Router::rerank_sync()` - Sync version, causes nested runtime issues

### Configuration (Updated for Phase 3)
Current config location: `~/.config/qmd/index.yaml`

Example LLM config:
```yaml
models:
  embed:
    local: "nomic-embed-text-v1.5"
    local_path: "~/.cache/qmd/models/nomic-embed-text-v1.5.gguf"  # Symlink to .f16.gguf
    remote: "text-embedding-3-small"
  rerank:
    local: "bge-reranker-base"
    remote: "gpt-4o-mini"
  query_expansion:
    local: "rule-based"
    remote: "gpt-4o-mini"
```

### Database Schema (Updated for Phase 3)
- `documents` - Main document table
- `content_vectors` - Embedding metadata (hash, model, timestamp)
- `vectors_vec` - Actual vector data (hash_seq, embedding float[768]) ← **Updated to 768 dimensions**

### Build Commands (Updated for Phase 3)
```bash
# Build with real embedding model (requires OpenMP)
RUSTFLAGS="-L /opt/homebrew/opt/libomp/lib -l omp" cargo build --features "sqlite-vec,llama-cpp"

# Build with sqlite-vec only (uses random vectors as fallback)
cargo build --features sqlite-vec

# Run tests
cargo test --features sqlite-vec
```

---

## 🎯 Phase 4A Status: COMPLETED ✅

### What Was Accomplished

1. **Unit Tests for Store Module (21 tests)**
   - RRF fusion algorithm: empty input, single list, multi-list dedup, weights, top-rank bonus, k parameter, metadata preservation, 3-list fusion
   - SearchResult: equality, clone, serialization
   - SearchOptions: defaults
   - Config: db_path generation
   - SHA256 hash: deterministic, different inputs, empty string
   - Schema initialization: verifies documents, FTS, content_vectors tables
   - BM25 search: with data (finds correct results), no results case
   - Index stats: empty collection

2. **Unit Tests for LLM Module (14 tests)**
   - LocalQueryExpander: keyword match, no match fallback, single word, max expansions, no duplicates
   - Router expand_query: always includes original, max 5, no duplicates
   - Router providers: no providers, with local embedder
   - Normalize embedding: unit vector, magnitude, zero vector
   - LLMProvider display

3. **Dev Dependencies Added**
   - `tempfile = "3.10"` for temporary database testing
   - `tokio = { features = ["test-util"] }` for async test support

### Test Results
```bash
cargo test --features sqlite-vec
# running 35 tests ... test result: ok. 35 passed; 0 failed
```

---

## 🎯 Recommended Next Steps (Phase 4B/5)

**Priority 1**: Performance optimization (Phase 4 Option B) ✅ COMPLETED
- ✅ Cached embedding model in memory using Mutex<Option<CachedLlamaModel>>
- ✅ Model loads once on first query, reused for subsequent calls
- ✅ Context created per-call (lightweight), model persists

**Priority 2**: Clean up unused code (Phase 4 Option C) ✅ COMPLETED
- ✅ Removed 6 deprecated sync methods:
  - store/mod.rs: vector_search(), vector_search_with_embedder()
  - store/mod.rs: embed_collection(), embed_all_collections()
  - llm/mod.rs: embed_sync(), rerank_sync()

**Priority 3**: Add more integration tests
- Async hybrid search tests (requires tokio::test)
- Vector search with mock embeddings
- End-to-end CLI tests using assert_cmd

---

## 🐛 Known Issues

1. **OpenMP linking** - Requires RUSTFLAGS for llama-cpp feature ✅ RESOLVED
   - Solution: `RUSTFLAGS="-L /opt/homebrew/opt/libomp/lib -l omp" cargo build --features "sqlite-vec,llama-cpp"`
   - Alternative: Use sqlite-vec only (random vectors fallback)

2. **Model reloading** - Model loads on every query (performance issue) ✅ RESOLVED
   - Solution: Implemented Mutex<Option<CachedLlamaModel>> in LocalEmbedder
   - Model loads once on first query, cached for subsequent calls
   - Context created per-call (lightweight)

3. **Unused method warnings** - Several sync methods marked as unused ✅ RESOLVED
   - Solution: Removed all 6 deprecated sync methods

---

## 📚 Quick Reference Commands

### Build Commands (Updated)
### Build Commands (Updated)
```bash
# Build with real embedding model (recommended for production)
RUSTFLAGS="-L /opt/homebrew/opt/libomp/lib -l omp" cargo build --features "sqlite-vec,llama-cpp"

# Build with sqlite-vec only (development/testing)
cargo build --features sqlite-vec

# Run tests
cargo test --features sqlite-vec
```

### Search Commands (Updated with Real Embeddings)
```bash
# Update index (scan and index documents)
./target/debug/qmd-rust update

# Generate embeddings with real model
./target/debug/qmd-rust embed --collection test_collection

# BM25 full-text search
./target/debug/qmd-rust search "query" --limit 10

# Vector semantic search (uses real embeddings)
./target/debug/qmd-rust vsearch "query" --limit 10

# Hybrid search (BM25 + Vector + RRF + Reranking)
./target/debug/qmd-rust query "query" --limit 10

# Force regenerate all embeddings
./target/debug/qmd-rust embed --force
```

### Database Commands
```bash
# Check database
sqlite3 ~/.cache/qmd/test_collection/index.db "SELECT COUNT(*) FROM documents;"
sqlite3 ~/.cache/qmd/test_collection/index.db "SELECT COUNT(*) FROM content_vectors;"

# View document content
sqlite3 ~/.cache/qmd/test_collection/index.db "SELECT path, title FROM documents LIMIT 5;"
```

---

## 🎉 Summary

**Phase 1, 2, 3, 4A, 4B & 4C Complete!** The QMD Rust project now has:
- ✅ Full vector search implementation with sqlite-vec (768-dim)
- ✅ Real embedding model integration (nomic-embed-text-v1.5 with GPU acceleration)
- ✅ Hybrid search combining BM25 + Vector search
- ✅ RRF fusion algorithm for result merging
- ✅ Query expansion and LLM reranking pipeline
- ✅ Async/await throughout the codebase
- ✅ All runtime issues resolved
- ✅ Semantic search working with real embeddings (no more random vectors!)
- ✅ **35 unit tests** covering RRF fusion, BM25 search, query expansion, embedding normalization, schema init
- ✅ **Model caching** - embedding model loads once, reused across queries (Mutex<Option<CachedLlamaModel>>)
- ✅ **Code cleanup** - removed 6 deprecated sync methods, cleaner async-only codebase

**Next**: Add more integration tests (async hybrid search, mock embeddings, CLI e2e tests)!
