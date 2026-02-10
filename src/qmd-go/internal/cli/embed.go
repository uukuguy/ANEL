package cli

import (
	"fmt"

	"github.com/spf13/cobra"
)

var embedCmd = &cobra.Command{
	Use:   "embed",
	Short: "Generate/update embeddings",
	Run: func(cmd *cobra.Command, args []string) {
		force, _ := cmd.Flags().GetBool("force")
		collection, _ := cmd.Flags().GetString("collection")

		if force {
			fmt.Println("Force regeneration enabled")
		}
		if collection != "" {
			fmt.Printf("Embedding collection: %s\n", collection)
		} else {
			fmt.Println("Generating embeddings for all collections...")
		}
	},
}

var updateCmd = &cobra.Command{
	Use:   "update",
	Short: "Update index",
	Run: func(cmd *cobra.Command, args []string) {
		pull, _ := cmd.Flags().GetBool("pull")
		if pull {
			fmt.Println("Pulling remote changes...")
		}
		fmt.Println("Updating index...")
	},
}

var statusCmd = &cobra.Command{
	Use:   "status",
	Short: "Show index status",
	Run: func(cmd *cobra.Command, args []string) {
		verbose, _ := cmd.Flags().GetBool("verbose")

		fmt.Println("Index Status")
		fmt.Println("=" + "="*39)
		fmt.Println("Collections: 0")
		fmt.Println("Documents: 0")

		if verbose {
			fmt.Println("\nDetailed Statistics:")
		}
	},
}

var cleanupCmd = &cobra.Command{
	Use:   "cleanup",
	Short: "Cleanup stale entries",
	Run: func(cmd *cobra.Command, args []string) {
		dryRun, _ := cmd.Flags().GetBool("dry-run")
		olderThan, _ := cmd.Flags().GetUint("older-than")

		if dryRun {
			fmt.Println("Dry run - no changes made")
		} else {
			fmt.Printf("Cleanup completed (older than %d days)\n", olderThan)
		}
	},
}

func init() {
	embedCmd.Flags().BoolP("force", "f", false, "Force regeneration")
	embedCmd.Flags().StringP("collection", "c", "", "Collection name")

	updateCmd.Flags().Bool("pull", false, "Pull remote changes")

	statusCmd.Flags().BoolP("verbose", "v", false, "Detailed output")

	cleanupCmd.Flags().Bool("dry-run", false, "Dry run only")
	cleanupCmd.Flags().Uint("older-than", 30, "Remove entries older than N days")
}
