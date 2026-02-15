//! LanceDB backend implementation for QMD
//!
//! Provides LanceDB as an alternative backend for both:
//! - Full-text search (BM25)
//! - Vector similarity search
//!
//! LanceDB provides embedded database capabilities with:
//! - FTS (Full-Text Search) via Index::FTS
//! - Vector similarity search with cosine distance

use crate::store::SearchResult;
use anyhow::{Context, Result};
use arrow_array::{
    Array, FixedSizeListArray, Float32Array, Int64Array, RecordBatch, RecordBatchIterator,
    StringArray,
};
use arrow_schema::{DataType, Field, Schema};
use futures::TryStreamExt;
use lance_index::scalar::FullTextSearchQuery;
use lancedb::index::Index;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{connect, Connection as LanceConnection, DistanceType};
use std::path::PathBuf;
use std::sync::Arc;

/// LanceDB backend for QMD
///
/// This implementation uses LanceDB for both FTS and vector search.
/// LanceDB provides embedded database mode (no server required).
pub struct LanceDbBackend {
    /// Base path for LanceDB databases
    pub db_path: PathBuf,
    /// Embedding dimension (for vector tables)
    pub embedding_dim: usize,
    /// LanceDB database connection
    db: Option<LanceConnection>,
}

impl LanceDbBackend {
    /// Create a new LanceDbBackend instance
    pub fn new(db_path: PathBuf, embedding_dim: usize) -> Self {
        Self {
            db_path,
            embedding_dim,
            db: None,
        }
    }

    /// Connect to LanceDB
    pub async fn connect(&mut self) -> Result<()> {
        let db_uri = self.db_path.join("lancedb");
        std::fs::create_dir_all(&db_uri)?;
        let db = connect(db_uri.to_string_lossy().as_ref())
            .execute()
            .await
            .context("Failed to connect to LanceDB")?;
        self.db = Some(db);
        Ok(())
    }

    /// Get the database connection, returning an error if not connected
    fn get_db(&self) -> Result<&LanceConnection> {
        self.db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("LanceDB not connected. Call connect() first."))
    }

    /// Get the table name for a collection
    fn table_name(collection: &str) -> String {
        format!("qmd_{}", collection.replace(['/', '\\', '.'], "_"))
    }

    /// Build the Arrow schema for document tables
    fn doc_schema(&self) -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("path", DataType::Utf8, false),
            Field::new("title", DataType::Utf8, false),
            Field::new("body", DataType::Utf8, false),
            Field::new("hash", DataType::Utf8, false),
            Field::new(
                "embedding",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    self.embedding_dim as i32,
                ),
                true,
            ),
        ]))
    }

    /// Open or create a table for a collection
    async fn open_or_create_table(
        &self,
        collection: &str,
    ) -> Result<lancedb::Table> {
        let db = self.get_db()?;
        let table_name = Self::table_name(collection);

        // Try to open existing table
        match db.open_table(&table_name).execute().await {
            Ok(table) => Ok(table),
            Err(_) => {
                // Create empty table with schema
                let schema = self.doc_schema();
                let batch = RecordBatch::new_empty(schema.clone());
                let batches =
                    RecordBatchIterator::new(vec![Ok(batch)], schema.clone());
                let table = db
                    .create_table(&table_name, Box::new(batches))
                    .execute()
                    .await
                    .with_context(|| {
                        format!("Failed to create LanceDB table: {}", table_name)
                    })?;
                Ok(table)
            }
        }
    }

    /// Full-text search using LanceDB FTS
    pub async fn fts_search(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let table = self.open_or_create_table(collection).await?;

        let fts_query = FullTextSearchQuery::new(query.to_string());

        let stream = table
            .query()
            .full_text_search(fts_query)
            .limit(limit)
            .execute()
            .await;

        let batches = match stream {
            Ok(stream) => stream.try_collect::<Vec<_>>().await?,
            Err(_) => {
                // FTS index may not exist yet, return empty
                return Ok(Vec::new());
            }
        };

        let mut results = Vec::new();
        for batch in &batches {
            let paths = batch
                .column_by_name("path")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let titles = batch
                .column_by_name("title")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let hashes = batch
                .column_by_name("hash")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let scores = batch
                .column_by_name("_score")
                .and_then(|c| c.as_any().downcast_ref::<Float32Array>());

            if let (Some(paths), Some(titles), Some(hashes)) = (paths, titles, hashes) {
                for i in 0..batch.num_rows() {
                    let path = paths.value(i).to_string();
                    let title = titles.value(i).to_string();
                    let hash = hashes.value(i).to_string();
                    let score = scores.map(|s| s.value(i)).unwrap_or(0.0);

                    let lines = std::fs::read_to_string(&path)
                        .map(|content| content.lines().count())
                        .unwrap_or(0);

                    results.push(SearchResult {
                        docid: crate::store::make_docid(collection, &path),
                        path,
                        collection: collection.to_string(),
                        score,
                        lines,
                        title,
                        hash,
                        query: Some(query.to_string()),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Vector search using LanceDB
    pub async fn vector_search(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let table = self.open_or_create_table(collection).await?;

        let stream = table
            .vector_search(query_vector)
            .context("Failed to create vector query")?
            .distance_type(DistanceType::Cosine)
            .limit(limit)
            .execute()
            .await;

        let batches = match stream {
            Ok(stream) => stream.try_collect::<Vec<_>>().await?,
            Err(_) => {
                // Vector index may not exist or table is empty
                return Ok(Vec::new());
            }
        };

        let mut results = Vec::new();
        for batch in &batches {
            let paths = batch
                .column_by_name("path")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let titles = batch
                .column_by_name("title")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let hashes = batch
                .column_by_name("hash")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let distances = batch
                .column_by_name("_distance")
                .and_then(|c| c.as_any().downcast_ref::<Float32Array>());

            if let (Some(paths), Some(titles), Some(hashes)) = (paths, titles, hashes) {
                for i in 0..batch.num_rows() {
                    let path = paths.value(i).to_string();
                    let title = titles.value(i).to_string();
                    let hash = hashes.value(i).to_string();
                    // Convert cosine distance to similarity score (1 - distance)
                    let distance = distances.map(|d| d.value(i)).unwrap_or(1.0);
                    let score = 1.0 - distance;

                    let lines = std::fs::read_to_string(&path)
                        .map(|content| content.lines().count())
                        .unwrap_or(0);

                    results.push(SearchResult {
                        docid: crate::store::make_docid(collection, &path),
                        path,
                        collection: collection.to_string(),
                        score,
                        lines,
                        title,
                        hash,
                        query: None,
                    });
                }
            }
        }

        Ok(results)
    }

    /// Insert documents into LanceDB table
    pub async fn insert_documents(
        &self,
        collection: &str,
        documents: Vec<DocumentInput>,
    ) -> Result<()> {
        if documents.is_empty() {
            return Ok(());
        }

        let table = self.open_or_create_table(collection).await?;

        let len = documents.len();
        let mut ids = Vec::with_capacity(len);
        let mut paths = Vec::with_capacity(len);
        let mut titles = Vec::with_capacity(len);
        let mut bodies = Vec::with_capacity(len);
        let mut hashes = Vec::with_capacity(len);
        let mut embeddings: Vec<Option<Vec<f32>>> = Vec::with_capacity(len);

        for doc in &documents {
            ids.push(doc.id);
            paths.push(doc.path.as_str());
            titles.push(doc.title.as_str());
            bodies.push(doc.body.as_str());
            hashes.push(doc.hash.as_str());
            embeddings.push(doc.embedding.clone());
        }

        let schema = self.doc_schema();

        // Build embedding FixedSizeList array
        let dim = self.embedding_dim as i32;
        let embedding_values: Vec<f32> = embeddings
            .iter()
            .flat_map(|e| {
                e.clone()
                    .unwrap_or_else(|| vec![0.0; self.embedding_dim])
            })
            .collect();
        let values_array = Float32Array::from(embedding_values);
        let field = Arc::new(Field::new("item", DataType::Float32, true));
        let embedding_array = FixedSizeListArray::new(field, dim, Arc::new(values_array), None);

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(ids)),
                Arc::new(StringArray::from(paths)),
                Arc::new(StringArray::from(titles)),
                Arc::new(StringArray::from(bodies)),
                Arc::new(StringArray::from(hashes)),
                Arc::new(embedding_array) as Arc<dyn Array>,
            ],
        )?;

        let batches =
            RecordBatchIterator::new(vec![Ok(batch)], schema.clone());

        table
            .add(Box::new(batches))
            .execute()
            .await
            .context("Failed to insert documents into LanceDB")?;

        log::info!("Inserted {} documents into LanceDB collection '{}'", len, collection);
        Ok(())
    }

    /// Ensure FTS index exists on body and title columns
    pub async fn ensure_fts_index(&self, collection: &str) -> Result<()> {
        let table = self.open_or_create_table(collection).await?;

        // Create FTS index on body column
        if let Err(e) = table
            .create_index(&["body", "title"], Index::FTS(Default::default()))
            .execute()
            .await
        {
            log::warn!(
                "FTS index creation for '{}' (may already exist): {}",
                collection,
                e
            );
        }

        Ok(())
    }

    /// Ensure vector index exists on embedding column
    pub async fn ensure_vector_index(&self, collection: &str) -> Result<()> {
        let table = self.open_or_create_table(collection).await?;

        if let Err(e) = table
            .create_index(&["embedding"], Index::Auto)
            .execute()
            .await
        {
            log::warn!(
                "Vector index creation for '{}' (may already exist): {}",
                collection,
                e
            );
        }

        Ok(())
    }
}

/// Document input for LanceDB
pub struct DocumentInput {
    pub id: i64,
    pub path: String,
    pub title: String,
    pub body: String,
    pub hash: String,
    pub embedding: Option<Vec<f32>>,
}
