package cli

import (
	"fmt"
	"path/filepath"

	"github.com/qmd/qmd-go/internal/config"
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
	Run:   runCollectionAdd,
}

var collectionListCmd = &cobra.Command{
	Use:   "list",
	Short: "List collections",
	Run:   runCollectionList,
}

var collectionRemoveCmd = &cobra.Command{
	Use:   "remove <name>",
	Short: "Remove a collection",
	Args:  cobra.ExactArgs(1),
	Run:   runCollectionRemove,
}

func runCollectionAdd(cmd *cobra.Command, args []string) {
	path := args[0]
	name, _ := cmd.Flags().GetString("name")
	mask, _ := cmd.Flags().GetString("mask")
	description, _ := cmd.Flags().GetString("description")

	if name == "" {
		name = filepath.Base(path)
	}

	cfg, err := LoadConfig()
	if err != nil {
		fmt.Fprintf(cmd.OutOrStderr(), "Error loading config: %v\n", err)
		return
	}

	// Add collection to config
	cfg.Collections = append(cfg.Collections, config.CollectionConfig{
		Name:        name,
		Path:        path,
		Pattern:     &mask,
		Description: &description,
	})

	// Save config
	if err := cfg.Save(); err != nil {
		fmt.Fprintf(cmd.OutOrStderr(), "Error saving config: %v\n", err)
		return
	}

	fmt.Printf("Collection added: %s\n", name)
	fmt.Printf("  Path: %s\n", path)
	fmt.Printf("  Pattern: %s\n", mask)
}

func runCollectionList(cmd *cobra.Command, args []string) {
	cfg, err := LoadConfig()
	if err != nil {
		fmt.Fprintf(cmd.OutOrStderr(), "Error loading config: %v\n", err)
		return
	}

	if len(cfg.Collections) == 0 {
		fmt.Println("Collections:")
		fmt.Println("  (none configured)")
		return
	}

	fmt.Println("Collections:")
	for _, c := range cfg.Collections {
		fmt.Printf("  - %s: %s\n", c.Name, c.Path)
		if c.Pattern != nil {
			fmt.Printf("    Pattern: %s\n", *c.Pattern)
		}
	}
}

func runCollectionRemove(cmd *cobra.Command, args []string) {
	name := args[0]

	cfg, err := LoadConfig()
	if err != nil {
		fmt.Fprintf(cmd.OutOrStderr(), "Error loading config: %v\n", err)
		return
	}

	// Find and remove collection
	found := false
	newCollections := []config.CollectionConfig{}
	for _, c := range cfg.Collections {
		if c.Name == name {
			found = true
			continue
		}
		newCollections = append(newCollections, c)
	}

	if !found {
		fmt.Fprintf(cmd.OutOrStderr(), "Collection '%s' not found\n", name)
		return
	}

	cfg.Collections = newCollections

	// Save config
	if err := cfg.Save(); err != nil {
		fmt.Fprintf(cmd.OutOrStderr(), "Error saving config: %v\n", err)
		return
	}

	fmt.Printf("Collection '%s' removed\n", name)
}

func init() {
	collectionCmd.AddCommand(collectionAddCmd)
	collectionCmd.AddCommand(collectionListCmd)
	collectionCmd.AddCommand(collectionRemoveCmd)

	collectionAddCmd.Flags().StringP("name", "n", "", "Collection name")
	collectionAddCmd.Flags().StringP("mask", "m", "**/*", "File pattern")
	collectionAddCmd.Flags().StringP("description", "d", "", "Description")
}
