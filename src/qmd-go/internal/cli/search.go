package cli

import (
	"fmt"

	"github.com/qmd/qmd-go/internal/store"
	"github.com/spf13/cobra"
)

var searchCmd = &cobra.Command{
	Use:   "search <query>",
	Short: "BM25 full-text search",
	Args:  cobra.ExactArgs(1),
	Run:   runSearch,
}

var vsearchCmd = &cobra.Command{
	Use:   "vsearch <query>",
	Short: "Vector semantic search",
	Args:  cobra.ExactArgs(1),
	Run:   runVectorSearch,
}

var queryCmd = &cobra.Command{
	Use:   "query <query>",
	Short: "Hybrid search with reranking",
	Args:  cobra.ExactArgs(1),
	Run:   runQuery,
}

func runSearch(cmd *cobra.Command, args []string) {
	query := args[0]

	s, err := LoadStore()
	if err != nil {
		fmt.Fprintf(cmd.OutOrStderr(), "Error loading store: %v\n", err)
		return
	}

	collection, _ := cmd.Flags().GetString("collection")
	all, _ := cmd.Flags().GetBool("all")

	results, err := s.BM25Search(query, store.SearchOptions{
		Limit:      limit,
		Collection: collection,
		SearchAll:  all,
	})
	if err != nil {
		fmt.Fprintf(cmd.OutOrStderr(), "Error searching: %v\n", err)
		return
	}

	printResults(results, outputFormat)
}

func runVectorSearch(cmd *cobra.Command, args []string) {
	query := args[0]

	s, err := LoadStore()
	if err != nil {
		fmt.Fprintf(cmd.OutOrStderr(), "Error loading store: %v\n", err)
		return
	}

	collection, _ := cmd.Flags().GetString("collection")
	all, _ := cmd.Flags().GetBool("all")

	results, err := s.VectorSearch(query, store.SearchOptions{
		Limit:      limit,
		Collection: collection,
		SearchAll:  all,
	})
	if err != nil {
		fmt.Fprintf(cmd.OutOrStderr(), "Error searching: %v\n", err)
		return
	}

	printResults(results, outputFormat)
}

func runQuery(cmd *cobra.Command, args []string) {
	query := args[0]

	s, err := LoadStore()
	if err != nil {
		fmt.Fprintf(cmd.OutOrStderr(), "Error loading store: %v\n", err)
		return
	}

	collection, _ := cmd.Flags().GetString("collection")
	all, _ := cmd.Flags().GetBool("all")

	results, err := s.HybridSearch(query, store.SearchOptions{
		Limit:      limit,
		Collection: collection,
		SearchAll:  all,
	})
	if err != nil {
		fmt.Fprintf(cmd.OutOrStderr(), "Error searching: %v\n", err)
		return
	}

	printResults(results, outputFormat)
}

func init() {
	for _, cmd := range []*cobra.Command{searchCmd, vsearchCmd, queryCmd} {
		cmd.Flags().StringP("collection", "c", "", "Collection name")
		cmd.Flags().Bool("all", false, "Search all collections")
	}
}
