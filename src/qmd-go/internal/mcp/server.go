package mcp

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
)

// RunServer runs the MCP server
func RunServer(transport string, port int) error {
	if transport == "stdio" {
		return runStdioServer()
	}
	return runSSEServer(port)
}

func runStdioServer() error {
	fmt.Fprintln(os.Stderr, "Starting MCP server (stdio)")

	// Read JSON-RPC messages from stdin
	decoder := json.NewDecoder(os.Stdin)
	encoder := json.NewEncoder(os.Stdout)

	for decoder.More() {
		var message map[string]interface{}
		if err := decoder.Decode(&message); err != nil {
			break
		}

		response := handleMessage(message)
		if response != nil {
			if err := encoder.Encode(response); err != nil {
				break
			}
		}
	}

	return nil
}

func runSSEServer(port int) error {
	fmt.Fprintf(os.Stderr, "SSE transport not yet implemented, port %d\n", port)
	return nil
}

func handleMessage(message map[string]interface{}) map[string]interface{} {
	method, _ := message["method"].(string)
	id, _ := message["id"]

	if method == "initialize" {
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

	if method == "tools/list" {
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
								"query": map[string]interface{}{"type": "string", "description": "Search query"},
								"limit": map[string]interface{}{"type": "integer", "description": "Max results"},
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
								"query": map[string]interface{}{"type": "string", "description": "Search query"},
								"limit": map[string]interface{}{"type": "integer", "description": "Max results"},
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
								"query": map[string]interface{}{"type": "string", "description": "Search query"},
								"limit": map[string]interface{}{"type": "integer", "description": "Max results"},
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
								"from": map[string]interface{}{"type": "integer", "description": "Start line"},
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

	return nil
}
