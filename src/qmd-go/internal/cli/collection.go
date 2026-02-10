package cli

import (
	"fmt"

	"github.com/spf13/cobra"
)

var collectionCmd = &cobra.Command{
	Use:   "collection",
	Short: "Manage collections",
}

var collectionAddCmd = &cobra.Command{
	Use:   "add <path>",
	Short: "Add a collection",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		path := args[0]
		name, _ := cmd.Flags().GetString("name")
		mask, _ := cmd.Flags().GetString("mask")
		description, _ := cmd.Flags().GetString("description")

		if name == "" {
			name = filepath.Base(path)
		}

		fmt.Printf("Collection added: %s\n", name)
		fmt.Printf("  Path: %s\n", path)
		fmt.Printf("  Pattern: %s\n", mask)
	},
}

var collectionListCmd = &cobra.Command{
	Use:   "list",
	Short: "List collections",
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("Collections:")
		fmt.Println("  (none configured)")
	},
}

var collectionRemoveCmd = &cobra.Command{
	Use:   "remove <name>",
	Short: "Remove a collection",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		name := args[0]
		fmt.Printf("Collection '%s' removed\n", name)
	},
}

func init() {
	collectionCmd.AddCommand(collectionAddCmd)
	collectionCmd.AddCommand(collectionListCmd)
	collectionCmd.AddCommand(collectionRemoveCmd)

	collectionAddCmd.Flags().StringP("name", "n", "", "Collection name")
	collectionAddCmd.Flags().StringP("mask", "m", "**/*", "File pattern")
	collectionAddCmd.Flags().StringP("description", "d", "", "Description")
}
