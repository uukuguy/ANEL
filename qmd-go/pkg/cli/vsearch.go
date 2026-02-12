package cli

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var vsearchCmd = &cobra.Command{
	Use:   "vsearch [query]",
	Short: "Vector semantic search",
	Args:  cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("Vector search requires embedding model setup")
		fmt.Println("Use 'qmd embed' to generate embeddings first")
		os.Exit(1)
	},
}
