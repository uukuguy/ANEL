use crate::cli::McpArgs;
use crate::config::Config;
use crate::llm::Router;
use crate::store::{SearchOptions, Store};
use anyhow::Result;
use bytes::Bytes;
use http_body_util::BodyExt;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::service::ServiceExt;
use rmcp::{tool, tool_router, ErrorData as McpError, ServerHandler};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::convert::Infallible;

// ── Parameter types ──────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchParams {
    /// Search query string
    pub query: String,
    /// Maximum number of results (default: 20)
    pub limit: Option<usize>,
    /// Collection name to search in
    pub collection: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetParams {
    /// File path of the document to retrieve
    pub path: String,
    /// Start line number (0-based, default: 0)
    pub from: Option<usize>,
    /// Maximum number of lines to return (default: 50)
    pub limit: Option<usize>,
}

// ── MCP Server ───────────────────────────────────────────────────

#[derive(Clone)]
pub struct QmdMcpServer {
    store: Arc<Mutex<Store>>,
    llm: Arc<tokio::sync::Mutex<Router>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl QmdMcpServer {
    pub fn new(config: Config) -> Result<Self> {
        let store = Store::new(&config)?;
        let llm = Router::new(&config)?;
        Ok(Self {
            store: Arc::new(Mutex::new(store)),
            llm: Arc::new(tokio::sync::Mutex::new(llm)),
            tool_router: Self::tool_router(),
        })
    }

    #[tool(description = "BM25 full-text search across indexed documents")]
    async fn search(
        &self,
        params: Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = params.0;
        let options = make_search_options(&p);
        let store = self.store.lock().map_err(|e| {
            McpError::internal_error(format!("Store lock failed: {e}"), None)
        })?;
        match store.bm25_search(&p.query, options) {
            Ok(results) => Ok(CallToolResult::success(vec![Content::text(
                format_search_results(&results),
            )])),
            Err(e) => Err(McpError::internal_error(
                format!("BM25 search failed: {e}"),
                None,
            )),
        }
    }

    #[tool(description = "Vector semantic search using document embeddings")]
    async fn vsearch(
        &self,
        params: Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = params.0;
        let options = make_search_options(&p);

        // Step 1: Generate embedding (async, no store lock)
        let embedding = {
            let llm = self.llm.lock().await;
            llm.embed(&[p.query.as_str()]).await.map_err(|e| {
                McpError::internal_error(format!("Embedding failed: {e}"), None)
            })?
        };
        let query_vector = &embedding.embeddings[0];

        // Step 2: Sync vector search with pre-computed embedding
        let store = self.store.lock().map_err(|e| {
            McpError::internal_error(format!("Store lock failed: {e}"), None)
        })?;
        match store.vector_search_with_embedding(query_vector, options) {
            Ok(results) => Ok(CallToolResult::success(vec![Content::text(
                format_search_results(&results),
            )])),
            Err(e) => Err(McpError::internal_error(
                format!("Vector search failed: {e}"),
                None,
            )),
        }
    }

    #[tool(description = "Hybrid search combining BM25 and vector search with RRF fusion")]
    async fn query(
        &self,
        params: Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = params.0;
        let options = make_search_options(&p);

        // Step 1: Query expansion (sync LLM call)
        let expanded_queries = {
            let llm = self.llm.lock().await;
            llm.expand_query(&p.query).map_err(|e| {
                McpError::internal_error(format!("Query expansion failed: {e}"), None)
            })?
        };

        // Step 2: BM25 retrieval for all expanded queries
        let all_bm25_results = {
            let store = self.store.lock().map_err(|e| {
                McpError::internal_error(format!("Store lock failed: {e}"), None)
            })?;
            let mut results = Vec::new();
            for eq in &expanded_queries {
                if let Ok(r) = store.bm25_search(eq, options.clone()) {
                    results.extend(r);
                }
            }
            results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            results.truncate(100);
            results
        };

        // Step 3: Vector search (embed async, then sync DB query)
        let vector_results = {
            let embedding = {
                let llm = self.llm.lock().await;
                llm.embed(&[p.query.as_str()]).await.map_err(|e| {
                    McpError::internal_error(format!("Embedding failed: {e}"), None)
                })?
            };
            let query_vector = &embedding.embeddings[0];
            let store = self.store.lock().map_err(|e| {
                McpError::internal_error(format!("Store lock failed: {e}"), None)
            })?;
            store.vector_search_with_embedding(query_vector, options.clone()).map_err(|e| {
                McpError::internal_error(format!("Vector search failed: {e}"), None)
            })?
        };

        // Step 4: RRF fusion
        let result_lists = vec![all_bm25_results, vector_results];
        let weights = Some(vec![1.0, 1.5]);
        let mut fused = Store::rrf_fusion(&result_lists, weights, 60);
        fused.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        let candidates: Vec<_> = fused.into_iter().take(30).collect();

        // Step 5: Try LLM reranking
        let llm = self.llm.lock().await;
        let final_results = if llm.has_reranker() {
            match llm.rerank(&p.query, &candidates).await {
                Ok(scores) => {
                    let mut reranked: Vec<_> = candidates
                        .into_iter()
                        .zip(scores)
                        .map(|(mut doc, score)| {
                            doc.score = score;
                            doc
                        })
                        .collect();
                    reranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
                    reranked
                }
                Err(_) => candidates,
            }
        } else {
            candidates
        };

        Ok(CallToolResult::success(vec![Content::text(
            format_search_results(&final_results),
        )]))
    }

    #[tool(description = "Get document content by file path with optional line range")]
    async fn get(
        &self,
        params: Parameters<GetParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = params.0;
        let from = p.from.unwrap_or(0);
        let limit = p.limit.unwrap_or(50);

        match std::fs::read_to_string(&p.path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let total = lines.len();
                let start = from.min(total);
                let end = (start + limit).min(total);
                let selected = &lines[start..end];
                let text = format!(
                    "File: {} (lines {}-{} of {})\n\n{}",
                    p.path,
                    start + 1,
                    end,
                    total,
                    selected.join("\n")
                );
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(
                format!("Failed to read file '{}': {e}", p.path),
                None,
            )),
        }
    }

    #[tool(description = "Show index statistics including document counts and collection info")]
    async fn status(&self) -> Result<CallToolResult, McpError> {
        let store = self.store.lock().map_err(|e| {
            McpError::internal_error(format!("Store lock failed: {e}"), None)
        })?;
        match store.get_stats() {
            Ok(stats) => {
                let mut text = format!(
                    "Index Status:\n  Collections: {}\n  Documents: {}\n  Indexed: {}\n  Pending: {}\n  Chunks: {}\n",
                    stats.collection_count,
                    stats.document_count,
                    stats.indexed_count,
                    stats.pending_count,
                    stats.chunk_count,
                );
                if !stats.collection_stats.is_empty() {
                    text.push_str("\nPer-collection:\n");
                    for (name, count) in &stats.collection_stats {
                        text.push_str(&format!("  {}: {} docs\n", name, count));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(
                format!("Failed to get stats: {e}"),
                None,
            )),
        }
    }
}

impl ServerHandler for QmdMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("QMD - AI-powered document search with hybrid BM25 and vector search. Use 'search' for keyword matching, 'vsearch' for semantic search, 'query' for best results combining both, 'get' to read document content, and 'status' to check index health.".into()),
            ..Default::default()
        }
    }
}

// ── Public entry point ───────────────────────────────────────────

pub fn run_server(args: &McpArgs, config: &Config) -> Result<()> {
    match args.transport.as_str() {
        "stdio" => run_stdio_server(config),
        "http" | "sse" => run_http_server(args, config),
        _ => anyhow::bail!("Unknown transport: {}", args.transport),
    }
}

fn run_stdio_server(config: &Config) -> Result<()> {
    let server = QmdMcpServer::new(config.clone())?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let transport = rmcp::transport::io::stdio();
        let service = server.serve(transport).await?;
        service.waiting().await?;
        Ok(())
    })
}

fn run_http_server(args: &McpArgs, config: &Config) -> Result<()> {
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService,
    };
    use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
    use std::sync::Arc;
    use axum::routing::post;
    use axum::Router;
    use tower::ServiceExt;
    use std::net::SocketAddr;
    use http_body_util::combinators::BoxBody;

    // Create server instance
    let server = QmdMcpServer::new(config.clone())?;

    // Get server for service factory
    let server_clone = server.clone();

    // Configure HTTP server
    let server_config = StreamableHttpServerConfig::default();

    // Use LocalSessionManager for simple in-memory session handling
    let session_manager: Arc<LocalSessionManager> = Arc::new(LocalSessionManager::default());

    // Create HTTP service with service factory
    let http_service = StreamableHttpService::new(
        move || Ok(server_clone.clone()),
        session_manager.clone(),
        server_config,
    );

    // Create axum app with MCP endpoint
    let app = Router::new()
        .route("/mcp", post(move |req| {
            let mut service = http_service.clone();
            async move {
                let response: Result<axum::http::Response<BoxBody<Bytes, Infallible>>, _> = ServiceExt::oneshot(&mut service, req).await;
                match response {
                    Ok(resp) => resp,
                    Err(e) => {
                        log::error!("MCP service error: {}", e);
                        axum::http::Response::new(
                            http_body_util::Full::new(Bytes::from(format!("Error: {}", e)))
                                .boxed()
                        )
                    }
                }
            }
        }));

    // Bind and run HTTP server
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    let rt = tokio::runtime::Runtime::new()?;

    log::info!("MCP HTTP Server listening on http://{}", addr);
    log::info!("Model will stay loaded in memory for fast subsequent queries");

    rt.block_on(async {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        Ok::<(), anyhow::Error>(())
    })
}

// ── Helpers ──────────────────────────────────────────────────────

fn make_search_options(p: &SearchParams) -> SearchOptions {
    SearchOptions {
        limit: p.limit.unwrap_or(20),
        min_score: 0.0,
        collection: p.collection.clone(),
        search_all: p.collection.is_none(),
    }
}

fn format_search_results(results: &[crate::store::SearchResult]) -> String {
    if results.is_empty() {
        return "No results found.".to_string();
    }
    results
        .iter()
        .enumerate()
        .map(|(i, r)| {
            format!(
                "{}. [{}] {} (score: {:.4}, docid: {})\n   Path: {}",
                i + 1,
                r.collection,
                r.title,
                r.score,
                r.docid,
                r.path,
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
