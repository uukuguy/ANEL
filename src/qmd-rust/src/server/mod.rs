// QMD HTTP Server module
// Provides independent HTTP API server with REST endpoints

pub mod handlers;
pub mod middleware;
pub mod observability;

use crate::config::Config;
use crate::llm::Router;
use crate::store::Store;
use anyhow::Result;
use axum::Router as AxumRouter;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use middleware::{RateLimitState, AuthState};
use observability::{Metrics, Tracing};

/// QMD HTTP Server state
#[derive(Clone)]
pub struct ServerState {
    pub store: Arc<Mutex<Store>>,
    pub llm: Arc<Mutex<Router>>,
    pub config: Config,
    pub rate_limit_state: Arc<RateLimitState>,
    pub auth_state: Arc<AuthState>,
    pub auth_enabled: bool,
    pub metrics: Arc<Metrics>,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    /// Rate limit: max requests per window
    pub rate_limit_max: usize,
    /// Rate limit: window in seconds
    pub rate_limit_window_secs: u64,
    /// Enable authentication
    pub auth_enabled: bool,
    /// API keys (key -> description)
    pub api_keys: Vec<(String, String)>,
    /// Whitelist IPs (skip auth)
    pub whitelist_ips: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: 4,
            rate_limit_max: 100,
            rate_limit_window_secs: 60,
            auth_enabled: false,
            api_keys: vec![],
            whitelist_ips: vec![],
        }
    }
}

/// Initialize logging for the server
pub fn init_logging() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "qmd_server=debug,tower=warn,axum=warn".into());

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Run the HTTP server
pub fn run_server(config: &ServerConfig, app_config: &Config) -> Result<()> {
    // Initialize runtime
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        // Create server state
        let store = Store::new(app_config)?;
        let llm = Router::new(app_config)?;

        // Create rate limiter state
        let rate_limit_state = Arc::new(RateLimitState::new(
            config.rate_limit_max,
            config.rate_limit_window_secs,
        ));

        // Create auth state
        let auth_state = Arc::new(AuthState::new(
            config.api_keys.clone(),
            config.whitelist_ips.clone(),
        ));

        // Create metrics
        let metrics = Arc::new(Metrics::new());

        let state = ServerState {
            store: Arc::new(Mutex::new(store)),
            llm: Arc::new(Mutex::new(llm)),
            config: app_config.clone(),
            rate_limit_state,
            auth_state,
            auth_enabled: config.auth_enabled,
            metrics,
        };

        // Build router with all routes
        let app = build_router(state)?;

        // Bind address
        let addr: SocketAddr = format!("{}:{}", config.host, config.port)
            .parse()
            .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], config.port)));

        // Start server
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("QMD HTTP Server listening on http://{}", addr);
        tracing::info!("API endpoints available:");
        tracing::info!("  GET  /health          - Health check");
        tracing::info!("  GET  /collections     - List collections");
        tracing::info!("  POST /search          - BM25 search");
        tracing::info!("  POST /vsearch         - Vector search");
        tracing::info!("  POST /query           - Hybrid search (BM25 + Vector + RRF + Rerank)");
        tracing::info!("  GET  /stats           - Index statistics");
        tracing::info!("  GET  /metrics        - Prometheus metrics");
        tracing::info!("  GET  /documents/:path - Get document content");
        tracing::info!("  POST /mcp             - MCP protocol (JSON-RPC)");
        if config.auth_enabled {
            tracing::info!("  Auth: API Key required (X-API-Key header)");
        }
        if !config.whitelist_ips.is_empty() {
            tracing::info!("  Auth: Whitelisted IPs: {:?}", config.whitelist_ips);
        }
        tracing::info!("  Rate limit: {} req/{}s", config.rate_limit_max, config.rate_limit_window_secs);

        axum::serve(listener, app).await?;
        Ok::<(), anyhow::Error>(())
    })
}

/// Build the router with all endpoints
fn build_router(state: ServerState) -> Result<AxumRouter> {
    use axum::routing::{get, post};
    use tower_http::cors::{Any, CorsLayer};

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = AxumRouter::new()
        // Health and info
        .route("/health", get(handlers::health))
        .route("/collections", get(handlers::list_collections))
        .route("/stats", get(handlers::stats))
        .route("/metrics", get(handlers::metrics))
        // Search endpoints
        .route("/search", post(handlers::search))
        .route("/vsearch", post(handlers::vsearch))
        .route("/query", post(handlers::query))
        // Document retrieval
        .route("/documents/:path", get(handlers::get_document))
        // MCP protocol
        .route("/mcp", post(handlers::mcp))
        .layer(cors)
        .with_state(state);

    Ok(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.workers, 4);
    }
}
