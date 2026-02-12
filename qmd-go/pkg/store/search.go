package store

import (
	"fmt"
	"math"
	"sort"
)

// VectorSearch performs vector similarity search
func (s *Store) VectorSearch(queryVector []float32, opts SearchOptions) ([]SearchResult, error) {
	if opts.Limit == 0 {
		opts.Limit = 20
	}

	// Normalize query vector
	queryVector = normalizeVector(queryVector)

	// Get all documents with embeddings
	rows, err := s.db.Query(`
		SELECT d.collection, d.path, d.title, d.hash, cv.embedding
		FROM content_vectors cv
		JOIN documents d ON d.hash = cv.hash
		WHERE d.active = 1
		AND (d.collection = ? OR ? = 1)
	`, opts.Collection, opts.All)
	if err != nil {
		return nil, fmt.Errorf("failed to query vectors: %w", err)
	}
	defer rows.Close()

	type scoredDoc struct {
		result SearchResult
		score  float32
	}

	var scoredDocs []scoredDoc
	for rows.Next() {
		var r SearchResult
		var embeddingJSON string
		if err := rows.Scan(&r.Collection, &r.Path, &r.Title, &r.Hash, &embeddingJSON); err != nil {
			return nil, fmt.Errorf("failed to scan row: %w", err)
		}

		// Parse embedding (stored as JSON array)
		embedding, err := parseEmbedding(embeddingJSON)
		if err != nil {
			continue
		}

		// Compute cosine similarity
		score := cosineSimilarity(queryVector, embedding)
		if score >= opts.MinScore {
			r.DocID = r.Collection + ":" + r.Path
			r.Score = score
			scoredDocs = append(scoredDocs, scoredDoc{result: r, score: score})
		}
	}

	// Sort by score descending
	sort.Slice(scoredDocs, func(i, j int) bool {
		return scoredDocs[i].score > scoredDocs[j].score
	})

	// Limit results
	if len(scoredDocs) > opts.Limit {
		scoredDocs = scoredDocs[:opts.Limit]
	}

	var results []SearchResult
	for _, sd := range scoredDocs {
		results = append(results, sd.result)
	}

	return results, nil
}

// HybridSearch performs hybrid search using RRF fusion
func (s *Store) HybridSearch(bm25Results, vectorResults []SearchResult, k int) []SearchResult {
	if k == 0 {
		k = 60 // Default RRF parameter
	}

	// Collect all unique documents
	docMap := make(map[string]*SearchResult)

	// Add BM25 results with reciprocal rank
	for i, r := range bm25Results {
		docKey := r.DocID
		if _, exists := docMap[docKey]; !exists {
			docMap[docKey] = &bm25Results[i]
		}
		rrfScore := float32(1.0 / float64(int(k)+i+1))
		docMap[docKey].Score += rrfScore
	}

	// Add vector results with reciprocal rank
	for i, r := range vectorResults {
		docKey := r.DocID
		if existing, exists := docMap[docKey]; exists {
			rrfScore := float32(1.0 / float64(int(k)+i+1))
			existing.Score += rrfScore
		} else {
			vectorResults[i].Score = float32(1.0 / float64(int(k)+i+1))
			docMap[docKey] = &vectorResults[i]
		}
	}

	// Convert to slice and sort by score
	results := make([]SearchResult, 0, len(docMap))
	for _, r := range docMap {
		results = append(results, *r)
	}

	sort.Slice(results, func(i, j int) bool {
		return results[i].Score > results[j].Score
	})

	return results
}

// RRF fusion algorithm
func RRF(resultLists [][]SearchResult, weights []float32, k uint32) []SearchResult {
	if k == 0 {
		k = 60
	}

	if len(weights) == 0 {
		weights = make([]float32, len(resultLists))
		for i := range weights {
			weights[i] = 1.0
		}
	}

	docMap := make(map[string]*SearchResult)

	for listIdx, results := range resultLists {
		weight := weights[listIdx]
		for i, r := range results {
			docKey := r.DocID
			rrfScore := weight * float32(1.0/float64(int(k)+i+1))
			if existing, exists := docMap[docKey]; exists {
				existing.Score += rrfScore
			} else {
				newR := r
				newR.Score = rrfScore
				docMap[docKey] = &newR
			}
		}
	}

	results := make([]SearchResult, 0, len(docMap))
	for _, r := range docMap {
		results = append(results, *r)
	}

	sort.Slice(results, func(i, j int) bool {
		return results[i].Score > results[j].Score
	})

	return results
}

// normalizeVector normalizes a vector to unit length
func normalizeVector(v []float32) []float32 {
	var sum float32
	for _, x := range v {
		sum += x * x
	}
	sum = float32(math.Sqrt(float64(sum)))
	if sum == 0 {
		return v
	}
	result := make([]float32, len(v))
	for i, x := range v {
		result[i] = x / sum
	}
	return result
}

// cosineSimilarity computes cosine similarity between two vectors
func cosineSimilarity(a, b []float32) float32 {
	if len(a) != len(b) {
		return 0
	}

	var dotProduct, normA, normB float32
	for i := range a {
		dotProduct += a[i] * b[i]
		normA += a[i] * a[i]
		normB += b[i] * b[i]
	}

	norm := float32(math.Sqrt(float64(normA))) * float32(math.Sqrt(float64(normB)))
	if norm == 0 {
		return 0
	}

	return dotProduct / norm
}

// parseEmbedding parses JSON embedding string to float32 slice
func parseEmbedding(jsonStr string) ([]float32, error) {
	// Simple JSON array parsing
	// Format: [0.1, 0.2, 0.3, ...]
	jsonStr = jsonStr[1 : len(jsonStr)-1] // Remove [ and ]
	parts := splitCSV(jsonStr)

	embedding := make([]float32, 0, len(parts))
	for _, part := range parts {
		var f float32
		if _, err := fmt.Sscanf(part, "%f", &f); err == nil {
			embedding = append(embedding, f)
		}
	}

	if len(embedding) == 0 {
		return nil, fmt.Errorf("empty embedding")
	}

	return embedding, nil
}

// splitCSV splits CSV string (simple implementation)
func splitCSV(s string) []string {
	var result []string
	var current string
	inQuotes := false

	for _, c := range s {
		switch c {
		case '"':
			inQuotes = !inQuotes
		case ',':
			if !inQuotes {
				result = append(result, current)
				current = ""
				continue
			}
		}
		current += string(c)
	}
	if current != "" {
		result = append(result, current)
	}

	return result
}
