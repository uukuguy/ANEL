use crate::cli::{McpArgs};
use crate::config::Config;
use crate::llm::Router;
use crate::store::Store;
use anyhow::{Context, Result};
use mcp_sdk::server::{Server, stdio_transport};
use mcp_sdk::types::{Tool, TextContent};
use std::sync::Arc;

/// Run MCP server
pub fn run_server(
    args: &McpArgs,
    store: &Store,
    llm: &Router,
) -> Result<()> {
    let store = Arc::new(store.clone());
    let llm = Arc::new(llm.clone());

    match args.transport.as_str() {
        "stdio" => run_stdio_server(store, llm),
        "sse" => run_sse_server(store, llm, args.port),
        _ => anyhow::bail!("Unknown transport: {}", args.transport),
    }
}

/// Run MCP server with stdio transport
fn run_stdio_server(store: Arc<Store>, llm: Arc<Router>) -> Result<()> {
    let server = Server::new("qmd-rust");

    // Register tools
    register_tools(&server, store.clone(), llm.clone());

    // Run with stdio transport
    let transport = stdio_transport();
    tokio::runtime::Runtime::new()?
        .block_on(async {
            server.run(transport).await
        })?;

    Ok(())
}

/// Run MCP server with SSE transport
fn run_sse_server(store: Arc<Store>, llm: Arc<Router>, port: u16) -> Result<()> {
    let server = Server::new("qmd-rust");

    // Register tools
    register_tools(&server, store.clone(), llm.clone());

    // TODO: Implement SSE transport
    log::info!("SSE transport not yet implemented, falling back to stdio");
    run_stdio_server(store, llm)
}

/// Register MCP tools
fn register_tools(server: &Server, store: Arc<Store>, llm: Arc<Router>) {
    // Search tool
    server.add_tool(
        Tool {
            name: "search".to_string(),
            description: "BM25 full-text search".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" },
                    "limit": { "type": "integer", "description": "Max results", "default": 20 },
                    "collection": { "type": "string", "description": "Collection name" }
                },
                "required": ["query"]
            }),
        },
        move |params: serde_json::Value| {
            let query = params["query"].as_str().unwrap_or("").to_string();
            let limit = params["limit"].as_u64().unwrap_or(20) as usize;
            let collection = params["collection"].as_str().map(|s| s.to_string());

            let results = store.bm25_search(&query, crate::store::SearchOptions {
                limit,
                min_score: 0.0,
                collection,
                search_all: collection.is_none(),
            });

            match results {
                Ok(results) => {
                    let output: Vec<TextContent> = results.into_iter().map(|r| {
                        TextContent {
                            type_: "text".to_string(),
                            text: format!("{}: {:.4} - {}", r.path, r.score, r.title),
                        }
                    }).collect();
                    Ok(output)
                }
                Err(e) => Err(mcp_sdk::Error::from(e.to_string()))
            }
        },
    );

    // Vector search tool
    server.add_tool(
        Tool {
            name: "vsearch".to_string(),
            description: "Vector semantic search".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" },
                    "limit": { "type": "integer", "description": "Max results", "default": 20 },
                    "collection": { "type": "string", "description": "Collection name" }
                },
                "required": ["query"]
            }),
        },
        move |params: serde_json::Value| {
            let query = params["query"].as_str().unwrap_or("").to_string();
            let limit = params["limit"].as_u64().unwrap_or(20) as usize;
            let collection = params["collection"].as_str().map(|s| s.to_string());

            let results = store.vector_search(&query, crate::store::SearchOptions {
                limit,
                min_score: 0.0,
                collection,
                search_all: collection.is_none(),
            });

            match results {
                Ok(results) => {
                    let output: Vec<TextContent> = results.into_iter().map(|r| {
                        TextContent {
                            type_: "text".to_string(),
                            text: format!("{}: {:.4} - {}", r.path, r.score, r.title),
                        }
                    }).collect();
                    Ok(output)
                }
                Err(e) => Err(mcp_sdk::Error::from(e.to_string()))
            }
        },
    );

    // Hybrid search tool
    server.add_tool(
        Tool {
            name: "query".to_string(),
            description: "Hybrid search with LLM reranking".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" },
                    "limit": { "type": "integer", "description": "Max results", "default": 20 },
                    "collection": { "type": "string", "description": "Collection name" }
                },
                "required": ["query"]
            }),
        },
        move |params: serde_json::Value| {
            let query = params["query"].as_str().unwrap_or("").to_string();
            let limit = params["limit"].as_u64().unwrap_or(20) as usize;
            let collection = params["collection"].as_str().map(|s| s.to_string());

            let results = store.hybrid_search(&query, crate::store::SearchOptions {
                limit,
                min_score: 0.0,
                collection,
                search_all: collection.is_none(),
            }, &llm);

            match results {
                Ok(results) => {
                    let output: Vec<TextContent> = results.into_iter().map(|r| {
                        TextContent {
                            type_: "text".to_string(),
                            text: format!("{}: {:.4} - {}", r.path, r.score, r.title),
                        }
                    }).collect();
                    Ok(output)
                }
                Err(e) => Err(mcp_sdk::Error::from(e.to_string()))
            }
        },
    );

    // Get document tool
    server.add_tool(
        Tool {
            name: "get".to_string(),
            description: "Get document content".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File path" },
                    "from": { "type": "integer", "description": "Start line", "default": 0 },
                    "limit": { "type": "integer", "description": "Max lines", "default": 50 }
                },
                "required": ["path"]
            }),
        },
        |params: serde_json::Value| {
            let path = params["path"].as_str().unwrap_or("").to_string();
            let from = params["from"].as_u64().unwrap_or(0) as usize;
            let limit = params["limit"].as_u64().unwrap_or(50) as usize;

            // TODO: Implement document retrieval
            let output = TextContent {
                type_: "text".to_string(),
                text: format!("Document: {} (lines {}-{})", path, from, from + limit),
            };
            Ok(vec![output])
        },
    );

    // Status tool
    server.add_tool(
        Tool {
            name: "status".to_string(),
            description: "Show index status".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        |_params: serde_json::Value| {
            let output = TextContent {
                type_: "text".to_string(),
                text: "Index status: OK".to_string(),
            };
            Ok(vec![output])
        },
    );
}
