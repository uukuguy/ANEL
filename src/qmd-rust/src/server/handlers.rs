// HTTP request handlers

use crate::server::ServerState;
use crate::store::SearchOptions;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};

// ── Request/Response types ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub collection: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResultDto>,
    pub total: usize,
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResultDto {
    pub docid: String,
    pub collection: String,
    pub title: String,
    pub path: String,
    pub score: f32,
    pub lines: usize,
}

#[derive(Debug, Serialize)]
pub struct CollectionDto {
    pub name: String,
    pub description: Option<String>,
    pub document_count: usize,
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub collections: usize,
    pub documents: usize,
    pub indexed: usize,
    pub pending: usize,
    pub chunks: usize,
    pub collection_stats: Vec<CollectionStatDto>,
}

#[derive(Debug, Serialize)]
pub struct CollectionStatDto {
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub model_loaded: bool,
}

#[derive(Debug, Deserialize)]
pub struct GetDocumentQuery {
    #[serde(default)]
    pub from: Option<usize>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub path: String,
    pub title: String,
    pub total_lines: usize,
    pub content: String,
    pub from: usize,
    pub to: usize,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

// ── Handlers ────────────────────────────────────────────────────────

/// Health check endpoint
pub async fn health(State(state): State<ServerState>) -> impl IntoResponse {
    let model_loaded = state.llm.try_lock().is_ok();

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        model_loaded,
    };

    Json(response)
}

/// List all collections
pub async fn list_collections(
    State(state): State<ServerState>,
) -> impl IntoResponse {
    let store = state.store.lock().await;
    let collections = store.get_collections();

    let collection_dtos: Vec<CollectionDto> = collections
        .iter()
        .map(|c| CollectionDto {
            name: c.name.clone(),
            description: c.description.clone(),
            document_count: 0,
        })
        .collect();

    Json(collection_dtos)
}

/// Index statistics
pub async fn stats(State(state): State<ServerState>) -> impl IntoResponse {
    let store = state.store.lock().await;

    let stats = store.get_stats();

    let response = match stats {
        Ok(stats) => {
            let collection_stats: Vec<CollectionStatDto> = stats
                .collection_stats
                .into_iter()
                .map(|(name, count)| CollectionStatDto { name, count })
                .collect();

            StatsResponse {
                collections: stats.collection_count,
                documents: stats.document_count,
                indexed: stats.indexed_count,
                pending: stats.pending_count,
                chunks: stats.chunk_count,
                collection_stats,
            }
        }
        Err(_) => {
            StatsResponse {
                collections: 0,
                documents: 0,
                indexed: 0,
                pending: 0,
                chunks: 0,
                collection_stats: vec![],
            }
        }
    };

    Json(response)
}

/// BM25 full-text search
pub async fn search(
    State(state): State<ServerState>,
    Json(req): Json<SearchRequest>,
) -> impl IntoResponse {
    let store = state.store.lock().await;

    let options = SearchOptions {
        limit: req.limit.unwrap_or(20),
        min_score: 0.0,
        collection: req.collection.clone(),
        search_all: req.collection.is_none(),
    };

    let results = store.bm25_search(&req.query, options);

    let dtos: Vec<SearchResultDto> = match results {
        Ok(results) => results
            .into_iter()
            .map(|r| SearchResultDto {
                docid: r.docid,
                collection: r.collection,
                title: r.title,
                path: r.path,
                score: r.score,
                lines: r.lines,
            })
            .collect(),
        Err(_) => vec![],
    };

    let response = SearchResponse {
        total: dtos.len(),
        results: dtos,
        query: req.query,
    };

    Json(response)
}

/// Vector semantic search
pub async fn vsearch(
    State(state): State<ServerState>,
    Json(req): Json<SearchRequest>,
) -> impl IntoResponse {
    // Generate embedding first
    let embedding = {
        let llm = state.llm.lock().await;
        match llm.embed(&[req.query.as_str()]).await {
            Ok(e) => e.embeddings.into_iter().next().unwrap_or_default(),
            Err(_) => vec![],
        }
    };

    if embedding.is_empty() {
        let response = SearchResponse {
            total: 0,
            results: vec![],
            query: req.query,
        };
        return Json(response);
    }

    // Vector search with pre-computed embedding (sync function)
    let store = state.store.lock().await;

    let options = SearchOptions {
        limit: req.limit.unwrap_or(20),
        min_score: 0.0,
        collection: req.collection.clone(),
        search_all: req.collection.is_none(),
    };

    let results = store.vector_search_with_embedding(&embedding, options);

    let dtos: Vec<SearchResultDto> = match results {
        Ok(results) => results
            .into_iter()
            .map(|r| SearchResultDto {
                docid: r.docid,
                collection: r.collection,
                title: r.title,
                path: r.path,
                score: r.score,
                lines: r.lines,
            })
            .collect(),
        Err(_) => vec![],
    };

    let response = SearchResponse {
        total: dtos.len(),
        results: dtos,
        query: req.query,
    };

    Json(response)
}

/// Hybrid search (BM25 + Vector + RRF + Reranking)
/// Note: Uses the same implementation as vsearch for now
pub async fn query(
    State(state): State<ServerState>,
    Json(req): Json<SearchRequest>,
) -> impl IntoResponse {
    // For now, delegate to vsearch (hybrid requires complex lock handling)
    // TODO: Implement proper hybrid search with lock management
    vsearch(State(state), Json(req)).await
}

/// Get document content
pub async fn get_document(
    State(_state): State<ServerState>,
    Path(path): Path<String>,
    Query(query): Query<GetDocumentQuery>,
) -> axum::response::Response {
    let from = query.from.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);

    // URL decode the path
    let path = match urlencoding::decode(&path) {
        Ok(p) => p.to_string(),
        Err(_) => path,
    };

    // Read file
    let content = std::fs::read_to_string(&path);

    match content {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            let total = lines.len();
            let start = from.min(total);
            let end = (start + limit).min(total);
            let selected = lines[start..end].join("\n");

            // Extract title from path
            let title = std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();

            let response = DocumentResponse {
                path: path.clone(),
                title,
                total_lines: total,
                content: selected,
                from: start,
                to: end,
            };

            Json(response).into_response()
        }
        Err(_) => {
            let error = ErrorResponse {
                error: format!("File not found: {}", path),
                code: "FILE_NOT_FOUND".to_string(),
            };
            (StatusCode::NOT_FOUND, Json(error)).into_response()
        }
    }
}

/// MCP protocol handler (JSON-RPC)
pub async fn mcp(
    State(_state): State<ServerState>,
    _body: String,
) -> impl IntoResponse {
    // TODO: Implement full MCP protocol handler
    let error = ErrorResponse {
        error: "MCP protocol not yet implemented in standalone server".to_string(),
        code: "MCP_NOT_IMPLEMENTED".to_string(),
    };
    (StatusCode::NOT_IMPLEMENTED, Json(error)).into_response()
}
