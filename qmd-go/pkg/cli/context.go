package cli

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var contextCmd = &cobra.Command{
	Use:   "context [action]",
	Short: "Context management",
	Args:  cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		action := args[0]
		switch action {
		case "add":
			fmt.Println("Context add not implemented")
		case "list":
			fmt.Println("Context list not implemented")
		case "rm":
			fmt.Println("Context rm not implemented")
		default:
			fmt.Fprintf(os.Stderr, "Unknown action: %s\n", action)
			os.Exit(1)
		}
	},
}
