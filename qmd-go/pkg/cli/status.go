package cli

import (
	"encoding/json"
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var statusCmd = &cobra.Command{
	Use:   "status",
	Short: "Show index status",
	Run: func(cmd *cobra.Command, args []string) {
		s, err := getStore()
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			os.Exit(1)
		}
		defer s.Close()

		stats, err := s.GetStats()
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			os.Exit(1)
		}

		if outputFormat == "json" {
			json.NewEncoder(os.Stdout).Encode(stats)
		} else {
			fmt.Printf("Collections: %d\n", stats.CollectionCount)
			fmt.Printf("Documents: %d\n", stats.DocumentCount)
			fmt.Printf("Indexed: %d\n", stats.IndexedCount)
			fmt.Printf("Chunks: %d\n", stats.ChunkCount)
			fmt.Println("\nPer-collection:")
			for name, count := range stats.CollectionStats {
				fmt.Printf("  %s: %d\n", name, count)
			}
		}
	},
}
