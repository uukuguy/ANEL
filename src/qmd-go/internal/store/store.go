package store

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"github.com/qmd/qmd-go/internal/config"
	"github.com/qmd/qmd-go/internal/llm"
	_ "github.com/mattn/go-sqlite3"
)

// SearchResult represents a search result
type SearchResult struct {
	Path       string
	Collection string
	Score      float32
	Lines      int
	Title      string
	Hash       string
}

// SearchOptions represents search options
type SearchOptions struct {
	Limit      int
	MinScore   float32
	Collection string
	SearchAll  bool
}

// IndexStats represents index statistics
type IndexStats struct {
	CollectionCount int
	DocumentCount   int
	IndexedCount    int
	PendingCount    int
}

// Store represents the main storage structure
type Store struct {
	config     *config.Config
	connections map[string]*sql.DB
	llmRouter  *llm.Router
	qdrant     *QdrantBackend
}

// New creates a new Store
func New(cfg *config.Config) (*Store, error) {
	store := &Store{
		config:     cfg,
		connections: make(map[string]*sql.DB),
		llmRouter:  llm.New(cfg),
	}

	// Initialize Qdrant backend if configured
	if cfg.Vector.Backend == "qdrant" {
		qdrant, err := NewQdrantBackend(
			cfg.Vector.Qdrant.URL,
			cfg.Vector.Qdrant.APIKey,
			cfg.Vector.Qdrant.Collection,
			uint64(cfg.Vector.Qdrant.VectorSize),
		)
		if err != nil {
			fmt.Printf("Warning: Qdrant backend not available: %v\n", err)
		} else {
			store.qdrant = qdrant
		}
	}

	// Initialize connections for each collection
	for _, col := range cfg.Collections {
		if _, err := store.GetConnection(col.Name); err != nil {
			return nil, err
		}
	}

	return store, nil
}

// GetConnection gets or creates a database connection
func (s *Store) GetConnection(collection string) (*sql.DB, error) {
	if db, ok := s.connections[collection]; ok {
		return db, nil
	}

	dbPath := filepath.Join(s.config.CachePath, collection, "index.db")

	if err := os.MkdirAll(filepath.Dir(dbPath), 0755); err != nil {
		return nil, err
	}

	db, err := sql.Open("sqlite3", dbPath)
	if err != nil {
		return nil, err
	}

	if err := s.initSchema(db); err != nil {
		return nil, err
	}

	s.connections[collection] = db
	return db, nil
}

// initSchema initializes the database schema
func (s *Store) initSchema(db *sql.DB) error {
	schema := `
		CREATE TABLE IF NOT EXISTS documents (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			collection TEXT NOT NULL,
			path TEXT NOT NULL,
			title TEXT NOT NULL,
			hash TEXT NOT NULL UNIQUE,
			created_at TEXT NOT NULL,
			modified_at TEXT NOT NULL,
			active INTEGER NOT NULL DEFAULT 1
		);

		CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
			filepath, title, body,
			tokenize='porter unicode61',
			content='documents',
			content_rowid='id'
		);

		CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents BEGIN
			INSERT INTO documents_fts(rowid, filepath, title, body)
			VALUES(new.id, new.collection || '/' || new.path, new.title,
				   (SELECT doc FROM content WHERE hash = new.hash));
		END;

		CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
			INSERT INTO documents_fts(documents_fts, rowid, filepath, title, body)
			VALUES('delete', old.id, old.collection || '/' || old.path, old.title, NULL);
		END;

		CREATE TABLE IF NOT EXISTS content (
			hash TEXT PRIMARY KEY,
			doc TEXT NOT NULL,
			size INTEGER NOT NULL DEFAULT 0
		);

		CREATE VIRTUAL TABLE IF NOT EXISTS vectors_vec USING vec0(
			hash_seq TEXT PRIMARY KEY,
			embedding float[384] distance_metric=cosine
		);

		CREATE TABLE IF NOT EXISTS content_vectors (
			hash TEXT NOT NULL,
			seq INTEGER NOT NULL DEFAULT 0,
			pos INTEGER NOT NULL DEFAULT 0,
			model TEXT NOT NULL,
			embedded_at TEXT NOT NULL,
			PRIMARY KEY (hash, seq)
		);

		CREATE INDEX IF NOT EXISTS idx_documents_collection ON documents(collection);
		CREATE INDEX IF NOT EXISTS idx_documents_hash ON documents(hash);
	`

	_, err := db.Exec(schema)
	return err
}

// BM25Search performs BM25 full-text search
func (s *Store) BM25Search(query string, options SearchOptions) ([]SearchResult, error) {
	var results []SearchResult

	collections := s.getCollections(options)

	for _, collection := range collections {
		db, err := s.GetConnection(collection)
		if err != nil {
			continue
		}

		rows, err := db.Query(`
			SELECT rowid, bm25(documents_fts), title, path
			FROM documents_fts
			WHERE documents_fts MATCH ? AND active = 1
			ORDER BY bm25(documents_fts)
			LIMIT ?
		`, fmt.Sprintf("%s NOT active:0", query), options.Limit)

		if err != nil {
			continue
		}

		for rows.Next() {
			var result SearchResult
			var rowID int64
			var score float64

			rows.Scan(&rowID, &score, &result.Title, &result.Path)
			result.Score = float32(score)
			result.Collection = collection
			result.Hash = fmt.Sprintf("%d", rowID)
			results = append(results, result)
		}
	}

	return results, nil
}

// VectorSearch performs vector search
func (s *Store) VectorSearch(query string, options SearchOptions) ([]SearchResult, error) {
	// Check vector backend configuration
	switch s.config.Vector.Backend {
	case "qmd_builtin":
		// Use sqlite-vec
		return s.VectorSearchSQLite(query, options)
	case "qdrant":
		// Use Qdrant backend
		return s.VectorSearchQdrant(query, options)
	default:
		// Fall back to BM25
		return s.BM25Search(query, options)
	}
}

// VectorSearchSQLite performs vector search using sqlite-vec
func (s *Store) VectorSearchSQLite(query string, options SearchOptions) ([]SearchResult, error) {
	ctx := context.Background()

	// Generate query embedding
	embeddingResult, err := s.llmRouter.Embed(ctx, []string{query})
	if err != nil {
		// Fall back to BM25
		return s.BM25Search(query, options)
	}

	queryVector := embeddingResult.Embeddings[0]

	// Search using sqlite-vec
	results := []SearchResult{}
	collections := s.getCollections(options)

	for _, collection := range collections {
		db, err := s.GetConnection(collection)
		if err != nil {
			continue
		}

		// Convert vector to JSON
		vectorJSON, _ := json.Marshal(queryVector)

		rows, err := db.Query(`
			SELECT
				v.hash_seq,
				v.embedding,
				d.title,
				d.path,
				d.hash,
				d.collection
			FROM vectors_vec v
			JOIN documents d ON v.hash_seq LIKE d.hash || '%'
			WHERE d.active = 1
			ORDER BY v.embedding <=> ?
			LIMIT ?
		`, string(vectorJSON), options.Limit)

		if err != nil {
			// sqlite-vec may not be available
			continue
		}

		for rows.Next() {
			var hashSeq string
			var embedding float64
			var title, path, hash, coll string

			rows.Scan(&hashSeq, &embedding, &title, &path, &hash, &coll)

			// Convert distance to score
			score := 1.0 / (1.0 + embedding)

			results = append(results, SearchResult{
				Path:       coll + "/" + path,
				Collection: coll,
				Score:      float32(score),
				Lines:      0,
				Title:      title,
				Hash:       hash,
			})
		}
	}

	if len(results) == 0 {
		// Fall back to BM25
		return s.BM25Search(query, options)
	}

	return results, nil
}

// HybridSearch performs hybrid search with reranking
func (s *Store) HybridSearch(query string, options SearchOptions) ([]SearchResult, error) {
	// Query expansion
	_ = s.expandQuery(query)

	// Parallel retrieval
	bm25Results, _ := s.BM25Search(query, options)
	vectorResults, _ := s.VectorSearch(query, options)

	// RRF fusion
	fused := s.rrfFusion([][]SearchResult{bm25Results, vectorResults}, nil, 60)

	// Top 30 for reranking
	candidates := fused
	if len(candidates) > 30 {
		candidates = candidates[:30]
	}

	return candidates, nil
}

func (s *Store) getCollections(options SearchOptions) []string {
	if options.SearchAll {
		collections := make([]string, len(s.config.Collections))
		for i, col := range s.config.Collections {
			collections[i] = col.Name
		}
		return collections
	}

	if options.Collection != "" {
		return []string{options.Collection}
	}

	if len(s.config.Collections) > 0 {
		return []string{s.config.Collections[0].Name}
	}

	return []string{}
}

func (s *Store) expandQuery(query string) []string {
	// TODO: Implement query expansion
	return []string{query}
}

func (s *Store) rrfFusion(resultLists [][]SearchResult, weights []float32, k int) []SearchResult {
	// TODO: Implement RRF fusion
	if len(resultLists) == 0 {
		return []SearchResult{}
	}

	return resultLists[0]
}

// GetStats returns index statistics
func (s *Store) GetStats() (*IndexStats, error) {
	stats := &IndexStats{
		CollectionCount: len(s.config.Collections),
	}

	for _, collection := range s.config.Collections {
		db, err := s.GetConnection(collection.Name)
		if err != nil {
			continue
		}

		var count int
		err = db.QueryRow("SELECT COUNT(*) FROM documents WHERE active = 1").Scan(&count)
		if err != nil {
			continue
		}

		stats.DocumentCount += count
	}

	stats.IndexedCount = stats.DocumentCount
	return stats, nil
}
