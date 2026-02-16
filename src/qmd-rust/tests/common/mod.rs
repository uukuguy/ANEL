use qmd_rust::config::{Config, CollectionConfig, BM25BackendConfig, VectorBackendConfig, ModelsConfig};
use rusqlite::Connection;
use std::path::Path;

/// Create a Config pointing at a temp directory with one collection.
/// The collection's `path` is set to `content_dir` so update_index can scan it.
pub fn create_test_config(cache_dir: &Path, collection_name: &str, content_dir: &Path) -> Config {
    Config {
        bm25: BM25BackendConfig::default(),
        vector: VectorBackendConfig::default(),
        collections: vec![CollectionConfig {
            name: collection_name.to_string(),
            path: content_dir.to_path_buf(),
            pattern: Some("**/*".to_string()),
            description: None,
        }],
        models: ModelsConfig::default(),
        cache_path: cache_dir.to_path_buf(),
    }
}

/// Create a Config with multiple collections.
pub fn create_multi_collection_config(
    cache_dir: &Path,
    collections: &[(&str, &Path)],
) -> Config {
    Config {
        bm25: BM25BackendConfig::default(),
        vector: VectorBackendConfig::default(),
        collections: collections
            .iter()
            .map(|(name, path)| CollectionConfig {
                name: name.to_string(),
                path: path.to_path_buf(),
                pattern: Some("**/*".to_string()),
                description: None,
            })
            .collect(),
        models: ModelsConfig::default(),
        cache_path: cache_dir.to_path_buf(),
    }
}

/// Open a SQLite connection at `path`, initialize sqlite-vec and the full schema.
/// Returns the connection for manual data insertion in tests.
pub fn init_test_db(path: &Path) -> Connection {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    // Use Store::new indirectly — we open the connection and init schema manually
    // because Store::new requires a full Config with collections.
    // Instead, replicate the schema init that Store does.
    init_sqlite_vec();
    let conn = Connection::open(path).unwrap();
    init_schema(&conn);
    conn
}

/// Load the sqlite-vec extension globally (idempotent).
fn init_sqlite_vec() {
    #[cfg(feature = "sqlite-vec")]
    {
        use sqlite_vec::sqlite3_vec_init;
        use std::os::raw::c_void;
        unsafe {
            rusqlite::ffi::sqlite3_auto_extension(
                Some(std::mem::transmute(sqlite3_vec_init as *const c_void)),
            );
        }
    }
}

/// Replicate Store::init_schema so integration tests can prepare databases
/// without needing a full Store instance.
fn init_schema(conn: &Connection) {
    // Content table - content-addressable storage
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS content (
            hash TEXT PRIMARY KEY,
            doc TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        "#,
    )
    .unwrap();

    // Documents table - file system layer mapping virtual paths to content hashes
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            collection TEXT NOT NULL,
            path TEXT NOT NULL,
            title TEXT NOT NULL,
            hash TEXT NOT NULL,
            created_at TEXT NOT NULL,
            modified_at TEXT NOT NULL,
            active INTEGER NOT NULL DEFAULT 1,
            FOREIGN KEY (hash) REFERENCES content(hash) ON DELETE CASCADE,
            UNIQUE(collection, path)
        );
        "#,
    )
    .unwrap();

    conn.execute_batch(
        r#"
        CREATE INDEX IF NOT EXISTS idx_documents_collection ON documents(collection, active);
        CREATE INDEX IF NOT EXISTS idx_documents_hash ON documents(hash);
        CREATE INDEX IF NOT EXISTS idx_documents_path ON documents(path, active);
        "#,
    )
    .unwrap();

    conn.execute_batch(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
            filepath, title, body,
            tokenize='porter unicode61'
        );
        "#,
    )
    .unwrap();

    // FTS triggers - now references content table via documents.hash
    conn.execute_batch(
        r#"
        CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents
        WHEN new.active = 1
        BEGIN
            INSERT INTO documents_fts(rowid, filepath, title, body)
            SELECT
                new.id,
                new.collection || '/' || new.path,
                new.title,
                (SELECT doc FROM content WHERE hash = new.hash)
            WHERE new.active = 1;
        END;

        CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
            DELETE FROM documents_fts WHERE rowid = old.id;
        END;

        CREATE TRIGGER IF NOT EXISTS documents_au AFTER UPDATE ON documents BEGIN
            DELETE FROM documents_fts WHERE rowid = old.id AND new.active = 0;
            INSERT OR REPLACE INTO documents_fts(rowid, filepath, title, body)
            SELECT
                new.id,
                new.collection || '/' || new.path,
                new.title,
                (SELECT doc FROM content WHERE hash = new.hash)
            WHERE new.active = 1;
        END;
        "#,
    )
    .unwrap();

    // vectors_vec requires sqlite-vec extension; skip if not available
    let _ = conn.execute_batch(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS vectors_vec USING vec0(
            hash_seq TEXT PRIMARY KEY,
            embedding float[768] distance_metric=cosine
        );
        "#,
    );

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS content_vectors (
            hash TEXT NOT NULL,
            seq INTEGER NOT NULL DEFAULT 0,
            pos INTEGER NOT NULL DEFAULT 0,
            model TEXT NOT NULL,
            embedded_at TEXT NOT NULL,
            PRIMARY KEY (hash, seq)
        );
        "#,
    )
    .unwrap();

    // LLM cache table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS llm_cache (
            cache_key TEXT PRIMARY KEY,
            model TEXT NOT NULL,
            response TEXT NOT NULL,
            created_at TEXT NOT NULL,
            expires_at TEXT
        );
        "#,
    )
    .unwrap();
}

/// Insert a test document into an already-initialized database.
pub fn insert_test_doc(conn: &Connection, collection: &str, path: &str, title: &str, content: &str, hash: &str) {
    // First insert content
    conn.execute(
        "INSERT OR REPLACE INTO content (hash, doc, created_at) VALUES (?1, ?2, datetime('now'))",
        rusqlite::params![hash, content],
    )
    .unwrap();

    // Then insert document reference
    conn.execute(
        "INSERT INTO documents (collection, path, title, hash, created_at, modified_at, active)
         VALUES (?1, ?2, ?3, ?4, datetime('now'), datetime('now'), 1)
         ON CONFLICT(collection, path) DO UPDATE SET title = excluded.title, hash = excluded.hash, active = 1",
        rusqlite::params![collection, path, title, hash],
    )
    .unwrap();
}
