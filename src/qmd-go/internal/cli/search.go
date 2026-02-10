package cli

import (
	"fmt"

	"github.com/spf13/cobra"
)

var searchCmd = &cobra.Command{
	Use:   "search <query>",
	Short: "BM25 full-text search",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		query := args[0]
		fmt.Printf("Searching: %s\n", query)
	},
}

var vsearchCmd = &cobra.Command{
	Use:   "vsearch <query>",
	Short: "Vector semantic search",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		query := args[0]
		fmt.Printf("Vector search: %s\n", query)
	},
}

var queryCmd = &cobra.Command{
	Use:   "query <query>",
	Short: "Hybrid search with reranking",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		query := args[0]
		fmt.Printf("Hybrid query: %s\n", query)
	},
}

func init() {
	for _, cmd := range []*cobra.Command{searchCmd, vsearchCmd, queryCmd} {
		cmd.Flags().StringP("collection", "c", "", "Collection name")
		cmd.Flags().Bool("all", false, "Search all collections")
	}
}
