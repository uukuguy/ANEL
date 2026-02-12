package cli

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/qmd/qmd-go/pkg/config"
	"github.com/spf13/cobra"
)

var collectionCmd = &cobra.Command{
	Use:   "collection [action]",
	Short: "Collection management",
	Args:  cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		action := args[0]
		switch action {
		case "add":
			addCollection(cmd)
		case "list":
			listCollections()
		case "remove":
			removeCollection(cmd)
		case "rename":
			renameCollection(cmd)
		default:
			fmt.Fprintf(os.Stderr, "Unknown action: %s\n", action)
			os.Exit(1)
		}
	},
}

var colPath string
var colPattern string
var colDesc string

func addCollection(cmd *cobra.Command) {
	if colPath == "" {
		fmt.Fprintln(os.Stderr, "Error: --path is required")
		os.Exit(1)
	}
	if collection == "" {
		collection = filepath.Base(colPath)
	}

	col := config.CollectionConfig{
		Name:        collection,
		Path:        colPath,
		Pattern:     colPattern,
		Description: colDesc,
	}

	cfg.AddCollection(col)
	if err := cfg.Save(""); err != nil {
		fmt.Fprintf(os.Stderr, "Error saving config: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("Collection '%s' added\n", collection)
}

func listCollections() {
	if len(cfg.Collections) == 0 {
		fmt.Println("No collections")
		return
	}
	fmt.Println("Collections:")
	for _, c := range cfg.Collections {
		fmt.Printf("  %s: %s\n", c.Name, c.Path)
	}
}

func removeCollection(cmd *cobra.Command) {
	if collection == "" {
		fmt.Fprintln(os.Stderr, "Error: --collection is required")
		os.Exit(1)
	}
	cfg.RemoveCollection(collection)
	if err := cfg.Save(""); err != nil {
		fmt.Fprintf(os.Stderr, "Error saving config: %v\n", err)
		os.Exit(1)
	}
	fmt.Printf("Collection '%s' removed\n", collection)
}

func renameCollection(cmd *cobra.Command) {
	newName, _ := cmd.Flags().GetString("new-name")
	if collection == "" || newName == "" {
		fmt.Fprintln(os.Stderr, "Error: --collection and --new-name are required")
		os.Exit(1)
	}
	for i := range cfg.Collections {
		if cfg.Collections[i].Name == collection {
			cfg.Collections[i].Name = newName
			break
		}
	}
	if err := cfg.Save(""); err != nil {
		fmt.Fprintf(os.Stderr, "Error saving config: %v\n", err)
		os.Exit(1)
	}
	fmt.Printf("Collection '%s' renamed to '%s'\n", collection, newName)
}

func init() {
	collectionCmd.Flags().StringVar(&colPath, "path", "", "Collection path")
	collectionCmd.Flags().StringVar(&colPattern, "pattern", "", "File pattern")
	collectionCmd.Flags().StringVar(&colDesc, "description", "", "Description")
	collectionCmd.Flags().String("new-name", "", "New name for rename")
}
