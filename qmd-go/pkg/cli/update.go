package cli

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/qmd/qmd-go/pkg/config"
	"github.com/spf13/cobra"
)

var updateCmd = &cobra.Command{
	Use:   "update [collection]",
	Short: "Update index",
	Args:  cobra.MinimumNArgs(0),
	Run: func(cmd *cobra.Command, args []string) {
		s, err := getStore()
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			os.Exit(1)
		}
		defer s.Close()

		colName := collection
		if colName == "" && len(args) > 0 {
			colName = args[0]
		}

		collections := cfg.Collections
		if colName != "" {
			for _, c := range collections {
				if c.Name == colName {
					collections = []config.CollectionConfig{c}
					break
				}
			}
		}

		totalDocs := 0
		for _, c := range collections {
			docs, err := scanDirectory(c.Path, c.Pattern)
			if err != nil {
				fmt.Fprintf(os.Stderr, "Error scanning %s: %v\n", c.Path, err)
				continue
			}

			for _, doc := range docs {
				_, err := s.AddDocument(c.Name, doc.path, doc.title, doc.content)
				if err != nil {
					fmt.Fprintf(os.Stderr, "Error adding %s: %v\n", doc.path, err)
					continue
				}
				totalDocs++
			}
			fmt.Printf("Indexed %d documents from %s\n", len(docs), c.Name)
		}

		fmt.Printf("Total: %d documents indexed\n", totalDocs)
	},
}

type docInfo struct {
	path    string
	title   string
	content string
}

func scanDirectory(dirPath, pattern string) ([]docInfo, error) {
	var docs []docInfo

	err := filepath.Walk(dirPath, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() {
			return nil
		}

		// Skip hidden files
		if filepath.Base(path)[0] == '.' {
			return nil
		}

		content, err := os.ReadFile(path)
		if err != nil {
			return nil
		}

		docs = append(docs, docInfo{
			path:    path,
			title:   filepath.Base(path),
			content: string(content),
		})

		return nil
	})

	return docs, err
}
