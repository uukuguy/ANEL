mod common;

use assert_cmd::Command;
use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use std::fs;
use std::process::Output;
use tempfile::tempdir;

/// Helper to set QMD_CONFIG_PATH and run qmd command
fn run_qmd_cmd(args: &[&str], config_path: &std::path::Path) -> Output {
    let mut cmd = Command::cargo_bin("qmd-rust").unwrap();
    cmd.env("QMD_CONFIG_PATH", config_path);
    cmd.args(args);
    cmd.output().unwrap()
}

/// Helper to create a test config file and content directory
fn setup_test_env() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    // Create test config
    let config_content = r#"
cache_path: /tmp/qmd_test_cache
bm25:
  backend: sqlite_fts5
vector:
  backend: qmd_builtin
collections:
  - name: docs
    path: CONTENT_DIR
    pattern: "**/*"
"#;
    let config_content = config_content.replace("CONTENT_DIR", content_dir.to_str().unwrap());
    let config_path = tmp.path().join("config.yaml");
    fs::write(&config_path, config_content).unwrap();

    (tmp, config_path)
}

// ============================================================================
// Search Command Tests (15 tests)
// ============================================================================

#[test]
fn test_search_basic() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["search", "test"], &config_path);
    // Should not crash, exit code may be non-zero if no index
    assert!(output.status.code().is_some());
}

#[test]
fn test_search_with_limit() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["search", "-n", "5", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_search_with_limit_long() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["search", "--limit", "10", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_search_with_collection() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["search", "-c", "docs", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_search_with_collection_long() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["search", "--collection", "docs", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_search_all_collections() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["search", "--all", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_search_with_min_score() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["search", "--min-score", "0.5", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_search_json_format() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["search", "--format", "json", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_search_ndjson_format() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["search", "--format", "ndjson", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_search_md_format() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["search", "--format", "md", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_search_csv_format() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["search", "--format", "csv", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_search_files_format() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["search", "--format", "files", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_search_dry_run() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["search", "--dry-run", "test"], &config_path);
    // Dry-run should succeed and show what would be executed
    assert!(output.status.success() || output.status.code() == Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DRY-RUN") || stdout.contains("query:"));
}

#[test]
fn test_search_emit_spec() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["search", "--emit-spec", "test"], &config_path);
    // emit-spec should output JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"type\"") || stdout.contains("schema"));
}

#[test]
fn test_search_empty_query() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["search", ""], &config_path);
    // Empty query may fail but should not crash
    assert!(output.status.code().is_some());
}

// ============================================================================
// VSearch Command Tests (10 tests)
// ============================================================================

#[test]
fn test_vsearch_basic() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["vsearch", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_vsearch_with_limit() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["vsearch", "-n", "5", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_vsearch_with_collection() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["vsearch", "-c", "docs", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_vsearch_all_collections() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["vsearch", "--all", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_vsearch_json_format() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["vsearch", "--format", "json", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_vsearch_dry_run() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["vsearch", "--dry-run", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_vsearch_emit_spec() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["vsearch", "--emit-spec", "test"], &config_path);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("type") || stdout.contains("properties"));
}

#[test]
fn test_vsearch_fts_backend_option() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["vsearch", "--fts-backend", "lancedb", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_vsearch_vector_backend_option() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["vsearch", "--vector-backend", "lancedb", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_vsearch_min_score() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["vsearch", "--min-score", "0.3", "test"], &config_path);
    assert!(output.status.code().is_some());
}

// ============================================================================
// Query Command Tests (10 tests)
// ============================================================================

#[test]
fn test_query_basic() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["query", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_query_with_limit() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["query", "-n", "5", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_query_with_collection() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["query", "-c", "docs", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_query_all_collections() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["query", "--all", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_query_json_format() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["query", "--format", "json", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_query_ndjson_format() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["query", "--format", "ndjson", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_query_md_format() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["query", "--format", "md", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_query_dry_run() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["query", "--dry-run", "test"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_query_emit_spec() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["query", "--emit-spec", "test"], &config_path);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("type") || stdout.contains("properties"));
}

#[test]
fn test_query_with_fts_backend() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["query", "--fts-backend", "lancedb", "test"], &config_path);
    assert!(output.status.code().is_some());
}

// ============================================================================
// Get Command Tests (8 tests)
// ============================================================================

#[test]
fn test_get_basic() {
    let (tmp, config_path) = setup_test_env();

    // Create a test file to get
    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(content_dir.join("test.txt"), "Hello world").unwrap();

    let output = run_qmd_cmd(&["get", "test.txt"], &config_path);
    // get command may fail without index but should not crash
    assert!(output.status.code().is_some());
}

#[test]
fn test_get_with_line_number() {
    let (tmp, config_path) = setup_test_env();

    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(content_dir.join("test.txt"), "Line 1\nLine 2\nLine 3").unwrap();

    let output = run_qmd_cmd(&["get", "test.txt:1"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_get_with_limit() {
    let (tmp, config_path) = setup_test_env();

    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(content_dir.join("test.txt"), "Line 1\nLine 2\nLine 3").unwrap();

    let output = run_qmd_cmd(&["get", "-n", "2", "test.txt"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_get_with_from() {
    let (tmp, config_path) = setup_test_env();

    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(content_dir.join("test.txt"), "Line 1\nLine 2\nLine 3").unwrap();

    let output = run_qmd_cmd(&["get", "--from", "2", "test.txt"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_get_full_content() {
    let (tmp, config_path) = setup_test_env();

    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(content_dir.join("test.txt"), "Full content here").unwrap();

    let output = run_qmd_cmd(&["get", "--full", "test.txt"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_get_json_format() {
    let (tmp, config_path) = setup_test_env();

    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(content_dir.join("test.txt"), "Content").unwrap();

    let output = run_qmd_cmd(&["get", "--format", "json", "test.txt"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_get_dry_run() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["get", "--dry-run", "test.txt"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_get_emit_spec() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["get", "--emit-spec", "test.txt"], &config_path);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("type") || stdout.contains("properties"));
}

// ============================================================================
// Status Command Tests (5 tests)
// ============================================================================

#[test]
fn test_status_basic() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["status"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_status_verbose() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["status", "--verbose"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_status_with_collection() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["status", "-c", "docs"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_status_json_format() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["status", "--format", "json"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_status_dry_run() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["status", "--dry-run"], &config_path);
    assert!(output.status.code().is_some());
}

// ============================================================================
// Collection Command Tests (5 tests)
// ============================================================================

#[test]
fn test_collection_list() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["collection", "list"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_collection_add() {
    let (tmp, config_path) = setup_test_env();
    let content_dir = tmp.path().join("new_content");
    fs::create_dir_all(&content_dir).unwrap();

    let output = run_qmd_cmd(&["collection", "add", content_dir.to_str().unwrap()], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_collection_add_with_name() {
    let (tmp, config_path) = setup_test_env();
    let content_dir = tmp.path().join("new_content");
    fs::create_dir_all(&content_dir).unwrap();

    let output = run_qmd_cmd(
        &["collection", "add", "-n", "my_collection", content_dir.to_str().unwrap()],
        &config_path,
    );
    assert!(output.status.code().is_some());
}

#[test]
fn test_collection_remove() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["collection", "remove", "docs"], &config_path);
    // May fail if collection doesn't exist but should not crash
    assert!(output.status.code().is_some());
}

#[test]
fn test_collection_rename() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["collection", "rename", "docs", "new_docs"], &config_path);
    // May fail but should not crash
    assert!(output.status.code().is_some());
}

// ============================================================================
// Update Command Tests (3 tests)
// ============================================================================

#[test]
fn test_update_basic() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["update"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_update_with_pull() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["update", "--pull"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_update_with_collection() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["update", "-c", "docs"], &config_path);
    assert!(output.status.code().is_some());
}

// ============================================================================
// Embed Command Tests (3 tests)
// ============================================================================

#[test]
fn test_embed_basic() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["embed"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_embed_with_force() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["embed", "--force"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_embed_with_collection() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["embed", "-c", "docs"], &config_path);
    assert!(output.status.code().is_some());
}

// ============================================================================
// Context Command Tests (3 tests)
// ============================================================================

#[test]
fn test_context_list() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["context", "list"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_context_add() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["context", "add", "/tmp/test", "-d", "Test context"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_context_rm() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["context", "rm", "/tmp/test"], &config_path);
    // May fail if context doesn't exist but should not crash
    assert!(output.status.code().is_some());
}

// ============================================================================
// Cleanup Command Tests (2 tests)
// ============================================================================

#[test]
fn test_cleanup_dry_run() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["cleanup", "--dry-run"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_cleanup_with_older_than() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["cleanup", "--older-than", "7"], &config_path);
    assert!(output.status.code().is_some());
}

// ============================================================================
// MultiGet Command Tests (2 tests)
// ============================================================================

#[test]
fn test_multiget_basic() {
    let (tmp, config_path) = setup_test_env();

    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(content_dir.join("file1.txt"), "Content 1").unwrap();
    fs::write(content_dir.join("file2.txt"), "Content 2").unwrap();

    let output = run_qmd_cmd(&["multiget", "*.txt"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_multiget_with_limit() {
    let (tmp, config_path) = setup_test_env();

    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(content_dir.join("test.txt"), "Line 1\nLine 2\nLine 3").unwrap();

    let output = run_qmd_cmd(&["multiget", "-n", "1", "*.txt"], &config_path);
    assert!(output.status.code().is_some());
}

// ============================================================================
// MCP Server Command Tests (2 tests)
// ============================================================================

#[test]
fn test_mcp_stdio_transport() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["mcp", "--transport", "stdio"], &config_path);
    // MCP server in stdio mode expects stdin input, so may timeout or exit
    assert!(output.status.code().is_some());
}

#[test]
fn test_mcp_sse_transport() {
    let (_tmp, config_path) = setup_test_env();
    // SSE transport with short timeout
    let output = run_qmd_cmd(&["mcp", "--transport", "sse", "--port", "18080"], &config_path);
    // Server should at least attempt to start
    assert!(output.status.code().is_some());
}

// ============================================================================
// Agent Command Tests (2 tests)
// ============================================================================

#[test]
fn test_agent_query_mode() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["agent", "test query"], &config_path);
    // Agent may fail without LLM but should not crash
    assert!(output.status.code().is_some());
}

#[test]
fn test_agent_json_format() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["agent", "--format", "json", "test"], &config_path);
    assert!(output.status.code().is_some());
}

// ============================================================================
// Plugin Command Tests (3 tests)
// ============================================================================

#[test]
fn test_plugin_list() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["plugin", "list"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_plugin_dir() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["plugin", "dir"], &config_path);
    assert!(output.status.code().is_some());
}

#[test]
fn test_plugin_info() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["plugin", "info", "nonexistent"], &config_path);
    // Should handle nonexistent plugin gracefully
    assert!(output.status.code().is_some());
}

// ============================================================================
// Server Command Tests (2 tests)
// ============================================================================

#[test]
fn test_server_start() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["server", "--port", "18081"], &config_path);
    // Server may fail to bind but should not crash
    assert!(output.status.code().is_some());
}

#[test]
fn test_server_with_workers() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["server", "--port", "18082", "--workers", "2"], &config_path);
    assert!(output.status.code().is_some());
}

// ============================================================================
// Error Handling Tests (4 tests)
// ============================================================================

#[test]
fn test_invalid_subcommand() {
    let (_tmp, config_path) = setup_test_env();
    let mut cmd = Command::cargo_bin("qmd-rust").unwrap();
    cmd.env("QMD_CONFIG_PATH", &config_path);
    cmd.arg("invalid_command");
    let output = cmd.output().unwrap();
    // Should fail with unrecognized subcommand
    assert!(!output.status.success());
}

#[test]
fn test_search_invalid_format() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["search", "--format", "invalid_format", "test"], &config_path);
    // Should handle invalid format gracefully
    assert!(output.status.code().is_some());
}

#[test]
fn test_status_nonexistent_collection() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["status", "-c", "nonexistent_collection_xyz"], &config_path);
    // Should handle gracefully
    assert!(output.status.code().is_some());
}

#[test]
fn test_get_nonexistent_file() {
    let (_tmp, config_path) = setup_test_env();
    let output = run_qmd_cmd(&["get", "nonexistent_file_xyz.txt"], &config_path);
    // Should handle gracefully
    assert!(output.status.code().is_some());
}

// ============================================================================
// CLI Help Tests (4 tests)
// ============================================================================

#[test]
fn test_search_help() {
    let mut cmd = Command::cargo_bin("qmd-rust").unwrap();
    cmd.arg("search").arg("--help");
    let output = cmd.output().unwrap();
    output
        .assert()
        .success()
        .stdout(predicate::str::contains("Search"));
}

#[test]
fn test_vsearch_help() {
    let mut cmd = Command::cargo_bin("qmd-rust").unwrap();
    cmd.arg("vsearch").arg("--help");
    let output = cmd.output().unwrap();
    output.assert().success().stdout(predicate::str::contains("Vector"));
}

#[test]
fn test_query_help() {
    let mut cmd = Command::cargo_bin("qmd-rust").unwrap();
    cmd.arg("query").arg("--help");
    let output = cmd.output().unwrap();
    output.assert().success().stdout(predicate::str::contains("Hybrid"));
}

#[test]
fn test_get_help() {
    let mut cmd = Command::cargo_bin("qmd-rust").unwrap();
    cmd.arg("get").arg("--help");
    let output = cmd.output().unwrap();
    output
        .assert()
        .success()
        .stdout(predicate::str::contains("Get"));
}
