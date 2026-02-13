pub mod chunker;
pub mod lance_backend;

#[cfg(feature = "lancedb")]
use lance_backend::LanceDbBackend;

use crate::config::{Config, BM25Backend, VectorBackend};
use crate::llm::Router;
use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Mutex;
use log::{info, warn};

/// Search result structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub docid: String,
    pub path: String,
    pub collection: String,
    pub score: f32,
    pub lines: usize,
    pub title: String,
    pub hash: String,
    pub query: Option<String>,
}

/// Generate a stable document ID from collection and path
pub fn make_docid(collection: &str, path: &str) -> String {
    format!("{}:{}", collection, path)
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
    pub chunk_count: usize,
    pub collection_stats: HashMap<String, usize>,
}

/// Main Store structure
pub struct Store {
    config: Config,
    connections: HashMap<String, Connection>,
    #[cfg(feature = "lancedb")]
    lance_backend: Option<Mutex<LanceDbBackend>>,
}

impl Store {
    /// Create a new Store instance
    pub fn new(config: &Config) -> Result<Self> {
        // Initialize sqlite-vec extension if available
        Self::init_sqlite_vec()?;

        // Initialize LanceDB backend if configured (either BM25 or vector)
        #[cfg(feature = "lancedb")]
        let lance_backend = if matches!(config.bm25.backend, BM25Backend::LanceDb)
            || matches!(config.vector.backend, VectorBackend::LanceDb)
        {
            let embedding_dim = 384; // Default, should match embedder config
            let db_path = config.cache_path.clone();
            let mut backend = LanceDbBackend::new(db_path, embedding_dim);

            // Use tokio runtime to connect
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async { backend.connect().await })?;

            Some(Mutex::new(backend))
        } else {
            None
        };

        #[cfg(not(feature = "lancedb"))]
        let _ = config; // Suppress unused warning

        let store = Self {
            config: config.clone(),
            connections: HashMap::new(),
            #[cfg(feature = "lancedb")]
            lance_backend,
        };

        // Initialize database connections for each collection
        for collection in &store.config.collections {
            store.get_connection(&collection.name)?;
        }

        Ok(store)
    }

    /// Get collections from config
    pub fn get_collections(&self) -> &[crate::config::CollectionConfig] {
        &self.config.collections
    }

    /// Initialize sqlite-vec extension
    fn init_sqlite_vec() -> Result<()> {
        #[cfg(feature = "sqlite-vec")]
        {
            use sqlite_vec::sqlite3_vec_init;
            use std::os::raw::c_void;

            unsafe {
                rusqlite::ffi::sqlite3_auto_extension(
                    Some(std::mem::transmute(sqlite3_vec_init as *const c_void)),
                );
            }
            info!("sqlite-vec extension loaded");
        }
        #[cfg(not(feature = "sqlite-vec"))]
        {
            warn!("sqlite-vec feature not enabled");
        }
        Ok(())
    }

    /// Get or create database connection for a collection
    pub fn get_connection(&self, collection: &str) -> Result<Connection> {
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

        // Create tables one by one with error handling
        conn.execute_batch(r#"
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
        "#)?;

        conn.execute_batch(r#"
            -- FTS5 virtual table for full-text search
            CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
                filepath, title, body,
                tokenize='porter unicode61'
            );
        "#)?;

        conn.execute_batch(r#"
            -- Triggers to keep FTS index synchronized
            CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents BEGIN
                INSERT INTO documents_fts(rowid, filepath, title, body)
                VALUES(new.id, new.collection || '/' || new.path, new.title, new.doc);
            END;

            CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
                INSERT INTO documents_fts(documents_fts, rowid, filepath, title, body)
                VALUES('delete', old.id, old.collection || '/' || old.path, old.title, NULL);
            END;

            CREATE TRIGGER IF NOT EXISTS documents_au AFTER UPDATE ON documents BEGIN
                INSERT INTO documents_fts(documents_fts, rowid, filepath, title, body)
                VALUES('delete', old.id, old.collection || '/' || old.path, old.title, NULL);
                INSERT INTO documents_fts(rowid, filepath, title, body)
                VALUES(new.id, new.collection || '/' || new.path, new.title, new.doc);
            END;
        "#)?;

        // Vector storage — requires sqlite-vec extension; skip gracefully if unavailable
        if let Err(e) = conn.execute_batch(r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS vectors_vec USING vec0(
                hash_seq TEXT PRIMARY KEY,
                embedding float[768] distance_metric=cosine
            );
        "#) {
            warn!("Could not create vectors_vec table (sqlite-vec may not be loaded): {}", e);
        }

        conn.execute_batch(r#"
            -- Vector metadata
            CREATE TABLE IF NOT EXISTS content_vectors (
                hash TEXT NOT NULL,
                seq INTEGER NOT NULL DEFAULT 0,
                pos INTEGER NOT NULL DEFAULT 0,
                model TEXT NOT NULL,
                embedded_at TEXT NOT NULL,
                PRIMARY KEY (hash, seq)
            );
        "#)?;

        conn.execute_batch(r#"
            -- Collections table
            CREATE TABLE IF NOT EXISTS collections (
                name TEXT PRIMARY KEY,
                path TEXT NOT NULL,
                pattern TEXT,
                description TEXT
            );
        "#)?;

        conn.execute_batch(r#"
            -- Create indexes
            CREATE INDEX IF NOT EXISTS idx_documents_collection ON documents(collection);
            CREATE INDEX IF NOT EXISTS idx_documents_hash ON documents(hash);
        "#)?;

        conn.execute_batch(r#"
            -- Path contexts for relevance hints
            CREATE TABLE IF NOT EXISTS path_contexts (
                path TEXT PRIMARY KEY,
                description TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
        "#)?;

        conn.execute_batch(r#"
            -- LLM response cache
            CREATE TABLE IF NOT EXISTS llm_cache (
                cache_key TEXT PRIMARY KEY,
                model TEXT NOT NULL,
                response TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT
            );
        "#)?;

        Ok(())
    }

    /// BM25 full-text search
    pub fn bm25_search(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> {
        // Determine which backend to use based on configuration
        match &self.config.bm25.backend {
            BM25Backend::SqliteFts5 => self.bm25_sqlite_search(query, options),
            #[cfg(feature = "lancedb")]
            BM25Backend::LanceDb => self.bm25_lance_search(query, options),
            #[cfg(not(feature = "lancedb"))]
            BM25Backend::LanceDb => {
                anyhow::bail!("LanceDB backend not enabled. Build with --features lancedb")
            }
        }
    }

    /// LanceDB FTS search implementation
    #[cfg(feature = "lancedb")]
    fn bm25_lance_search(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> {
        use futures::executor::block_on;

        let mut all_results = Vec::new();

        let collections: Vec<&str> = if options.search_all {
            self.config.collections.iter().map(|c| c.name.as_str()).collect()
        } else if let Some(ref name) = options.collection {
            vec![name.as_str()]
        } else if let Some(col) = self.config.collections.first() {
            vec![col.name.as_str()]
        } else {
            return Ok(all_results);
        };

        let limit = options.limit;

        for collection in collections {
            if let Some(ref backend_mutex) = self.lance_backend {
                if let Ok(backend) = backend_mutex.lock() {
                    let rt = tokio::runtime::Runtime::new()?;
                    let results = rt.block_on(async {
                        backend.fts_search(collection, query, limit).await
                    });
                    if let Ok(mut results) = results {
                        all_results.append(&mut results);
                    }
                }
            }
        }

        Ok(all_results)
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

        let limit = options.limit;

        for collection in collections {
            if let Ok(conn) = self.get_connection(collection) {
                let fts_query = query.to_string();

                let mut stmt = conn.prepare(
                    "SELECT rowid, bm25(documents_fts), title, filepath
                     FROM documents_fts
                     WHERE documents_fts MATCH ?
                     ORDER BY bm25(documents_fts)
                     LIMIT ?"
                )?;

                let rows: Vec<(i32, f64, String, String)> = stmt
                    .query_map((&fts_query, limit as i64), |row| {
                        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                    })?
                    .filter_map(|r| r.ok())
                    .collect();

                for (rowid, score, title, filepath) in rows {
                    let docid = make_docid(collection, &filepath);
                    // Calculate line count by reading the file
                    let lines = std::fs::read_to_string(&filepath)
                        .map(|content| content.lines().count())
                        .unwrap_or(0);
                    results.push(SearchResult {
                        docid,
                        path: filepath,
                        collection: collection.to_string(),
                        score: score as f32,
                        lines,
                        title,
                        hash: rowid.to_string(),
                        query: Some(query.to_string()),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Vector search with explicit embedder (async version)
    ///
    /// Uses the provided LLM to generate embeddings and performs similarity search.
    pub async fn vector_search_with_embedder_async(
        &self,
        query: &str,
        options: SearchOptions,
        llm: &Router,
    ) -> Result<Vec<SearchResult>> {
        // Generate embedding for the query (async)
        let embedding_result = llm.embed(&[query]).await?;

        info!("Generated embedding with {} dimensions, provider: {}",
              embedding_result.embeddings[0].len(), embedding_result.provider);

        // Get the embedding vector
        let query_vector = &embedding_result.embeddings[0];

        self.vector_search_with_embedding(query_vector, options)
    }

    /// Vector search with a pre-computed embedding vector (sync)
    ///
    /// Performs similarity search using a pre-computed embedding vector.
    /// Useful when the embedding has already been generated externally.
    pub fn vector_search_with_embedding(
        &self,
        query_vector: &[f32],
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        // Dispatch based on vector backend configuration
        match &self.config.vector.backend {
            VectorBackend::QmdBuiltin => {
                self.vector_search_sqlite(query_vector, options)
            }
            #[cfg(feature = "lancedb")]
            VectorBackend::LanceDb => {
                self.vector_search_lance(query_vector, options)
            }
            #[cfg(not(feature = "lancedb"))]
            VectorBackend::LanceDb => {
                anyhow::bail!("LanceDB backend not enabled. Build with --features lancedb")
            }
        }
    }

    /// Vector search using SQLite (qmd_builtin)
    fn vector_search_sqlite(
        &self,
        query_vector: &[f32],
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>> {
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
            if let Ok(conn) = self.get_connection(collection) {
                let collection_results = self.vector_search_in_db(&conn, query_vector, options.limit)?;
                results.extend(collection_results);
            }
        }

        // Sort by score (cosine distance - lower is better)
        results.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());

        Ok(results)
    }

    /// Vector search using LanceDB
    #[cfg(feature = "lancedb")]
    fn vector_search_lance(
        &self,
        query_vector: &[f32],
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        let mut all_results = Vec::new();

        let collections: Vec<&str> = if options.search_all {
            self.config.collections.iter().map(|c| c.name.as_str()).collect()
        } else if let Some(ref name) = options.collection {
            vec![name.as_str()]
        } else if let Some(col) = self.config.collections.first() {
            vec![col.name.as_str()]
        } else {
            return Ok(all_results);
        };

        for collection in collections {
            if let Some(ref backend_mutex) = self.lance_backend {
                if let Ok(backend) = backend_mutex.lock() {
                    let rt = tokio::runtime::Runtime::new()?;
                    let results = rt.block_on(async {
                        backend.vector_search(collection, query_vector, options.limit).await
                    });
                    if let Ok(results) = results {
                        all_results.extend(results);
                    }
                }
            }
        }

        Ok(all_results)
    }

    /// Perform vector search in a single database
    fn vector_search_in_db(
        &self,
        _conn: &Connection,
        _query_vector: &[f32],
        _limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // Try sqlite-vec first
        #[cfg(feature = "sqlite-vec")]
        {
            results = self.vector_search_sqlite_vec(_conn, _query_vector, _limit)?;
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
    ///
    /// Aggregates chunks back to document level by taking the best (minimum distance)
    /// chunk per document via GROUP BY.
    #[cfg(feature = "sqlite-vec")]
    fn vector_search_sqlite_vec(
        &self,
        conn: &Connection,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // Convert query vector to JSON array format for sqlite-vec
        let query_vec_json = serde_json::to_string(query_vector)?;

        // Use sqlite-vec's vec_distance_cosine function for similarity search.
        // GROUP BY cv.hash aggregates multiple chunks back to one result per document,
        // taking the best (minimum distance) chunk score.
        let mut stmt = conn.prepare(
            "SELECT
                cv.hash,
                d.path,
                d.title,
                d.collection,
                MIN(vec_distance_cosine(v.embedding, ?)) as distance
             FROM content_vectors cv
             JOIN vectors_vec v ON v.hash_seq = cv.hash || '_' || cv.seq
             JOIN documents d ON d.hash = cv.hash
             WHERE d.active = 1
             GROUP BY cv.hash
             ORDER BY distance ASC
             LIMIT ?"
        )?;

        let rows: Vec<(String, String, String, String, f64)> = stmt
            .query_map(rusqlite::params![query_vec_json, limit as i64], |row| {
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

        for (hash, path, title, collection, distance) in rows {
            let docid = make_docid(&collection, &path);
            // Calculate line count
            let lines = std::fs::read_to_string(&path)
                .map(|content| content.lines().count())
                .unwrap_or(0);
            results.push(SearchResult {
                docid,
                path,
                collection,
                score: distance as f32,
                lines,
                title,
                hash,
                query: None,
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
    /// 3. Vector search for original query
    /// 4. RRF fusion of all results
    /// 5. LLM reranking of top candidates (if available)
    pub async fn hybrid_search(
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

        // Step 3: Vector search for original query
        let vector_results = self.vector_search_with_embedder_async(query, options.clone(), llm).await?;

        info!("BM25 results: {}, Vector results: {}", all_bm25_results.len(), vector_results.len());

        // Step 4: RRF fusion of BM25 and vector results
        let result_lists = vec![all_bm25_results, vector_results];
        let weights = Some(vec![1.0, 1.5]); // Give more weight to vector search
        let mut fused = Self::rrf_fusion(&result_lists, weights, 60);

        // Sort by RRF score (higher is better)
        fused.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Step 5: Top 30 for reranking
        let candidates: Vec<SearchResult> = fused.into_iter().take(30).collect();

        // Step 6: Try LLM reranking if available
        let final_results = if llm.has_reranker() {
            info!("LLM reranking available, applying to top candidates");
            match llm.rerank(query, &candidates).await {
                Ok(scores) => {
                    // Apply reranking scores
                    let mut reranked: Vec<_> = candidates
                        .into_iter()
                        .zip(scores)
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
    pub fn rrf_fusion(
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
        let mut results: Vec<_> = doc_map.into_iter().collect();
        results.sort_by(|a, b| b.1.0.partial_cmp(&a.1.0).unwrap());

        // Apply Top-Rank Bonus and construct final results
        results.into_iter().enumerate().map(|(rank, (path, data))| {
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
                docid: make_docid(&data.1, &path),
                path,
                collection: data.1,
                score: final_score,
                lines: data.2,
                title: data.3,
                hash: data.4,
                query: None,
            }
        }).collect()
    }

    /// Set a path context (upsert)
    pub fn set_path_context(&self, collection: &str, path: &str, description: &str) -> Result<()> {
        let conn = self.get_connection(collection)?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO path_contexts (path, description, created_at, updated_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(path) DO UPDATE SET
                description = excluded.description,
                updated_at = excluded.updated_at",
            [path, description, &now, &now],
        )?;
        Ok(())
    }

    /// Get a path context
    pub fn get_path_context(&self, collection: &str, path: &str) -> Result<Option<(String, String)>> {
        let conn = self.get_connection(collection)?;
        let result = conn.query_row(
            "SELECT description, updated_at FROM path_contexts WHERE path = ?",
            [path],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        );
        match result {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List all path contexts in a collection
    pub fn list_path_contexts(&self, collection: &str) -> Result<Vec<(String, String)>> {
        let conn = self.get_connection(collection)?;
        let mut stmt = conn.prepare(
            "SELECT path, description FROM path_contexts ORDER BY path"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Remove a path context
    pub fn remove_path_context(&self, collection: &str, path: &str) -> Result<bool> {
        let conn = self.get_connection(collection)?;
        let deleted = conn.execute(
            "DELETE FROM path_contexts WHERE path = ?",
            [path],
        )?;
        Ok(deleted > 0)
    }

    /// Get a cached LLM response
    pub fn cache_get(&self, collection: &str, cache_key: &str) -> Result<Option<String>> {
        let conn = self.get_connection(collection)?;
        let now = chrono::Utc::now().to_rfc3339();
        let result = conn.query_row(
            "SELECT response FROM llm_cache
             WHERE cache_key = ? AND (expires_at IS NULL OR expires_at > ?)",
            [cache_key, &now],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(response) => Ok(Some(response)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Set a cached LLM response
    pub fn cache_set(
        &self,
        collection: &str,
        cache_key: &str,
        model: &str,
        response: &str,
        ttl_seconds: Option<i64>,
    ) -> Result<()> {
        let conn = self.get_connection(collection)?;
        let now = chrono::Utc::now();
        let created_at = now.to_rfc3339();
        let expires_at = ttl_seconds.map(|ttl| {
            (now + chrono::Duration::seconds(ttl)).to_rfc3339()
        });
        conn.execute(
            "INSERT INTO llm_cache (cache_key, model, response, created_at, expires_at)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(cache_key) DO UPDATE SET
                model = excluded.model,
                response = excluded.response,
                created_at = excluded.created_at,
                expires_at = excluded.expires_at",
            rusqlite::params![cache_key, model, response, created_at, expires_at],
        )?;
        Ok(())
    }

    /// Clear expired cache entries
    pub fn cache_clear_expired(&self, collection: &str) -> Result<usize> {
        let conn = self.get_connection(collection)?;
        let now = chrono::Utc::now().to_rfc3339();
        let deleted = conn.execute(
            "DELETE FROM llm_cache WHERE expires_at IS NOT NULL AND expires_at <= ?",
            [&now],
        )?;
        Ok(deleted)
    }

    /// Clear all cache entries
    pub fn cache_clear_all(&self, collection: &str) -> Result<usize> {
        let conn = self.get_connection(collection)?;
        let deleted = conn.execute("DELETE FROM llm_cache", [])?;
        Ok(deleted)
    }

    /// Update index
    pub fn update_index(&self) -> Result<()> {
        use std::io::Read;

        for collection in &self.config.collections {
            info!("Updating collection: {}", collection.name);

            // Expand the path
            let base_path = shellexpand::tilde(&collection.path.to_string_lossy())
                .parse::<std::path::PathBuf>()?;

            // Get glob pattern
            let pattern = collection.pattern.as_deref().unwrap_or("**/*");
            let glob_path = base_path.join(pattern);
            let glob_pattern = glob_path.to_string_lossy();

            info!("Scanning files with pattern: {}", glob_pattern);

            // Find matching files
            let entries = glob::glob(&glob_pattern)?;

            let mut file_count = 0;
            let mut skip_count = 0;

            for entry in entries {
                match entry {
                    Ok(path) => {
                        if !path.is_file() {
                            continue;
                        }

                        // Read file content
                        let mut file = std::fs::File::open(&path)?;
                        let mut content = String::new();
                        file.read_to_string(&mut content)?;

                        // Calculate hash of content
                        let hash = Self::calculate_hash(&content);

                        // Get relative path from base
                        let rel_path = path.strip_prefix(&base_path)
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .into_owned();

                        // Get file metadata
                        let metadata = std::fs::metadata(&path)?;
                        let modified: chrono::DateTime<chrono::Utc> = metadata.modified()?.into();
                        let created: chrono::DateTime<chrono::Utc> = metadata.created()?.into();

                        // Extract title from filename
                        let title = path.file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned();

                        // Insert or update document
                        let conn = match self.get_connection(&collection.name) {
                            Ok(c) => c,
                            Err(e) => {
                                warn!("Failed to get connection for {}: {}", collection.name, e);
                                continue;
                            }
                        };

                        // Check if document exists and is modified
                        let existing_hash: Option<String> = conn.query_row(
                            "SELECT hash FROM documents WHERE path = ? AND collection = ?",
                            [&rel_path, &collection.name],
                            |row| row.get(0)
                        ).ok();

                        if existing_hash.as_ref() == Some(&hash) {
                            // Document unchanged, skip
                            skip_count += 1;
                            continue;
                        }

                        // Upsert document (includes doc content directly now)
                        let doc_ref: &str = &content;
                        conn.execute(
                            "INSERT INTO documents (collection, path, title, hash, doc, created_at, modified_at, active)
                             VALUES (?, ?, ?, ?, ?, ?, ?, 1)
                             ON CONFLICT(hash) DO UPDATE SET
                                path = excluded.path,
                                title = excluded.title,
                                doc = excluded.doc,
                                modified_at = excluded.modified_at,
                                active = 1",
                            [&collection.name, &rel_path, &title, &hash, doc_ref,
                             &created.to_rfc3339(), &modified.to_rfc3339()],
                        )?;

                        file_count += 1;
                    }
                    Err(e) => {
                        warn!("Failed to access path: {:?}", e);
                    }
                }
            }

            info!("Updated {} files ({} unchanged)", file_count, skip_count);
        }

        Ok(())
    }

    /// Calculate SHA256 hash of content
    fn calculate_hash(content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get index statistics
    pub fn get_stats(&self) -> Result<IndexStats> {
        let mut stats = IndexStats {
            collection_count: self.config.collections.len(),
            ..Default::default()
        };

        for collection in &self.config.collections {
            if let Ok(conn) = self.get_connection(&collection.name) {
                let count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM documents WHERE active = 1",
                    [],
                    |row| row.get(0)
                ).unwrap_or(0);

                let chunks: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM content_vectors",
                    [],
                    |row| row.get(0)
                ).unwrap_or(0);

                stats.document_count += count as usize;
                stats.chunk_count += chunks as usize;
                stats.collection_stats.insert(collection.name.clone(), count as usize);
            }
        }

        stats.indexed_count = stats.document_count;
        stats.pending_count = 0;

        Ok(stats)
    }

    /// Find stale entries - files that no longer exist on disk
    pub fn find_stale_entries(&self, _older_than: u32) -> Result<Vec<String>> {
        let mut stale_paths = Vec::new();

        for collection in &self.config.collections {
            if let Ok(conn) = self.get_connection(&collection.name) {
                let mut stmt = conn.prepare("SELECT path FROM documents WHERE active = 1")?;
                let paths: Vec<String> = stmt
                    .query_map([], |row| row.get(0))?
                    .filter_map(|r| r.ok())
                    .collect();

                for path in paths {
                    // Check if file exists
                    if !std::path::Path::new(&path).exists() {
                        stale_paths.push(path);
                    }
                }
            }
        }

        Ok(stale_paths)
    }

    /// Remove stale entries from database
    pub fn remove_stale_entries(&self, entries: &[String]) -> Result<()> {
        for collection in &self.config.collections {
            if let Ok(conn) = self.get_connection(&collection.name) {
                for path in entries {
                    // Soft delete - mark as inactive
                    conn.execute(
                        "UPDATE documents SET active = 0 WHERE path = ?",
                        [path],
                    )?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CollectionConfig;

    /// Initialize sqlite-vec for tests that need database access
    fn init_test_db(path: &std::path::Path) -> Connection {
        Store::init_sqlite_vec().unwrap();
        let conn = Connection::open(path).unwrap();
        Store::init_schema(&conn).unwrap();
        conn
    }

    /// Helper to create a SearchResult for testing
    fn make_result(path: &str, score: f32) -> SearchResult {
        SearchResult {
            docid: make_docid("test", path),
            path: path.to_string(),
            collection: "test".to_string(),
            score,
            lines: 0,
            title: path.to_string(),
            hash: format!("hash_{}", path),
            query: None,
        }
    }

    // ==================== RRF Fusion Tests ====================

    #[test]
    fn test_rrf_fusion_empty_input() {
        let result = Store::rrf_fusion(&[], None, 60);
        assert!(result.is_empty());
    }

    #[test]
    fn test_rrf_fusion_single_empty_list() {
        let result = Store::rrf_fusion(&[vec![]], None, 60);
        assert!(result.is_empty());
    }

    #[test]
    fn test_rrf_fusion_single_list() {
        let list = vec![
            make_result("doc1.md", 0.9),
            make_result("doc2.md", 0.8),
            make_result("doc3.md", 0.7),
        ];
        let result = Store::rrf_fusion(&[list], None, 60);

        assert_eq!(result.len(), 3);
        // First result should have highest RRF score (1/60 + top-rank bonus)
        assert_eq!(result[0].path, "doc1.md");
        assert_eq!(result[1].path, "doc2.md");
        assert_eq!(result[2].path, "doc3.md");
        // Verify ordering: scores should be descending
        assert!(result[0].score > result[1].score);
        assert!(result[1].score > result[2].score);
    }

    #[test]
    fn test_rrf_fusion_two_lists_deduplication() {
        // Same document appears in both lists
        let list1 = vec![
            make_result("doc1.md", 0.9),
            make_result("doc2.md", 0.8),
        ];
        let list2 = vec![
            make_result("doc2.md", 0.95), // doc2 appears in both
            make_result("doc3.md", 0.7),
        ];
        let result = Store::rrf_fusion(&[list1, list2], None, 60);

        // Should have 3 unique documents
        assert_eq!(result.len(), 3);

        // doc2 should rank highest because it appears in both lists
        // RRF(doc2) = 1/(60+1) + 1/(60+0) = ~0.0164 + ~0.0167 = ~0.033
        // RRF(doc1) = 1/(60+0) = ~0.0167
        // RRF(doc3) = 1/(60+1) = ~0.0164
        assert_eq!(result[0].path, "doc2.md");
    }

    #[test]
    fn test_rrf_fusion_with_weights() {
        let list1 = vec![make_result("doc1.md", 0.9)];
        let list2 = vec![make_result("doc2.md", 0.8)];

        // Give list2 much higher weight
        let weights = Some(vec![1.0, 10.0]);
        let result = Store::rrf_fusion(&[list1, list2], weights, 60);

        assert_eq!(result.len(), 2);
        // doc2 should rank higher due to 10x weight
        assert_eq!(result[0].path, "doc2.md");
        assert_eq!(result[1].path, "doc1.md");
    }

    #[test]
    fn test_rrf_fusion_top_rank_bonus() {
        // Single list with 15 items to test all bonus tiers
        let list: Vec<SearchResult> = (0..15)
            .map(|i| make_result(&format!("doc{}.md", i), 1.0 - i as f32 * 0.01))
            .collect();
        let result = Store::rrf_fusion(&[list], None, 60);

        assert_eq!(result.len(), 15);

        // Rank 0 gets +0.05 bonus
        // Rank 1,2 get +0.02 bonus
        // Rank 3-9 get +0.01 bonus
        // Rank 10+ get no bonus

        // Base RRF for rank 0: 1/60 ≈ 0.01667, + 0.05 = ~0.06667
        // Base RRF for rank 1: 1/61 ≈ 0.01639, + 0.02 = ~0.03639
        let first_score = result[0].score;
        let second_score = result[1].score;
        assert!(first_score > 0.06, "First place should have bonus: {}", first_score);
        assert!(second_score > 0.03, "Second place should have bonus: {}", second_score);
    }

    #[test]
    fn test_rrf_fusion_k_parameter() {
        let list = vec![
            make_result("doc1.md", 0.9),
            make_result("doc2.md", 0.8),
        ];

        // With k=1, scores are 1/(1+0)=1.0 and 1/(1+1)=0.5
        let result_k1 = Store::rrf_fusion(&[list.clone()], None, 1);
        // With k=100, scores are 1/(100+0)=0.01 and 1/(100+1)≈0.0099
        let result_k100 = Store::rrf_fusion(&[list], None, 100);

        // k=1 should produce larger score differences
        let diff_k1 = result_k1[0].score - result_k1[1].score;
        let diff_k100 = result_k100[0].score - result_k100[1].score;
        assert!(diff_k1 > diff_k100, "Smaller k should produce larger score gaps");
    }

    #[test]
    fn test_rrf_fusion_preserves_metadata() {
        let list = vec![SearchResult {
            docid: make_docid("my_collection", "test/doc.md"),
            path: "test/doc.md".to_string(),
            collection: "my_collection".to_string(),
            score: 0.5,
            lines: 42,
            title: "Test Document".to_string(),
            hash: "abc123".to_string(),
            query: None,
        }];
        let result = Store::rrf_fusion(&[list], None, 60);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, "test/doc.md");
        assert_eq!(result[0].collection, "my_collection");
        assert_eq!(result[0].lines, 42);
        assert_eq!(result[0].title, "Test Document");
        assert_eq!(result[0].hash, "abc123");
    }

    #[test]
    fn test_rrf_fusion_three_lists() {
        let list1 = vec![make_result("a.md", 0.9), make_result("b.md", 0.8)];
        let list2 = vec![make_result("b.md", 0.95), make_result("c.md", 0.7)];
        let list3 = vec![make_result("b.md", 0.85), make_result("a.md", 0.6)];

        let result = Store::rrf_fusion(&[list1, list2, list3], None, 60);

        assert_eq!(result.len(), 3);
        // b.md appears in all 3 lists, should rank first
        assert_eq!(result[0].path, "b.md");
        // a.md appears in 2 lists, should rank second
        assert_eq!(result[1].path, "a.md");
    }

    // ==================== SearchResult Tests ====================

    #[test]
    fn test_search_result_equality() {
        let r1 = make_result("doc.md", 0.5);
        let r2 = make_result("doc.md", 0.5);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_search_result_clone() {
        let r1 = make_result("doc.md", 0.5);
        let r2 = r1.clone();
        assert_eq!(r1.path, r2.path);
        assert_eq!(r1.score, r2.score);
    }

    #[test]
    fn test_search_result_serialize() {
        let r = make_result("doc.md", 0.75);
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"path\":\"doc.md\""));
        assert!(json.contains("\"score\":0.75"));
    }

    // ==================== SearchOptions Tests ====================

    #[test]
    fn test_search_options_defaults() {
        let opts = SearchOptions {
            limit: 10,
            min_score: 0.0,
            collection: None,
            search_all: false,
        };
        assert_eq!(opts.limit, 10);
        assert!(!opts.search_all);
    }

    // ==================== Config Integration Tests ====================

    #[test]
    fn test_config_db_path() {
        let config = Config::default();
        let db_path = config.db_path_for("test_collection");
        assert!(db_path.to_string_lossy().contains("test_collection"));
        assert!(db_path.to_string_lossy().ends_with("index.db"));
    }

    // ==================== Hash Calculation Tests ====================

    #[test]
    fn test_calculate_hash_deterministic() {
        let hash1 = Store::calculate_hash("hello world");
        let hash2 = Store::calculate_hash("hello world");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_calculate_hash_different_inputs() {
        let hash1 = Store::calculate_hash("hello");
        let hash2 = Store::calculate_hash("world");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_calculate_hash_empty_string() {
        let hash = Store::calculate_hash("");
        assert!(!hash.is_empty());
        // SHA256 of empty string is well-known
        assert_eq!(hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    // ==================== Store with TempDir Tests ====================

    #[test]
    fn test_store_init_schema() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("test.db");
        let conn = init_test_db(&db_path);

        // Verify tables exist
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='documents'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Verify FTS table exists
        let fts_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='documents_fts'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(fts_count, 1);

        // Verify content_vectors table exists
        let cv_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='content_vectors'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(cv_count, 1);
    }

    #[test]
    fn test_bm25_search_with_data() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("test_col").join("index.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();

        let config = Config {
            collections: vec![CollectionConfig {
                name: "test_col".to_string(),
                path: tmp.path().to_path_buf(),
                pattern: None,
                description: None,
            }],
            cache_path: tmp.path().to_path_buf(),
            ..Config::default()
        };

        // Use init_test_db to load sqlite-vec extension
        let conn = init_test_db(&db_path);

        // Insert test documents
        conn.execute(
            "INSERT INTO documents (collection, path, title, hash, doc, created_at, modified_at, active)
             VALUES ('test_col', 'rust_guide.md', 'Rust Programming Guide', 'hash1', 'Rust is a systems programming language', datetime('now'), datetime('now'), 1)",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO documents (collection, path, title, hash, doc, created_at, modified_at, active)
             VALUES ('test_col', 'python_guide.md', 'Python Tutorial', 'hash2', 'Python is a dynamic programming language', datetime('now'), datetime('now'), 1)",
            [],
        ).unwrap();

        drop(conn);

        let store = Store {
            config,
            connections: HashMap::new(),
            #[cfg(feature = "lancedb")]
            lance_backend: None,
        };

        let opts = SearchOptions {
            limit: 10,
            min_score: 0.0,
            collection: Some("test_col".to_string()),
            search_all: false,
        };

        let results = store.bm25_search("Rust programming", opts).unwrap();
        assert!(!results.is_empty(), "BM25 search should find 'Rust programming'");
        assert_eq!(results[0].path, "test_col/rust_guide.md");
    }

    #[test]
    fn test_bm25_search_no_results() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("test_col").join("index.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();

        let config = Config {
            collections: vec![CollectionConfig {
                name: "test_col".to_string(),
                path: tmp.path().to_path_buf(),
                pattern: None,
                description: None,
            }],
            cache_path: tmp.path().to_path_buf(),
            ..Config::default()
        };

        let conn = init_test_db(&db_path);

        conn.execute(
            "INSERT INTO documents (collection, path, title, hash, doc, created_at, modified_at, active)
             VALUES ('test_col', 'doc.md', 'Test', 'hash1', 'Hello world', datetime('now'), datetime('now'), 1)",
            [],
        ).unwrap();

        drop(conn);

        let store = Store {
            config,
            connections: HashMap::new(),
            #[cfg(feature = "lancedb")]
            lance_backend: None,
        };

        let opts = SearchOptions {
            limit: 10,
            min_score: 0.0,
            collection: Some("test_col".to_string()),
            search_all: false,
        };

        let results = store.bm25_search("nonexistent_xyz_query", opts).unwrap();
        assert!(results.is_empty(), "Should find no results for unrelated query");
    }

    #[test]
    fn test_get_stats_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("test_col").join("index.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();

        let config = Config {
            collections: vec![CollectionConfig {
                name: "test_col".to_string(),
                path: tmp.path().to_path_buf(),
                pattern: None,
                description: None,
            }],
            cache_path: tmp.path().to_path_buf(),
            ..Config::default()
        };

        // Init schema so the table exists
        let conn = init_test_db(&db_path);
        drop(conn);

        let store = Store {
            config,
            connections: HashMap::new(),
            #[cfg(feature = "lancedb")]
            lance_backend: None,
        };

        let stats = store.get_stats().unwrap();
        assert_eq!(stats.collection_count, 1);
        assert_eq!(stats.document_count, 0);
    }
}
