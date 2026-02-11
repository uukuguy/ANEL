mod common;

use common::{create_test_config, create_multi_collection_config, init_test_db, insert_test_doc};
use qmd_rust::store::{Store, SearchOptions};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_store_new_creates_schema() {
    let tmp = tempdir().unwrap();
    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    let config = create_test_config(tmp.path(), "test_col", &content_dir);
    let store = Store::new(&config).unwrap();

    // Verify we can get a connection (schema was created)
    let conn = store.get_connection("test_col").unwrap();

    // Verify core tables exist
    for table in &["documents", "documents_fts", "content_vectors", "collections"] {
        let count: i64 = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{}'",
                    table
                ),
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "Table '{}' should exist", table);
    }
}

#[test]
fn test_update_index_scans_files() {
    let tmp = tempdir().unwrap();
    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    // Create test files
    fs::write(content_dir.join("hello.txt"), "Hello world from Rust").unwrap();
    fs::write(content_dir.join("guide.md"), "# Rust Guide\nLearn Rust programming").unwrap();

    let config = create_test_config(tmp.path(), "docs", &content_dir);
    let store = Store::new(&config).unwrap();

    store.update_index().unwrap();

    // Verify documents were indexed
    let conn = store.get_connection("docs").unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM documents WHERE active = 1", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 2, "Should have indexed 2 files");
}

#[test]
fn test_update_index_skip_unchanged() {
    let tmp = tempdir().unwrap();
    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    fs::write(content_dir.join("stable.txt"), "This content does not change").unwrap();

    let config = create_test_config(tmp.path(), "docs", &content_dir);
    let store = Store::new(&config).unwrap();

    // First index
    store.update_index().unwrap();

    let conn = store.get_connection("docs").unwrap();
    let count_after_first: i64 = conn
        .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count_after_first, 1);

    drop(conn);

    // Second index — same content, should skip
    store.update_index().unwrap();

    let conn2 = store.get_connection("docs").unwrap();
    let count_after_second: i64 = conn2
        .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))
        .unwrap();
    // Should still be 1 document (not duplicated)
    assert_eq!(count_after_second, 1, "Unchanged file should not create duplicate");
}

#[test]
fn test_bm25_search_multiple_collections() {
    let tmp = tempdir().unwrap();

    // Set up two collections with separate content dirs
    let content_a = tmp.path().join("content_a");
    let content_b = tmp.path().join("content_b");
    fs::create_dir_all(&content_a).unwrap();
    fs::create_dir_all(&content_b).unwrap();

    let config = create_multi_collection_config(
        tmp.path(),
        &[("col_a", &content_a), ("col_b", &content_b)],
    );

    // Pre-populate databases with test data
    let db_a = tmp.path().join("col_a").join("index.db");
    let db_b = tmp.path().join("col_b").join("index.db");

    let conn_a = init_test_db(&db_a);
    insert_test_doc(&conn_a, "col_a", "rust.md", "Rust Guide", "Rust is a systems programming language", "hash_a1");
    drop(conn_a);

    let conn_b = init_test_db(&db_b);
    insert_test_doc(&conn_b, "col_b", "go.md", "Go Guide", "Go is a compiled programming language", "hash_b1");
    drop(conn_b);

    let store = Store::new(&config).unwrap();

    let opts = SearchOptions {
        limit: 10,
        min_score: 0.0,
        collection: None,
        search_all: true,
    };

    let results = store.bm25_search("programming language", opts).unwrap();
    assert!(results.len() >= 2, "search_all should find docs across both collections, got {}", results.len());

    let collections: Vec<&str> = results.iter().map(|r| r.collection.as_str()).collect();
    assert!(collections.contains(&"col_a"), "Should find results in col_a");
    assert!(collections.contains(&"col_b"), "Should find results in col_b");
}

#[test]
fn test_bm25_search_single_collection_filter() {
    let tmp = tempdir().unwrap();

    let content_a = tmp.path().join("content_a");
    let content_b = tmp.path().join("content_b");
    fs::create_dir_all(&content_a).unwrap();
    fs::create_dir_all(&content_b).unwrap();

    let config = create_multi_collection_config(
        tmp.path(),
        &[("col_a", &content_a), ("col_b", &content_b)],
    );

    let db_a = tmp.path().join("col_a").join("index.db");
    let db_b = tmp.path().join("col_b").join("index.db");

    let conn_a = init_test_db(&db_a);
    insert_test_doc(&conn_a, "col_a", "rust.md", "Rust Guide", "Rust programming language", "hash_a1");
    drop(conn_a);

    let conn_b = init_test_db(&db_b);
    insert_test_doc(&conn_b, "col_b", "go.md", "Go Guide", "Go programming language", "hash_b1");
    drop(conn_b);

    let store = Store::new(&config).unwrap();

    // Search only col_a
    let opts = SearchOptions {
        limit: 10,
        min_score: 0.0,
        collection: Some("col_a".to_string()),
        search_all: false,
    };

    let results = store.bm25_search("programming", opts).unwrap();
    assert!(!results.is_empty(), "Should find results in col_a");
    for r in &results {
        assert_eq!(r.collection, "col_a", "All results should be from col_a");
    }
}

#[test]
fn test_get_stats_with_documents() {
    let tmp = tempdir().unwrap();
    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    let config = create_test_config(tmp.path(), "docs", &content_dir);

    // Pre-populate database
    let db_path = tmp.path().join("docs").join("index.db");
    let conn = init_test_db(&db_path);
    insert_test_doc(&conn, "docs", "a.md", "Doc A", "Content A", "hash1");
    insert_test_doc(&conn, "docs", "b.md", "Doc B", "Content B", "hash2");
    insert_test_doc(&conn, "docs", "c.md", "Doc C", "Content C", "hash3");
    drop(conn);

    let store = Store::new(&config).unwrap();
    let stats = store.get_stats().unwrap();

    assert_eq!(stats.collection_count, 1);
    assert_eq!(stats.document_count, 3);
    assert_eq!(stats.indexed_count, 3);
    assert_eq!(stats.pending_count, 0);
    assert_eq!(*stats.collection_stats.get("docs").unwrap(), 3);
}

#[test]
fn test_get_stats_multiple_collections() {
    let tmp = tempdir().unwrap();

    let content_a = tmp.path().join("content_a");
    let content_b = tmp.path().join("content_b");
    fs::create_dir_all(&content_a).unwrap();
    fs::create_dir_all(&content_b).unwrap();

    let config = create_multi_collection_config(
        tmp.path(),
        &[("alpha", &content_a), ("beta", &content_b)],
    );

    let db_a = tmp.path().join("alpha").join("index.db");
    let db_b = tmp.path().join("beta").join("index.db");

    let conn_a = init_test_db(&db_a);
    insert_test_doc(&conn_a, "alpha", "a.md", "A", "Content", "h1");
    insert_test_doc(&conn_a, "alpha", "b.md", "B", "Content", "h2");
    drop(conn_a);

    let conn_b = init_test_db(&db_b);
    insert_test_doc(&conn_b, "beta", "c.md", "C", "Content", "h3");
    drop(conn_b);

    let store = Store::new(&config).unwrap();
    let stats = store.get_stats().unwrap();

    assert_eq!(stats.collection_count, 2);
    assert_eq!(stats.document_count, 3);
    assert_eq!(*stats.collection_stats.get("alpha").unwrap(), 2);
    assert_eq!(*stats.collection_stats.get("beta").unwrap(), 1);
}
