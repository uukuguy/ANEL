use crate::anel::{self, TraceContext};
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
use rmcp::service::{RequestContext, RoleServer, ServiceExt};
use rmcp::{tool, tool_router, ErrorData as McpError, ServerHandler};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::convert::Infallible;
use std::future::Future;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

// ── Stream Tap (Audit Layer) ─────────────────────────────────────

/// Audit logger that records every MCP tool invocation as NDJSON to stderr.
#[derive(Clone, Debug)]
struct StreamTap {
    identity: Option<String>,
    trace_id: String,
}

impl StreamTap {
    fn new() -> Self {
        let ctx = TraceContext::from_env();
        Self {
            identity: std::env::var(anel::env::IDENTITY_TOKEN).ok(),
            trace_id: ctx.get_or_generate_trace_id(),
        }
    }

    fn log(&self, tool_name: &str, args_summary: &str, status: &str, duration_ms: u64) {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let record = serde_json::json!({
            "type": "audit",
            "timestamp": timestamp_ms,
            "tool": tool_name,
            "trace_id": self.trace_id,
            "X-Agent-Identity": self.identity,
            "args": args_summary,
            "status": status,
            "duration_ms": duration_ms,
        });
        eprintln!("{}", serde_json::to_string(&record).unwrap_or_default());
    }
}

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
    tap: StreamTap,
    dry_run: bool,
}

// ── Dry-run / audit helpers (outside #[tool_router] block) ───────

impl QmdMcpServer {
    fn check_dry_run(&self, tool_name: &str, args: &str) -> Option<CallToolResult> {
        if self.dry_run {
            self.tap.log(tool_name, args, "dry-run", 0);
            Some(CallToolResult::success(vec![Content::text(
                format!("[DRY-RUN] Would execute tool '{}' with args: {}", tool_name, args),
            )]))
        } else {
            None
        }
    }
}

#[tool_router]
impl QmdMcpServer {
    pub fn new(config: Config) -> Result<Self> {
        let store = Store::new(&config)?;
        let llm = Router::new(&config)?;
        let dry_run = std::env::var(anel::env::DRY_RUN)
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let tap = StreamTap::new();
        Ok(Self {
            store: Arc::new(Mutex::new(store)),
            llm: Arc::new(tokio::sync::Mutex::new(llm)),
            tool_router: Self::tool_router(),
            tap,
            dry_run,
        })
    }

    #[tool(description = "BM25 full-text search across indexed documents")]
    async fn search(
        &self,
        params: Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = params.0;
        let args_summary = serde_json::to_string(&serde_json::json!({
            "query": &p.query, "limit": p.limit, "collection": &p.collection
        })).unwrap_or_default();

        if let Some(result) = self.check_dry_run("search", &args_summary) {
            return Ok(result);
        }

        let start = Instant::now();
        let options = make_search_options(&p);
        let store = self.store.lock().map_err(|e| {
            self.tap.log("search", &args_summary, "error", start.elapsed().as_millis() as u64);
            McpError::internal_error(format!("Store lock failed: {e}"), None)
        })?;
        match store.bm25_search(&p.query, options) {
            Ok(results) => {
                self.tap.log("search", &args_summary, "ok", start.elapsed().as_millis() as u64);
                Ok(CallToolResult::success(vec![Content::text(
                    format_search_results(&results),
                )]))
            }
            Err(e) => {
                self.tap.log("search", &args_summary, "error", start.elapsed().as_millis() as u64);
                Err(McpError::internal_error(
                    format!("BM25 search failed: {e}"),
                    None,
                ))
            }
        }
    }

    #[tool(description = "Vector semantic search using document embeddings")]
    async fn vsearch(
        &self,
        params: Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = params.0;
        let args_summary = serde_json::to_string(&serde_json::json!({
            "query": &p.query, "limit": p.limit, "collection": &p.collection
        })).unwrap_or_default();

        if let Some(result) = self.check_dry_run("vsearch", &args_summary) {
            return Ok(result);
        }

        let start = Instant::now();
        let options = make_search_options(&p);

        // Step 1: Generate embedding (async, no store lock)
        let embedding = {
            let llm = self.llm.lock().await;
            llm.embed(&[p.query.as_str()]).await.map_err(|e| {
                self.tap.log("vsearch", &args_summary, "error", start.elapsed().as_millis() as u64);
                McpError::internal_error(format!("Embedding failed: {e}"), None)
            })?
        };
        let query_vector = &embedding.embeddings[0];

        // Step 2: Sync vector search with pre-computed embedding
        let store = self.store.lock().map_err(|e| {
            self.tap.log("vsearch", &args_summary, "error", start.elapsed().as_millis() as u64);
            McpError::internal_error(format!("Store lock failed: {e}"), None)
        })?;
        match store.vector_search_with_embedding(query_vector, options) {
            Ok(results) => {
                self.tap.log("vsearch", &args_summary, "ok", start.elapsed().as_millis() as u64);
                Ok(CallToolResult::success(vec![Content::text(
                    format_search_results(&results),
                )]))
            }
            Err(e) => {
                self.tap.log("vsearch", &args_summary, "error", start.elapsed().as_millis() as u64);
                Err(McpError::internal_error(
                    format!("Vector search failed: {e}"),
                    None,
                ))
            }
        }
    }

    #[tool(description = "Hybrid search combining BM25 and vector search with RRF fusion")]
    async fn query(
        &self,
        params: Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = params.0;
        let args_summary = serde_json::to_string(&serde_json::json!({
            "query": &p.query, "limit": p.limit, "collection": &p.collection
        })).unwrap_or_default();

        if let Some(result) = self.check_dry_run("query", &args_summary) {
            return Ok(result);
        }

        let start = Instant::now();
        let options = make_search_options(&p);

        // Step 1: Query expansion (sync LLM call)
        let expanded_queries = {
            let llm = self.llm.lock().await;
            llm.expand_query(&p.query).map_err(|e| {
                self.tap.log("query", &args_summary, "error", start.elapsed().as_millis() as u64);
                McpError::internal_error(format!("Query expansion failed: {e}"), None)
            })?
        };

        // Step 2: BM25 retrieval for all expanded queries
        let all_bm25_results = {
            let store = self.store.lock().map_err(|e| {
                self.tap.log("query", &args_summary, "error", start.elapsed().as_millis() as u64);
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
                    self.tap.log("query", &args_summary, "error", start.elapsed().as_millis() as u64);
                    McpError::internal_error(format!("Embedding failed: {e}"), None)
                })?
            };
            let query_vector = &embedding.embeddings[0];
            let store = self.store.lock().map_err(|e| {
                self.tap.log("query", &args_summary, "error", start.elapsed().as_millis() as u64);
                McpError::internal_error(format!("Store lock failed: {e}"), None)
            })?;
            store.vector_search_with_embedding(query_vector, options.clone()).map_err(|e| {
                self.tap.log("query", &args_summary, "error", start.elapsed().as_millis() as u64);
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

        self.tap.log("query", &args_summary, "ok", start.elapsed().as_millis() as u64);
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
        let args_summary = serde_json::to_string(&serde_json::json!({
            "path": &p.path, "from": p.from, "limit": p.limit
        })).unwrap_or_default();

        if let Some(result) = self.check_dry_run("get", &args_summary) {
            return Ok(result);
        }

        let start = Instant::now();
        let from = p.from.unwrap_or(0);
        let limit = p.limit.unwrap_or(50);

        match std::fs::read_to_string(&p.path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let total = lines.len();
                let line_start = from.min(total);
                let end = (line_start + limit).min(total);
                let selected = &lines[line_start..end];
                let text = format!(
                    "File: {} (lines {}-{} of {})\n\n{}",
                    p.path,
                    line_start + 1,
                    end,
                    total,
                    selected.join("\n")
                );
                self.tap.log("get", &args_summary, "ok", start.elapsed().as_millis() as u64);
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => {
                self.tap.log("get", &args_summary, "error", start.elapsed().as_millis() as u64);
                Err(McpError::internal_error(
                    format!("Failed to read file '{}': {e}", p.path),
                    None,
                ))
            }
        }
    }

    #[tool(description = "Show index statistics including document counts and collection info")]
    async fn status(&self) -> Result<CallToolResult, McpError> {
        let args_summary = "{}".to_string();

        if let Some(result) = self.check_dry_run("status", &args_summary) {
            return Ok(result);
        }

        let start = Instant::now();
        let store = self.store.lock().map_err(|e| {
            self.tap.log("status", &args_summary, "error", start.elapsed().as_millis() as u64);
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
                self.tap.log("status", &args_summary, "ok", start.elapsed().as_millis() as u64);
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => {
                self.tap.log("status", &args_summary, "error", start.elapsed().as_millis() as u64);
                Err(McpError::internal_error(
                    format!("Failed to get stats: {e}"),
                    None,
                ))
            }
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

    fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourceTemplatesResult, McpError>> + Send + '_ {
        // Return empty templates - resources are accessed via 'get' tool
        std::future::ready(Ok(ListResourceTemplatesResult::default()))
    }

    fn read_resource(
        &self,
        _request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        // Return method not found - resources are accessed via 'get' tool instead
        std::future::ready(Err(McpError::method_not_found::<ReadResourceRequestMethod>()))
    }
}

// ── Public entry point ───────────────────────────────────────────

use crate::anel::AnelSpec;

pub fn run_server(args: &McpArgs, config: &Config) -> Result<()> {
    // Handle --emit-spec: output ANEL specification and exit
    if args.emit_spec {
        let spec = AnelSpec::mcp();
        println!("{}", serde_json::to_string_pretty(&spec)?);
        return Ok(());
    }

    // Handle --dry-run: validate parameters without executing
    if args.dry_run {
        println!("[DRY-RUN] Would execute mcp server with:");
        println!("  transport: {}", args.transport);
        println!("  port: {}", args.port);
        println!("  format: {}", args.format);
        return Ok(());
    }

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
