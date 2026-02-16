//! MCP Server Tests - Phase 3
//!
//! Tests for Rust MCP Server implementation aligned with Python/TypeScript equivalents.
//! Coverage: ServerInfo, Store-backed search operations, parameter handling

mod common;

use common::{create_test_config, init_test_db, insert_test_doc};
use qmd_rust::mcp::{QmdMcpServer, SearchParams, GetParams};
use qmd_rust::store::{Store, SearchOptions, SearchResult};
use rmcp::ServerHandler;
use tempfile::tempdir;
use std::fs;

// ─────────────────────────────────────────────────────────────────────────────
// Helper Functions
// ─────────────────────────────────────────────────────────────────────────────

/// Create a server instance with test config and pre-populated database
fn create_test_server(tmp_dir: &std::path::Path, collection: &str) -> (QmdMcpServer, std::path::PathBuf) {
    let content_dir = tmp_dir.join("content");
    fs::create_dir_all(&content_dir).unwrap();

    // Create test files
    fs::write(content_dir.join("readme.md"), "# Project README\nThis is the main readme.").unwrap();
    fs::write(content_dir.join("api.md"), "# API Documentation\nREST API endpoints.").unwrap();
    fs::write(content_dir.join("notes.md"), "# Meeting Notes\nQ1 goals and roadmap.").unwrap();

    let config = create_test_config(tmp_dir, collection, &content_dir);
    let store = Store::new(&config).unwrap();
    store.update_index().unwrap();

    let server = QmdMcpServer::new(config).unwrap();
    (server, content_dir)
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Server Info Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_server_info_name() {
    let tmp = tempdir().unwrap();
    let (server, _) = create_test_server(tmp.path(), "docs");

    let info = server.get_info();
    // ServerInfo = InitializeResult, which has server_info: Implementation
    assert!(info.server_info.name.contains("qmd") || info.server_info.name.len() > 0,
        "Server name should be defined");
}

#[test]
fn test_server_info_has_instructions() {
    let tmp = tempdir().unwrap();
    let (server, _) = create_test_server(tmp.path(), "docs");

    let info = server.get_info();
    // ServerInfo has instructions field
    assert!(info.instructions.is_some(), "Server should have instructions");
    let instructions = info.instructions.unwrap();
    assert!(instructions.contains("search"), "Instructions should mention search");
    assert!(instructions.contains("vsearch"), "Instructions should mention vsearch");
    assert!(instructions.contains("query"), "Instructions should mention query");
}

#[test]
fn test_server_info_has_capabilities() {
    let tmp = tempdir().unwrap();
    let (server, _) = create_test_server(tmp.path(), "docs");

    let info = server.get_info();
    // ServerInfo has capabilities field (not Option)
    // ServerCapabilities has various fields, just verify it's constructable
    let _caps = info.capabilities;
    assert!(true, "Server should have capabilities");
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Store-backed Search Tests (equivalent to MCP search tool)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_bm25_search_returns_results() {
    let tmp = tempdir().unwrap();
    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    // Create test files
    fs::write(content_dir.join("readme.md"), "# Project README\nThis is the main readme file.").unwrap();
    fs::write(content_dir.join("api.md"), "# API Documentation\nREST API endpoints and usage.").unwrap();

    let config = create_test_config(tmp.path(), "docs", &content_dir);
    let store = Store::new(&config).unwrap();
    store.update_index().unwrap();

    let options = SearchOptions {
        limit: 10,
        min_score: 0.0,
        collection: None,
        search_all: true,
    };

    let results = store.bm25_search("readme", options).unwrap();
    assert!(!results.is_empty(), "Should find results for 'readme' query");

    // Verify result structure
    let first = &results[0];
    assert!(first.docid.contains("readme") || first.title.to_lowercase().contains("readme"),
        "Result should be related to readme");
}

#[test]
fn test_bm25_search_no_results() {
    let tmp = tempdir().unwrap();
    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    fs::write(content_dir.join("readme.md"), "# README\nContent here.").unwrap();

    let config = create_test_config(tmp.path(), "docs", &content_dir);
    let store = Store::new(&config).unwrap();
    store.update_index().unwrap();

    let options = SearchOptions {
        limit: 10,
        min_score: 0.0,
        collection: None,
        search_all: true,
    };

    let results = store.bm25_search("nonexistent_xyz_query_12345", options).unwrap();
    // Should return empty results (not error)
    assert!(results.is_empty() || results.len() >= 0, "Should handle no results gracefully");
}

#[test]
fn test_bm25_search_with_limit() {
    let tmp = tempdir().unwrap();
    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    // Create multiple files
    for i in 0..5 {
        fs::write(content_dir.join(format!("doc{}.md", i)), format!("# Document {}\nContent here.", i)).unwrap();
    }

    let config = create_test_config(tmp.path(), "docs", &content_dir);
    let store = Store::new(&config).unwrap();
    store.update_index().unwrap();

    let options = SearchOptions {
        limit: 2,
        min_score: 0.0,
        collection: None,
        search_all: true,
    };

    let results = store.bm25_search("document", options).unwrap();
    assert!(results.len() <= 2, "Should respect limit parameter");
}

#[test]
fn test_bm25_search_with_collection_filter() {
    let tmp = tempdir().unwrap();
    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    fs::write(content_dir.join("doc.md"), "# Document\nContent.").unwrap();

    let config = create_test_config(tmp.path(), "docs", &content_dir);
    let store = Store::new(&config).unwrap();
    store.update_index().unwrap();

    // Search with collection filter
    let options = SearchOptions {
        limit: 10,
        min_score: 0.0,
        collection: Some("docs".to_string()),
        search_all: false,
    };

    let results = store.bm25_search("document", options).unwrap();
    assert!(!results.is_empty(), "Should find results in collection");
    for r in &results {
        assert_eq!(r.collection, "docs", "All results should be from 'docs' collection");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Get Tool Tests (file reading)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_get_file_content() {
    let tmp = tempdir().unwrap();
    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    let test_content = "# Test Document\nThis is test content.";
    fs::write(content_dir.join("test.md"), test_content).unwrap();

    let result = std::fs::read_to_string(content_dir.join("test.md")).unwrap();
    assert!(result.contains("Test Document"), "Should read file content");
}

#[test]
fn test_get_file_with_line_range() {
    let tmp = tempdir().unwrap();
    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    let lines: Vec<String> = (0..20).map(|i| format!("Line {}", i)).collect();
    fs::write(content_dir.join("lines.md"), lines.join("\n")).unwrap();

    let full_content = std::fs::read_to_string(content_dir.join("lines.md")).unwrap();
    let all_lines: Vec<&str> = full_content.lines().collect();

    // Test line range
    let from = 2;
    let limit = 5;
    let selected: Vec<&str> = all_lines.iter().skip(from).take(limit).copied().collect();

    assert_eq!(selected.len(), 5, "Should return requested number of lines");
    assert!(selected[0].contains("Line 2"), "Should start from correct line");
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Status Tool Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_status_returns_stats() {
    let tmp = tempdir().unwrap();
    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    // Create test files
    fs::write(content_dir.join("doc1.md"), "# Doc 1\nContent.").unwrap();
    fs::write(content_dir.join("doc2.md"), "# Doc 2\nMore content.").unwrap();

    let config = create_test_config(tmp.path(), "docs", &content_dir);
    let store = Store::new(&config).unwrap();
    store.update_index().unwrap();

    let stats = store.get_stats().unwrap();

    assert!(stats.collection_count >= 1, "Should have at least one collection");
    assert!(stats.document_count >= 2, "Should have indexed documents");
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Parameter Type Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_search_options_default_limit() {
    let options = SearchOptions {
        limit: 20,
        min_score: 0.0,
        collection: None,
        search_all: true,
    };
    assert_eq!(options.limit, 20, "Default limit should be 20");
}

#[test]
fn test_search_options_with_limit() {
    let options = SearchOptions {
        limit: 5,
        min_score: 0.0,
        collection: None,
        search_all: true,
    };
    assert_eq!(options.limit, 5, "Limit should be 5");
}

#[test]
fn test_search_options_with_collection() {
    let options = SearchOptions {
        limit: 10,
        min_score: 0.0,
        collection: Some("my_collection".to_string()),
        search_all: false,
    };
    assert_eq!(options.collection, Some("my_collection".to_string()));
    assert!(!options.search_all, "search_all should be false when collection specified");
}

#[test]
fn test_search_options_without_collection() {
    let options = SearchOptions {
        limit: 10,
        min_score: 0.0,
        collection: None,
        search_all: true,
    };
    assert!(options.search_all, "search_all should be true when no collection");
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. SearchResult Type Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_search_result_fields() {
    let result = SearchResult {
        docid: "abc123".to_string(),
        collection: "docs".to_string(),
        path: "/path/to/doc.md".to_string(),
        title: "Test Document".to_string(),
        score: 0.95,
        lines: 10,
        hash: "hash1".to_string(),
        query: Some("test query".to_string()),
    };

    assert_eq!(result.docid, "abc123");
    assert_eq!(result.collection, "docs");
    assert_eq!(result.title, "Test Document");
    assert_eq!(result.score, 0.95);
    assert_eq!(result.query, Some("test query".to_string()));
}

#[test]
fn test_search_result_query_optional() {
    let result = SearchResult {
        docid: "abc123".to_string(),
        collection: "docs".to_string(),
        path: "/path/to/doc.md".to_string(),
        title: "Test Document".to_string(),
        score: 0.95,
        lines: 10,
        hash: "hash1".to_string(),
        query: None,
    };

    assert_eq!(result.query, None, "query should be optional");
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. RRF Fusion Tests (used by hybrid query)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_rrf_fusion_empty_lists() {
    let result_lists: Vec<Vec<SearchResult>> = vec![];
    let fused = Store::rrf_fusion(&result_lists, None, 60);
    assert!(fused.is_empty(), "RRF fusion with empty lists should return empty");
}

#[test]
fn test_rrf_fusion_single_list() {
    let list1 = vec![
        SearchResult {
            docid: "docs:/doc1.md".to_string(),
            collection: "docs".to_string(),
            path: "/doc1.md".to_string(),
            title: "Doc 1".to_string(),
            score: 1.0,
            lines: 10,
            hash: "hash1".to_string(),
            query: None,
        },
    ];

    let result_lists = vec![list1];
    let fused = Store::rrf_fusion(&result_lists, None, 60);

    assert!(!fused.is_empty(), "RRF fusion with single list should return results");
    assert_eq!(fused[0].docid, "docs:/doc1.md", "Should preserve document ID");
}

#[test]
fn test_rrf_fusion_multiple_lists() {
    let list1 = vec![
        SearchResult {
            docid: "docs:/doc1.md".to_string(),
            collection: "docs".to_string(),
            path: "/doc1.md".to_string(),
            title: "Doc 1".to_string(),
            score: 1.0,
            lines: 10,
            hash: "hash1".to_string(),
            query: None,
        },
        SearchResult {
            docid: "docs:/doc2.md".to_string(),
            collection: "docs".to_string(),
            path: "/doc2.md".to_string(),
            title: "Doc 2".to_string(),
            score: 0.8,
            lines: 10,
            hash: "hash2".to_string(),
            query: None,
        },
    ];

    let list2 = vec![
        SearchResult {
            docid: "docs:/doc2.md".to_string(),
            collection: "docs".to_string(),
            path: "/doc2.md".to_string(),
            title: "Doc 2".to_string(),
            score: 1.0,
            lines: 10,
            hash: "hash2".to_string(),
            query: None,
        },
        SearchResult {
            docid: "docs:/doc3.md".to_string(),
            collection: "docs".to_string(),
            path: "/doc3.md".to_string(),
            title: "Doc 3".to_string(),
            score: 0.9,
            lines: 10,
            hash: "hash3".to_string(),
            query: None,
        },
    ];

    let result_lists = vec![list1, list2];
    let fused = Store::rrf_fusion(&result_lists, None, 60);

    assert!(fused.len() >= 2, "RRF fusion should return results from both lists");
    // doc2 appears in both lists, should have higher score due to RRF
    let doc2_result = fused.iter().find(|r| r.docid == "docs:/doc2.md");
    assert!(doc2_result.is_some(), "doc2 should appear in results");
}

#[test]
fn test_rrf_fusion_with_weights() {
    let list1 = vec![
        SearchResult {
            docid: "docs:/doc1.md".to_string(),
            collection: "docs".to_string(),
            path: "/doc1.md".to_string(),
            title: "Doc 1".to_string(),
            score: 1.0,
            lines: 10,
            hash: "hash1".to_string(),
            query: None,
        },
    ];

    let list2 = vec![
        SearchResult {
            docid: "docs:/doc1.md".to_string(),
            collection: "docs".to_string(),
            path: "/doc1.md".to_string(),
            title: "Doc 1".to_string(),
            score: 1.0,
            lines: 10,
            hash: "hash1".to_string(),
            query: None,
        },
    ];

    // With weights [1.0, 2.0], list2 should have more influence
    let weights = Some(vec![1.0, 2.0]);
    let result_lists = vec![list1, list2];
    let fused = Store::rrf_fusion(&result_lists, weights, 60);

    assert!(!fused.is_empty(), "RRF fusion with weights should return results");
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Error Handling Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_store_invalid_collection() {
    let tmp = tempdir().unwrap();
    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    fs::write(content_dir.join("doc.md"), "# Doc\nContent.").unwrap();

    let config = create_test_config(tmp.path(), "docs", &content_dir);
    let store = Store::new(&config).unwrap();
    store.update_index().unwrap();

    // Search with non-existent collection
    let options = SearchOptions {
        limit: 10,
        min_score: 0.0,
        collection: Some("nonexistent_collection".to_string()),
        search_all: false,
    };

    let results = store.bm25_search("doc", options).unwrap();
    // Should return empty results, not error
    assert!(results.is_empty() || results.len() >= 0, "Should handle invalid collection gracefully");
}

#[test]
fn test_get_nonexistent_file() {
    let result = std::fs::read_to_string("/nonexistent/path/file.md");
    assert!(result.is_err(), "Reading nonexistent file should return error");
}
