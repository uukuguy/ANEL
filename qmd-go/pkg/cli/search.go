package cli

import (
	"fmt"
	"os"

	"github.com/qmd/qmd-go/pkg/formatter"
	"github.com/qmd/qmd-go/pkg/store"
	"github.com/spf13/cobra"
)

var searchCmd = &cobra.Command{
	Use:   "search [query]",
	Short: "BM25 full-text search",
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

		results, err := s.BM25Search(query, opts)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Search error: %v\n", err)
			os.Exit(1)
		}

		f := formatter.New(outputFormat, limit)
		fmt.Print(f.FormatSearchResults(results))
	},
}

func init() {
	searchCmd.Flags().BoolP("all", "a", false, "Search all collections")
}
