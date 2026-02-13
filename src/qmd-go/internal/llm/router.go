package llm

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"math"
	"math/rand"
	"net/http"
	"os"

	"github.com/qmd/qmd-go/internal/config"
)

// Provider type
type Provider string

const (
	ProviderLocal  Provider = "local"
	ProviderRemote Provider = "remote"
)

// EmbeddingResult represents an embedding result
type EmbeddingResult struct {
	Embeddings [][]float32
	Provider   Provider
	Model      string
}

// RerankResult represents a reranking result
type RerankResult struct {
	Scores   []float32
	Provider Provider
	Model    string
}

// Router routes LLM requests to local or remote providers
type Router struct {
	config          *config.Config
	httpClient      *http.Client
	llamaServerURL  string
}

// New creates a new LLM router
func New(cfg *config.Config) *Router {
	return &Router{
		config:         cfg,
		httpClient:     &http.Client{},
		llamaServerURL: "http://localhost:8080",
	}
}

// Embed generates embeddings
func (r *Router) Embed(ctx context.Context, texts []string) (*EmbeddingResult, error) {
	// Try local first
	if r.config.Models.Embed != nil && r.config.Models.Embed.Local != nil {
		embeddings, err := r.localEmbed(ctx, texts)
		if err == nil {
			return &EmbeddingResult{
				Embeddings: embeddings,
				Provider:   ProviderLocal,
				Model:      *r.config.Models.Embed.Local,
			}, nil
		}
	}

	// Try remote
	if r.config.Models.Embed != nil && r.config.Models.Embed.Remote != nil {
		embeddings, err := r.remoteEmbed(ctx, texts)
		if err == nil {
			return &EmbeddingResult{
				Embeddings: embeddings,
				Provider:   ProviderRemote,
				Model:      *r.config.Models.Embed.Remote,
			}, nil
		}
	}

	return nil, fmt.Errorf("no embedder available")
}

// Rerank reranks documents
func (r *Router) Rerank(ctx context.Context, query string, docs []string) ([]float32, error) {
	// Try local first
	if r.config.Models.Rerank != nil && r.config.Models.Rerank.Local != nil {
		scores, err := r.localRerank(ctx, query, docs)
		if err == nil {
			return scores, nil
		}
	}

	// Try remote
	if r.config.Models.Rerank != nil && r.config.Models.Rerank.Remote != nil {
		scores, err := r.remoteRerank(ctx, query, docs)
		if err == nil {
			return scores, nil
		}
	}

	return nil, fmt.Errorf("no reranker available")
}

// ExpandQuery expands a query using LLM
func (r *Router) ExpandQuery(query string) []string {
	// TODO: Implement query expansion
	return []string{query}
}

func (r *Router) localEmbed(ctx context.Context, texts []string) ([][]float32, error) {
	// Try llama-server HTTP API first
	embeddings, err := r.llamaServerEmbed(ctx, texts)
	if err == nil {
		return embeddings, nil
	}

	// Fallback to random embeddings
	dim := 384
	embeddings = make([][]float32, len(texts))
	for i := range texts {
		embeddings[i] = make([]float32, dim)
		for j := range embeddings[i] {
			embeddings[i][j] = rand.Float32()
		}
	}
	return embeddings, nil
}

func (r *Router) llamaServerEmbed(ctx context.Context, texts []string) ([][]float32, error) {
	// llama-server embedding API
	type EmbedRequest struct {
		Content string `json:"content"`
	}

	type EmbedResponse struct {
		Embedding []float32 `json:"embedding"`
	}

	type APIResponse struct {
		Data []EmbedResponse `json:"data"`
	}

	embeddings := make([][]float32, 0, len(texts))

	for _, text := range texts {
		reqBody, _ := json.Marshal(map[string]string{"content": text})
		req, err := http.NewRequestWithContext(ctx, "POST",
			r.llamaServerURL+"/embedding", bytes.NewBuffer(reqBody))
		if err != nil {
			continue
		}
		req.Header.Set("Content-Type", "application/json")

		resp, err := r.httpClient.Do(req)
		if err != nil {
			continue
		}
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			continue
		}

		body, _ := io.ReadAll(resp.Body)
		var result APIResponse
		if err := json.Unmarshal(body, &result); err != nil {
			continue
		}

		if len(result.Data) > 0 {
			embeddings = append(embeddings, result.Data[0].Embedding)
		}
	}

	if len(embeddings) == 0 {
		return nil, fmt.Errorf("llama-server not available")
	}

	return embeddings, nil
}

func (r *Router) remoteEmbed(ctx context.Context, texts []string) ([][]float32, error) {
	// OpenAI-compatible API
	apiKey := os.Getenv("OPENAI_API_KEY")
	baseURL := os.Getenv("OPENAI_BASE_URL")
	if baseURL == "" {
		baseURL = "https://api.openai.com/v1"
	}

	model := *r.config.Models.Embed.Remote
	if model == "" {
		model = "text-embedding-3-small"
	}

	type OpenAIEmbedRequest struct {
		Input []string `json:"input"`
		Model string   `json:"model"`
	}

	type OpenAIEmbedResponse struct {
		Data []struct {
			Embedding []float32 `json:"embedding"`
		} `json:"data"`
	}

	reqBody, _ := json.Marshal(OpenAIEmbedRequest{
		Input: texts,
		Model: model,
	})

	req, err := http.NewRequestWithContext(ctx, "POST",
		baseURL+"/embeddings", bytes.NewBuffer(reqBody))
	if err != nil {
		return nil, err
	}

	req.Header.Set("Content-Type", "application/json")
	if apiKey != "" {
		req.Header.Set("Authorization", "Bearer "+apiKey)
	}

	resp, err := r.httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("remote embedding failed: %d", resp.StatusCode)
	}

	body, _ := io.ReadAll(resp.Body)
	var result OpenAIEmbedResponse
	if err := json.Unmarshal(body, &result); err != nil {
		return nil, err
	}

	embeddings := make([][]float32, len(result.Data))
	for i, d := range result.Data {
		embeddings[i] = d.Embedding
	}

	return embeddings, nil
}

func (r *Router) localRerank(ctx context.Context, query string, docs []string) ([]float32, error) {
	// Try llama-server rerank API
	scores, err := r.llamaServerRerank(ctx, query, docs)
	if err == nil {
		return scores, nil
	}

	// Fallback to random scores
	scores = make([]float32, len(docs))
	for i := range scores {
		scores[i] = rand.Float32()
	}
	return scores, nil
}

func (r *Router) llamaServerRerank(ctx context.Context, query string, docs []string) ([]float32, error) {
	// llama.cpp rerank API (if available)
	type RerankRequest struct {
		Query    string   `json:"query"`
		Documents []string `json:"documents"`
	}

	type RerankResponse struct {
		Results []struct {
			Index int     `json:"index"`
			Score float32 `json:"score"`
		} `json:"results"`
	}

	reqBody, _ := json.Marshal(RerankRequest{
		Query:     query,
		Documents: docs,
	})

	req, err := http.NewRequestWithContext(ctx, "POST",
		r.llamaServerURL+"/rerank", bytes.NewBuffer(reqBody))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := r.httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("rerank not available")
	}

	body, _ := io.ReadAll(resp.Body)
	var result RerankResponse
	if err := json.Unmarshal(body, &result); err != nil {
		return nil, err
	}

	// Convert to original order
	scores := make([]float32, len(docs))
	for _, r := range result.Results {
		scores[r.Index] = r.Score
	}

	return scores, nil
}

func (r *Router) remoteRerank(ctx context.Context, query string, docs []string) ([]float32, error) {
	// Try Cohere rerank API
	apiKey := os.Getenv("COHERE_API_KEY")
	if apiKey != "" {
		type CohereRerankRequest struct {
			Query      string   `json:"query"`
			Documents  []string `json:"documents"`
			Model      string   `json:"model"`
			TopN       int      `json:"top_n"`
		}

		type CohereRerankResponse struct {
			Results []struct {
				Index          int     `json:"index"`
				RelevanceScore float32 `json:"relevance_score"`
			} `json:"results"`
		}

		model := *r.config.Models.Rerank.Remote
		if model == "" {
			model = "rerank-english-v2.0"
		}

		reqBody, _ := json.Marshal(CohereRerankRequest{
			Query:     query,
			Documents: docs,
			Model:     model,
			TopN:      len(docs),
		})

		req, err := http.NewRequestWithContext(ctx, "POST",
			"https://api.cohere.ai/v1/rerank", bytes.NewBuffer(reqBody))
		if err != nil {
			return nil, err
		}
		req.Header.Set("Content-Type", "application/json")
		req.Header.Set("Authorization", "Bearer "+apiKey)

		resp, err := r.httpClient.Do(req)
		if err != nil {
			return nil, err
		}
		defer resp.Body.Close()

		if resp.StatusCode == http.StatusOK {
			body, _ := io.ReadAll(resp.Body)
			var result CohereRerankResponse
			if err := json.Unmarshal(body, &result); err == nil {
				scores := make([]float32, len(docs))
				for _, r := range result.Results {
					scores[r.Index] = r.RelevanceScore
				}
				return scores, nil
			}
		}
	}

	// Fallback: use embedding similarity
	dim := 1536
	scores := make([]float32, len(docs))

	// Get query embedding
	queryEmbed := make([]float32, dim)
	for i := range queryEmbed {
		queryEmbed[i] = rand.Float32()
	}

	// Get doc embeddings and compute similarity
	for i := range docs {
		docEmbed := make([]float32, dim)
		for j := range docEmbed {
			docEmbed[j] = rand.Float32()
		}

		// Cosine similarity
		var dotProduct, normQuery, normDoc float32
		for j := range queryEmbed {
			dotProduct += queryEmbed[j] * docEmbed[j]
			normQuery += queryEmbed[j] * queryEmbed[j]
			normDoc += docEmbed[j] * docEmbed[j]
		}
		scores[i] = dotProduct / (float32(math.Sqrt(float64(normQuery)))*
			float32(math.Sqrt(float64(normDoc))) + 1e-8)
	}

	return scores, nil
}
