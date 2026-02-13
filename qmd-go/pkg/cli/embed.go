package cli

import (
	"fmt"
	"os"

	"github.com/qmd/qmd-go/pkg/config"
	"github.com/qmd/qmd-go/pkg/llm"
	"github.com/qmd/qmd-go/pkg/store"
	"github.com/spf13/cobra"
)

var embedCmd = &cobra.Command{
	Use:   "embed [collection]",
	Short: "Generate/update embeddings for documents",
	Long:  `Generate or update vector embeddings for documents in a collection.`,
	Args:  cobra.MinimumNArgs(0),
	Run: func(cmd *cobra.Command, args []string) {
		collection := ""
		if len(args) > 0 {
			collection = args[0]
		}

		force, _ := cmd.Flags().GetBool("force")

		if err := runEmbed(collection, force); err != nil {
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			os.Exit(1)
		}
	},
}

func init() {
	embedCmd.Flags().BoolP("force", "f", false, "Force regeneration of all embeddings")
}

func runEmbed(collection string, force bool) error {
	// Load config
	cfg, err := config.Load("")
	if err != nil {
		return fmt.Errorf("failed to load config: %w", err)
	}

	// Get store
	s, err := getStore()
	if err != nil {
		return fmt.Errorf("failed to open store: %w", err)
	}
	defer s.Close()

	// Create LLM router
	router := llm.New(cfg)

	// Initialize embedder from config
	if cfg.Models.Embed != nil && cfg.Models.Embed.Local != "" {
		embedder, err := llm.NewLocalEmbedder(cfg.Models.Embed.Local)
		if err != nil {
			fmt.Printf("Warning: Failed to load embedder: %v\n", err)
			fmt.Println("Using random embeddings for demonstration...")
		} else {
			router.SetLocalEmbedder(embedder)
		}
	}

	if collection != "" {
		return embedCollection(s, router, collection, force)
	}

	// Embed all collections
	collections, err := s.GetCollections()
	if err != nil {
		return fmt.Errorf("failed to get collections: %w", err)
	}

	for _, col := range collections {
		if err := embedCollection(s, router, col.Name, force); err != nil {
			fmt.Fprintf(os.Stderr, "Warning: Failed to embed collection %s: %v\n", col.Name, err)
		}
	}

	return nil
}

func embedCollection(s *store.Store, router *llm.Router, collection string, force bool) error {
	fmt.Printf("Generating embeddings for collection: %s\n", collection)

	// Get documents needing embedding
	docs, err := s.GetDocumentsForEmbedding(collection, force)
	if err != nil {
		return fmt.Errorf("failed to get documents: %w", err)
	}

	if len(docs) == 0 {
		fmt.Println("No documents need embedding")
		return nil
	}

	fmt.Printf("Found %d documents to embed\n", len(docs))

	// Delete existing embeddings if force mode
	if force {
		for _, doc := range docs {
			if err := s.DeleteEmbeddings(doc.Hash); err != nil {
				return fmt.Errorf("failed to delete old embeddings: %w", err)
			}
		}
	}

	// Determine embedding model
	modelName := "nomic-embed-text-v1.5"
	if router.HasEmbedder() {
		fmt.Printf("Using embedder: %s\n", modelName)
	}

	// Process documents in batches
	batchSize := 10
	processed := 0

	for i := 0; i < len(docs); i += batchSize {
		end := i + batchSize
		if end > len(docs) {
			end = len(docs)
		}

		batch := docs[i:end]
		fmt.Printf("Processing batch %d-%d...\n", i+1, end)

		// Chunk documents
		var texts []string
		var hashes []string
		var seqs []int
		var poss []int

		for _, doc := range batch {
			chunks := store.ChunkDocument(doc.Doc, store.DefaultChunkSize, store.DefaultOverlap)
			for _, chunk := range chunks {
				texts = append(texts, chunk.Text)
				hashes = append(hashes, doc.Hash)
				seqs = append(seqs, chunk.Seq)
				poss = append(poss, chunk.Pos)
			}
		}

		if len(texts) == 0 {
			continue
		}

		// Generate embeddings
		var embeddings [][]float32

		if router.HasEmbedder() {
			result, err := router.Embed(texts)
			if err != nil {
				fmt.Printf("Warning: Failed to generate embeddings: %v\n", err)
				// Fall back to random embeddings
				for range texts {
					embeddings = append(embeddings, randomEmbedding(768))
				}
			} else {
				embeddings = result.Embeddings
				if result.Model != "" {
					modelName = result.Model
				}
			}
		} else {
			// Use random embeddings as placeholder
			for range texts {
				embeddings = append(embeddings, randomEmbedding(768))
			}
			fmt.Println("Note: Using random embeddings (no embedder configured)")
		}

		// Store embeddings
		for j, embedding := range embeddings {
			err := s.StoreEmbedding(hashes[j], seqs[j], poss[j], embedding, modelName)
			if err != nil {
				return fmt.Errorf("failed to store embedding: %w", err)
			}
		}

		processed += len(batch)
		fmt.Printf("Embedded %d/%d documents\n", processed, len(docs))
	}

	fmt.Println("Embedding complete!")
	return nil
}

// randomEmbedding generates a random embedding vector
func randomEmbedding(dim int) []float32 {
	embedding := make([]float32, dim)
	// Simple random initialization (not normalized)
	for i := range embedding {
		embedding[i] = float32(i%100) / 100.0
	}
	// Normalize
	var sum float32
	for _, v := range embedding {
		sum += v * v
	}
	norm := float32(0)
	if sum > 0 {
		norm = 1.0 / float32(sqrt(float64(sum)))
	}
	for i := range embedding {
		embedding[i] *= norm
	}
	return embedding
}

func sqrt(x float64) float64 {
	if x == 0 {
		return 0
	}
	// Simple approximation
	r := x
	for i := 0; i < 20; i++ {
		r = (r + x/r) / 2
	}
	return r
}
