package cli

import (
	"fmt"
	"log"
	"os"
	"path/filepath"

	"github.com/qmd/qmd-go/internal/anel"
	"github.com/qmd/qmd-go/internal/config"
	"github.com/qmd/qmd-go/internal/store"
	"github.com/spf13/cobra"
)

// RootCmd is the root command
var RootCmd = &cobra.Command{
	Use:   "qmd",
	Short: "QMD - AI-powered search with hybrid BM25 and vector search",
	Long:  `QMD provides AI-powered search with hybrid BM25 and vector search capabilities.`,
	PersistentPreRunE: func(cmd *cobra.Command, args []string) error {
		// Check for --emit-spec flag before running command
		emitSpec, _ := cmd.Flags().GetBool("emit-spec")
		if emitSpec {
			spec := anel.GetSpecForCommand(cmd.Name())
			if spec != nil {
				fmt.Println(spec.ToJSON())
				os.Exit(0)
			}
		}
		return nil
	},
}

// Global options
var (
	configPath  string
	outputFormat string
	limit       int
	ftsBackend  string
	vectorBackend string
)

// ANEL global options
var (
	emitSpec  bool
	dryRun    bool
)

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

var searchOpts SearchOptions

func init() {
	// Global flags
	RootCmd.PersistentFlags().StringVarP(&outputFormat, "format", "f", "cli", "Output format: cli, json, md, csv, files")
	RootCmd.PersistentFlags().IntVarP(&limit, "limit", "n", 20, "Max results")
	RootCmd.PersistentFlags().StringVar(&ftsBackend, "fts-backend", "sqlite_fts5", "BM25 backend: sqlite_fts5, lancedb")
	RootCmd.PersistentFlags().StringVar(&vectorBackend, "vector-backend", "qmd_builtin", "Vector backend: qmd_builtin, lancedb, qdrant")
	RootCmd.PersistentFlags().StringVar(&configPath, "config", "", "Config file path")

	// ANEL flags
	RootCmd.PersistentFlags().BoolVar(&emitSpec, "emit-spec", false, "Output JSON Schema and exit")
	RootCmd.PersistentFlags().BoolVar(&dryRun, "dry-run", false, "Validate parameters but don't execute")

	// Check environment variables for ANEL overrides
	if os.Getenv(anel.EnvEmitSpec) != "" {
		emitSpec = true
	}
	if os.Getenv(anel.EnvDryRun) != "" {
		dryRun = true
	}

	// Add subcommands
	RootCmd.AddCommand(collectionCmd)
	RootCmd.AddCommand(contextCmd)
	RootCmd.AddCommand(getCmd)
	RootCmd.AddCommand(searchCmd)
	RootCmd.AddCommand(vsearchCmd)
	RootCmd.AddCommand(queryCmd)
	RootCmd.AddCommand(embedCmd)
	RootCmd.AddCommand(updateCmd)
	RootCmd.AddCommand(statusCmd)
	RootCmd.AddCommand(cleanupCmd)
	RootCmd.AddCommand(mcpCmd)
}

// LoadConfig loads configuration
func LoadConfig() (*config.Config, error) {
	cfgPath := configPath
	if cfgPath == "" {
		cfgPath = expandPath("~/.config/qmd/index.yaml")
	}

	if _, err := os.Stat(cfgPath); os.IsNotExist(err) {
		return config.DefaultConfig(), nil
	}

	data, err := os.ReadFile(cfgPath)
	if err != nil {
		return nil, err
	}

	return config.LoadConfigFromData(data)
}

// LoadStore loads the store
func LoadStore() (*store.Store, error) {
	cfg, err := LoadConfig()
	if err != nil {
		return nil, err
	}

	return store.New(cfg)
}

func expandPath(path string) string {
	if home, err := os.UserHomeDir(); err == nil {
		if len(path) > 1 && path[:2] == "~/" {
			return filepath.Join(home, path[2:])
		}
	}
	return path
}

// printResults prints search results in the specified format
func printResults(results []store.SearchResult, format string) {
	switch format {
	case "json":
		for _, r := range results {
			fmt.Printf(`{"path": "%s", "collection": "%s", "score": %f, "lines": %d, "title": "%s"}`+"\n",
				r.Path, r.Collection, r.Score, r.Lines, r.Title)
		}
	case "csv":
		fmt.Println("path,collection,score,lines,title")
		for _, r := range results {
			fmt.Printf("%s,%s,%f,%d,%s\n", r.Path, r.Collection, r.Score, r.Lines, r.Title)
		}
	default: // cli
		for _, r := range results {
			fmt.Printf("[%.3f] %s (%s)\n", r.Score, r.Path, r.Collection)
			fmt.Printf("    Title: %s, Lines: %d\n", r.Title, r.Lines)
		}
	}
}

func init() {
	log.SetFlags(0)
}
