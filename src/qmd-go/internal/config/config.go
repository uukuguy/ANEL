package config

import (
	"os"
	"path/filepath"

	"gopkg.in/yaml.v3"
)

const (
	DefaultConfigPath = "~/.config/qmd/index.yaml"
	DefaultCachePath  = "~/.cache/qmd"
)

// BM25Backend type
type BM25Backend string

const (
	BM25BackendSqliteFTS5 BM25Backend = "sqlite_fts5"
	BM25BackendLanceDB    BM25Backend = "lancedb"
)

// VectorBackend type
type VectorBackend string

const (
	VectorBackendQmdBuiltin VectorBackend = "qmd_builtin"
	VectorBackendLanceDB    VectorBackend = "lancedb"
)

// CollectionConfig represents a collection configuration
type CollectionConfig struct {
	Name        string  `yaml:"name"`
	Path        string  `yaml:"path"`
	Pattern     *string `yaml:"pattern,omitempty"`
	Description *string `yaml:"description,omitempty"`
}

// BM25Config represents BM25 backend configuration
type BM25Config struct {
	Backend BM25Backend `yaml:"backend"`
}

// VectorConfig represents vector backend configuration
type VectorConfig struct {
	Backend VectorBackend `yaml:"backend"`
	Model   string        `yaml:"model"`
}

// LLMModelConfig represents LLM model configuration
type LLMModelConfig struct {
	Local  *string `yaml:"local,omitempty"`
	Remote *string `yaml:"remote,omitempty"`
}

// ModelsConfig represents models configuration
type ModelsConfig struct {
	Embed          *LLMModelConfig `yaml:"embed,omitempty"`
	Rerank         *LLMModelConfig `yaml:"rerank,omitempty"`
	QueryExpansion *LLMModelConfig `yaml:"query_expansion,omitempty"`
}

// Config represents the main configuration
type Config struct {
	BM25        BM25Config        `yaml:"bm25"`
	Vector      VectorConfig      `yaml:"vector"`
	Collections []CollectionConfig `yaml:"collections"`
	Models      ModelsConfig      `yaml:"models"`
	CachePath   string            `yaml:"cache_path"`
}

// DefaultConfig returns default configuration
func DefaultConfig() *Config {
	return &Config{
		BM25: BM25Config{
			Backend: BM25BackendSqliteFTS5,
		},
		Vector: VectorConfig{
			Backend: VectorBackendQmdBuiltin,
			Model:   "embeddinggemma-300M",
		},
		Collections: []CollectionConfig{},
		Models:      ModelsConfig{},
		CachePath:   DefaultCachePath,
	}
}

// LoadConfig loads configuration from file
func LoadConfig() (*Config, error) {
	return LoadConfigFromFile(expandPath(DefaultConfigPath))
}

// LoadConfigFromFile loads configuration from a specific file
func LoadConfigFromFile(path string) (*Config, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		if os.IsNotExist(err) {
			return DefaultConfig(), nil
		}
		return nil, err
	}

	return LoadConfigFromData(data)
}

// LoadConfigFromData loads configuration from byte data
func LoadConfigFromData(data []byte) (*Config, error) {
	config := &Config{
		BM25: BM25Config{
			Backend: BM25BackendSqliteFTS5,
		},
		Vector: VectorConfig{
			Backend: VectorBackendQmdBuiltin,
			Model:   "embeddinggemma-300M",
		},
		CachePath: DefaultCachePath,
	}

	if err := yaml.Unmarshal(data, config); err != nil {
		return nil, err
	}

	return config, nil
}

// Save saves configuration to file
func (c *Config) Save() error {
	path := expandPath(DefaultConfigPath)

	if err := os.MkdirAll(filepath.Dir(path), 0755); err != nil {
		return err
	}

	data, err := yaml.Marshal(c)
	if err != nil {
		return err
	}

	return os.WriteFile(path, data, 0644)
}

// DBPath returns database path for a collection
func (c *Config) DBPath(collection string) string {
	return filepath.Join(c.CachePath, collection, "index.db")
}

func expandPath(path string) string {
	if home, err := os.UserHomeDir(); err == nil {
		if len(path) > 1 && path[:2] == "~/" {
			return filepath.Join(home, path[2:])
		}
	}
	return path
}
