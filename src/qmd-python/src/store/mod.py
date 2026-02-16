"""Storage backend for QMD."""

from dataclasses import dataclass
from pathlib import Path
from typing import List, Optional, Dict
import sqlite3
from collections import defaultdict


@dataclass
class SearchResult:
    """Search result."""

    path: str
    collection: str
    score: float
    lines: int
    title: str
    hash: str


@dataclass
class SearchOptions:
    """Search options."""

    limit: int = 20
    min_score: float = 0.0
    collection: Optional[str] = None
    search_all: bool = False


@dataclass
class IndexStats:
    """Index statistics."""

    collection_count: int = 0
    document_count: int = 0
    indexed_count: int = 0
    pending_count: int = 0
    collection_stats: Dict[str, int] = None

    def __post_init__(self):
        if self.collection_stats is None:
            self.collection_stats = {}


class Store:
    """Main storage class."""

    def __init__(self, config):
        """Initialize store."""
        self.config = config
        self.connections: Dict[str, sqlite3.Connection] = {}
        self._qdrant_backend = None

        # Initialize connections for each collection
        for collection in self.config.collections:
            self.get_connection(collection.name)

    @property
    def qdrant_backend(self):
        """Lazy load Qdrant backend."""
        if self._qdrant_backend is None:
            from .qdrant_backend import QdrantBackend, QdrantConfig
            qdrant_config = QdrantConfig(
                url=self.config.vector.qdrant.url,
                api_key=self.config.vector.qdrant.api_key,
                collection=self.config.vector.qdrant.collection,
                vector_size=self.config.vector.qdrant.vector_size,
            )
            self._qdrant_backend = QdrantBackend(qdrant_config)
        return self._qdrant_backend

    def get_connection(self, collection: str) -> sqlite3.Connection:
        """Get database connection for a collection."""
        if collection in self.connections:
            return self.connections[collection]

        db_path = self.config.db_path_for(collection)
        db_path.parent.mkdir(parents=True, exist_ok=True)

        conn = sqlite3.connect(str(db_path))
        self._init_schema(conn)
        self.connections[collection] = conn

        return conn

    def _init_schema(self, conn: sqlite3.Connection) -> None:
        """Initialize database schema - compatible with original qmd."""
        conn.executescript("""
            -- Content-addressable storage - source of truth for document content
            CREATE TABLE IF NOT EXISTS content (
                hash TEXT PRIMARY KEY,
                doc TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            -- Documents table - file system layer mapping virtual paths to content hashes
            CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                collection TEXT NOT NULL,
                path TEXT NOT NULL,
                title TEXT NOT NULL,
                hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL,
                active INTEGER NOT NULL DEFAULT 1,
                FOREIGN KEY (hash) REFERENCES content(hash) ON DELETE CASCADE,
                UNIQUE(collection, path)
            );

            -- Indexes
            CREATE INDEX IF NOT EXISTS idx_documents_collection ON documents(collection, active);
            CREATE INDEX IF NOT EXISTS idx_documents_hash ON documents(hash);
            CREATE INDEX IF NOT EXISTS idx_documents_path ON documents(path, active);

            -- FTS5 virtual table
            CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
                filepath, title, body,
                tokenize='porter unicode61'
            );

            -- FTS triggers - now references content table via documents.hash
            CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents
            WHEN new.active = 1
            BEGIN
                INSERT INTO documents_fts(rowid, filepath, title, body)
                SELECT
                    new.id,
                    new.collection || '/' || new.path,
                    new.title,
                    (SELECT doc FROM content WHERE hash = new.hash)
                WHERE new.active = 1;
            END;

            CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
                DELETE FROM documents_fts WHERE rowid = old.id;
            END;

            CREATE TRIGGER IF NOT EXISTS documents_au AFTER UPDATE ON documents BEGIN
                DELETE FROM documents_fts WHERE rowid = old.id AND new.active = 0;
                INSERT OR REPLACE INTO documents_fts(rowid, filepath, title, body)
                SELECT
                    new.id,
                    new.collection || '/' || new.path,
                    new.title,
                    (SELECT doc FROM content WHERE hash = new.hash)
                WHERE new.active = 1;
            END;

            -- Vectors table (sqlite-vec)
            CREATE VIRTUAL TABLE IF NOT EXISTS vectors_vec USING vec0(
                hash_seq TEXT PRIMARY KEY,
                embedding float[384] distance_metric=cosine
            );

            -- Vector metadata
            CREATE TABLE IF NOT EXISTS content_vectors (
                hash TEXT NOT NULL,
                seq INTEGER NOT NULL DEFAULT 0,
                pos INTEGER NOT NULL DEFAULT 0,
                model TEXT NOT NULL,
                embedded_at TEXT NOT NULL,
                PRIMARY KEY (hash, seq)
            );

            -- LLM response cache
            CREATE TABLE IF NOT EXISTS llm_cache (
                cache_key TEXT PRIMARY KEY,
                model TEXT NOT NULL,
                response TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT
            );
        """)

    def bm25_search(
        self, query: str, options: SearchOptions
    ) -> List[SearchResult]:
        """BM25 full-text search."""
        results: List[SearchResult] = []

        collections = self._get_collections(options)

        for collection in collections:
            conn = self.get_connection(collection)
            cursor = conn.execute(
                """
                SELECT rowid, bm25(documents_fts), title, path
                FROM documents_fts
                WHERE documents_fts MATCH ? AND active = 1
                ORDER BY bm25(documents_fts)
                LIMIT ?
                """,
                (f"{query} NOT active:0", options.limit),
            )

            for row in cursor:
                results.append(SearchResult(
                    path=f"{collection}/{row[3]}",
                    collection=collection,
                    score=row[1],
                    lines=0,
                    title=row[2],
                    hash=str(row[0]),
                ))

        return results

    def vector_search(
        self, query: str, options: SearchOptions, llm=None
    ) -> List[SearchResult]:
        """Vector semantic search."""
        # Dispatch based on backend configuration
        backend = self.config.vector.backend

        if backend == "qmd_builtin":
            return self._vector_search_sqlite(query, options, llm)
        elif backend == "qdrant":
            return self._vector_search_qdrant(query, options, llm)
        elif backend == "lancedb":
            return self._vector_search_lancedb(query, options)
        else:
            # Fall back to BM25
            return self.bm25_search(query, options)

    def _vector_search_sqlite(
        self, query: str, options: SearchOptions, llm=None
    ) -> List[SearchResult]:
        """Vector search using sqlite-vec."""
        import asyncio
        import json

        # Generate query embedding
        query_vector = None

        if llm is not None:
            try:
                if asyncio.iscoroutinefunction(llm.embed):
                    result = asyncio.run(llm.embed([query]))
                else:
                    result = llm.embed([query])
                query_vector = result.embeddings[0]
            except Exception as e:
                print(f"Failed to generate embedding: {e}")

        if query_vector is None:
            # Fall back to BM25 if embedding fails
            return self.bm25_search(query, options)

        # Search using sqlite-vec
        results = []
        collections = self._get_collections(options)

        for collection in collections:
            conn = self.get_connection(collection)

            try:
                # Convert vector to JSON for sqlite-vec
                vector_json = json.dumps(query_vector)

                cursor = conn.execute("""
                    SELECT
                        v.hash_seq,
                        v.embedding,
                        d.title,
                        d.path,
                        d.hash,
                        d.collection
                    FROM vectors_vec v
                    JOIN documents d ON v.hash_seq LIKE d.hash || '%'
                    WHERE d.active = 1
                    ORDER BY v.embedding <=> ?
                    LIMIT ?
                """, (vector_json, options.limit))

                for row in cursor:
                    # Get line count
                    lines = self._get_line_count(row[4])

                    results.append(SearchResult(
                        path=f"{row[5]}/{row[3]}",
                        collection=row[5],
                        score=1.0 / (1.0 + row[1]),  # Convert distance to score
                        lines=lines,
                        title=row[2],
                        hash=row[4],
                    ))
            except Exception as e:
                # sqlite-vec may not be available, fall back to BM25
                print(f"sqlite-vec search failed: {e}")

        return results

    def _vector_search_qdrant(
        self, query: str, options: SearchOptions, llm=None
    ) -> List[SearchResult]:
        """Vector search using Qdrant."""
        import asyncio

        # Generate query embedding
        query_vector = None

        if llm is not None:
            # Use provided LLM for embedding
            try:
                # Check if llm has sync or async embed method
                if hasattr(llm, 'embed'):
                    # Try sync first
                    try:
                        result = llm.embed([query])
                        query_vector = result.embeddings[0]
                    except TypeError:
                        # Try async
                        loop = asyncio.get_event_loop()
                        if loop.is_running():
                            # If already in async context, create new loop
                            import concurrent.futures
                            with concurrent.futures.ThreadPoolExecutor() as executor:
                                future = executor.submit(
                                    asyncio.run, llm.embed([query])
                                )
                                result = future.result()
                                query_vector = result.embeddings[0]
                        else:
                            result = loop.run_until_complete(llm.embed([query]))
                            query_vector = result.embeddings[0]
            except Exception as e:
                print(f"Failed to generate embedding: {e}")

        if query_vector is None:
            # Fall back to BM25 if embedding fails
            return self.bm25_search(query, options)

        # Search Qdrant
        try:
            results = self.qdrant_backend.search(
                query_vector=query_vector,
                limit=options.limit,
            )

            # Convert to SearchResult
            search_results = []
            for r in results:
                # Get line count from SQLite
                lines = self._get_line_count(r["hash"])

                search_results.append(SearchResult(
                    path=r["path"],
                    collection=r.get("collection", ""),
                    score=r["score"],
                    lines=lines,
                    title=r.get("title", ""),
                    hash=r.get("hash", ""),
                ))

            return search_results
        except Exception as e:
            print(f"Qdrant search failed: {e}")
            return self.bm25_search(query, options)

    def _get_line_count(self, hash_value: str) -> int:
        """Get line count for a document by hash."""
        # Query all collections for the document
        for collection in self._get_collections(SearchOptions()):
            conn = self.get_connection(collection)
            cursor = conn.execute(
                "SELECT doc FROM content WHERE hash = ?",
                (hash_value,),
            )
            row = cursor.fetchone()
            if row:
                return len(row[0].split('\n'))
        return 0

    def _vector_search_lancedb(
        self, query: str, options: SearchOptions
    ) -> List[SearchResult]:
        """Vector search using LanceDB."""
        # TODO: Implement LanceDB backend
        # Placeholder: fall back to BM25
        return self.bm25_search(query, options)

    def hybrid_search(
        self, query: str, options: SearchOptions, llm=None
    ) -> List[SearchResult]:
        """Hybrid search with reranking."""
        # Query expansion
        expanded = self._expand_query(query)

        # Parallel retrieval
        bm25_results = self.bm25_search(query, options)
        vector_results = self.vector_search(query, options)

        # RRF fusion
        fused = self._rrf_fusion([bm25_results, vector_results], k=60)

        # Top 30 for reranking
        candidates = fused[:30]

        # LLM reranking
        if llm:
            candidates = self._rerank(query, candidates, llm)

        return candidates

    def _get_collections(self, options: SearchOptions) -> List[str]:
        """Get collections to search."""
        if options.search_all:
            return [c.name for c in self.config.collections]
        elif options.collection:
            return [options.collection]
        elif self.config.collections:
            return [self.config.collections[0].name]
        return []

    def _expand_query(self, query: str) -> List[str]:
        """Expand query using LLM."""
        # TODO: Implement query expansion
        return [query]

    def _rerank(
        self, query: str, candidates: List[SearchResult], llm
    ) -> List[SearchResult]:
        """Rerank candidates using LLM."""
        import asyncio

        if not candidates:
            return candidates

        # Get document content for reranking
        docs = []
        for result in candidates:
            # Get content from SQLite
            for collection in self._get_collections(SearchOptions()):
                conn = self.get_connection(collection)
                cursor = conn.execute(
                    "SELECT doc FROM content WHERE hash = ?",
                    (result.hash,),
                )
                row = cursor.fetchone()
                if row:
                    docs.append(row[0][:1000])  # Limit content length
                    break
            else:
                docs.append("")

        # Get rerank scores from LLM
        try:
            if asyncio.iscoroutinefunction(llm.rerank):
                scores = asyncio.run(llm.rerank(query, docs))
            else:
                scores = llm.rerank(query, docs)

            # Reorder candidates by score
            scored_candidates = list(zip(candidates, scores))
            scored_candidates.sort(key=lambda x: x[1], reverse=True)

            # Update scores
            reranked = []
            for result, score in scored_candidates:
                result.score = score
                reranked.append(result)

            return reranked
        except Exception as e:
            print(f"Reranking failed: {e}")
            return candidates

    def _rrf_fusion(
        self, result_lists: List[List[SearchResult]], weights: List[float] = None, k: int = 60
    ) -> List[SearchResult]:
        """Reciprocal Rank Fusion."""
        from collections import defaultdict

        weights = weights or [1.0] * len(result_lists)
        scores: Dict[str, tuple[float, str, int]] = defaultdict(lambda: (0.0, "", 0))

        for list_idx, results in enumerate(result_lists):
            weight = weights[list_idx] if list_idx < len(weights) else 1.0

            for rank, result in enumerate(results):
                rrf_score = weight / (k + rank + 1)
                key = result.hash
                current = scores[key]
                scores[key] = (
                    current[0] + rrf_score,
                    result.path,
                    result.lines,
                )

        # Sort and apply Top-Rank Bonus
        sorted_results = sorted(
            scores.values(),
            key=lambda x: x[0],
            reverse=True
        )

        final: List[SearchResult] = []
        for rank, (score, path, lines) in enumerate(sorted_results):
            final_score = score
            if rank == 0:
                final_score += 0.05
            elif rank < 3:
                final_score += 0.02

            final.append(SearchResult(
                path=path,
                collection="",
                score=final_score,
                lines=lines,
                title="",
                hash="",
            ))

        return final

    def get_stats(self) -> IndexStats:
        """Get index statistics."""
        stats = IndexStats()
        stats.collection_count = len(self.config.collections)

        for collection in self.config.collections:
            conn = self.get_connection(collection.name)
            cursor = conn.execute(
                "SELECT COUNT(*) FROM documents WHERE active = 1"
            )
            count = cursor.fetchone()[0]
            stats.document_count += count
            stats.collection_stats[collection.name] = count

        stats.indexed_count = stats.document_count
        return stats

    def embed_collection(self, collection: str, llm, force: bool = False) -> None:
        """Generate embeddings for a collection."""
        import asyncio
        from datetime import datetime

        conn = self.get_connection(collection)

        # Get documents that need embedding
        if force:
            cursor = conn.execute(
                "SELECT id, hash, title FROM documents WHERE active = 1",
            )
        else:
            cursor = conn.execute("""
                SELECT d.id, d.hash, d.title
                FROM documents d
                LEFT JOIN content_vectors cv ON d.hash = cv.hash
                WHERE d.active = 1 AND cv.hash IS NULL
            """)

        documents = list(cursor.fetchall())

        if not documents:
            print(f"  No documents need embedding")
            return

        print(f"  Processing {len(documents)} documents...")

        # Process documents in batches
        batch_size = 10
        for i in range(0, len(documents), batch_size):
            batch = documents[i:i + batch_size]

            # Get content for batch
            docs_content = []
            doc_hashes = [doc[1] for doc in batch]

            for doc_hash in doc_hashes:
                cursor = conn.execute(
                    "SELECT doc FROM content WHERE hash = ?",
                    (doc_hash,),
                )
                row = cursor.fetchone()
                docs_content.append((doc_hash, row[0] if row else ""))

            # Chunk and embed
            all_chunks = []
            chunk_metadata = []

            for doc_hash, content in docs_content:
                if not content:
                    continue
                # Simple chunking by paragraphs
                chunks = content.split("\n\n")
                for chunk_idx, chunk in enumerate(chunks):
                    if len(chunk) < 10:
                        continue
                    all_chunks.append(chunk)
                    chunk_metadata.append({
                        "hash": doc_hash,
                        "seq": chunk_idx,
                        "pos": 0,
                    })

            if not all_chunks:
                continue

            # Generate embeddings
            try:
                if asyncio.iscoroutinefunction(llm.embed):
                    result = asyncio.run(llm.embed(all_chunks))
                else:
                    result = llm.embed(all_chunks)

                embeddings = result.embeddings

                # Upsert to Qdrant if configured
                if self.config.vector.backend == "qdrant":
                    vectors_to_upsert = []
                    for meta, emb in zip(chunk_metadata, embeddings):
                        vectors_to_upsert.append({
                            "id": int(meta["hash"], 36) if isinstance(meta["hash"], str) else meta["hash"],
                            "vector": emb,
                            "path": "",
                            "title": "",
                            "body": all_chunks[chunk_metadata.index(meta)],
                            "hash": meta["hash"],
                            "collection": collection,
                        })

                    self.qdrant_backend.upsert_vectors(vectors_to_upsert)

                # Store in SQLite
                now = datetime.now().isoformat()
                for meta, emb in zip(chunk_metadata, embeddings):
                    # Store embedding in content_vectors table
                    # Note: sqlite-vec stores as binary blob
                    import json
                    emb_json = json.dumps(emb)

                    conn.execute("""
                        INSERT OR REPLACE INTO content_vectors (hash, seq, pos, model, embedded_at)
                        VALUES (?, ?, ?, ?, ?)
                    """, (meta["hash"], meta["seq"], meta["pos"], result.model, now))

                conn.commit()

            except Exception as e:
                print(f"  Error generating embeddings: {e}")

        print(f"  Done processing {len(documents)} documents")

    def embed_all_collections(self, llm, force: bool = False) -> None:
        """Generate embeddings for all collections."""
        for collection in self.config.collections:
            self.embed_collection(collection.name, llm, force)

    def update_index(self) -> None:
        """Update index."""
        pass

    def find_stale_entries(self, older_than: int = 30) -> List[str]:
        """Find stale entries."""
        return []

    def remove_stale_entries(self, entries: List[str]) -> None:
        """Remove stale entries."""
        pass
