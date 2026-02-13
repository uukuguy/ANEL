package llm

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"

	"github.com/qmd/qmd-go/pkg/config"
)

// Provider type
type Provider string

const (
	ProviderLocal  Provider = "local"
	ProviderRemote Provider = "remote"
)

// EmbeddingResult represents embedding results
type EmbeddingResult struct {
	Embeddings [][]float32
	Provider   Provider
	Model      string
}

// Embedder interface
type Embedder interface {
	Embed(texts []string) (*EmbeddingResult, error)
}

// Reranker interface
type Reranker interface {
	Rerank(query string, docs []string) ([]float32, error)
}

// Router is the main LLM router
type Router struct {
	config          *config.Config
	localEmbedder   Embedder
	remoteEmbedder  Embedder
	localReranker  Reranker
	remoteReranker Reranker
	mu              sync.Mutex
}

// New creates a new Router
func New(cfg *config.Config) *Router {
	return &Router{
		config: cfg,
	}
}

// HasEmbedder returns true if embedder is available
func (r *Router) HasEmbedder() bool {
	return r.localEmbedder != nil || r.remoteEmbedder != nil
}

// HasReranker returns true if reranker is available
func (r *Router) HasReranker() bool {
	return r.localReranker != nil || r.remoteReranker != nil
}

// Embed generates embeddings for texts
func (r *Router) Embed(texts []string) (*EmbeddingResult, error) {
	if r.localEmbedder != nil {
		return r.localEmbedder.Embed(texts)
	}
	if r.remoteEmbedder != nil {
		return r.remoteEmbedder.Embed(texts)
	}
	return nil, fmt.Errorf("no embedder available")
}

// Rerank reranks documents
func (r *Router) Rerank(query string, docs []string) ([]float32, error) {
	if r.localReranker != nil {
		return r.localReranker.Rerank(query, docs)
	}
	if r.remoteReranker != nil {
		return r.remoteReranker.Rerank(query, docs)
	}
	return nil, fmt.Errorf("no reranker available")
}

// SetLocalEmbedder sets local embedder
func (r *Router) SetLocalEmbedder(e Embedder) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.localEmbedder = e
}

// SetRemoteEmbedder sets remote embedder
func (r *Router) SetRemoteEmbedder(e Embedder) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.remoteEmbedder = e
}

// SetLocalReranker sets local reranker
func (r *Router) SetLocalReranker(rr Reranker) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.localReranker = rr
}

// SetRemoteReranker sets remote reranker
func (r *Router) SetRemoteReranker(rr Reranker) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.remoteReranker = rr
}

// LocalEmbedder represents a local embedding model
type LocalEmbedder struct {
	modelPath string
	modelName string
}

// NewLocalEmbedder creates a new LocalEmbedder
func NewLocalEmbedder(modelName string) (*LocalEmbedder, error) {
	homeDir, _ := os.UserHomeDir()
	modelPath := filepath.Join(homeDir, ".cache", "qmd", "models", modelName)

	if _, err := os.Stat(modelPath); os.IsNotExist(err) {
		return nil, fmt.Errorf("model not found: %s", modelPath)
	}

	return &LocalEmbedder{
		modelPath: modelPath,
		modelName: modelName,
	}, nil
}

// Embed generates embeddings using llama.cpp
func (e *LocalEmbedder) Embed(texts []string) (*EmbeddingResult, error) {
	// Use llama.cpp CLI for embedding
	args := []string{
		"-m", e.modelPath,
		"--embedding", "true",
		"-p", texts[0],
	}

	cmd := exec.Command("llama-cli", args...)
	output, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("llama-cli failed: %w", err)
	}

	// Parse embedding from output
	embedding, err := parseLLamaEmbedding(string(output))
	if err != nil {
		return nil, err
	}

	return &EmbeddingResult{
		Embeddings: [][]float32{embedding},
		Provider:   ProviderLocal,
		Model:      e.modelName,
	}, nil
}

// RemoteEmbedder represents a remote embedding API
type RemoteEmbedder struct {
	model   string
	apiURL  string
	apiKey  string
}

// NewRemoteEmbedder creates a new RemoteEmbedder
func NewRemoteEmbedder(model, apiURL, apiKey string) *RemoteEmbedder {
	return &RemoteEmbedder{
		model:  model,
		apiURL: apiURL,
		apiKey: apiKey,
	}
}

// Embed generates embeddings using remote API
func (e *RemoteEmbedder) Embed(texts []string) (*EmbeddingResult, error) {
	// Use OpenAI-compatible API
	type Request struct {
		Input []string `json:"input"`
		Model string   `json:"model"`
	}

	type Response struct {
		Data []struct {
			Embedding []float32 `json:"embedding"`
		} `json:"data"`
	}

	reqBody, _ := json.Marshal(Request{
		Input: texts,
		Model: e.model,
	})

	// Note: In real implementation, use HTTP client with apiKey
	_ = reqBody

	return &EmbeddingResult{
		Embeddings: make([][]float32, len(texts)),
		Provider:   ProviderRemote,
		Model:      e.model,
	}, nil
}

// LocalReranker represents a local reranking model
type LocalReranker struct {
	modelPath string
	modelName string
}

// NewLocalReranker creates a new LocalReranker
func NewLocalReranker(modelName string) (*LocalReranker, error) {
	homeDir, _ := os.UserHomeDir()
	modelPath := filepath.Join(homeDir, ".cache", "qmd", "models", modelName)

	if _, err := os.Stat(modelPath); os.IsNotExist(err) {
		return nil, fmt.Errorf("model not found: %s", modelPath)
	}

	return &LocalReranker{
		modelPath: modelPath,
		modelName: modelName,
	}, nil
}

// Rerank reranks documents using local model
func (r *LocalReranker) Rerank(query string, docs []string) ([]float32, error) {
	// Format: BGE-reranker prompt format
	// "[QUERY] [/s] [DOCUMENT]"
	prompt := fmt.Sprintf("%s</s>%s", query, docs[0])

	args := []string{
		"-m", r.modelPath,
		"-p", prompt,
	}

	cmd := exec.Command("llama-cli", args...)
	_, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("llama-cli failed: %w", err)
	}

	// Parse yes/no token probabilities for relevance score
	scores := make([]float32, len(docs))
	for i := range docs {
		scores[i] = 0.5 // Placeholder - needs logit extraction
	}

	return scores, nil
}

// RemoteReranker represents a remote reranking API
type RemoteReranker struct {
	model  string
	apiURL string
	apiKey string
}

// NewRemoteReranker creates a new RemoteReranker
func NewRemoteReranker(model, apiURL, apiKey string) *RemoteReranker {
	return &RemoteReranker{
		model:  model,
		apiURL: apiURL,
		apiKey: apiKey,
	}
}

// Rerank reranks documents using remote API
func (r *RemoteReranker) Rerank(query string, docs []string) ([]float32, error) {
	// Use OpenAI-compatible reranking API
	type Request struct {
		Query   string   `json:"query"`
		Documents []string `json:"documents"`
		Model   string   `json:"model"`
	}

	type Response struct {
		Results []struct {
			Index  int     `json:"index"`
			Score float32 `json:"score"`
		} `json:"results"`
	}

	reqBody, _ := json.Marshal(Request{
		Query:     query,
		Documents: docs,
		Model:     r.model,
	})

	_ = reqBody

	scores := make([]float32, len(docs))
	for i := range docs {
		scores[i] = 1.0 / float32(i+1) // Placeholder
	}

	return scores, nil
}

// parseLLamaEmbedding parses embedding from llama.cpp output
func parseLLamaEmbedding(output string) ([]float32, error) {
	// Simple parsing - look for [0.1, 0.2, ...] pattern
	start := strings.Index(output, "[")
	end := strings.LastIndex(output, "]")
	if start == -1 || end == -1 {
		return nil, fmt.Errorf("no embedding found in output")
	}

	embedStr := output[start : end+1]
	var embedding []float32
	if err := json.Unmarshal([]byte(embedStr), &embedding); err != nil {
		return nil, fmt.Errorf("failed to parse embedding: %w", err)
	}

	return embedding, nil
}

// QueryExpander expands queries with synonyms
type QueryExpander struct{}

var expansionTerms = map[string][]string{
	"how":      {"how to", "guide", "tutorial"},
	"what":     {"what is", "definition", "explanation"},
	"why":      {"reason", "explanation", "purpose"},
	"config":   {"configuration", "settings", "setup"},
	"install":  {"installation", "setup", "deployment"},
	"error":    {"error", "issue", "problem", "bug"},
	"api":      {"api", "interface", "endpoint"},
	"doc":      {"documentation", "docs", "guide"},
}

// ExpandQuery expands query with synonyms
func (e *QueryExpander) ExpandQuery(query string) []string {
	words := strings.Fields(strings.ToLower(query))
	expanded := []string{query}

	for _, word := range words {
		if terms, ok := expansionTerms[word]; ok {
			expanded = append(expanded, terms...)
		}
	}

	return expanded
}
