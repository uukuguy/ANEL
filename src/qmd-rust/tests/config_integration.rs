mod common;

use qmd_rust::config::{Config, CollectionConfig, BM25BackendConfig, VectorBackendConfig, ModelsConfig, LLMModelConfig, BM25Backend, VectorBackend};
use std::path::PathBuf;

// ==================== Default Values ====================

#[test]
fn test_config_default_values() {
    let config = Config::default();

    assert!(config.collections.is_empty());
    assert!(matches!(config.bm25.backend, BM25Backend::SqliteFts5));
    assert!(matches!(config.vector.backend, VectorBackend::QmdBuiltin));
    assert_eq!(config.vector.model, "embeddinggemma-300M");
    assert!(config.models.embed.is_none());
    assert!(config.models.rerank.is_none());
    assert!(config.models.query_expansion.is_none());
    // cache_path should contain "qmd"
    assert!(
        config.cache_path.to_string_lossy().contains("qmd"),
        "Default cache_path should contain 'qmd': {:?}",
        config.cache_path
    );
}

// ==================== Serialization Roundtrip ====================

#[test]
fn test_config_serialize_deserialize_roundtrip() {
    let config = Config {
        bm25: BM25BackendConfig::default(),
        vector: VectorBackendConfig::default(),
        collections: vec![
            CollectionConfig {
                name: "my_project".to_string(),
                path: "/tmp/test/project".into(),
                pattern: Some("**/*.rs".to_string()),
                description: Some("Rust source files".to_string()),
            },
        ],
        models: ModelsConfig {
            embed: Some(LLMModelConfig {
                local: Some("nomic-embed".to_string()),
                remote: None,
            }),
            rerank: None,
            query_expansion: Some(LLMModelConfig {
                local: Some("rule-based".to_string()),
                remote: None,
            }),
        },
        cache_path: "/tmp/test/cache".into(),
    };

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&config).unwrap();

    // Deserialize back
    let restored: Config = serde_yaml::from_str(&yaml).unwrap();

    assert_eq!(restored.collections.len(), 1);
    assert_eq!(restored.collections[0].name, "my_project");
    assert_eq!(restored.collections[0].pattern, Some("**/*.rs".to_string()));
    assert_eq!(restored.collections[0].description, Some("Rust source files".to_string()));
    assert!(restored.models.embed.is_some());
    assert_eq!(
        restored.models.embed.as_ref().unwrap().local,
        Some("nomic-embed".to_string())
    );
    assert!(restored.models.rerank.is_none());
    assert!(restored.models.query_expansion.is_some());
}

#[test]
fn test_config_serialize_deserialize_minimal() {
    // Minimal YAML — all fields should get defaults via serde
    let yaml = r#"
collections: []
cache_path: /tmp/cache
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();

    assert!(config.collections.is_empty());
    assert!(matches!(config.bm25.backend, BM25Backend::SqliteFts5));
    assert!(matches!(config.vector.backend, VectorBackend::QmdBuiltin));
}

// ==================== Path Generation ====================

#[test]
fn test_config_db_path_structure() {
    let config = Config {
        cache_path: "/tmp/qmd_test".into(),
        ..Config::default()
    };

    let db_path = config.db_path_for("my_collection");
    assert_eq!(
        db_path.to_string_lossy(),
        "/tmp/qmd_test/my_collection/index.db"
    );
}

#[test]
fn test_config_cache_dir_structure() {
    let config = Config {
        cache_path: "/tmp/qmd_test".into(),
        ..Config::default()
    };

    let cache_dir = config.cache_dir_for("my_collection");
    assert_eq!(
        cache_dir.to_string_lossy(),
        "/tmp/qmd_test/my_collection"
    );
}

#[test]
fn test_config_db_path_different_collections() {
    let config = Config {
        cache_path: "/data/qmd".into(),
        ..Config::default()
    };

    let path_a = config.db_path_for("alpha");
    let path_b = config.db_path_for("beta");

    assert_ne!(path_a, path_b);
    assert!(path_a.to_string_lossy().contains("alpha"));
    assert!(path_b.to_string_lossy().contains("beta"));
}

// ==================== Backend Serialization ====================

#[test]
fn test_bm25_backend_serde() {
    let yaml = r#"backend: sqlite_fts5"#;
    let config: BM25BackendConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(matches!(config.backend, BM25Backend::SqliteFts5));

    let yaml2 = r#"backend: lancedb"#;
    let config2: BM25BackendConfig = serde_yaml::from_str(yaml2).unwrap();
    assert!(matches!(config2.backend, BM25Backend::LanceDb));
}

#[test]
fn test_vector_backend_serde() {
    let yaml = r#"
backend: qmd_builtin
model: test-model
"#;
    let config: VectorBackendConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(matches!(config.backend, VectorBackend::QmdBuiltin));
    assert_eq!(config.model, "test-model");
}

// ==================== Config Save/Load (Phase 5) ====================

#[test]
fn test_config_save_and_load_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let config_path = tmp.path().join("index.yaml");

    let config = Config {
        bm25: BM25BackendConfig::default(),
        vector: VectorBackendConfig::default(),
        collections: vec![
            CollectionConfig {
                name: "test_col".to_string(),
                path: "/tmp/test/docs".into(),
                pattern: Some("**/*.md".to_string()),
                description: Some("Test collection".to_string()),
            },
        ],
        models: ModelsConfig::default(),
        cache_path: "/tmp/test/cache".into(),
    };

    // Serialize and write
    let content = serde_yaml::to_string(&config).unwrap();
    std::fs::write(&config_path, &content).unwrap();

    // Read back and deserialize
    let loaded_content = std::fs::read_to_string(&config_path).unwrap();
    let loaded: Config = serde_yaml::from_str(&loaded_content).unwrap();

    assert_eq!(loaded.collections.len(), 1);
    assert_eq!(loaded.collections[0].name, "test_col");
    assert_eq!(loaded.collections[0].path, PathBuf::from("/tmp/test/docs"));
    assert_eq!(loaded.collections[0].pattern, Some("**/*.md".to_string()));
    assert_eq!(loaded.collections[0].description, Some("Test collection".to_string()));
}

#[test]
fn test_config_add_collection_persists() {
    let tmp = tempfile::tempdir().unwrap();
    let config_path = tmp.path().join("index.yaml");

    // Start with empty config
    let mut config = Config {
        cache_path: tmp.path().join("cache"),
        ..Config::default()
    };

    assert!(config.collections.is_empty());

    // Add a collection
    config.collections.push(CollectionConfig {
        name: "notes".to_string(),
        path: tmp.path().join("notes"),
        pattern: Some("**/*.md".to_string()),
        description: None,
    });

    // Serialize and write
    let content = serde_yaml::to_string(&config).unwrap();
    std::fs::write(&config_path, &content).unwrap();

    // Read back
    let loaded_content = std::fs::read_to_string(&config_path).unwrap();
    let loaded: Config = serde_yaml::from_str(&loaded_content).unwrap();

    assert_eq!(loaded.collections.len(), 1);
    assert_eq!(loaded.collections[0].name, "notes");
}

#[test]
fn test_config_remove_collection_persists() {
    let tmp = tempfile::tempdir().unwrap();
    let config_path = tmp.path().join("index.yaml");

    let mut config = Config {
        collections: vec![
            CollectionConfig {
                name: "keep".to_string(),
                path: "/tmp/keep".into(),
                pattern: None,
                description: None,
            },
            CollectionConfig {
                name: "remove_me".to_string(),
                path: "/tmp/remove".into(),
                pattern: None,
                description: None,
            },
        ],
        cache_path: tmp.path().join("cache"),
        ..Config::default()
    };

    // Remove second collection
    let idx = config.collections.iter().position(|c| c.name == "remove_me").unwrap();
    config.collections.remove(idx);

    // Serialize and write
    let content = serde_yaml::to_string(&config).unwrap();
    std::fs::write(&config_path, &content).unwrap();

    // Read back
    let loaded_content = std::fs::read_to_string(&config_path).unwrap();
    let loaded: Config = serde_yaml::from_str(&loaded_content).unwrap();

    assert_eq!(loaded.collections.len(), 1);
    assert_eq!(loaded.collections[0].name, "keep");
}

#[test]
fn test_config_rename_collection_persists() {
    let tmp = tempfile::tempdir().unwrap();
    let config_path = tmp.path().join("index.yaml");

    let mut config = Config {
        collections: vec![
            CollectionConfig {
                name: "old_name".to_string(),
                path: "/tmp/project".into(),
                pattern: Some("**/*.rs".to_string()),
                description: Some("My project".to_string()),
            },
        ],
        cache_path: tmp.path().join("cache"),
        ..Config::default()
    };

    // Rename
    config.collections[0].name = "new_name".to_string();

    // Serialize and write
    let content = serde_yaml::to_string(&config).unwrap();
    std::fs::write(&config_path, &content).unwrap();

    // Read back
    let loaded_content = std::fs::read_to_string(&config_path).unwrap();
    let loaded: Config = serde_yaml::from_str(&loaded_content).unwrap();

    assert_eq!(loaded.collections.len(), 1);
    assert_eq!(loaded.collections[0].name, "new_name");
    // Other fields preserved
    assert_eq!(loaded.collections[0].pattern, Some("**/*.rs".to_string()));
    assert_eq!(loaded.collections[0].description, Some("My project".to_string()));
}

#[test]
fn test_config_duplicate_collection_detection() {
    let config = Config {
        collections: vec![
            CollectionConfig {
                name: "existing".to_string(),
                path: "/tmp/a".into(),
                pattern: None,
                description: None,
            },
        ],
        ..Config::default()
    };

    // Check duplicate detection logic
    let has_duplicate = config.collections.iter().any(|c| c.name == "existing");
    assert!(has_duplicate);

    let no_duplicate = config.collections.iter().any(|c| c.name == "new_one");
    assert!(!no_duplicate);
}
