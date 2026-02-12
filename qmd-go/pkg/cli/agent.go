package cli

import (
	"bufio"
	"fmt"
	"os"
	"strings"

	"github.com/qmd/qmd-go/pkg/formatter"
	"github.com/qmd/qmd-go/pkg/store"
	"github.com/spf13/cobra"
)

var agentCmd = &cobra.Command{
	Use:   "agent",
	Short: "Run in agent mode (interactive)",
	Run: func(cmd *cobra.Command, args []string) {
		runAgent()
	},
}

// QueryIntent represents the query intent
type QueryIntent string

const (
	IntentKeyword QueryIntent = "keyword"  // BM25
	IntentSemantic QueryIntent = "semantic" // Vector
	IntentHybrid  QueryIntent = "hybrid"    // Hybrid
)

// classifyIntent classifies the query intent
func classifyIntent(query string) QueryIntent {
	// Simple rule-based classification
	lowerQuery := strings.ToLower(query)

	// Check for natural language patterns
	nlPatterns := []string{"explain", "describe", "what is", "how does", "why", "meaning"}
	for _, p := range nlPatterns {
		if strings.Contains(lowerQuery, p) {
			return IntentSemantic
		}
	}

	// Check for technical terms (likely keyword search)
	techPatterns := []string{"error", "exception", "api", "function", "class", "method"}
	for _, p := range techPatterns {
		if strings.Contains(lowerQuery, p) {
			return IntentKeyword
		}
	}

	// Default to hybrid
	return IntentHybrid
}

// runAgent runs the interactive agent
func runAgent() {
	s, err := getStore()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
	defer s.Close()

	fmt.Println("QMD Agent - Interactive Search")
	fmt.Println("Type 'quit' or 'exit' to exit")
	fmt.Println("Type '/bm25', '/vector', or '/hybrid' to force search method")
	fmt.Println()

	reader := bufio.NewReader(os.Stdin)
	forceMethod := ""

	for {
		fmt.Print("> ")
		line, _ := reader.ReadString('\n')
		line = strings.TrimSpace(line)

		if line == "" {
			continue
		}

		if line == "quit" || line == "exit" {
			break
		}

		// Check for forced method
		if strings.HasPrefix(line, "/bm25") {
			forceMethod = "bm25"
			line = strings.TrimPrefix(line, "/bm25 ")
			fmt.Println("Forced BM25 search")
		} else if strings.HasPrefix(line, "/vector") {
			forceMethod = "vector"
			line = strings.TrimPrefix(line, "/vector ")
			fmt.Println("Forced Vector search")
		} else if strings.HasPrefix(line, "/hybrid") {
			forceMethod = "hybrid"
			line = strings.TrimPrefix(line, "/hybrid ")
			fmt.Println("Forced Hybrid search")
		}

		if line == "" {
			continue
		}

		// Classify intent
		intent := classifyIntent(line)
		if forceMethod != "" {
			switch forceMethod {
			case "bm25":
				intent = IntentKeyword
			case "vector":
				intent = IntentSemantic
			case "hybrid":
				intent = IntentHybrid
			}
			forceMethod = ""
		}

		opts := store.SearchOptions{
			Limit:    limit,
			MinScore: minScore,
			All:      true,
		}

		var results []store.SearchResult

		switch intent {
		case IntentKeyword:
			results, _ = s.BM25Search(line, opts)
		case IntentSemantic:
			fmt.Println("Vector search requires embedding model setup")
			continue
		case IntentHybrid:
			results, _ = s.BM25Search(line, opts)
		}

		f := formatter.New(outputFormat, limit)
		fmt.Print(f.FormatSearchResults(results))
	}

	fmt.Println("Goodbye!")
}
