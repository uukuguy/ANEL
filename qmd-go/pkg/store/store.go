package store

import (
	"crypto/sha256"
	"database/sql"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	_ "github.com/mattn/go-sqlite3"
)

// SearchResult represents a search result
type SearchResult struct {
	DocID     string
	Path      string
	Collection string
	Score     float32
	Lines     int
	Title     string
	Hash      string
}

// SearchOptions represents search options
type SearchOptions struct {
	Limit      int
	MinScore   float32
	Collection string
	All        bool
}

// IndexStats represents index statistics
type IndexStats struct {
	CollectionCount int
	DocumentCount   int
	IndexedCount    int
	PendingCount    int
	ChunkCount      int
	CollectionStats map[string]int
}

// Store represents the document store
type Store struct {
	dbPath string
	db     *sql.DB
}

// New creates a new Store
func New(dbPath string) (*Store, error) {
	dir := filepath.Dir(dbPath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create directory: %w", err)
	}

	db, err := sql.Open("sqlite3", dbPath)
	if err != nil {
		return nil, fmt.Errorf("failed to open database: %w", err)
	}

	store := &Store{dbPath: dbPath, db: db}
	if err := store.init(); err != nil {
		db.Close()
		return nil, err
	}

	return store, nil
}

// Close closes the store
func (s *Store) Close() error {
	return s.db.Close()
}

// init initializes the database schema
func (s *Store) init() error {
	schema := `
	-- Documents table
	CREATE TABLE IF NOT EXISTS documents (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		collection TEXT NOT NULL,
		path TEXT NOT NULL,
		title TEXT NOT NULL,
		hash TEXT NOT NULL UNIQUE,
		doc TEXT NOT NULL,
		created_at TEXT NOT NULL,
		modified_at TEXT NOT NULL,
		active INTEGER NOT NULL DEFAULT 1
	);

	-- FTS5 full-text search
	CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
		filepath, title, body,
		tokenize='porter unicode61'
	);

	-- Collections
	CREATE TABLE IF NOT EXISTS collections (
		name TEXT PRIMARY KEY,
		path TEXT NOT NULL,
		pattern TEXT,
		description TEXT
	);

	-- Path contexts (relevance hints)
	CREATE TABLE IF NOT EXISTS path_contexts (
		path TEXT PRIMARY KEY,
		description TEXT NOT NULL,
		created_at TEXT NOT NULL,
		updated_at TEXT NOT NULL
	);

	-- LLM response cache
	CREATE TABLE IF NOT EXISTS llm_cache (
		cache_key TEXT PRIMARY KEY,
		model TEXT NOT NULL,
		response TEXT NOT NULL,
		created_at TEXT NOT NULL,
		expires_at TEXT
	);

	-- Content vectors metadata
	CREATE TABLE IF NOT EXISTS content_vectors (
		hash TEXT NOT NULL,
		seq INTEGER NOT NULL DEFAULT 0,
		pos INTEGER NOT NULL DEFAULT 0,
		model TEXT NOT NULL,
		embedded_at TEXT NOT NULL,
		PRIMARY KEY (hash, seq)
	);

	-- Vector storage (for qmd built-in vector search)
	CREATE TABLE IF NOT EXISTS vectors_vec (
		hash_seq TEXT PRIMARY KEY,
		embedding TEXT NOT NULL
	);

	-- Indexes
	CREATE INDEX IF NOT EXISTS idx_documents_collection ON documents(collection);
	CREATE INDEX IF NOT EXISTS idx_documents_hash ON documents(hash);
	CREATE INDEX IF NOT EXISTS idx_documents_active ON documents(active);
	CREATE INDEX IF NOT EXISTS idx_content_vectors_hash ON content_vectors(hash);
	`

	_, err := s.db.Exec(schema)
	return err
}

// AddDocument adds a document to the store
func (s *Store) AddDocument(collection, path, title, content string) (string, error) {
	hash := HashContent(content)
	now := time.Now().UTC().Format(time.RFC3339)

	// Insert or replace document
	_, err := s.db.Exec(`
		INSERT INTO documents (collection, path, title, hash, doc, created_at, modified_at, active)
		VALUES (?, ?, ?, ?, ?, ?, ?, 1)
		ON CONFLICT(hash) DO UPDATE SET
			path = excluded.path,
			title = excluded.title,
			modified_at = excluded.modified_at,
			active = 1
	`, collection, path, title, hash, content, now, now)

	if err != nil {
		return "", fmt.Errorf("failed to add document: %w", err)
	}

	// Update FTS index
	_, err = s.db.Exec(`
		INSERT INTO documents_fts (filepath, title, body)
		VALUES (?, ?, ?)
		ON CONFLICT(rowid) DO UPDATE SET
			filepath = excluded.filepath,
			title = excluded.title,
			body = excluded.body
	`, path, title, content)

	if err != nil {
		return "", fmt.Errorf("failed to update FTS: %w", err)
	}

	return hash, nil
}

// RemoveDocument removes a document from the store
func (s *Store) RemoveDocument(hash string) error {
	_, err := s.db.Exec("UPDATE documents SET active = 0 WHERE hash = ?", hash)
	if err != nil {
		return fmt.Errorf("failed to remove document: %w", err)
	}

	_, err = s.db.Exec("DELETE FROM documents_fts WHERE rowid IN (SELECT rowid FROM documents WHERE hash = ?)", hash)
	return err
}

// BM25Search performs BM25 full-text search
func (s *Store) BM25Search(query string, opts SearchOptions) ([]SearchResult, error) {
	if opts.Limit == 0 {
		opts.Limit = 20
	}

	// Escape special FTS5 characters
	query = EscapeFTS5(query)

	var args []interface{}
	collectionClause := ""
	if !opts.All && opts.Collection != "" {
		collectionClause = "AND d.collection = ?"
		args = append(args, opts.Collection)
	}

	sql := fmt.Sprintf(`
		SELECT d.collection, d.path, d.title, d.hash, bm25(documents_fts), d.doc
		FROM documents_fts f
		JOIN documents d ON d.path = f.filepath
		WHERE documents_fts MATCH ? %s AND d.active = 1
		ORDER BY bm25(documents_fts)
		LIMIT ?
	`, collectionClause)

	args = append(args, query, opts.Limit)

	rows, err := s.db.Query(sql, args...)
	if err != nil {
		return nil, fmt.Errorf("failed to search: %w", err)
	}
	defer rows.Close()

	var results []SearchResult
	for rows.Next() {
		var r SearchResult
		var doc string
		if err := rows.Scan(&r.Collection, &r.Path, &r.Title, &r.Hash, &r.Score, &doc); err != nil {
			return nil, fmt.Errorf("failed to scan row: %w", err)
		}
		r.DocID = r.Collection + ":" + r.Path
		r.Lines = strings.Count(doc, "\n") + 1
		results = append(results, r)
	}

	return results, nil
}

// GetStats returns index statistics
func (s *Store) GetStats() (*IndexStats, error) {
	stats := &IndexStats{
		CollectionStats: make(map[string]int),
	}

	// Count collections
	err := s.db.QueryRow("SELECT COUNT(DISTINCT collection) FROM documents WHERE active = 1").Scan(&stats.CollectionCount)
	if err != nil {
		return nil, err
	}

	// Count documents
	err = s.db.QueryRow("SELECT COUNT(*) FROM documents WHERE active = 1").Scan(&stats.DocumentCount)
	if err != nil {
		return nil, err
	}

	// Count indexed (documents with FTS entries)
	err = s.db.QueryRow("SELECT COUNT(*) FROM documents_fts").Scan(&stats.IndexedCount)
	if err != nil {
		return nil, err
	}

	// Count chunks
	err = s.db.QueryRow("SELECT COUNT(*) FROM content_vectors").Scan(&stats.ChunkCount)
	if err != nil {
		return nil, err
	}

	// Get per-collection stats
	rows, err := s.db.Query(`
		SELECT collection, COUNT(*) as count
		FROM documents
		WHERE active = 1
		GROUP BY collection
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	for rows.Next() {
		var collection string
		var count int
		if err := rows.Scan(&collection, &count); err != nil {
			return nil, err
		}
		stats.CollectionStats[collection] = count
	}

	return stats, nil
}

// GetDocument returns a document by path
func (s *Store) GetDocument(path string) (*SearchResult, string, error) {
	var r SearchResult
	var content string
	err := s.db.QueryRow(`
		SELECT collection, path, title, hash, doc
		FROM documents
		WHERE path = ? AND active = 1
	`, path).Scan(&r.Collection, &r.Path, &r.Title, &r.Hash, &content)

	if err == sql.ErrNoRows {
		return nil, "", fmt.Errorf("document not found")
	}
	if err != nil {
		return nil, "", err
	}

	r.DocID = r.Collection + ":" + r.Path
	r.Lines = strings.Count(content, "\n") + 1
	return &r, content, nil
}

// DocumentInfo represents document info for embedding
type DocumentInfo struct {
	ID     int64
	Hash   string
	Doc    string
}

// GetDocumentsForEmbedding returns documents that need embedding
func (s *Store) GetDocumentsForEmbedding(collection string, force bool) ([]DocumentInfo, error) {
	var rows *sql.Rows
	var err error

	if force {
		rows, err = s.db.Query(`
			SELECT id, hash, doc FROM documents
			WHERE collection = ? AND active = 1
		`, collection)
	} else {
		rows, err = s.db.Query(`
			SELECT id, hash, doc FROM documents
			WHERE collection = ? AND active = 1
			AND hash NOT IN (SELECT DISTINCT hash FROM content_vectors)
		`, collection)
	}

	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var docs []DocumentInfo
	for rows.Next() {
		var d DocumentInfo
		if err := rows.Scan(&d.ID, &d.Hash, &d.Doc); err != nil {
			return nil, err
		}
		docs = append(docs, d)
	}

	return docs, nil
}

// StoreEmbedding stores a single embedding
func (s *Store) StoreEmbedding(hash string, seq int, pos int, embedding []float32, model string) error {
	embeddingJSON, err := json.Marshal(embedding)
	if err != nil {
		return err
	}

	now := time.Now().UTC().Format(time.RFC3339)

	// Store in content_vectors metadata table
	_, err = s.db.Exec(`
		INSERT OR REPLACE INTO content_vectors (hash, seq, pos, model, embedded_at)
		VALUES (?, ?, ?, ?, ?)
	`, hash, seq, pos, model, now)
	if err != nil {
		return err
	}

	// Store in vectors_vec table
	hashSeq := fmt.Sprintf("%s_%d", hash, seq)
	_, err = s.db.Exec(`
		INSERT OR REPLACE INTO vectors_vec (hash_seq, embedding)
		VALUES (?, ?)
	`, hashSeq, string(embeddingJSON))

	return err
}

// DeleteEmbeddings deletes all embeddings for a document hash
func (s *Store) DeleteEmbeddings(hash string) error {
	// Delete from vectors_vec
	_, err := s.db.Exec(`DELETE FROM vectors_vec WHERE hash_seq LIKE ?`, hash+"_%")
	if err != nil {
		return err
	}

	// Delete from content_vectors
	_, err = s.db.Exec(`DELETE FROM content_vectors WHERE hash = ?`, hash)
	return err
}

// CollectionInfo represents collection info
type CollectionInfo struct {
	Name        string
	Path        string
	Pattern     string
	Description string
}

// GetCollections returns all collections
func (s *Store) GetCollections() ([]CollectionInfo, error) {
	rows, err := s.db.Query(`
		SELECT DISTINCT collection
		FROM documents
		WHERE active = 1
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var collections []CollectionInfo
	for rows.Next() {
		var name string
		if err := rows.Scan(&name); err != nil {
			return nil, err
		}
		collections = append(collections, CollectionInfo{Name: name})
	}

	// Also get from collections table
	rows2, err := s.db.Query(`SELECT name, path, pattern, description FROM collections`)
	if err != nil {
		return nil, err
	}
	defer rows2.Close()

	for rows2.Next() {
		var c CollectionInfo
		if err := rows2.Scan(&c.Name, &c.Path, &c.Pattern, &c.Description); err != nil {
			return nil, err
		}
		// Check if already exists
		found := false
		for _, existing := range collections {
			if existing.Name == c.Name {
				found = true
				break
			}
		}
		if !found {
			collections = append(collections, c)
		}
	}

	return collections, nil
}

// HashContent computes SHA256 hash of content
func HashContent(content string) string {
	h := sha256.Sum256([]byte(content))
	return fmt.Sprintf("%x", h)
}

// EscapeFTS5 escapes special FTS5 characters
func EscapeFTS5(query string) string {
	// Simple escaping - just quote the query for FTS5
	if strings.ContainsAny(query, `"(-)`) {
		return `"` + query + `"`
	}
	return query
}
