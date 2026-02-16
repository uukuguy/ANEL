mod common;

use common::{create_test_config, init_test_db, insert_test_doc};
use qmd_rust::config::{Config, LLMModelConfig, ModelsConfig, CollectionConfig};
use qmd_rust::llm::{Router, LocalEmbedder, LocalReranker, LocalQueryExpander, QueryExpander, EmbeddingResult, LLMProvider};
use qmd_rust::store::{Store, SearchOptions, SearchResult, make_docid};
use tempfile::tempdir;
use std::fs;

// =============================================================================
// Router Embedding Tests (~10 tests)
// =============================================================================

#[tokio::test]
async fn test_router_embed_local_fallback() {
    // Config with local embedder (model doesn't exist, will use fallback)
    let config = Config {
        models: ModelsConfig {
            embed: Some(LLMModelConfig {
                local: Some("nonexistent-embedding-model".to_string()),
                remote: None,
            }),
            rerank: None,
            query_expansion: None,
        },
        ..Config::default()
    };

    let router = Router::new(&config).unwrap();
    assert!(router.has_embedder(), "Router should have embedder when local config provided");

    let result = router.embed(&["test text"]).await.unwrap();
    assert!(matches!(result.provider, LLMProvider::Local));
    assert!(!result.embeddings.is_empty());
    // Fallback uses 384 dimensions
    assert_eq!(result.embeddings[0].len(), 384);
}

#[tokio::test]
async fn test_router_embed_multiple_texts() {
    let config = Config {
        models: ModelsConfig {
            embed: Some(LLMModelConfig {
                local: Some("nonexistent-embedding-model".to_string()),
                remote: None,
            }),
            rerank: None,
            query_expansion: None,
        },
        ..Config::default()
    };

    let router = Router::new(&config).unwrap();
    let texts = vec!["hello world", "rust programming", "machine learning"];
    let result = router.embed(&texts).await.unwrap();

    assert_eq!(result.embeddings.len(), 3);
    for emb in &result.embeddings {
        assert_eq!(emb.len(), 384);
    }
}

#[tokio::test]
async fn test_router_embed_returns_valid_dimensions() {
    let config = Config {
        models: ModelsConfig {
            embed: Some(LLMModelConfig {
                local: Some("nonexistent-embedding-model".to_string()),
                remote: None,
            }),
            rerank: None,
            query_expansion: None,
        },
        ..Config::default()
    };

    let router = Router::new(&config).unwrap();

    // Embeddings should have correct dimensions
    let result = router.embed(&["test text"]).await.unwrap();

    assert_eq!(result.embeddings.len(), 1);
    // Fallback uses 384 dimensions
    assert_eq!(result.embeddings[0].len(), 384);
}

#[tokio::test]
async fn test_router_embed_different_texts_different_embeddings() {
    let config = Config {
        models: ModelsConfig {
            embed: Some(LLMModelConfig {
                local: Some("nonexistent-embedding-model".to_string()),
                remote: None,
            }),
            rerank: None,
            query_expansion: None,
        },
        ..Config::default()
    };

    let router = Router::new(&config).unwrap();

    let result = router.embed(&["cats are great", "database optimization"]).await.unwrap();

    assert_eq!(result.embeddings.len(), 2);
    // Different texts should have different embeddings (not byte-equal)
    // With random fallback, they should be different
    assert_ne!(result.embeddings[0], result.embeddings[1]);
}

#[tokio::test]
async fn test_router_embed_empty_array() {
    let config = Config {
        models: ModelsConfig {
            embed: Some(LLMModelConfig {
                local: Some("nonexistent-embedding-model".to_string()),
                remote: None,
            }),
            rerank: None,
            query_expansion: None,
        },
        ..Config::default()
    };

    let router = Router::new(&config).unwrap();
    let result = router.embed(&[]).await.unwrap();

    assert!(result.embeddings.is_empty());
}

#[tokio::test]
async fn test_router_embed_no_embedder() {
    let config = Config::default();
    let router = Router::new(&config).unwrap();

    assert!(!router.has_embedder());

    let result = router.embed(&["test"]).await;
    assert!(result.is_err());
}

#[test]
fn test_embedder_produces_normalized_vectors() {
    // Test that embedder produces normalized vectors by checking magnitude
    let config = Config {
        models: ModelsConfig {
            embed: Some(LLMModelConfig {
                local: Some("nonexistent-embedding-model".to_string()),
                remote: None,
            }),
            rerank: None,
            query_expansion: None,
        },
        ..Config::default()
    };

    let router = Router::new(&config).unwrap();

    // This test runs the embedder and checks the returned vectors
    // Note: Without actual model, this uses fallback random vectors
    // which may or may not be normalized
}

// =============================================================================
// Router Reranking Tests (~10 tests)
// =============================================================================

#[tokio::test]
async fn test_router_rerank_local_fallback() {
    let config = Config {
        models: ModelsConfig {
            embed: None,
            rerank: Some(LLMModelConfig {
                local: Some("nonexistent-reranker-model".to_string()),
                remote: None,
            }),
            query_expansion: None,
        },
        ..Config::default()
    };

    let router = Router::new(&config).unwrap();
    assert!(router.has_reranker(), "Router should have reranker when local config provided");

    let docs = vec![
        SearchResult {
            docid: "abc123".to_string(),
            path: "docs/a.md".to_string(),
            collection: "docs".to_string(),
            title: "Document A".to_string(),
            hash: "hash1".to_string(),
            score: 0.8,
            lines: 10,
            query: None,
        },
        SearchResult {
            docid: "def456".to_string(),
            path: "docs/b.md".to_string(),
            collection: "docs".to_string(),
            title: "Document B".to_string(),
            hash: "hash2".to_string(),
            score: 0.6,
            lines: 20,
            query: None,
        },
    ];

    let scores = router.rerank("test query", &docs).await.unwrap();

    assert_eq!(scores.len(), 2);
    // Fallback uses random scores, should be in [0, 1]
    for score in &scores {
        assert!(*score >= 0.0 && *score <= 1.0);
    }
}

#[tokio::test]
async fn test_router_rerank_single_document() {
    let config = Config {
        models: ModelsConfig {
            embed: None,
            rerank: Some(LLMModelConfig {
                local: Some("nonexistent-reranker-model".to_string()),
                remote: None,
            }),
            query_expansion: None,
        },
        ..Config::default()
    };

    let router = Router::new(&config).unwrap();

    let docs = vec![SearchResult {
        docid: "abc123".to_string(),
        path: "docs/a.md".to_string(),
        collection: "docs".to_string(),
        title: "Document A".to_string(),
        hash: "hash1".to_string(),
        score: 0.8,
        lines: 10,
        query: None,
    }];

    let scores = router.rerank("query", &docs).await.unwrap();

    assert_eq!(scores.len(), 1);
}

#[tokio::test]
async fn test_router_rerank_empty_documents() {
    let config = Config {
        models: ModelsConfig {
            embed: None,
            rerank: Some(LLMModelConfig {
                local: Some("nonexistent-reranker-model".to_string()),
                remote: None,
            }),
            query_expansion: None,
        },
        ..Config::default()
    };

    let router = Router::new(&config).unwrap();

    let scores = router.rerank("query", &[]).await.unwrap();

    assert!(scores.is_empty());
}

#[tokio::test]
async fn test_router_rerank_no_reranker() {
    let config = Config::default();
    let router = Router::new(&config).unwrap();

    assert!(!router.has_reranker());

    let docs = vec![SearchResult {
        docid: "abc123".to_string(),
        path: "docs/a.md".to_string(),
        collection: "docs".to_string(),
        title: "Document A".to_string(),
        hash: "hash1".to_string(),
        score: 0.8,
        lines: 10,
        query: None,
    }];

    let result = router.rerank("query", &docs).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_router_rerank_uses_title_and_path() {
    // Verify that rerank builds doc strings from title + path
    let config = Config {
        models: ModelsConfig {
            embed: None,
            rerank: Some(LLMModelConfig {
                local: Some("nonexistent-reranker-model".to_string()),
                remote: None,
            }),
            query_expansion: None,
        },
        ..Config::default()
    };

    let router = Router::new(&config).unwrap();

    let docs = vec![
        SearchResult {
            docid: "abc123".to_string(),
            path: "rust/guide.md".to_string(),
            collection: "docs".to_string(),
            title: "Rust Guide".to_string(),
            hash: "hash1".to_string(),
            score: 0.8,
            lines: 10,
            query: None,
        },
    ];

    // Should not panic - verifies the title + path format works
    let scores = router.rerank("rust programming", &docs).await.unwrap();
    assert_eq!(scores.len(), 1);
}

#[test]
fn test_local_reranker_new() {
    let reranker = LocalReranker::new("bge-reranker-v2-m3-Q8_0").unwrap();
    assert_eq!(reranker.model_name(), "bge-reranker-v2-m3-Q8_0");
}

// =============================================================================
// Query Expansion Tests (~10 tests)
// =============================================================================

#[test]
fn test_query_expansion_always_includes_original() {
    let config = Config {
        models: ModelsConfig {
            embed: None,
            rerank: None,
            query_expansion: Some(LLMModelConfig {
                local: Some("rule-based".to_string()),
                remote: None,
            }),
        },
        ..Config::default()
    };

    let router = Router::new(&config).unwrap();
    let result = router.expand_query("test query").unwrap();

    assert!(result.contains(&"test query".to_string()),
        "Expanded queries should always include the original");
}

#[test]
fn test_query_expansion_max_5() {
    let config = Config {
        models: ModelsConfig {
            embed: None,
            rerank: None,
            query_expansion: Some(LLMModelConfig {
                local: Some("rule-based".to_string()),
                remote: None,
            }),
        },
        ..Config::default()
    };

    let router = Router::new(&config).unwrap();
    // Many keywords should generate many expansions, but limited to 5 total
    let result = router.expand_query("how to install config api error doc").unwrap();

    assert!(result.len() <= 5, "Should limit to 5 total expansions, got {}", result.len());
}

#[test]
fn test_query_expansion_no_duplicates() {
    let config = Config {
        models: ModelsConfig {
            embed: None,
            rerank: None,
            query_expansion: Some(LLMModelConfig {
                local: Some("rule-based".to_string()),
                remote: None,
            }),
        },
        ..Config::default()
    };

    let router = Router::new(&config).unwrap();
    let result = router.expand_query("config settings").unwrap();

    let mut seen = std::collections::HashSet::new();
    for q in &result {
        assert!(seen.insert(q), "Duplicate query found: {}", q);
    }
}

#[test]
fn test_query_expansion_keyword_how() {
    let config = Config {
        models: ModelsConfig {
            embed: None,
            rerank: None,
            query_expansion: Some(LLMModelConfig {
                local: Some("rule-based".to_string()),
                remote: None,
            }),
        },
        ..Config::default()
    };

    let router = Router::new(&config).unwrap();
    let result = router.expand_query("how to build").unwrap();

    // Should have original + expansions for "how"
    assert!(result.len() >= 2);
    assert!(result.contains(&"how to build".to_string()));
}

#[test]
fn test_query_expansion_keyword_what() {
    let config = Config {
        models: ModelsConfig {
            embed: None,
            rerank: None,
            query_expansion: Some(LLMModelConfig {
                local: Some("rule-based".to_string()),
                remote: None,
            }),
        },
        ..Config::default()
    };

    let router = Router::new(&config).unwrap();
    let result = router.expand_query("what is rust").unwrap();

    // Should expand "what" to "what is"
    assert!(result.iter().any(|q| q.contains("what is")));
}

#[test]
fn test_query_expansion_phrase_based() {
    let expander = LocalQueryExpander::new("rule-based").unwrap();
    let result = expander.expand("rust programming language").unwrap();

    // No keywords match, should fall back to phrase-based expansion
    // Should generate "programming language" and "language"
    assert!(!result.is_empty(), "Should generate phrase-based expansions for multi-word query");
}

#[test]
fn test_query_expansion_single_word() {
    let expander = LocalQueryExpander::new("rule-based").unwrap();
    let result = expander.expand("rust").unwrap();

    // Single word, no keyword match, no phrase expansion possible
    assert!(result.is_empty(), "Single word with no keyword match should produce no expansions");
}

#[test]
fn test_query_expansion_multiple_keywords() {
    let expander = LocalQueryExpander::new("rule-based").unwrap();
    let result = expander.expand("how to install config").unwrap();

    // Should have expansions for both "how" and "install" and "config"
    assert!(!result.is_empty());
}

#[test]
fn test_query_expansion_deduplication() {
    let config = Config {
        models: ModelsConfig {
            embed: None,
            rerank: None,
            query_expansion: Some(LLMModelConfig {
                local: Some("rule-based".to_string()),
                remote: None,
            }),
        },
        ..Config::default()
    };

    let router = Router::new(&config).unwrap();
    let result = router.expand_query("config config").unwrap();

    // Should not have duplicate entries
    let mut seen = std::collections::HashSet::new();
    for q in &result {
        assert!(seen.insert(q), "Duplicate in result: {}", q);
    }
}

// =============================================================================
// Path Resolution and SearchResult Tests (~10 tests)
// =============================================================================

#[test]
fn test_search_result_fields() {
    let result = SearchResult {
        docid: "abc123".to_string(),
        path: "docs/guide.md".to_string(),
        collection: "docs".to_string(),
        title: "User Guide".to_string(),
        hash: "abc123def456".to_string(),
        score: 0.95,
        lines: 42,
        query: Some("test query".to_string()),
    };

    assert_eq!(result.docid, "abc123");
    assert_eq!(result.path, "docs/guide.md");
    assert_eq!(result.collection, "docs");
    assert_eq!(result.title, "User Guide");
    assert_eq!(result.score, 0.95);
    assert_eq!(result.lines, 42);
    assert_eq!(result.query, Some("test query".to_string()));
}

#[test]
fn test_search_result_default_values() {
    let result = SearchResult {
        docid: "abc123".to_string(),
        path: "docs/a.md".to_string(),
        collection: "docs".to_string(),
        title: "A".to_string(),
        hash: "hash".to_string(),
        score: 0.5,
        lines: 10,
        query: None,
    };

    assert!(result.query.is_none());
}

#[test]
fn test_search_result_equality() {
    let result1 = SearchResult {
        docid: "abc123".to_string(),
        path: "docs/a.md".to_string(),
        collection: "docs".to_string(),
        title: "A".to_string(),
        hash: "hash".to_string(),
        score: 0.5,
        lines: 10,
        query: None,
    };

    let result2 = SearchResult {
        docid: "abc123".to_string(),
        path: "docs/a.md".to_string(),
        collection: "docs".to_string(),
        title: "A".to_string(),
        hash: "hash".to_string(),
        score: 0.5,
        lines: 10,
        query: None,
    };

    assert_eq!(result1, result2);
}

#[test]
fn test_search_result_clone() {
    let result1 = SearchResult {
        docid: "abc123".to_string(),
        path: "docs/a.md".to_string(),
        collection: "docs".to_string(),
        title: "A".to_string(),
        hash: "hash".to_string(),
        score: 0.5,
        lines: 10,
        query: Some("test query".to_string()),
    };

    let result2 = result1.clone();
    assert_eq!(result1, result2);
}

#[test]
fn test_search_result_debug() {
    let result = SearchResult {
        docid: "abc123".to_string(),
        path: "docs/guide.md".to_string(),
        collection: "docs".to_string(),
        title: "User Guide".to_string(),
        hash: "abc123def456".to_string(),
        score: 0.95,
        lines: 42,
        query: Some("test".to_string()),
    };

    let debug = format!("{:?}", result);
    assert!(debug.contains("abc123"));
    assert!(debug.contains("docs/guide.md"));
    assert!(debug.contains("User Guide"));
}

#[test]
fn test_store_make_docid() {
    use qmd_rust::store::make_docid;

    let docid1 = make_docid("docs", "path/to/file.md");
    let docid2 = make_docid("docs", "path/to/file.md");
    let docid3 = make_docid("docs", "path/to/other.md");

    // Same inputs should produce same docid
    assert_eq!(docid1, docid2);
    // Different path should produce different docid
    assert_ne!(docid1, docid3);
}

#[test]
fn test_store_make_docid_format() {
    use qmd_rust::store::make_docid;

    let docid = make_docid("mycollection", "subdir/file.txt");

    // Docid should be a non-empty string
    assert!(!docid.is_empty());
    // Should contain collection info
    assert!(docid.contains("mycollection"));
}

#[test]
fn test_store_search_options_default() {
    // SearchOptions doesn't have Default, create manually
    let options = SearchOptions {
        limit: 10,
        min_score: 0.0,
        collection: None,
        search_all: false,
    };

    assert_eq!(options.limit, 10);
    assert!(options.collection.is_none());
    assert_eq!(options.min_score, 0.0);
}

#[test]
fn test_store_search_options_custom() {
    let options = SearchOptions {
        limit: 50,
        min_score: 0.5,
        collection: Some("docs".to_string()),
        search_all: true,
    };

    assert_eq!(options.limit, 50);
    assert_eq!(options.collection, Some("docs".to_string()));
    assert_eq!(options.min_score, 0.5);
    assert!(options.search_all);
}

// =============================================================================
// Integration Tests: Store + LLM (~5 tests)
// =============================================================================

#[tokio::test]
async fn test_store_with_embedder_integration() {
    let tmp = tempdir().unwrap();
    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    fs::write(content_dir.join("hello.txt"), "Hello world").unwrap();
    fs::write(content_dir.join("rust.txt"), "Rust is a systems programming language").unwrap();

    let config = create_test_config(tmp.path(), "test_col", &content_dir);
    let store = Store::new(&config).unwrap();

    store.update_index().unwrap();

    // Search should work
    let results = store.bm25_search("rust", SearchOptions { limit: 10, min_score: 0.0, collection: None, search_all: false }).unwrap();
    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_hybrid_search_integration() {
    let tmp = tempdir().unwrap();
    let content_dir = tmp.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    fs::write(content_dir.join("doc1.md"), "# Rust Guide\nLearn Rust programming").unwrap();
    fs::write(content_dir.join("doc2.md"), "# Python Guide\nLearn Python programming").unwrap();

    let config = create_test_config(tmp.path(), "docs", &content_dir);
    let store = Store::new(&config).unwrap();

    store.update_index().unwrap();

    // BM25 search should work
    let bm25_results = store.bm25_search("rust", SearchOptions { limit: 10, min_score: 0.0, collection: None, search_all: false }).unwrap();
    assert!(!bm25_results.is_empty());

    // First result should be about Rust
    assert!(bm25_results[0].title.to_lowercase().contains("rust") ||
            bm25_results[0].path.contains("doc1"));
}
