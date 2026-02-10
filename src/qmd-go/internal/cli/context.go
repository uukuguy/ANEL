package cli

import (
	"fmt"

	"github.com/spf13/cobra"
)

var contextCmd = &cobra.Command{
	Use:   "context",
	Short: "Manage contexts",
}

var contextAddCmd = &cobra.Command{
	Use:   "add [path]",
	Short: "Add a context",
	Args:  cobra.MaximumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		description, _ := cmd.Flags().GetString("description")
		fmt.Printf("Context added: %s\n", description)
	},
}

var contextListCmd = &cobra.Command{
	Use:   "list",
	Short: "List contexts",
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("Contexts:")
	},
}

var contextRemoveCmd = &cobra.Command{
	Use:   "rm <path>",
	Short: "Remove a context",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		path := args[0]
		fmt.Printf("Context '%s' removed\n", path)
	},
}

func init() {
	contextCmd.AddCommand(contextAddCmd)
	contextCmd.AddCommand(contextListCmd)
	contextCmd.AddCommand(contextRemoveCmd)

	contextAddCmd.Flags().StringP("description", "d", "", "Description")
	contextAddCmd.MarkFlagRequired("description")
}
