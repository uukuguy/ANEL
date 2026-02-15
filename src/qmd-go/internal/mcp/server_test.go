package mcp

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/qmd/qmd-go/internal/config"
)

// newTestServer creates a Server with nil store and default config.
// Suitable for tests that don't invoke store-dependent tools.
func newTestServer() *Server {
	cfg := config.DefaultConfig()
	return NewServer(nil, cfg)
}

// --- handleMessage dispatching ---

func TestHandleMessage_Initialize(t *testing.T) {
	srv := newTestServer()
	msg := map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      float64(1),
		"method":  "initialize",
	}
	resp := srv.handleMessage(msg)
	if resp == nil {
		t.Fatal("expected non-nil response for initialize")
	}
	assertJSONRPC(t, resp, float64(1))
	result := resp["result"].(map[string]interface{})
	if result["name"] != "qmd-go" {
		t.Errorf("expected name qmd-go, got %v", result["name"])
	}
}

func TestHandleMessage_ToolsList(t *testing.T) {
	srv := newTestServer()
	msg := map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      float64(2),
		"method":  "tools/list",
	}
	resp := srv.handleMessage(msg)
	if resp == nil {
		t.Fatal("expected non-nil response for tools/list")
	}
	assertJSONRPC(t, resp, float64(2))
}

func TestHandleMessage_ToolsCall(t *testing.T) {
	srv := newTestServer()
	tmpDir := t.TempDir()
	tmpFile := filepath.Join(tmpDir, "test.txt")
	if err := os.WriteFile(tmpFile, []byte("hello"), 0644); err != nil {
		t.Fatal(err)
	}

	msg := map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      float64(3),
		"method":  "tools/call",
		"params": map[string]interface{}{
			"name":      "get",
			"arguments": map[string]interface{}{"path": tmpFile},
		},
	}
	resp := srv.handleMessage(msg)
	if resp == nil {
		t.Fatal("expected non-nil response for tools/call")
	}
	assertJSONRPC(t, resp, float64(3))
}

func TestHandleMessage_UnknownMethod(t *testing.T) {
	srv := newTestServer()
	msg := map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      float64(99),
		"method":  "nonexistent/method",
	}
	resp := srv.handleMessage(msg)
	if resp != nil {
		t.Errorf("expected nil response for unknown method, got %v", resp)
	}
}

// --- handleInitialize ---

func TestHandleInitialize(t *testing.T) {
	srv := newTestServer()
	resp := srv.handleInitialize(float64(10))

	assertJSONRPC(t, resp, float64(10))

	result, ok := resp["result"].(map[string]interface{})
	if !ok {
		t.Fatal("result is not a map")
	}

	tests := []struct {
		key  string
		want interface{}
	}{
		{"name", "qmd-go"},
		{"version", "0.1.0"},
		{"protocolVersion", "2024-11-05"},
	}
	for _, tt := range tests {
		if result[tt.key] != tt.want {
			t.Errorf("result[%q] = %v, want %v", tt.key, result[tt.key], tt.want)
		}
	}

	caps, ok := result["capabilities"].(map[string]interface{})
	if !ok {
		t.Fatal("capabilities is not a map")
	}
	if _, ok := caps["tools"]; !ok {
		t.Error("capabilities missing 'tools'")
	}
	if _, ok := caps["resources"]; !ok {
		t.Error("capabilities missing 'resources'")
	}
}

// --- handleToolsList ---

func TestHandleToolsList(t *testing.T) {
	srv := newTestServer()
	resp := srv.handleToolsList(float64(20))

	assertJSONRPC(t, resp, float64(20))

	result := resp["result"].(map[string]interface{})
	tools, ok := result["tools"].([]map[string]interface{})
	if !ok {
		t.Fatal("tools is not []map[string]interface{}")
	}

	expectedTools := []string{"search", "vsearch", "query", "get", "status"}
	if len(tools) != len(expectedTools) {
		t.Fatalf("expected %d tools, got %d", len(expectedTools), len(tools))
	}

	for i, name := range expectedTools {
		if tools[i]["name"] != name {
			t.Errorf("tool[%d] name = %v, want %v", i, tools[i]["name"], name)
		}
	}
}

func TestHandleToolsList_InputSchemas(t *testing.T) {
	srv := newTestServer()
	resp := srv.handleToolsList(float64(21))
	result := resp["result"].(map[string]interface{})
	tools := result["tools"].([]map[string]interface{})

	// search, vsearch, query, get should have inputSchema
	toolsWithSchema := []string{"search", "vsearch", "query", "get"}
	for i, name := range toolsWithSchema {
		schema, ok := tools[i]["inputSchema"].(map[string]interface{})
		if !ok {
			t.Errorf("tool %q missing inputSchema", name)
			continue
		}
		if schema["type"] != "object" {
			t.Errorf("tool %q inputSchema type = %v, want object", name, schema["type"])
		}
		props, ok := schema["properties"].(map[string]interface{})
		if !ok {
			t.Errorf("tool %q inputSchema missing properties", name)
		}
		if len(props) == 0 {
			t.Errorf("tool %q inputSchema has empty properties", name)
		}
	}

	// status tool (index 4) has no inputSchema
	if _, ok := tools[4]["inputSchema"]; ok {
		t.Error("status tool should not have inputSchema")
	}
}

// --- handleToolsCall: get tool ---

func TestToolsCall_Get_ValidFile(t *testing.T) {
	srv := newTestServer()
	tmpDir := t.TempDir()
	tmpFile := filepath.Join(tmpDir, "sample.txt")
	content := "line0\nline1\nline2\nline3\nline4"
	if err := os.WriteFile(tmpFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	msg := map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      float64(30),
		"method":  "tools/call",
		"params": map[string]interface{}{
			"name":      "get",
			"arguments": map[string]interface{}{"path": tmpFile},
		},
	}
	resp := srv.handleMessage(msg)
	assertJSONRPC(t, resp, float64(30))

	text, isError := extractToolResult(t, resp)
	if isError {
		t.Errorf("expected no error, got: %s", text)
	}
	if text != content {
		t.Errorf("content mismatch:\ngot:  %q\nwant: %q", text, content)
	}
}

func TestToolsCall_Get_MissingPath(t *testing.T) {
	srv := newTestServer()
	msg := map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      float64(31),
		"method":  "tools/call",
		"params": map[string]interface{}{
			"name":      "get",
			"arguments": map[string]interface{}{},
		},
	}
	resp := srv.handleMessage(msg)
	assertJSONRPC(t, resp, float64(31))

	text, isError := extractToolResult(t, resp)
	if !isError {
		t.Error("expected isError=true for missing path")
	}
	if !strings.Contains(text, "path is required") {
		t.Errorf("expected 'path is required' error, got: %s", text)
	}
}

func TestToolsCall_Get_NonexistentFile(t *testing.T) {
	srv := newTestServer()
	msg := map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      float64(32),
		"method":  "tools/call",
		"params": map[string]interface{}{
			"name":      "get",
			"arguments": map[string]interface{}{"path": "/nonexistent/file.txt"},
		},
	}
	resp := srv.handleMessage(msg)
	assertJSONRPC(t, resp, float64(32))

	text, isError := extractToolResult(t, resp)
	if !isError {
		t.Error("expected isError=true for nonexistent file")
	}
	if !strings.Contains(text, "Error reading file") {
		t.Errorf("expected file read error, got: %s", text)
	}
}

func TestToolsCall_Get_WithFromAndLimit(t *testing.T) {
	srv := newTestServer()
	tmpDir := t.TempDir()
	tmpFile := filepath.Join(tmpDir, "lines.txt")
	content := "line0\nline1\nline2\nline3\nline4"
	if err := os.WriteFile(tmpFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	msg := map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      float64(33),
		"method":  "tools/call",
		"params": map[string]interface{}{
			"name": "get",
			"arguments": map[string]interface{}{
				"path":  tmpFile,
				"from":  float64(1),
				"limit": float64(2),
			},
		},
	}
	resp := srv.handleMessage(msg)
	assertJSONRPC(t, resp, float64(33))

	text, isError := extractToolResult(t, resp)
	if isError {
		t.Errorf("expected no error, got: %s", text)
	}
	// from=1 skips line0, limit=2 takes line1 and line2
	expected := "line1\nline2"
	if text != expected {
		t.Errorf("content mismatch:\ngot:  %q\nwant: %q", text, expected)
	}
}

// --- handleToolsCall: unknown tool ---

func TestToolsCall_UnknownTool(t *testing.T) {
	srv := newTestServer()
	msg := map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      float64(40),
		"method":  "tools/call",
		"params": map[string]interface{}{
			"name":      "nonexistent_tool",
			"arguments": map[string]interface{}{},
		},
	}
	resp := srv.handleMessage(msg)
	assertJSONRPC(t, resp, float64(40))

	text, isError := extractToolResult(t, resp)
	if !isError {
		t.Error("expected isError=true for unknown tool")
	}
	if !strings.Contains(text, "Unknown tool") {
		t.Errorf("expected 'Unknown tool' error, got: %s", text)
	}
}

// --- JSON-RPC format validation ---

func TestJSONRPC_Format_Initialize(t *testing.T) {
	srv := newTestServer()
	resp := srv.handleInitialize("string-id")
	assertJSONRPC(t, resp, "string-id")
	if _, ok := resp["result"]; !ok {
		t.Error("response missing 'result' field")
	}
}

func TestJSONRPC_Format_ToolsList(t *testing.T) {
	srv := newTestServer()
	resp := srv.handleToolsList(float64(100))
	assertJSONRPC(t, resp, float64(100))
	if _, ok := resp["result"]; !ok {
		t.Error("response missing 'result' field")
	}
}

func TestJSONRPC_Format_ToolsCall(t *testing.T) {
	srv := newTestServer()
	msg := map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      float64(101),
		"method":  "tools/call",
		"params": map[string]interface{}{
			"name":      "get",
			"arguments": map[string]interface{}{},
		},
	}
	resp := srv.handleMessage(msg)
	assertJSONRPC(t, resp, float64(101))
	if _, ok := resp["result"]; !ok {
		t.Error("response missing 'result' field")
	}
}

func TestJSONRPC_IDPreserved_NilID(t *testing.T) {
	srv := newTestServer()
	resp := srv.handleInitialize(nil)
	if resp["jsonrpc"] != "2.0" {
		t.Errorf("jsonrpc = %v, want 2.0", resp["jsonrpc"])
	}
	if resp["id"] != nil {
		t.Errorf("id = %v, want nil", resp["id"])
	}
}

// --- NewServer ---

func TestNewServer(t *testing.T) {
	cfg := config.DefaultConfig()
	srv := NewServer(nil, cfg)
	if srv == nil {
		t.Fatal("NewServer returned nil")
	}
}

// --- helpers ---

// assertJSONRPC checks that a response has "jsonrpc": "2.0" and the expected id.
func assertJSONRPC(t *testing.T, resp map[string]interface{}, expectedID interface{}) {
	t.Helper()
	if resp["jsonrpc"] != "2.0" {
		t.Errorf("jsonrpc = %v, want 2.0", resp["jsonrpc"])
	}
	if resp["id"] != expectedID {
		t.Errorf("id = %v, want %v", resp["id"], expectedID)
	}
}

// extractToolResult pulls the text and isError from a tools/call response.
func extractToolResult(t *testing.T, resp map[string]interface{}) (string, bool) {
	t.Helper()
	result, ok := resp["result"].(map[string]interface{})
	if !ok {
		t.Fatal("result is not a map")
	}
	contentArr, ok := result["content"].([]map[string]interface{})
	if !ok {
		t.Fatal("result.content is not []map[string]interface{}")
	}
	if len(contentArr) == 0 {
		t.Fatal("result.content is empty")
	}
	text, _ := contentArr[0]["text"].(string)
	isError, _ := result["isError"].(bool)
	return text, isError
}
