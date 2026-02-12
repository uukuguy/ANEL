package cli

import (
	"fmt"
	"os"

	"github.com/qmd/qmd-go/pkg/formatter"
	"github.com/qmd/qmd-go/pkg/store"
	"github.com/spf13/cobra"
)

var queryCmd = &cobra.Command{
	Use:   "query [query]",
	Short: "Hybrid search (BM25 + Vector + RRF + Reranking)",
	Args:  cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		query := args[0]
		all, _ := cmd.Flags().GetBool("all")

		s, err := getStore()
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			os.Exit(1)
		}
		defer s.Close()

		opts := store.SearchOptions{
			Limit:      limit,
			MinScore:   minScore,
			Collection: collection,
			All:        all,
		}

		// Perform BM25 search
		bm25Results, err := s.BM25Search(query, opts)
		if err != nil {
			fmt.Fprintf(os.Stderr, "BM25 search error: %v\n", err)
			os.Exit(1)
		}

		// Vector search (placeholder - needs embeddings)
		var vectorResults []store.SearchResult

		// Hybrid search with RRF
		results := s.HybridSearch(bm25Results, vectorResults, 60)

		f := formatter.New(outputFormat, limit)
		fmt.Print(f.FormatSearchResults(results))
	},
}

func init() {
	queryCmd.Flags().BoolP("all", "a", false, "Search all collections")
}
