package cli

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var cleanupCmd = &cobra.Command{
	Use:   "cleanup",
	Short: "Clean up stale entries",
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("Cleanup not implemented")
		os.Exit(1)
	},
}
