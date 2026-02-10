use crate::config::{Config, BM25Backend, VectorBackend};
use crate::llm::Router;
use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Search result structure
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub path: String,
    pub collection: String,
    pub score: f32,
    pub lines: usize,
    pub title: String,
    pub hash: String,
}

/// Search options
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub limit: usize,
    pub min_score: f32,
    pub collection: Option<String>,
    pub search_all: bool,
}

/// Index statistics
#[derive(Debug, Default)]
pub struct IndexStats {
    pub collection_count: usize,
    pub document_count: usize,
    pub indexed_count: usize,
    pub pending_count: usize,
    pub collection_stats: HashMap<String, usize>,
}

/// Main Store structure
pub struct Store {
    config: Config,
    connections: HashMap<String, Connection>,
    // Vector backend will be added when lanceDB is integrated
}

impl Store {
    /// Create a new Store instance
    pub fn new(config: &Config) -> Result<Self> {
        let store = Self {
            config: config.clone(),
            connections: HashMap::new(),
        };

        // Initialize database connections for each collection
        for collection in &store.config.collections {
            store.get_connection(&collection.name)?;
        }

        Ok(store)
    }

    /// Get or create database connection for a collection
    fn get_connection(&self, collection: &str) -> Result<Connection> {
        let db_path = self.config.db_path_for(collection);

        // Create parent directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database: {}", db_path.display()))?;

        // Initialize schema
        Self::init_schema(&conn)?;

        Ok(conn)
    }

    /// Initialize database schema
    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(r#"
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

            -- FTS5 virtual table for full-text search
            CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
                filepath, title, body,
                tokenize='porter unicode61',
                content='documents',
                content_rowid='id'
            );

            -- Triggers to keep FTS index synchronized
            CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents BEGIN
                INSERT INTO documents_fts(rowid, filepath, title, body)
                VALUES(new.id, new.collection || '/' || new.path, new.title,
                       (SELECT doc FROM content WHERE hash = new.hash));
            END;

            CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
                INSERT INTO documents_fts(documents_fts, rowid, filepath, title, body)
                VALUES('delete', old.id, old.collection || '/' || old.path, old.title, NULL);
            END;

            CREATE TRIGGER IF NOT EXISTS documents_au AFTER UPDATE ON documents BEGIN
                INSERT INTO documents_fts(documents_fts, rowid, filepath, title, body)
                VALUES('delete', old.id, old.collection || '/' || old.path, old.title, NULL);
                INSERT INTO documents_fts(rowid, filepath, title, body)
                VALUES(new.id, new.collection || '/' || new.path, new.title,
                       (SELECT doc FROM content WHERE hash = new.hash));
            END;

            -- Content table (actual document content)
            CREATE TABLE IF NOT EXISTS content (
                hash TEXT PRIMARY KEY,
                doc TEXT NOT NULL,
                size INTEGER NOT NULL DEFAULT 0
            );

            -- Vector storage (sqlite-vec if available)
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

            -- Collections table
            CREATE TABLE IF NOT EXISTS collections (
                name TEXT PRIMARY KEY,
                path TEXT NOT NULL,
                pattern TEXT,
                description TEXT
            );

            -- Create indexes
            CREATE INDEX IF NOT EXISTS idx_documents_collection ON documents(collection);
            CREATE INDEX IF NOT EXISTS idx_documents_hash ON documents(hash);
        "#)?;

        Ok(())
    }

    /// BM25 full-text search
    pub fn bm25_search(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> {
        // Determine which backend to use based on configuration
        match &self.config.bm25.backend {
            BM25Backend::SqliteFts5 => self.bm25_sqlite_search(query, options),
            BM25Backend::LanceDb => unimplemented!("LanceDB FTS not yet implemented"),
        }
    }

    /// SQLite FTS5 search implementation
    fn bm25_sqlite_search(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        let collections: Vec<&str> = if options.search_all {
            self.config.collections.iter().map(|c| c.name.as_str()).collect()
        } else if let Some(ref name) = options.collection {
            vec![name.as_str()]
        } else if let Some(col) = self.config.collections.first() {
            vec![col.name.as_str()]
        } else {
            return Ok(results);
        };

        for collection in collections {
            if let Some(conn) = self.get_connection(collection).ok() {
                let query = format!("{} NOT active:0", query);

                let mut stmt = conn.prepare(
                    "SELECT rowid, bm25(documents_fts), title, path
                     FROM documents_fts
                     WHERE documents_fts MATCH ?
                     ORDER BY bm25(documents_fts)
                     LIMIT ?"
                )?;

                let rows: Vec<(i32, f64, String, String)> = stmt
                    .query_map([&query], |row| {
                        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                    })?
                    .filter_map(|r| r.ok())
                    .collect();

                for (rowid, score, title, path) in rows {
                    results.push(SearchResult {
                        path: format!("{}/{}", collection, path),
                        collection: collection.to_string(),
                        score: score as f32,
                        lines: 0, // TODO: Calculate line count
                        title,
                        hash: rowid.to_string(),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Vector search (placeholder for sqlite-vec)
    pub fn vector_search(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> {
        match &self.config.vector.backend {
            VectorBackend::QmdBuiltin => self.vector_sqlite_search(query, options),
            VectorBackend::LanceDb => unimplemented!("LanceDB vectors not yet implemented"),
        }
    }

    /// Vector search implementation using sqlite-vec
    fn vector_sqlite_search(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> {
        // TODO: Generate query embedding and search
        // This requires the LLM module to generate embeddings

        // Placeholder: fall back to BM25 for now
        self.bm25_search(query, options)
    }

    /// Hybrid search with reranking
    pub fn hybrid_search(
        &self,
        query: &str,
        options: SearchOptions,
        llm: &Router,
    ) -> Result<Vec<SearchResult>> {
        // Step 1: Query expansion using LLM
        let expanded_queries = llm.expand_query(query)?; // TODO: Implement query expansion

        // Step 2: Parallel retrieval
        let bm25_results = self.bm25_search(query, options.clone())?;
        let vector_results = self.vector_search(query, options.clone())?;

        // Step 3: RRF fusion
        let fused = Self::rrf_fusion(&[bm25_results, vector_results], None, 60);

        // Step 4: Top 30 for reranking
        let candidates: Vec<SearchResult> = fused.into_iter().take(30).collect();

        // Step 5: LLM reranking
        let reranked = llm.rerank(query, &candidates)?; // TODO: Implement reranking

        Ok(reranked)
    }

    /// RRF (Reciprocal Rank Fusion) algorithm
    fn rrf_fusion(
        result_lists: &[Vec<SearchResult>],
        weights: Option<Vec<f32>>,
        k: u32,
    ) -> Vec<SearchResult> {
        use std::collections::HashMap;

        let weights = weights.unwrap_or_else(|| vec![1.0; result_lists.len()]);
        let mut scores: HashMap<String, (f32, String, usize)> = HashMap::new();

        for (list_idx, results) in result_lists.iter().enumerate() {
            let weight = weights.get(list_idx).copied().unwrap_or(1.0);

            for (rank, result) in results.iter().enumerate() {
                let rrf_score = weight as f64 / (k + rank + 1) as f64;
                let entry = scores.entry(result.hash.clone()).or_insert((0.0, result.path.clone(), result.lines));
                entry.0 += rrf_score as f32;
            }
        }

        // Top-Rank Bonus
        let mut sorted: Vec<_> = scores.into_values().collect();
        sorted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        sorted.into_iter().take(100).enumerate().map(|(rank, (score, path, lines))| {
            let mut final_score = score;
            if rank == 0 {
                final_score += 0.05; // First place bonus
            } else if rank < 3 {
                final_score += 0.02; // Top 3 bonus
            }
            SearchResult {
                path,
                collection: String::new(),
                score: final_score,
                lines,
                title: String::new(),
                hash: String::new(),
            }
        }).collect()
    }

    /// Embed a collection
    pub fn embed_collection(&self, collection: &str, llm: &Router, force: bool) -> Result<()> {
        // TODO: Implement embedding generation
        println!("Embedding collection: {}", collection);
        Ok(())
    }

    /// Embed all collections
    pub fn embed_all_collections(&self, llm: &Router, force: bool) -> Result<()> {
        for collection in &self.config.collections {
            self.embed_collection(&collection.name, llm, force)?;
        }
        Ok(())
    }

    /// Update index
    pub fn update_index(&self) -> Result<()> {
        // TODO: Scan files and update index
        Ok(())
    }

    /// Get index statistics
    pub fn get_stats(&self) -> Result<IndexStats> {
        let mut stats = IndexStats::default();

        stats.collection_count = self.config.collections.len();

        for collection in &self.config.collections {
            if let Ok(conn) = self.get_connection(&collection.name) {
                let count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM documents WHERE active = 1",
                    [],
                    |row| row.get(0)
                ).unwrap_or(0);

                stats.document_count += count as usize;
                stats.collection_stats.insert(collection.name.clone(), count as usize);
            }
        }

        stats.indexed_count = stats.document_count;
        stats.pending_count = 0;

        Ok(stats)
    }

    /// Find stale entries
    pub fn find_stale_entries(&self, _older_than: u32) -> Result<Vec<String>> {
        // TODO: Check for files that no longer exist
        Ok(Vec::new())
    }

    /// Remove stale entries
    pub fn remove_stale_entries(&self, _entries: &[String]) -> Result<()> {
        // TODO: Remove stale entries from database
        Ok(())
    }
}
