package cli

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/qmd/qmd-go/pkg/config"
	"github.com/qmd/qmd-go/pkg/store"
	"github.com/spf13/cobra"
)

var (
	cfg          *config.Config
	dbPath       string
	collection   string
	outputFormat string
	limit        int
	minScore     float32
)

// RootCmd represents the root command
var RootCmd = &cobra.Command{
	Use:   "qmd",
	Short: "QMD - AI-powered search tool",
	Long:  `QMD is an AI-powered search tool supporting hybrid BM25 and vector search.`,
	PersistentPreRun: func(cmd *cobra.Command, args []string) {
		var err error
		if dbPath == "" {
			dbPath = filepath.Join(os.Getenv("HOME"), ".cache", "qmd", "index.db")
		}
		cfg, err = config.Load("")
		if err != nil {
			fmt.Fprintf(os.Stderr, "Warning: Failed to load config: %v\n", err)
			cfg = config.DefaultConfig()
		}
	},
}

func init() {
	RootCmd.PersistentFlags().StringVarP(&dbPath, "db", "d", "", "Database path")
	RootCmd.PersistentFlags().StringVarP(&collection, "collection", "c", "", "Collection name")
	RootCmd.PersistentFlags().StringVarP(&outputFormat, "format", "f", "cli", "Output format (cli, json, md, csv, files, xml)")
	RootCmd.PersistentFlags().IntVarP(&limit, "limit", "l", 20, "Maximum number of results")
	RootCmd.PersistentFlags().Float32Var(&minScore, "min-score", 0.0, "Minimum score threshold")

	RootCmd.AddCommand(collectionCmd)
	RootCmd.AddCommand(contextCmd)
	RootCmd.AddCommand(getCmd)
	RootCmd.AddCommand(multiGetCmd)
	RootCmd.AddCommand(searchCmd)
	RootCmd.AddCommand(vsearchCmd)
	RootCmd.AddCommand(queryCmd)
	RootCmd.AddCommand(embedCmd)
	RootCmd.AddCommand(updateCmd)
	RootCmd.AddCommand(statusCmd)
	RootCmd.AddCommand(cleanupCmd)
	RootCmd.AddCommand(mcpCmd)
	RootCmd.AddCommand(agentCmd)
}

// getStore returns a Store instance
func getStore() (*store.Store, error) {
	if dbPath == "" {
		dbPath = filepath.Join(os.Getenv("HOME"), ".cache", "qmd", "index.db")
	}
	return store.New(dbPath)
}
