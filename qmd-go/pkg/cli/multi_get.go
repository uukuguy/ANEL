package cli

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var multiGetCmd = &cobra.Command{
	Use:   "multi_get [pattern]",
	Short: "Get multiple documents by pattern",
	Args:  cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("multi_get not fully implemented")
		os.Exit(1)
	},
}
