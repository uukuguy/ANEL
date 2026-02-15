// HTTP request handlers

use crate::server::ServerState;
use crate::store::SearchOptions;
use axum::{
    extract::{Path, Query, State},
    http::{header::HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};

/// Check rate limit and authentication
/// Returns None if passed, Some(response) if failed
pub async fn check_rate_limit_and_auth(
    state: &ServerState,
    headers: &HeaderMap,
) -> Option<impl IntoResponse> {
    let client_ip = get_client_ip(headers);

    // Check rate limit
    let (allowed, remaining, reset_secs) = state.rate_limit_state.check(&client_ip).await;
    if !allowed {
        let error = ErrorResponse {
            error: "Rate limit exceeded".to_string(),
            code: "RATE_LIMIT_EXCEEDED".to_string(),
        };
        return Some((StatusCode::TOO_MANY_REQUESTS, Json(error)));
    }

    // Check authentication if enabled
    if state.auth_enabled {
        let api_key = headers.get("x-api-key").and_then(|v| v.to_str().ok());
        if !state.auth_state.is_allowed(api_key, &client_ip).await {
            let error = ErrorResponse {
                error: "Authentication required".to_string(),
                code: "UNAUTHORIZED".to_string(),
            };
            return Some((StatusCode::UNAUTHORIZED, Json(error)));
        }
    }

    None
}

/// Helper to extract client IP from headers
fn get_client_ip(headers: &HeaderMap) -> String {
    // Check X-Forwarded-For header first
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            return forwarded_str.split(',').next().unwrap_or("unknown").trim().to_string();
        }
    }

    // Check X-Real-IP header
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }

    "unknown".to_string()
}

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
pub async fn query(
    State(state): State<ServerState>,
    Json(req): Json<SearchRequest>,
) -> impl IntoResponse {
    let query = req.query.clone();
    let limit = req.limit.unwrap_or(20);
    let collection = req.collection.clone();
    let search_all = collection.is_none();

    // Search options for BM25 and Vector
    let options = SearchOptions {
        limit: limit * 2, // Fetch more for reranking
        min_score: 0.0,
        collection: collection.clone(),
        search_all,
    };

    // Step 1: BM25 search (hold Store lock)
    let bm25_results = {
        let store = state.store.lock().await;
        store.bm25_search(&query, options.clone())
    };

    let bm25_results = match bm25_results {
        Ok(r) => r,
        Err(_) => vec![],
    };

    // Step 2: Vector search (release Store lock first, then re-acquire)
    let vector_results = {
        // Generate embedding
        let embedding = {
            let llm = state.llm.lock().await;
            match llm.embed(&[&query]).await {
                Ok(e) => e.embeddings.into_iter().next().unwrap_or_default(),
                Err(_) => vec![],
            }
        };

        if embedding.is_empty() {
            vec![]
        } else {
            let store = state.store.lock().await;
            match store.vector_search_with_embedding(&embedding, options) {
                Ok(r) => r,
                Err(_) => vec![],
            }
        }
    };

    // Step 3: RRF Fusion (no locks held)
    let fused_results = {
        use crate::store::Store;
        let result_lists = [bm25_results.clone(), vector_results.clone()];
        Store::rrf_fusion(
            &result_lists,
            None,
            limit as u32,
        )
    };

    // Step 4: LLM Reranking (hold LLM lock)
    // rerank returns Vec<f32> (scores), need to reorder results
    let final_results = if !fused_results.is_empty() {
        let scores = {
            let llm = state.llm.lock().await;
            match llm.rerank(&query, &fused_results).await {
                Ok(s) => s,
                Err(_) => fused_results.iter().map(|r| r.score).collect(),
            }
        };

        // Reorder fused_results based on rerank scores
        let mut paired: Vec<(crate::store::SearchResult, f32)> = fused_results
            .into_iter()
            .zip(scores.into_iter())
            .collect();

        // Sort by rerank score descending
        paired.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Update scores and return
        paired
            .into_iter()
            .map(|(mut r, score)| {
                r.score = score;
                r
            })
            .collect()
    } else {
        fused_results
    };

    let dtos: Vec<SearchResultDto> = final_results
        .into_iter()
        .map(|r| SearchResultDto {
            docid: r.docid,
            collection: r.collection,
            title: r.title,
            path: r.path,
            score: r.score,
            lines: r.lines,
        })
        .collect();

    let response = SearchResponse {
        total: dtos.len(),
        results: dtos,
        query: req.query,
    };

    Json(response)
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
/// Note: For production, use standalone MCP HTTP server: `qmd mcp --transport http --port 8081`
/// This endpoint provides basic MCP protocol info
pub async fn mcp(
    State(_state): State<ServerState>,
    _body: String,
) -> impl IntoResponse {
    // For now, return instructions to use standalone MCP server
    // Full MCP integration requires more complex state management
    let error = ErrorResponse {
        error: "Use standalone MCP server instead: qmd mcp --transport http --port 8081".to_string(),
        code: "MCP_USE_STANDALONE".to_string(),
    };
    (StatusCode::NOT_IMPLEMENTED, Json(error)).into_response()
}

/// Prometheus metrics endpoint
pub async fn metrics(State(state): State<ServerState>) -> impl IntoResponse {
    let m = &state.metrics;

    // Generate simple Prometheus format output
    let output = format!(
r#"# HELP qmd_http_requests_total Total HTTP requests received
# TYPE qmd_http_requests_total counter
qmd_http_requests_total {}

# HELP qmd_http_requests_in_flight Requests currently being processed
# TYPE qmd_http_requests_in_flight gauge
qmd_http_requests_in_flight {}

# HELP qmd_http_request_duration_seconds HTTP request latency
# TYPE qmd_http_request_duration_seconds histogram
qmd_http_request_duration_seconds_bucket{{le="0.005"}} 0
qmd_http_request_duration_seconds_bucket{{le="0.01"}} 0
qmd_http_request_duration_seconds_bucket{{le="0.05"}} 0
qmd_http_request_duration_seconds_bucket{{le="0.1"}} 0
qmd_http_request_duration_seconds_bucket{{le="0.5"}} 0
qmd_http_request_duration_seconds_bucket{{le="1"}} 0
qmd_http_request_duration_seconds_bucket{{le="+Inf"}} 0
qmd_http_request_duration_seconds_sum 0
qmd_http_request_duration_seconds_count 0

# HELP qmd_search_total Total BM25 search requests
# TYPE qmd_search_total counter
qmd_search_total {}

# HELP qmd_vsearch_total Total vector search requests
# TYPE qmd_vsearch_total counter
qmd_vsearch_total {}

# HELP qmd_query_total Total hybrid query requests
# TYPE qmd_query_total counter
qmd_query_total {}

# HELP qmd_errors_total Total errors
# TYPE qmd_errors_total counter
qmd_errors_total {}

# HELP qmd_llm_embeddings_total Total embedding requests
# TYPE qmd_llm_embeddings_total counter
qmd_llm_embeddings_total {}

# HELP qmd_llm_rerank_total Total rerank requests
# TYPE qmd_llm_rerank_total counter
qmd_llm_rerank_total {}

# HELP qmd_llm_errors_total Total LLM errors
# TYPE qmd_llm_errors_total counter
qmd_llm_errors_total {}
"#,
        m.get_requests_total(),
        m.get_requests_in_flight(),
        m.get_search_total(),
        m.get_vsearch_total(),
        m.get_query_total(),
        m.get_errors_total(),
        m.get_llm_embeddings_total(),
        m.get_llm_rerank_total(),
        m.get_llm_errors()
    );

    (StatusCode::OK, [("Content-Type", "text/plain; version=0.0.4")], output)
}
