package cli

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var embedCmd = &cobra.Command{
	Use:   "embed [collection]",
	Short: "Generate/update embeddings",
	Args:  cobra.MinimumNArgs(0),
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("Embed requires embedding model setup")
		fmt.Println("Configure models in config file")
		os.Exit(1)
	},
}
