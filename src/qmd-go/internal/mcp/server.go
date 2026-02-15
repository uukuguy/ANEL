package mcp

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"strings"

	"github.com/qmd/qmd-go/internal/config"
	"github.com/qmd/qmd-go/internal/store"
)

// Server holds the MCP server state
type Server struct {
	store  *store.Store
	config *config.Config
}

// NewServer creates a new MCP server
func NewServer(s *store.Store, cfg *config.Config) *Server {
	return &Server{store: s, config: cfg}
}

// RunServer runs the MCP server (backward-compatible entry point)
func RunServer(transport string, port int) error {
	cfg, err := config.LoadConfig()
	if err != nil {
		return fmt.Errorf("failed to load config: %w", err)
	}
	s, err := store.New(cfg)
	if err != nil {
		return fmt.Errorf("failed to create store: %w", err)
	}
	srv := NewServer(s, cfg)
	if transport == "stdio" {
		return srv.runStdioServer()
	}
	return srv.runSSEServer(port)
}

// RunServerWithStore runs the MCP server with an existing store
func RunServerWithStore(transport string, port int, s *store.Store, cfg *config.Config) error {
	srv := NewServer(s, cfg)
	if transport == "stdio" {
		return srv.runStdioServer()
	}
	return srv.runSSEServer(port)
}

func (srv *Server) runStdioServer() error {
	fmt.Fprintln(os.Stderr, "Starting MCP server (stdio)")

	scanner := bufio.NewScanner(os.Stdin)
	// Increase buffer size for large messages
	scanner.Buffer(make([]byte, 0, 1024*1024), 1024*1024)

	for scanner.Scan() {
		line := scanner.Text()
		if strings.TrimSpace(line) == "" {
			continue
		}

		var message map[string]interface{}
		if err := json.Unmarshal([]byte(line), &message); err != nil {
			continue
		}

		response := srv.handleMessage(message)
		if response != nil {
			data, err := json.Marshal(response)
			if err != nil {
				continue
			}
			fmt.Println(string(data))
		}
	}

	return nil
}

func (srv *Server) runSSEServer(port int) error {
	fmt.Fprintf(os.Stderr, "SSE transport not yet implemented, port %d\n", port)
	return nil
}

func (srv *Server) handleMessage(message map[string]interface{}) map[string]interface{} {
	method, _ := message["method"].(string)
	id := message["id"]

	switch method {
	case "initialize":
		return srv.handleInitialize(id)
	case "tools/list":
		return srv.handleToolsList(id)
	case "tools/call":
		return srv.handleToolsCall(id, message)
	default:
		return nil
	}
}

func (srv *Server) handleInitialize(id interface{}) map[string]interface{} {
	return map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      id,
		"result": map[string]interface{}{
			"name":            "qmd-go",
			"version":         "0.1.0",
			"protocolVersion": "2024-11-05",
			"capabilities": map[string]interface{}{
				"tools":     map[string]interface{}{},
				"resources": map[string]interface{}{},
			},
		},
	}
}

func (srv *Server) handleToolsList(id interface{}) map[string]interface{} {
	return map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      id,
		"result": map[string]interface{}{
			"tools": []map[string]interface{}{
				{
					"name":        "search",
					"description": "BM25 full-text search",
					"inputSchema": map[string]interface{}{
						"type": "object",
						"properties": map[string]interface{}{
							"query":      map[string]interface{}{"type": "string", "description": "Search query"},
							"limit":      map[string]interface{}{"type": "integer", "description": "Max results"},
							"collection": map[string]interface{}{"type": "string", "description": "Collection name"},
						},
						"required": []string{"query"},
					},
				},
				{
					"name":        "vsearch",
					"description": "Vector semantic search",
					"inputSchema": map[string]interface{}{
						"type": "object",
						"properties": map[string]interface{}{
							"query":      map[string]interface{}{"type": "string", "description": "Search query"},
							"limit":      map[string]interface{}{"type": "integer", "description": "Max results"},
							"collection": map[string]interface{}{"type": "string", "description": "Collection name"},
						},
						"required": []string{"query"},
					},
				},
				{
					"name":        "query",
					"description": "Hybrid search with reranking",
					"inputSchema": map[string]interface{}{
						"type": "object",
						"properties": map[string]interface{}{
							"query":      map[string]interface{}{"type": "string", "description": "Search query"},
							"limit":      map[string]interface{}{"type": "integer", "description": "Max results"},
							"collection": map[string]interface{}{"type": "string", "description": "Collection name"},
						},
						"required": []string{"query"},
					},
				},
				{
					"name":        "get",
					"description": "Get document content",
					"inputSchema": map[string]interface{}{
						"type": "object",
						"properties": map[string]interface{}{
							"path":  map[string]interface{}{"type": "string", "description": "File path"},
							"from":  map[string]interface{}{"type": "integer", "description": "Start line"},
							"limit": map[string]interface{}{"type": "integer", "description": "Max lines"},
						},
						"required": []string{"path"},
					},
				},
				{
					"name":        "status",
					"description": "Show index status",
				},
			},
		},
	}
}

func (srv *Server) handleToolsCall(id interface{}, message map[string]interface{}) map[string]interface{} {
	params, _ := message["params"].(map[string]interface{})
	toolName, _ := params["name"].(string)
	args, _ := params["arguments"].(map[string]interface{})

	var content string
	var isError bool

	switch toolName {
	case "search":
		content, isError = srv.toolSearch(args)
	case "vsearch":
		content, isError = srv.toolVSearch(args)
	case "query":
		content, isError = srv.toolQuery(args)
	case "get":
		content, isError = srv.toolGet(args)
	case "status":
		content, isError = srv.toolStatus()
	default:
		content = fmt.Sprintf("Unknown tool: %s", toolName)
		isError = true
	}

	return map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      id,
		"result": map[string]interface{}{
			"content": []map[string]interface{}{
				{
					"type": "text",
					"text": content,
				},
			},
			"isError": isError,
		},
	}
}

func (srv *Server) parseSearchArgs(args map[string]interface{}) (string, store.SearchOptions) {
	query, _ := args["query"].(string)
	limit := 20
	if l, ok := args["limit"].(float64); ok {
		limit = int(l)
	}
	collection := ""
	if c, ok := args["collection"].(string); ok {
		collection = c
	}

	options := store.SearchOptions{
		Limit:      limit,
		Collection: collection,
		SearchAll:  collection == "",
	}
	return query, options
}

func (srv *Server) toolSearch(args map[string]interface{}) (string, bool) {
	query, options := srv.parseSearchArgs(args)
	if query == "" {
		return "Error: query is required", true
	}

	results, err := srv.store.BM25Search(query, options)
	if err != nil {
		return fmt.Sprintf("Error: %v", err), true
	}

	return srv.formatSearchResults(results), false
}

func (srv *Server) toolVSearch(args map[string]interface{}) (string, bool) {
	query, options := srv.parseSearchArgs(args)
	if query == "" {
		return "Error: query is required", true
	}

	results, err := srv.store.VectorSearch(query, options)
	if err != nil {
		return fmt.Sprintf("Error: %v", err), true
	}

	return srv.formatSearchResults(results), false
}

func (srv *Server) toolQuery(args map[string]interface{}) (string, bool) {
	query, options := srv.parseSearchArgs(args)
	if query == "" {
		return "Error: query is required", true
	}

	results, err := srv.store.HybridSearch(query, options)
	if err != nil {
		return fmt.Sprintf("Error: %v", err), true
	}

	return srv.formatSearchResults(results), false
}

func (srv *Server) toolGet(args map[string]interface{}) (string, bool) {
	path, _ := args["path"].(string)
	if path == "" {
		return "Error: path is required", true
	}

	fromLine := 0
	if f, ok := args["from"].(float64); ok {
		fromLine = int(f)
	}
	limit := 0
	if l, ok := args["limit"].(float64); ok {
		limit = int(l)
	}

	data, err := os.ReadFile(path)
	if err != nil {
		return fmt.Sprintf("Error reading file: %v", err), true
	}

	content := string(data)
	lines := strings.Split(content, "\n")

	// Apply from/limit
	if fromLine > 0 && fromLine < len(lines) {
		lines = lines[fromLine:]
	}
	if limit > 0 && limit < len(lines) {
		lines = lines[:limit]
	}

	return strings.Join(lines, "\n"), false
}

func (srv *Server) toolStatus() (string, bool) {
	stats, err := srv.store.GetStats()
	if err != nil {
		return fmt.Sprintf("Error: %v", err), true
	}

	var sb strings.Builder
	sb.WriteString("Index Status\n")
	sb.WriteString("============\n")
	sb.WriteString(fmt.Sprintf("Collections: %d\n", stats.CollectionCount))
	sb.WriteString(fmt.Sprintf("Documents:   %d\n", stats.DocumentCount))
	sb.WriteString(fmt.Sprintf("Indexed:     %d\n", stats.IndexedCount))
	sb.WriteString(fmt.Sprintf("Pending:     %d\n", stats.PendingCount))

	return sb.String(), false
}

func (srv *Server) formatSearchResults(results []store.SearchResult) string {
	if len(results) == 0 {
		return "No results found."
	}

	var sb strings.Builder
	sb.WriteString(fmt.Sprintf("Found %d results:\n\n", len(results)))

	for i, r := range results {
		sb.WriteString(fmt.Sprintf("%d. [%.3f] %s\n", i+1, r.Score, r.Path))
		if r.Title != "" {
			sb.WriteString(fmt.Sprintf("   Title: %s\n", r.Title))
		}
		if r.Collection != "" {
			sb.WriteString(fmt.Sprintf("   Collection: %s\n", r.Collection))
		}
	}

	return sb.String()
}
