package cli

import (
	"fmt"

	"github.com/spf13/cobra"
)

var getCmd = &cobra.Command{
	Use:   "get <file>[:line]",
	Short: "Get document content",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		file := args[0]
		limit, _ := cmd.Flags().GetInt("limit")
		from, _ := cmd.Flags().GetInt("from")
		full, _ := cmd.Flags().GetBool("full")

		fmt.Printf("Getting: %s (limit: %d, from: %d, full: %v)\n", file, limit, from, full)
	},
}

var multiGetCmd = &cobra.Command{
	Use:   "multi-get <pattern>",
	Short: "Get multiple documents",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		pattern := args[0]
		limit, _ := cmd.Flags().GetInt("limit")
		maxBytes, _ := cmd.Flags().GetInt("max-bytes")

		fmt.Printf("Pattern: %s (limit: %d, max-bytes: %d)\n", pattern, limit, maxBytes)
	},
}

func init() {
	getCmd.Flags().IntP("limit", "l", 50, "Number of lines")
	getCmd.Flags().Int("from", 0, "Start line")
	getCmd.Flags().Bool("full", false, "Full content")

	multiGetCmd.Flags().IntP("limit", "l", 50, "Lines per file")
	multiGetCmd.Flags().Int("max-bytes", 0, "Max bytes per file")
}
