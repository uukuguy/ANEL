package config

import (
	"testing"
)

func TestDefaultConfig(t *testing.T) {
	cfg := DefaultConfig()

	if cfg.BM25.Backend != BM25BackendSqliteFTS5 {
		t.Errorf("BM25.Backend = %s, want sqlite_fts5", cfg.BM25.Backend)
	}
	if cfg.Vector.Backend != VectorBackendQmdBuiltin {
		t.Errorf("Vector.Backend = %s, want qmd_builtin", cfg.Vector.Backend)
	}
	if cfg.Vector.Model != "embeddinggemma-300M" {
		t.Errorf("Vector.Model = %s, want embeddinggemma-300M", cfg.Vector.Model)
	}
	if cfg.Vector.Qdrant.URL != "http://localhost:6333" {
		t.Errorf("Qdrant.URL = %s, want http://localhost:6333", cfg.Vector.Qdrant.URL)
	}
	if cfg.Vector.Qdrant.Collection != "qmd_documents" {
		t.Errorf("Qdrant.Collection = %s, want qmd_documents", cfg.Vector.Qdrant.Collection)
	}
	if cfg.Vector.Qdrant.VectorSize != 384 {
		t.Errorf("Qdrant.VectorSize = %d, want 384", cfg.Vector.Qdrant.VectorSize)
	}
	if len(cfg.Collections) != 0 {
		t.Errorf("Collections should be empty, got %d", len(cfg.Collections))
	}
	if cfg.CachePath != DefaultCachePath {
		t.Errorf("CachePath = %s, want %s", cfg.CachePath, DefaultCachePath)
	}
}

func TestLoadConfigFromData_Minimal(t *testing.T) {
	data := []byte(`
bm25:
  backend: sqlite_fts5
`)
	cfg, err := LoadConfigFromData(data)
	if err != nil {
		t.Fatalf("LoadConfigFromData failed: %v", err)
	}

	if cfg.BM25.Backend != BM25BackendSqliteFTS5 {
		t.Errorf("BM25.Backend = %s, want sqlite_fts5", cfg.BM25.Backend)
	}
	// Defaults should be preserved
	if cfg.Vector.Backend != VectorBackendQmdBuiltin {
		t.Errorf("Vector.Backend default not preserved: %s", cfg.Vector.Backend)
	}
}

func TestLoadConfigFromData_WithCollections(t *testing.T) {
	data := []byte(`
collections:
  - name: notes
    path: ~/notes
    pattern: "**/*.md"
  - name: docs
    path: ~/docs
`)
	cfg, err := LoadConfigFromData(data)
	if err != nil {
		t.Fatalf("LoadConfigFromData failed: %v", err)
	}

	if len(cfg.Collections) != 2 {
		t.Fatalf("Collections count = %d, want 2", len(cfg.Collections))
	}
	if cfg.Collections[0].Name != "notes" {
		t.Errorf("Collections[0].Name = %s, want notes", cfg.Collections[0].Name)
	}
	if cfg.Collections[1].Name != "docs" {
		t.Errorf("Collections[1].Name = %s, want docs", cfg.Collections[1].Name)
	}
	if cfg.Collections[0].Pattern == nil || *cfg.Collections[0].Pattern != "**/*.md" {
		t.Errorf("Collections[0].Pattern = %v, want **/*.md", cfg.Collections[0].Pattern)
	}
}

func TestLoadConfigFromData_QdrantBackend(t *testing.T) {
	data := []byte(`
vector:
  backend: qdrant
  model: embeddinggemma-300M
  qdrant:
    url: http://qdrant:6333
    api_key: secret
    collection: my_docs
    vector_size: 768
`)
	cfg, err := LoadConfigFromData(data)
	if err != nil {
		t.Fatalf("LoadConfigFromData failed: %v", err)
	}

	if cfg.Vector.Backend != VectorBackendQdrant {
		t.Errorf("Vector.Backend = %s, want qdrant", cfg.Vector.Backend)
	}
	if cfg.Vector.Qdrant.URL != "http://qdrant:6333" {
		t.Errorf("Qdrant.URL = %s, want http://qdrant:6333", cfg.Vector.Qdrant.URL)
	}
	if cfg.Vector.Qdrant.APIKey != "secret" {
		t.Errorf("Qdrant.APIKey not set correctly")
	}
	if cfg.Vector.Qdrant.VectorSize != 768 {
		t.Errorf("Qdrant.VectorSize = %d, want 768", cfg.Vector.Qdrant.VectorSize)
	}
}

func TestLoadConfigFromData_WithModels(t *testing.T) {
	localEmbed := "embeddinggemma-300M"
	localRerank := "qwen3-reranker"
	data := []byte(`
models:
  embed:
    local: embeddinggemma-300M
  rerank:
    local: qwen3-reranker
`)
	cfg, err := LoadConfigFromData(data)
	if err != nil {
		t.Fatalf("LoadConfigFromData failed: %v", err)
	}

	if cfg.Models.Embed == nil || *cfg.Models.Embed.Local != localEmbed {
		t.Errorf("Models.Embed.Local = %v, want %s", cfg.Models.Embed, localEmbed)
	}
	if cfg.Models.Rerank == nil || *cfg.Models.Rerank.Local != localRerank {
		t.Errorf("Models.Rerank.Local = %v, want %s", cfg.Models.Rerank, localRerank)
	}
}

func TestLoadConfigFromData_InvalidYAML(t *testing.T) {
	data := []byte(`{{{invalid yaml`)
	_, err := LoadConfigFromData(data)
	if err == nil {
		t.Error("LoadConfigFromData should fail on invalid YAML")
	}
}

func TestLoadConfigFromFile_NonExistent(t *testing.T) {
	cfg, err := LoadConfigFromFile("/tmp/nonexistent-qmd-config-test.yaml")
	if err != nil {
		t.Fatalf("LoadConfigFromFile should return default for missing file, got error: %v", err)
	}

	// Should return default config
	if cfg.BM25.Backend != BM25BackendSqliteFTS5 {
		t.Errorf("Should return default config, got BM25.Backend = %s", cfg.BM25.Backend)
	}
}

func TestDBPath(t *testing.T) {
	cfg := DefaultConfig()
	cfg.CachePath = "/tmp/qmd-test"

	path := cfg.DBPath("notes")
	if path != "/tmp/qmd-test/notes/index.db" {
		t.Errorf("DBPath = %s, want /tmp/qmd-test/notes/index.db", path)
	}
}

func TestBackendConstants(t *testing.T) {
	if BM25BackendSqliteFTS5 != "sqlite_fts5" {
		t.Errorf("BM25BackendSqliteFTS5 = %s", BM25BackendSqliteFTS5)
	}
	if BM25BackendLanceDB != "lancedb" {
		t.Errorf("BM25BackendLanceDB = %s", BM25BackendLanceDB)
	}
	if VectorBackendQmdBuiltin != "qmd_builtin" {
		t.Errorf("VectorBackendQmdBuiltin = %s", VectorBackendQmdBuiltin)
	}
	if VectorBackendLanceDB != "lancedb" {
		t.Errorf("VectorBackendLanceDB = %s", VectorBackendLanceDB)
	}
	if VectorBackendQdrant != "qdrant" {
		t.Errorf("VectorBackendQdrant = %s", VectorBackendQdrant)
	}
}
