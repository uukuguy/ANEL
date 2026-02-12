package cli

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var getCmd = &cobra.Command{
	Use:   "get [path]",
	Short: "Get document content",
	Args:  cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		path := args[0]
		from, _ := cmd.Flags().GetInt("from")
		limit, _ := cmd.Flags().GetInt("limit")

		s, err := getStore()
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			os.Exit(1)
		}
		defer s.Close()

		doc, content, err := s.GetDocument(path)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			os.Exit(1)
		}

		lines := []byte(content)
		if from > 0 && from < len(lines) {
			lines = lines[from:]
		}
		if limit > 0 && limit < len(lines) {
			lines = lines[:limit]
		}

		fmt.Printf("Title: %s\n", doc.Title)
		fmt.Printf("Collection: %s\n", doc.Collection)
		fmt.Printf("Hash: %s\n", doc.Hash)
		fmt.Println("---")
		fmt.Print(string(lines))
	},
}

func init() {
	getCmd.Flags().Int("from", 0, "Start line")
	getCmd.Flags().Int("limit", 0, "Line limit")
}
