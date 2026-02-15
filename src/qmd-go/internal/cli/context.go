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
		// Check for dry-run mode
		dryRun, _ := cmd.Flags().GetBool("dry-run")
		if dryRun {
			description, _ := cmd.Flags().GetString("description")
			path := ""
			if len(args) > 0 {
				path = args[0]
			}
			fmt.Println("[DRY-RUN] Would execute context add with:")
			fmt.Printf("  path: %s\n", path)
			fmt.Printf("  description: %s\n", description)
			return
		}
		description, _ := cmd.Flags().GetString("description")
		fmt.Printf("Context added: %s\n", description)
	},
}

var contextListCmd = &cobra.Command{
	Use:   "list",
	Short: "List contexts",
	Run: func(cmd *cobra.Command, args []string) {
		// Check for dry-run mode
		dryRun, _ := cmd.Flags().GetBool("dry-run")
		if dryRun {
			fmt.Println("[DRY-RUN] Would execute context list")
			return
		}
		fmt.Println("Contexts:")
	},
}

var contextRemoveCmd = &cobra.Command{
	Use:   "rm <path>",
	Short: "Remove a context",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		// Check for dry-run mode
		dryRun, _ := cmd.Flags().GetBool("dry-run")
		if dryRun {
			path := args[0]
			fmt.Println("[DRY-RUN] Would execute context rm with:")
			fmt.Printf("  path: %s\n", path)
			return
		}
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
