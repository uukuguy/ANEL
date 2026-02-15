package anel

import (
	"encoding/json"
	"testing"
)

// allCommands lists every command that should have a spec
var allCommands = []string{
	"search", "vsearch", "query", "get", "collection",
	"context", "embed", "update", "status", "cleanup",
	"agent", "mcp",
}

func TestGetSpecForCommand_AllCommands(t *testing.T) {
	for _, cmd := range allCommands {
		t.Run(cmd, func(t *testing.T) {
			spec := GetSpecForCommand(cmd)
			if spec == nil {
				t.Fatalf("GetSpecForCommand(%q) returned nil", cmd)
			}
			if spec.Command != cmd {
				t.Errorf("Command = %s, want %s", spec.Command, cmd)
			}
			if spec.Version != Version {
				t.Errorf("Version = %s, want %s", spec.Version, Version)
			}
		})
	}
}

func TestGetSpecForCommand_Unknown(t *testing.T) {
	spec := GetSpecForCommand("nonexistent")
	if spec != nil {
		t.Errorf("GetSpecForCommand(nonexistent) should return nil, got %v", spec)
	}
}

func TestAllSpecs_ValidJSON(t *testing.T) {
	for _, cmd := range allCommands {
		t.Run(cmd+"_json", func(t *testing.T) {
			spec := GetSpecForCommand(cmd)
			jsonStr := spec.ToJSON()

			var parsed map[string]interface{}
			if err := json.Unmarshal([]byte(jsonStr), &parsed); err != nil {
				t.Fatalf("Spec for %q produced invalid JSON: %v\n%s", cmd, err, jsonStr)
			}

			// Must have all required fields
			for _, field := range []string{"version", "command", "input_schema", "output_schema", "error_codes"} {
				if _, ok := parsed[field]; !ok {
					t.Errorf("Spec for %q missing field %q", cmd, field)
				}
			}
		})
	}
}

func TestAllSpecs_InputSchemaIsObject(t *testing.T) {
	for _, cmd := range allCommands {
		t.Run(cmd, func(t *testing.T) {
			spec := GetSpecForCommand(cmd)

			var schema map[string]interface{}
			if err := json.Unmarshal(spec.InputSchema, &schema); err != nil {
				t.Fatalf("InputSchema for %q is invalid JSON: %v", cmd, err)
			}

			if schema["type"] != "object" {
				t.Errorf("InputSchema type = %v, want object", schema["type"])
			}

			if _, ok := schema["properties"]; !ok {
				t.Errorf("InputSchema for %q missing 'properties'", cmd)
			}
		})
	}
}

func TestAllSpecs_OutputSchemaIsObject(t *testing.T) {
	for _, cmd := range allCommands {
		t.Run(cmd, func(t *testing.T) {
			spec := GetSpecForCommand(cmd)

			var schema map[string]interface{}
			if err := json.Unmarshal(spec.OutputSchema, &schema); err != nil {
				t.Fatalf("OutputSchema for %q is invalid JSON: %v", cmd, err)
			}

			if schema["type"] != "object" {
				t.Errorf("OutputSchema type = %v, want object", schema["type"])
			}
		})
	}
}

func TestAllSpecs_HaveErrorCodes(t *testing.T) {
	for _, cmd := range allCommands {
		t.Run(cmd, func(t *testing.T) {
			spec := GetSpecForCommand(cmd)
			if len(spec.ErrorCodes) == 0 {
				t.Errorf("Spec for %q has no error codes", cmd)
			}
		})
	}
}

// --- Individual spec validation ---

func TestSearchSpec_RequiresQuery(t *testing.T) {
	spec := SearchSpec()
	var schema map[string]interface{}
	json.Unmarshal(spec.InputSchema, &schema)

	required := schema["required"].([]interface{})
	if len(required) == 0 || required[0] != "query" {
		t.Error("SearchSpec should require 'query'")
	}
}

func TestSearchSpec_ErrorCodes(t *testing.T) {
	spec := SearchSpec()
	codes := map[ErrorCode]bool{}
	for _, c := range spec.ErrorCodes {
		codes[c] = true
	}

	expected := []ErrorCode{ErrorCodeSearchFailed, ErrorCodeIndexNotReady, ErrorCodeQueryParseError}
	for _, e := range expected {
		if !codes[e] {
			t.Errorf("SearchSpec missing error code %s", e)
		}
	}
}

func TestVSearchSpec_RequiresQuery(t *testing.T) {
	spec := VSearchSpec()
	var schema map[string]interface{}
	json.Unmarshal(spec.InputSchema, &schema)

	required := schema["required"].([]interface{})
	if len(required) == 0 || required[0] != "query" {
		t.Error("VSearchSpec should require 'query'")
	}
}

func TestVSearchSpec_HasEmbeddingError(t *testing.T) {
	spec := VSearchSpec()
	found := false
	for _, c := range spec.ErrorCodes {
		if c == ErrorCodeEmbeddingFailed {
			found = true
			break
		}
	}
	if !found {
		t.Error("VSearchSpec should include EMBEDDING_FAILED error code")
	}
}

func TestQuerySpec_RequiresQuery(t *testing.T) {
	spec := QuerySpec()
	var schema map[string]interface{}
	json.Unmarshal(spec.InputSchema, &schema)

	required := schema["required"].([]interface{})
	if len(required) == 0 || required[0] != "query" {
		t.Error("QuerySpec should require 'query'")
	}
}

func TestGetSpec_RequiresFile(t *testing.T) {
	spec := GetSpec()
	var schema map[string]interface{}
	json.Unmarshal(spec.InputSchema, &schema)

	required := schema["required"].([]interface{})
	if len(required) == 0 || required[0] != "file" {
		t.Error("GetSpec should require 'file'")
	}
}

func TestCollectionSpec_HasActions(t *testing.T) {
	spec := CollectionSpec()
	var schema map[string]interface{}
	json.Unmarshal(spec.InputSchema, &schema)

	props := schema["properties"].(map[string]interface{})
	action := props["action"].(map[string]interface{})
	enum := action["enum"].([]interface{})

	expected := map[string]bool{"add": true, "list": true, "remove": true, "rename": true}
	for _, v := range enum {
		delete(expected, v.(string))
	}
	if len(expected) > 0 {
		t.Errorf("CollectionSpec missing actions: %v", expected)
	}
}

func TestContextSpec_RequiresAction(t *testing.T) {
	spec := ContextSpec()
	var schema map[string]interface{}
	json.Unmarshal(spec.InputSchema, &schema)

	required := schema["required"].([]interface{})
	if len(required) == 0 || required[0] != "action" {
		t.Error("ContextSpec should require 'action'")
	}
}

func TestEmbedSpec_HasForceOption(t *testing.T) {
	spec := EmbedSpec()
	var schema map[string]interface{}
	json.Unmarshal(spec.InputSchema, &schema)

	props := schema["properties"].(map[string]interface{})
	if _, ok := props["force"]; !ok {
		t.Error("EmbedSpec should have 'force' property")
	}
}

func TestUpdateSpec_HasPullOption(t *testing.T) {
	spec := UpdateSpec()
	var schema map[string]interface{}
	json.Unmarshal(spec.InputSchema, &schema)

	props := schema["properties"].(map[string]interface{})
	if _, ok := props["pull"]; !ok {
		t.Error("UpdateSpec should have 'pull' property")
	}
}

func TestStatusSpec_HasVerboseOption(t *testing.T) {
	spec := StatusSpec()
	var schema map[string]interface{}
	json.Unmarshal(spec.InputSchema, &schema)

	props := schema["properties"].(map[string]interface{})
	if _, ok := props["verbose"]; !ok {
		t.Error("StatusSpec should have 'verbose' property")
	}
}

func TestMcpSpec_HasTransportOption(t *testing.T) {
	spec := McpSpec()
	var schema map[string]interface{}
	json.Unmarshal(spec.InputSchema, &schema)

	props := schema["properties"].(map[string]interface{})
	if _, ok := props["transport"]; !ok {
		t.Error("McpSpec should have 'transport' property")
	}
}
