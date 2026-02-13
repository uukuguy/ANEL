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

        # Initialize connections for each collection
        for collection in self.config.collections:
            self.get_connection(collection.name)

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
        """Initialize database schema."""
        conn.executescript("""
            -- Documents table
            CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                collection TEXT NOT NULL,
                path TEXT NOT NULL,
                title TEXT NOT NULL,
                hash TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL,
                active INTEGER NOT NULL DEFAULT 1
            );

            -- FTS5 virtual table
            CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
                filepath, title, body,
                tokenize='porter unicode61',
                content='documents',
                content_rowid='id'
            );

            -- Triggers
            CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents BEGIN
                INSERT INTO documents_fts(rowid, filepath, title, body)
                VALUES(new.id, new.collection || '/' || new.path, new.title,
                       (SELECT doc FROM content WHERE hash = new.hash));
            END;

            CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
                INSERT INTO documents_fts(documents_fts, rowid, filepath, title, body)
                VALUES('delete', old.id, old.collection || '/' || old.path, old.title, NULL);
            END;

            -- Content table
            CREATE TABLE IF NOT EXISTS content (
                hash TEXT PRIMARY KEY,
                doc TEXT NOT NULL,
                size INTEGER NOT NULL DEFAULT 0
            );

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

            -- Indexes
            CREATE INDEX IF NOT EXISTS idx_documents_collection ON documents(collection);
            CREATE INDEX IF NOT EXISTS idx_documents_hash ON documents(hash);
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
        self, query: str, options: SearchOptions
    ) -> List[SearchResult]:
        """Vector semantic search."""
        # Dispatch based on backend configuration
        backend = self.config.vector.backend

        if backend == "qmd_builtin":
            return self._vector_search_sqlite(query, options)
        elif backend == "qdrant":
            return self._vector_search_qdrant(query, options)
        elif backend == "lancedb":
            return self._vector_search_lancedb(query, options)
        else:
            # Fall back to BM25
            return self.bm25_search(query, options)

    def _vector_search_sqlite(
        self, query: str, options: SearchOptions
    ) -> List[SearchResult]:
        """Vector search using sqlite-vec."""
        # TODO: Generate query embedding and search
        # Placeholder: fall back to BM25
        return self.bm25_search(query, options)

    def _vector_search_qdrant(
        self, query: str, options: SearchOptions
    ) -> List[SearchResult]:
        """Vector search using Qdrant."""
        # TODO: Implement Qdrant backend
        # Placeholder: fall back to BM25
        return self.bm25_search(query, options)

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
        # TODO: Implement LLM reranking
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
        # TODO: Implement embedding generation
        pass

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
