//! Qdrant backend implementation for QMD
//!
//! Provides Qdrant as a vector backend for QMD. Qdrant is a high-performance
//! vector search engine that can be used either as a local server or cloud service.
//!
//! Key features:
//! - Vector similarity search with cosine distance
//! - Cloud and local deployment support
//! - RESTful API with gRPC transport
//! - Automatic collection management

use crate::store::SearchResult;
use anyhow::{Context, Result};
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, SearchPointsBuilder,
    Value, VectorParamsBuilder,
};
use qdrant_client::{Payload, Qdrant};
use std::collections::HashMap;

/// Helper function to extract string from Value
fn value_to_string(value: &Value) -> String {
    match value.kind.as_ref() {
        Some(qdrant_client::qdrant::value::Kind::StringValue(s)) => s.clone(),
        _ => String::new(),
    }
}

/// Qdrant backend for QMD vector search
pub struct QdrantBackend {
    /// Qdrant client
    client: Qdrant,
    /// Collection name
    collection: String,
    /// Vector size/dimension
    vector_size: usize,
    /// Cache of collection existence
    collections_cache: HashMap<String, bool>,
}

impl QdrantBackend {
    /// Create a new QdrantBackend instance
    ///
    /// # Arguments
    /// * `url` - Qdrant server URL (e.g., "http://localhost:6333")
    /// * `api_key` - Optional API key for Qdrant Cloud authentication
    /// * `collection` - Collection name for QMD documents
    /// * `vector_size` - Vector dimension (must match embedding model)
    pub async fn new(
        url: &str,
        api_key: Option<&str>,
        collection: &str,
        vector_size: usize,
    ) -> Result<Self> {
        // Build client using the builder pattern
        let mut builder = Qdrant::from_url(url);

        // Add API key if provided
        if let Some(key) = api_key {
            builder = builder.api_key(key);
        }

        let client = builder
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create Qdrant client: {}", e))?;

        Ok(Self {
            client,
            collection: collection.to_string(),
            vector_size,
            collections_cache: HashMap::new(),
        })
    }

    /// Ensure collection exists, create if not
    pub async fn ensure_collection(&mut self) -> Result<()> {
        // Check cache first
        if self.collections_cache.get(&self.collection) == Some(&true) {
            return Ok(());
        }

        // Check if collection exists
        let collection_info = self.client.collection_info(&self.collection).await;
        let collection_exists = match collection_info {
            Ok(info) => info.result.is_some(),
            Err(_) => false,
        };

        if !collection_exists {
            log::info!("Creating Qdrant collection: {}", self.collection);

            // Create collection with cosine distance using builder
            self.client
                .create_collection(
                    CreateCollectionBuilder::new(&self.collection).vectors_config(
                        VectorParamsBuilder::new(self.vector_size as u64, Distance::Cosine),
                    ),
                )
                .await
                .context("Failed to create Qdrant collection")?;

            log::info!("Created Qdrant collection: {}", self.collection);
        }

        // Cache the result
        self.collections_cache.insert(self.collection.clone(), true);
        Ok(())
    }

    /// Upsert vectors into Qdrant collection
    ///
    /// # Arguments
    /// * `documents` - Vector of DocumentInput to upsert
    pub async fn upsert_vectors(&mut self, documents: Vec<DocumentInput>) -> Result<()> {
        // Ensure collection exists
        self.ensure_collection().await?;

        // Build points for upsert using builder pattern
        let points: Vec<PointStruct> = documents
            .into_iter()
            .map(|doc| {
                // Parse id as u64
                let id = doc
                    .id
                    .parse::<u64>()
                    .unwrap_or_else(|_| rand::random::<u64>());

                let mut payload = Payload::new();
                payload.insert("path", doc.path);
                payload.insert("title", doc.title);
                payload.insert("body", doc.body);
                payload.insert("hash", doc.hash);
                payload.insert("collection", doc.collection);

                PointStruct::new(id, doc.vector, payload)
            })
            .collect();

        let points_count = points.len();

        // Upsert points
        if points_count > 0 {
            use qdrant_client::qdrant::UpsertPointsBuilder;

            self.client
                .upsert_points(UpsertPointsBuilder::new(&self.collection, points))
                .await
                .context("Failed to upsert vectors to Qdrant")?;

            log::info!("Upserted {} documents to Qdrant", points_count);
        }

        Ok(())
    }

    /// Vector search using Qdrant
    ///
    /// # Arguments
    /// * `query_vector` - The embedding vector to search with
    /// * `limit` - Maximum number of results to return
    /// * `_score_threshold` - Optional minimum score threshold (not used)
    pub async fn vector_search(
        &self,
        query_vector: &[f32],
        limit: usize,
        _score_threshold: Option<f32>,
    ) -> Result<Vec<SearchResult>> {
        // Build search request using builder pattern
        let search_request =
            SearchPointsBuilder::new(&self.collection, query_vector.to_vec(), limit as u64)
                .with_payload(true);

        // Execute search
        let search_result = self
            .client
            .search_points(search_request)
            .await
            .context("Failed to search Qdrant")?;

        // Convert results to SearchResult
        let mut results = Vec::with_capacity(search_result.result.len());

        for point in search_result.result {
            // Access payload directly - it's a HashMap<String, Value>
            let payload = &point.payload;

            let path = payload
                .get("path")
                .map(value_to_string)
                .unwrap_or_default();

            let title = payload
                .get("title")
                .map(value_to_string)
                .unwrap_or_default();

            let hash = payload
                .get("hash")
                .map(value_to_string)
                .unwrap_or_default();

            let collection = payload
                .get("collection")
                .map(value_to_string)
                .unwrap_or_default();

            // Calculate line count from file
            let lines = std::fs::read_to_string(&path)
                .map(|content| content.lines().count())
                .unwrap_or(0);

            results.push(SearchResult {
                docid: crate::store::make_docid(&collection, &path),
                path,
                collection,
                score: point.score,
                lines,
                title,
                hash,
                query: None,
            });
        }

        Ok(results)
    }

    /// Get collection info
    pub async fn get_collection_info(&self) -> Result<i32> {
        let info = self
            .client
            .collection_info(&self.collection)
            .await
            .context("Failed to get collection info")?;

        // Return the status as i32 (protobuf enum)
        Ok(info.result.map(|c| c.status).unwrap_or(0))
    }

    /// Delete collection
    #[allow(deprecated)]
    pub async fn delete_collection(&self) -> Result<()> {
        // Use the deprecated method for compatibility
        self.client
            .delete_collection(&self.collection)
            .await
            .context("Failed to delete collection")?;

        log::info!("Deleted Qdrant collection: {}", self.collection);
        Ok(())
    }

    /// Get collection name
    pub fn collection_name(&self) -> &str {
        &self.collection
    }
}

/// Document input for Qdrant upsert
pub struct DocumentInput {
    /// Unique ID (numeric string)
    pub id: String,
    /// Document path
    pub path: String,
    /// Document title
    pub title: String,
    /// Document body content
    pub body: String,
    /// Document hash
    pub hash: String,
    /// Collection name
    pub collection: String,
    /// Embedding vector
    pub vector: Vec<f32>,
}

impl DocumentInput {
    /// Create a new DocumentInput
    pub fn new(
        id: String,
        path: String,
        title: String,
        body: String,
        hash: String,
        collection: String,
        vector: Vec<f32>,
    ) -> Self {
        Self {
            id,
            path,
            title,
            body,
            hash,
            collection,
            vector,
        }
    }
}
