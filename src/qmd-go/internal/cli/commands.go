package cli

import (
	"fmt"
	"log"
	"os"
	"path/filepath"

	"github.com/spf13/cobra"
)

// RootCmd is the root command
var RootCmd = &cobra.Command{
	Use:   "qmd",
	Short: "QMD - AI-powered search with hybrid BM25 and vector search",
	Long:  `QMD provides AI-powered search with hybrid BM25 and vector search capabilities.`,
}

// Search options
type SearchOptions struct {
	Limit        int
	MinScore     float32
	Collection   string
	SearchAll    bool
	Format       string
	FTSBackend   string
	VectorBackend string
}

func init() {
	// Global flags
	RootCmd.PersistentFlags().StringVarP(&SearchOptions{}.Format, "format", "f", "cli", "Output format: cli, json, md, csv, files")
	RootCmd.PersistentFlags().IntVarP(&SearchOptions{}.Limit, "limit", "n", 20, "Max results")
	RootCmd.PersistentFlags().StringVar(&SearchOptions{}.FTSBackend, "fts-backend", "sqlite_fts5", "BM25 backend: sqlite_fts5, lancedb")
	RootCmd.PersistentFlags().StringVar(&SearchOptions{}.VectorBackend, "vector-backend", "qmd_builtin", "Vector backend: qmd_builtin, lancedb")

	// Add subcommands
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

// LoadConfig loads configuration
func LoadConfig() (*Config, error) {
	configPath := expandPath("~/.config/qmd/index.yaml")

	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		return DefaultConfig(), nil
	}

	data, err := os.ReadFile(configPath)
	if err != nil {
		return nil, err
	}

	return LoadConfigFromData(data)
}

func expandPath(path string) string {
	if home, err := os.UserHomeDir(); err == nil {
		if len(path) > 1 && path[:2] == "~/" {
			return filepath.Join(home, path[2:])
		}
	}
	return path
}
