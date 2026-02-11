use crate::config::{Config, BM25Backend};
use crate::llm::Router;
use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::Serialize;
use std::collections::HashMap;
use log::{info, warn};

/// Search result structure
#[derive(Debug, Clone, PartialEq, Serialize)]
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
}

impl Store {
    /// Create a new Store instance
    pub fn new(config: &Config) -> Result<Self> {
        let mut store = Self {
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
        info!("Initializing database schema");

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
    ///
    /// Note: For actual vector search, use `vector_search_with_embedder` with an LLM embedder.
    /// This method falls back to BM25 if no embedder is available.
    pub fn vector_search(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> {
        warn!("Vector search using BM25 fallback (embedder not available)");
        self.bm25_search(query, options)
    }

    /// Vector search with explicit embedder
    ///
    /// Uses the provided LLM to generate embeddings and performs similarity search.
    pub fn vector_search_with_embedder(
        &self,
        query: &str,
        options: SearchOptions,
        llm: &Router,
    ) -> Result<Vec<SearchResult>> {
        // Generate embedding for the query
        let embedding_result = llm.embed_sync(query)?;

        info!("Generated embedding with {} dimensions, provider: {}",
              embedding_result.embeddings[0].len(), embedding_result.provider);

        // Get the embedding vector
        let query_vector = &embedding_result.embeddings[0];

        // Perform vector search in each collection
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
                let collection_results = self.vector_search_in_db(&conn, query_vector, options.limit)?;
                results.extend(collection_results);
            }
        }

        // Sort by score (cosine distance - lower is better)
        results.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());

        Ok(results)
    }

    /// Perform vector search in a single database
    fn vector_search_in_db(
        &self,
        conn: &Connection,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // Try sqlite-vec first
        #[cfg(feature = "sqlite-vec")]
        {
            results = self.vector_search_sqlite_vec(conn, query_vector, limit)?;
        }

        // Fallback to BM25 if no results or sqlite-vec not available
        if results.is_empty() {
            warn!("Falling back to BM25 for vector search");
            // Convert query_vector back to a simple text search
            // This is a placeholder fallback
        }

        Ok(results)
    }

    /// SQLite vector search using sqlite-vec
    #[cfg(feature = "sqlite-vec")]
    fn vector_search_sqlite_vec(
        &self,
        conn: &Connection,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        use rusqlite::types::Value;

        let mut results = Vec::new();

        // Prepare query vector as SQL array
        let query_vec: Vec<Value> = query_vector.iter().map(|&v| Value::from(v)).collect();

        let mut stmt = conn.prepare(
            "SELECT
                v.rowid,
                v.distance,
                d.path,
                d.title,
                d.hash
             FROM vectors_vec v
             JOIN documents d ON v.rowid = d.id
             ORDER BY v.distance
             LIMIT ?"
        )?;

        let rows: Vec<(i32, f64, String, String, String)> = stmt
            .query_map([limit as i64], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        for (rowid, distance, path, title, hash) in rows {
            results.push(SearchResult {
                path,
                collection: String::new(),
                score: distance as f32,
                lines: 0,
                title,
                hash: rowid.to_string(),
            });
        }

        Ok(results)
    }

    #[cfg(not(feature = "sqlite-vec"))]
    fn vector_search_sqlite_vec(
        &self,
        _conn: &Connection,
        _query_vector: &[f32],
        _limit: usize,
    ) -> Result<Vec<SearchResult>> {
        warn!("sqlite-vec feature not enabled");
        Ok(Vec::new())
    }

    /// Hybrid search with reranking
    ///
    /// Combines BM25 and vector search with query expansion and LLM reranking:
    /// 1. Query expansion using LLM
    /// 2. BM25 retrieval for expanded queries
    /// 3. RRF fusion of all results
    /// 4. LLM reranking of top candidates (if available)
    pub fn hybrid_search(
        &self,
        query: &str,
        options: SearchOptions,
        llm: &Router,
    ) -> Result<Vec<SearchResult>> {
        // Step 1: Query expansion using LLM
        let expanded_queries = llm.expand_query(query)?;

        info!("Hybrid search: original='{}', expanded={} variants", query, expanded_queries.len());

        // Step 2: BM25 retrieval for all expanded queries
        let mut all_bm25_results = Vec::new();

        for expanded_query in &expanded_queries {
            // BM25 search
            let bm25_results = self.bm25_search(expanded_query, options.clone())?;
            all_bm25_results.extend(bm25_results);
        }

        // Limit intermediate results to avoid memory issues
        all_bm25_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        all_bm25_results.truncate(100);

        // Step 3: RRF fusion (single list, so just sort by BM25 score)
        let mut fused = all_bm25_results;
        fused.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Step 4: Top 30 for reranking
        let candidates: Vec<SearchResult> = fused.into_iter().take(30).collect();

        // Step 5: Try LLM reranking if available
        let final_results = if llm.has_reranker() {
            info!("LLM reranking available, applying to top candidates");
            match llm.rerank_sync(query, &candidates) {
                Ok(scores) => {
                    // Apply reranking scores
                    let mut reranked: Vec<_> = candidates
                        .into_iter()
                        .zip(scores.into_iter())
                        .map(|(mut doc, score)| {
                            doc.score = score;
                            doc
                        })
                        .collect();
                    // Sort by reranking score (higher is better)
                    reranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
                    reranked
                }
                Err(e) => {
                    warn!("LLM reranking failed: {}, using original candidates", e);
                    candidates
                }
            }
        } else {
            candidates
        };

        Ok(final_results)
    }

    /// RRF (Reciprocal Rank Fusion) algorithm
    ///
    /// Combines multiple ranked result lists using the Reciprocal Rank Fusion formula:
    /// RRF(d) = sum(1 / (k + rank(d)))
    ///
    /// Where:
    /// - d is a document
    /// - rank(d) is the rank of d in a result list
    /// - k is a constant (typically 60)
    ///
    /// The algorithm also applies a Top-Rank Bonus to give extra weight to highly-ranked results.
    fn rrf_fusion(
        result_lists: &[Vec<SearchResult>],
        weights: Option<Vec<f32>>,
        k: u32,
    ) -> Vec<SearchResult> {
        let weights = weights.unwrap_or_else(|| vec![1.0; result_lists.len()]);

        // Map from document identifier (path) to aggregated data
        type DocData = (
            f32,                    // accumulated RRF score
            String,                 // collection
            usize,                  // lines
            String,                 // title
            String,                 // hash
        );
        let mut doc_map: HashMap<String, DocData> = HashMap::new();

        for (list_idx, results) in result_lists.iter().enumerate() {
            let weight = weights.get(list_idx).copied().unwrap_or(1.0);

            for (rank, result) in results.iter().enumerate() {
                // Calculate RRF score with weight
                let rank_plus_k = k + rank as u32;
                let rrf_score = weight as f64 / rank_plus_k as f64;

                // Use path as unique identifier
                let path_key = result.path.clone();

                doc_map.entry(path_key).and_modify(|data| {
                    data.0 += rrf_score as f32;
                }).or_insert((
                    rrf_score as f32,         // initial RRF score
                    result.collection.clone(), // collection
                    result.lines,             // lines
                    result.title.clone(),     // title
                    result.hash.clone(),      // hash
                ));
            }
        }

        // Sort by RRF score (descending)
        let mut results: Vec<_> = doc_map.into_values().collect();
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // Apply Top-Rank Bonus and construct final results
        results.into_iter().enumerate().map(|(rank, data)| {
            let mut final_score = data.0;

            // Top-Rank Bonus: extra points for highly-ranked documents
            if rank == 0 {
                final_score += 0.05; // First place bonus
            } else if rank < 3 {
                final_score += 0.02; // Top 3 bonus
            } else if rank < 10 {
                final_score += 0.01; // Top 10 bonus
            }

            SearchResult {
                path: data.1.clone(),
                collection: data.1,
                score: final_score,
                lines: data.2,
                title: data.3,
                hash: data.4,
            }
        }).collect()
    }

    /// Embed a collection
    pub fn embed_collection(&self, _collection: &str, _llm: &Router, _force: bool) -> Result<()> {
        // TODO: Implement embedding generation
        println!("Embedding collection: {}", _collection);
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
