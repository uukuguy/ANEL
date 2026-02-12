"""Store module for QMD - database operations and search"""

import json
import os
import sqlite3
import hashlib
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Optional


@dataclass
class SearchResult:
    """Represents a search result"""
    doc_id: str
    path: str
    collection: str
    score: float
    lines: int
    title: str
    hash: str


@dataclass
class SearchOptions:
    """Search options"""
    limit: int = 20
    min_score: float = 0.0
    collection: Optional[str] = None
    all: bool = False


@dataclass
class IndexStats:
    """Index statistics"""
    collection_count: int
    document_count: int
    indexed_count: int
    pending_count: int
    chunk_count: int
    collection_stats: dict


class Store:
    """Document store with SQLite FTS5 and vector support"""

    def __init__(self, db_path: str):
        self.db_path = db_path
        Path(db_path).parent.mkdir(parents=True, exist_ok=True)
        self.conn = sqlite3.connect(db_path)
        self.conn.row_factory = sqlite3.Row
        self._init_db()

    def close(self):
        """Close the database connection"""
        self.conn.close()

    def _init_db(self):
        """Initialize database schema"""
        schema = """
        -- Documents table
        CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            collection TEXT NOT NULL,
            path TEXT NOT NULL,
            title TEXT NOT NULL,
            hash TEXT NOT NULL UNIQUE,
            doc TEXT NOT NULL,
            created_at TEXT NOT NULL,
            modified_at TEXT NOT NULL,
            active INTEGER NOT NULL DEFAULT 1
        );

        -- FTS5 full-text search
        CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
            filepath, title, body,
            tokenize='porter unicode61'
        );

        -- Collections
        CREATE TABLE IF NOT EXISTS collections (
            name TEXT PRIMARY KEY,
            path TEXT NOT NULL,
            pattern TEXT,
            description TEXT
        );

        -- Path contexts (relevance hints)
        CREATE TABLE IF NOT EXISTS path_contexts (
            path TEXT PRIMARY KEY,
            description TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- LLM response cache
        CREATE TABLE IF NOT EXISTS llm_cache (
            cache_key TEXT PRIMARY KEY,
            model TEXT NOT NULL,
            response TEXT NOT NULL,
            created_at TEXT NOT NULL,
            expires_at TEXT
        );

        -- Content vectors
        CREATE TABLE IF NOT EXISTS content_vectors (
            hash TEXT NOT NULL,
            seq INTEGER NOT NULL DEFAULT 0,
            embedding TEXT NOT NULL,
            pos INTEGER NOT NULL DEFAULT 0,
            model TEXT NOT NULL,
            embedded_at TEXT NOT NULL,
            PRIMARY KEY (hash, seq)
        );

        -- Indexes
        CREATE INDEX IF NOT EXISTS idx_documents_collection ON documents(collection);
        CREATE INDEX IF NOT EXISTS idx_documents_hash ON documents(hash);
        CREATE INDEX IF NOT EXISTS idx_documents_active ON documents(active);
        """
        self.conn.executescript(schema)
        self.conn.commit()

    def add_document(self, collection: str, path: str, title: str, content: str) -> str:
        """Add a document to the store"""
        doc_hash = self._hash_content(content)
        now = datetime.utcnow().isoformat()

        self.conn.execute(
            """
            INSERT INTO documents (collection, path, title, hash, doc, created_at, modified_at, active)
            VALUES (?, ?, ?, ?, ?, ?, ?, 1)
            ON CONFLICT(hash) DO UPDATE SET
                path = excluded.path,
                title = excluded.title,
                modified_at = excluded.modified_at,
                active = 1
            """,
            (collection, path, title, doc_hash, content, now, now)
        )

        # Update FTS index
        self.conn.execute(
            """
            INSERT INTO documents_fts (filepath, title, body)
            VALUES (?, ?, ?)
            ON CONFLICT(rowid) DO UPDATE SET
                filepath = excluded.filepath,
                title = excluded.title,
                body = excluded.body
            """,
            (path, title, content)
        )

        self.conn.commit()
        return doc_hash

    def remove_document(self, doc_hash: str):
        """Remove a document from the store"""
        self.conn.execute("UPDATE documents SET active = 0 WHERE hash = ?", (doc_hash,))
        self.conn.commit()

    def bm25_search(self, query: str, opts: SearchOptions) -> list[SearchResult]:
        """BM25 full-text search"""
        # Escape special FTS5 characters
        query = self._escape_fts5(query)

        collection_clause = ""
        params = [query, opts.limit]
        if not opts.all and opts.collection:
            collection_clause = "AND d.collection = ?"
            params.insert(1, opts.collection)

        sql = f"""
            SELECT d.collection, d.path, d.title, d.hash, bm25(documents_fts), d.doc
            FROM documents_fts f
            JOIN documents d ON d.path = f.filepath
            WHERE documents_fts MATCH ? {collection_clause} AND d.active = 1
            ORDER BY bm25(documents_fts)
            LIMIT ?
        """

        cursor = self.conn.execute(sql, params)
        results = []
        for row in cursor.fetchall():
            r = SearchResult(
                doc_id=f"{row['collection']}:{row['path']}",
                path=row['path'],
                collection=row['collection'],
                score=row[4],
                lines=row['doc'].count('\n') + 1,
                title=row['title'],
                hash=row['hash'],
            )
            results.append(r)

        return results

    def get_document(self, path: str) -> tuple[SearchResult, str]:
        """Get a document by path"""
        cursor = self.conn.execute(
            "SELECT collection, path, title, hash, doc FROM documents WHERE path = ? AND active = 1",
            (path,)
        )
        row = cursor.fetchone()
        if not row:
            raise ValueError("Document not found")

        r = SearchResult(
            doc_id=f"{row['collection']}:{row['path']}",
            path=row['path'],
            collection=row['collection'],
            score=0.0,
            lines=row['doc'].count('\n') + 1,
            title=row['title'],
            hash=row['hash'],
        )
        return r, row['doc']

    def get_stats(self) -> IndexStats:
        """Get index statistics"""
        stats = IndexStats(
            collection_stats={},
            collection_count=0,
            document_count=0,
            indexed_count=0,
            pending_count=0,
            chunk_count=0,
        )

        cursor = self.conn.execute("SELECT COUNT(DISTINCT collection) FROM documents WHERE active = 1")
        stats.collection_count = cursor.fetchone()[0]

        cursor = self.conn.execute("SELECT COUNT(*) FROM documents WHERE active = 1")
        stats.document_count = cursor.fetchone()[0]

        cursor = self.conn.execute("SELECT COUNT(*) FROM documents_fts")
        stats.indexed_count = cursor.fetchone()[0]

        cursor = self.conn.execute("SELECT COUNT(*) FROM content_vectors")
        stats.chunk_count = cursor.fetchone()[0]

        cursor = self.conn.execute("SELECT collection, COUNT(*) as count FROM documents WHERE active = 1 GROUP BY collection")
        for row in cursor.fetchall():
            stats.collection_stats[row['collection']] = row['count']

        return stats

    def vector_search(self, query_vector: list[float], opts: SearchOptions) -> list[SearchResult]:
        """Vector similarity search"""
        # Normalize query vector
        query_vector = self._normalize_vector(query_vector)

        cursor = self.conn.execute(
            "SELECT d.collection, d.path, d.title, d.hash, cv.embedding FROM content_vectors cv JOIN documents d ON d.hash = cv.hash WHERE d.active = 1"
        )

        results = []
        for row in cursor.fetchall():
            embedding = json.loads(row['embedding'])
            score = self._cosine_similarity(query_vector, embedding)
            if score >= opts.min_score:
                results.append(SearchResult(
                    doc_id=f"{row['collection']}:{row['path']}",
                    path=row['path'],
                    collection=row['collection'],
                    score=score,
                    lines=0,
                    title=row['title'],
                    hash=row['hash'],
                ))

        results.sort(key=lambda x: x.score, reverse=True)
        return results[:opts.limit]

    def hybrid_search(self, bm25_results: list[SearchResult], vector_results: list[SearchResult], k: int = 60) -> list[SearchResult]:
        """Hybrid search using RRF fusion"""
        doc_map = {}

        # Add BM25 results
        for i, r in enumerate(bm25_results):
            key = r.doc_id
            if key not in doc_map:
                doc_map[key] = r
            rrf_score = 1.0 / (k + i + 1)
            doc_map[key].score += rrf_score

        # Add vector results
        for i, r in enumerate(vector_results):
            key = r.doc_id
            rrf_score = 1.0 / (k + i + 1)
            if key in doc_map:
                doc_map[key].score += rrf_score
            else:
                r.score = rrf_score
                doc_map[key] = r

        results = list(doc_map.values())
        results.sort(key=lambda x: x.score, reverse=True)
        return results

    def store_embeddings(self, doc_hash: str, embeddings: list[list[float]], model: str):
        """Store embeddings in database"""
        now = datetime.utcnow().isoformat()
        for seq, emb in enumerate(embeddings):
            self.conn.execute(
                "INSERT OR REPLACE INTO content_vectors (hash, seq, embedding, model, embedded_at) VALUES (?, ?, ?, ?, ?)",
                (doc_hash, seq, json.dumps(emb), model, now)
            )
        self.conn.commit()

    @staticmethod
    def _hash_content(content: str) -> str:
        """Compute SHA256 hash of content"""
        return hashlib.sha256(content.encode()).hexdigest()

    @staticmethod
    def _escape_fts5(query: str) -> str:
        """Escape special FTS5 characters"""
        # Note: This is a simplified version
        return query.replace('"', '""')

    @staticmethod
    def _normalize_vector(v: list[float]) -> list[float]:
        """Normalize vector to unit length"""
        import math
        norm = math.sqrt(sum(x * x for x in v))
        if norm == 0:
            return v
        return [x / norm for x in v]

    @staticmethod
    def _cosine_similarity(a: list[float], b: list[float]) -> float:
        """Compute cosine similarity between two vectors"""
        import math
        if len(a) != len(b):
            return 0.0

        dot = sum(x * y for x, y in zip(a, b))
        norm_a = math.sqrt(sum(x * x for x in a))
        norm_b = math.sqrt(sum(x * x for x in b))

        if norm_a == 0 or norm_b == 0:
            return 0.0

        return dot / (norm_a * norm_b)
