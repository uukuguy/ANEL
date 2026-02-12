package config

import (
	"fmt"
	"os"
	"path/filepath"

	"gopkg.in/yaml.v3"
)

// BM25Backend type
type BM25Backend string

const (
	BM25SqliteFts5 BM25Backend = "sqlite_fts5"
	BM25LanceDB    BM25Backend = "lancedb"
)

// VectorBackend type
type VectorBackend string

const (
	VectorQmdBuiltin VectorBackend = "qmd_builtin"
	VectorLanceDB    VectorBackend = "lancedb"
)

// CollectionConfig represents a document collection
type CollectionConfig struct {
	Name        string   `yaml:"name"`
	Path        string   `yaml:"path"`
	Pattern     string   `yaml:"pattern,omitempty"`
	Description string   `yaml:"description,omitempty"`
}

// LLMModelConfig represents LLM model configuration
type LLMModelConfig struct {
	Local  string `yaml:"local,omitempty"`
	Remote string `yaml:"remote,omitempty"`
}

// ModelsConfig represents models configuration
type ModelsConfig struct {
	Embed          *LLMModelConfig `yaml:"embed,omitempty"`
	Rerank         *LLMModelConfig `yaml:"rerank,omitempty"`
	QueryExpansion *LLMModelConfig `yaml:"query_expansion,omitempty"`
}

// Config represents the main configuration
type Config struct {
	BM25       BM25Backend             `yaml:"bm25"`
	Vector     VectorBackendConfig     `yaml:"vector"`
	Collections []CollectionConfig     `yaml:"collections"`
	Models     ModelsConfig           `yaml:"models"`
	CachePath  string                 `yaml:"cache_path"`
}

// VectorBackendConfig represents vector backend configuration
type VectorBackendConfig struct {
	Backend VectorBackend `yaml:"backend"`
	Model   string        `yaml:"model"`
}

// DefaultConfig returns default configuration
func DefaultConfig() *Config {
	homeDir, _ := os.UserHomeDir()
	cachePath := filepath.Join(homeDir, ".cache", "qmd")

	return &Config{
		BM25:       BM25SqliteFts5,
		Vector:     VectorBackendConfig{Backend: VectorQmdBuiltin, Model: "nomic-embed-text-v1.5"},
		Collections: []CollectionConfig{},
		Models:     ModelsConfig{},
		CachePath:  cachePath,
	}
}

// Load loads configuration from file
func Load(configPath string) (*Config, error) {
	cfg := DefaultConfig()

	if configPath == "" {
		configPath = filepath.Join(os.Getenv("HOME"), ".config", "qmd", "index.yaml")
	}

	data, err := os.ReadFile(configPath)
	if err != nil {
		if os.IsNotExist(err) {
			return cfg, nil
		}
		return nil, fmt.Errorf("failed to read config file: %w", err)
	}

	if err := yaml.Unmarshal(data, cfg); err != nil {
		return nil, fmt.Errorf("failed to parse config file: %w", err)
	}

	return cfg, nil
}

// Save saves configuration to file
func (c *Config) Save(configPath string) error {
	if configPath == "" {
		configPath = filepath.Join(os.Getenv("HOME"), ".config", "qmd", "index.yaml")
	}

	dir := filepath.Dir(configPath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return fmt.Errorf("failed to create config directory: %w", err)
	}

	data, err := yaml.Marshal(c)
	if err != nil {
		return fmt.Errorf("failed to marshal config: %w", err)
	}

	if err := os.WriteFile(configPath, data, 0644); err != nil {
		return fmt.Errorf("failed to write config file: %w", err)
	}

	return nil
}

// GetCollection returns collection by name
func (c *Config) GetCollection(name string) *CollectionConfig {
	for i := range c.Collections {
		if c.Collections[i].Name == name {
			return &c.Collections[i]
		}
	}
	return nil
}

// AddCollection adds a new collection
func (c *Config) AddCollection(col CollectionConfig) {
	c.Collections = append(c.Collections, col)
}

// RemoveCollection removes a collection by name
func (c *Config) RemoveCollection(name string) {
	newCollections := make([]CollectionConfig, 0)
	for _, col := range c.Collections {
		if col.Name != name {
			newCollections = append(newCollections, col)
		}
	}
	c.Collections = newCollections
}

// CacheDir returns cache directory for a collection
func (c *Config) CacheDirFor(collection string) string {
	return filepath.Join(c.CachePath, collection)
}

// DBPath returns database path for a collection
func (c *Config) DBPathFor(collection string) string {
	return filepath.Join(c.CacheDirFor(collection), "index.db")
}
