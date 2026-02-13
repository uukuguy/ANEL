package main

import (
	"fmt"
	"log"
	"os"

	"github.com/qmd/qmd-go/internal/cli"
)

func main() {
	// Initialize logger
	log.SetFlags(0)

	// Load configuration (for validation)
	_, err := cli.LoadConfig()
	if err != nil {
		log.Printf("Warning: %v", err)
	}

	// Build and run CLI
	if err := cli.RootCmd.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
