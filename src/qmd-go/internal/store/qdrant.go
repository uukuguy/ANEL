package store

import (
	"context"
	"fmt"

	"github.com/qdrant/go-client/qdrant"
)

// QdrantBackend represents a Qdrant vector database backend
type QdrantBackend struct {
	client     *qdrant.Client
	collection string
	vectorSize uint64
}

// NewQdrantBackend creates a new Qdrant backend
func NewQdrantBackend(url, apiKey, collection string, vectorSize uint64) (*QdrantBackend, error) {
	client, err := qdrant.NewClient(&qdrant.Config{
		Host: url,
		APIKey: apiKey,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to create Qdrant client: %w", err)
	}

	backend := &QdrantBackend{
		client:     client,
		collection: collection,
		vectorSize: vectorSize,
	}

	// Ensure collection exists
	if err := backend.ensureCollection(); err != nil {
		return nil, err
	}

	return backend, nil
}

// ensureCollection creates the collection if it doesn't exist
func (b *QdrantBackend) ensureCollection() error {
	ctx := context.Background()

	// Check if collection exists
	exists, err := b.client.CollectionExists(ctx, b.collection)
	if err != nil {
		return fmt.Errorf("failed to check collection: %w", err)
	}

	if !exists {
		// Create collection
		err = b.client.CreateCollection(ctx, &qdrant.CreateCollection{
			CollectionName: b.collection,
			VectorsConfig: &qdrant.VectorsConfig{
				Config: &qdrant.VectorsConfig_Params{
					Params: &qdrant.VectorParams{
						Size:     b.vectorSize,
						Distance: qdrant.Distance_Cosine,
					},
				},
			},
		})
		if err != nil {
			return fmt.Errorf("failed to create collection: %w", err)
		}
	}

	return nil
}

// VectorSearchQdrant performs vector search using Qdrant
func (s *Store) VectorSearchQdrant(query string, options SearchOptions) ([]SearchResult, error) {
	ctx := context.Background()

	// Check if Qdrant backend is available
	if s.qdrant == nil {
		return []SearchResult{}, fmt.Errorf("Qdrant backend not available")
	}

	// Generate embedding for query
	embeddingResult, err := s.llmRouter.Embed(ctx, []string{query})
	if err != nil {
		return []SearchResult{}, fmt.Errorf("failed to generate embedding: %w", err)
	}

	queryVector := embeddingResult.Embeddings[0]

	// Search Qdrant
	results, err := s.qdrant.Search(queryVector, uint64(options.Limit))
	if err != nil {
		return []SearchResult{}, err
	}

	// Convert to SearchResult
	searchResults := make([]SearchResult, len(results))
	for i, r := range results {
		searchResults[i] = SearchResult{
			Path:       r["path"].(string),
			Collection: r["collection"].(string),
			Score:      float32(r["score"].(float64)),
			Lines:      0,
			Title:      r["title"].(string),
			Hash:       r["hash"].(string),
		}
	}

	return searchResults, nil
}

// Search performs vector search using Qdrant backend
func (b *QdrantBackend) Search(queryVector []float32, limit uint64) ([]map[string]interface{}, error) {
	ctx := context.Background()

	limitVal := limit

	// Use Query API
	searchResult, err := b.client.Query(ctx, &qdrant.QueryPoints{
		CollectionName: b.collection,
		Query:         qdrant.NewQuery(queryVector...),
		Limit:         &limitVal,
	})
	if err != nil {
		return nil, err
	}

	results := make([]map[string]interface{}, len(searchResult))
	for i, r := range searchResult {
		results[i] = map[string]interface{}{
			"id":         r.Id.GetNum(),
			"score":      float64(r.Score),
			"path":       r.Payload["path"].GetStringValue(),
			"title":      r.Payload["title"].GetStringValue(),
			"body":       r.Payload["body"].GetStringValue(),
			"hash":       r.Payload["hash"].GetStringValue(),
			"collection": r.Payload["collection"].GetStringValue(),
		}
	}

	return results, nil
}

// UpsertVectors inserts vectors into Qdrant
func (b *QdrantBackend) UpsertVectors(points []VectorPoint) error {
	ctx := context.Background()

	qdrantPoints := make([]*qdrant.PointStruct, len(points))
	for i, p := range points {
		payload := make(map[string]*qdrant.Value)
		payload["path"] = strToValue(p.Path)
		payload["title"] = strToValue(p.Title)
		payload["body"] = strToValue(p.Body)
		payload["hash"] = strToValue(p.Hash)
		payload["collection"] = strToValue(p.Collection)

		// Use helper function to create PointId
		id := qdrant.NewIDNum(p.ID)

		// Use helper function to create vectors
		vectors := qdrant.NewVectorsDense(p.Vector)

		qdrantPoints[i] = &qdrant.PointStruct{
			Id:      id,
			Vectors: vectors,
			Payload: payload,
		}
	}

	_, err := b.client.Upsert(ctx, &qdrant.UpsertPoints{
		CollectionName: b.collection,
		Points:        qdrantPoints,
	})

	return err
}

// strToValue converts a string to Qdrant Value
func strToValue(s string) *qdrant.Value {
	return &qdrant.Value{
		Kind: &qdrant.Value_StringValue{
			StringValue: s,
		},
	}
}

// VectorPoint represents a vector point for upsert
type VectorPoint struct {
	ID         uint64
	Path       string
	Title      string
	Body       string
	Hash       string
	Collection string
	Vector     []float32
}
