package anel

import "encoding/json"

// SearchSpec returns the ANEL spec for the search command
func SearchSpec() *AnelSpec {
	inputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"query": {"type": "string"},
			"limit": {"type": "integer", "default": 20},
			"min_score": {"type": "number", "default": 0.0},
			"collection": {"type": "string"},
			"all": {"type": "boolean", "default": false}
		},
		"required": ["query"]
	}`)

	outputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"results": {
				"type": "array",
				"items": {
					"type": "object",
					"properties": {
						"docid": {"type": "string"},
						"path": {"type": "string"},
						"score": {"type": "number"},
						"lines": {"type": "integer"}
					}
				}
			},
			"total": {"type": "integer"}
		}
	}`)

	return &AnelSpec{
		Version:      Version,
		Command:      "search",
		InputSchema:  inputSchema,
		OutputSchema: outputSchema,
		ErrorCodes: []ErrorCode{
			ErrorCodeSearchFailed,
			ErrorCodeIndexNotReady,
			ErrorCodeQueryParseError,
		},
	}
}

// VSearchSpec returns the ANEL spec for the vsearch command
func VSearchSpec() *AnelSpec {
	inputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"query": {"type": "string"},
			"limit": {"type": "integer", "default": 20},
			"collection": {"type": "string"},
			"all": {"type": "boolean", "default": false}
		},
		"required": ["query"]
	}`)

	outputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"results": {
				"type": "array",
				"items": {
					"type": "object",
					"properties": {
						"docid": {"type": "string"},
						"path": {"type": "string"},
						"score": {"type": "number"},
						"lines": {"type": "integer"}
					}
				}
			},
			"total": {"type": "integer"}
		}
	}`)

	return &AnelSpec{
		Version:      Version,
		Command:      "vsearch",
		InputSchema:  inputSchema,
		OutputSchema: outputSchema,
		ErrorCodes: []ErrorCode{
			ErrorCodeSearchFailed,
			ErrorCodeIndexNotReady,
			ErrorCodeEmbeddingFailed,
			ErrorCodeModelNotFound,
		},
	}
}

// QuerySpec returns the ANEL spec for the query (hybrid search) command
func QuerySpec() *AnelSpec {
	inputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"query": {"type": "string"},
			"limit": {"type": "integer", "default": 20},
			"collection": {"type": "string"},
			"all": {"type": "boolean", "default": false}
		},
		"required": ["query"]
	}`)

	outputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"results": {
				"type": "array",
				"items": {
					"type": "object",
					"properties": {
						"docid": {"type": "string"},
						"path": {"type": "string"},
						"score": {"type": "number"},
						"lines": {"type": "integer"},
						"reranked": {"type": "boolean"}
					}
				}
			},
			"total": {"type": "integer"}
		}
	}`)

	return &AnelSpec{
		Version:      Version,
		Command:      "query",
		InputSchema:  inputSchema,
		OutputSchema: outputSchema,
		ErrorCodes: []ErrorCode{
			ErrorCodeSearchFailed,
			ErrorCodeIndexNotReady,
			ErrorCodeEmbeddingFailed,
			ErrorCodeModelNotFound,
			ErrorCodeQueryParseError,
		},
	}
}

// GetSpec returns the ANEL spec for the get command
func GetSpec() *AnelSpec {
	inputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"file": {"type": "string", "description": "File path with optional :line suffix"},
			"limit": {"type": "integer", "default": 50},
			"from": {"type": "integer", "default": 0},
			"full": {"type": "boolean", "default": false}
		},
		"required": ["file"]
	}`)

	outputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"path": {"type": "string"},
			"lines": {"type": "array", "items": {"type": "string"}},
			"total_lines": {"type": "integer"}
		}
	}`)

	return &AnelSpec{
		Version:      Version,
		Command:      "get",
		InputSchema:  inputSchema,
		OutputSchema: outputSchema,
		ErrorCodes: []ErrorCode{
			ErrorCodeNotFound,
			ErrorCodeInvalidInput,
		},
	}
}

// CollectionSpec returns the ANEL spec for the collection command
func CollectionSpec() *AnelSpec {
	inputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"action": {"type": "string", "enum": ["add", "list", "remove", "rename"]},
			"name": {"type": "string"},
			"path": {"type": "string"},
			"mask": {"type": "string", "default": "**/*"},
			"description": {"type": "string"},
			"new_name": {"type": "string"}
		}
	}`)

	outputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"collections": {
				"type": "array",
				"items": {
					"type": "object",
					"properties": {
						"name": {"type": "string"},
						"path": {"type": "string"},
						"pattern": {"type": "string"},
						"description": {"type": "string"}
					}
				}
			},
			"action": {"type": "string"}
		}
	}`)

	return &AnelSpec{
		Version:      Version,
		Command:      "collection",
		InputSchema:  inputSchema,
		OutputSchema: outputSchema,
		ErrorCodes: []ErrorCode{
			ErrorCodeCollectionNotFound,
			ErrorCodeCollectionExists,
			ErrorCodeInvalidInput,
		},
	}
}

// EmbedSpec returns the ANEL spec for the embed command
func EmbedSpec() *AnelSpec {
	inputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"force": {"type": "boolean", "default": false},
			"collection": {"type": "string"}
		}
	}`)

	outputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"collections_processed": {"type": "integer"},
			"documents_embedded": {"type": "integer"},
			"chunks_embedded": {"type": "integer"},
			"model": {"type": "string"}
		}
	}`)

	return &AnelSpec{
		Version:      Version,
		Command:      "embed",
		InputSchema:  inputSchema,
		OutputSchema: outputSchema,
		ErrorCodes: []ErrorCode{
			ErrorCodeEmbeddingFailed,
			ErrorCodeModelNotFound,
			ErrorCodeModelLoadFailed,
			ErrorCodeCollectionNotFound,
		},
	}
}

// UpdateSpec returns the ANEL spec for the update command
func UpdateSpec() *AnelSpec {
	inputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"pull": {"type": "boolean", "default": false},
			"collection": {"type": "string"}
		}
	}`)

	outputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"collections_updated": {"type": "integer"},
			"documents_indexed": {"type": "integer"},
			"documents_removed": {"type": "integer"}
		}
	}`)

	return &AnelSpec{
		Version:      Version,
		Command:      "update",
		InputSchema:  inputSchema,
		OutputSchema: outputSchema,
		ErrorCodes: []ErrorCode{
			ErrorCodeIndexNotReady,
			ErrorCodeCollectionNotFound,
			ErrorCodeStorageError,
		},
	}
}

// StatusSpec returns the ANEL spec for the status command
func StatusSpec() *AnelSpec {
	inputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"verbose": {"type": "boolean", "default": false},
			"collection": {"type": "string"}
		}
	}`)

	outputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"collections": {
				"type": "array",
				"items": {
					"type": "object",
					"properties": {
						"name": {"type": "string"},
						"documents": {"type": "integer"},
						"chunks": {"type": "integer"},
						"embeddings": {"type": "integer"},
						"last_updated": {"type": "string"}
					}
				}
			}
		}
	}`)

	return &AnelSpec{
		Version:      Version,
		Command:      "status",
		InputSchema:  inputSchema,
		OutputSchema: outputSchema,
		ErrorCodes: []ErrorCode{
			ErrorCodeCollectionNotFound,
			ErrorCodeIndexNotReady,
		},
	}
}

// CleanupSpec returns the ANEL spec for the cleanup command
func CleanupSpec() *AnelSpec {
	inputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"dry_run": {"type": "boolean", "default": false},
			"older_than": {"type": "integer", "default": 30},
			"collection": {"type": "string"}
		}
	}`)

	outputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"entries_removed": {"type": "integer"},
			"dry_run": {"type": "boolean"}
		}
	}`)

	return &AnelSpec{
		Version:      Version,
		Command:      "cleanup",
		InputSchema:  inputSchema,
		OutputSchema: outputSchema,
		ErrorCodes: []ErrorCode{
			ErrorCodeCollectionNotFound,
			ErrorCodeStorageError,
		},
	}
}

// AgentSpec returns the ANEL spec for the agent command
func AgentSpec() *AnelSpec {
	inputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"interactive": {"type": "boolean", "default": false},
			"query": {"type": "string"},
			"mcp": {"type": "boolean", "default": false},
			"transport": {"type": "string", "default": "stdio"}
		}
	}`)

	outputSchema := json.RawMessage(`{
		"type": "object",
		"properties": {
			"intent": {"type": "string"},
			"results": {"type": "array"},
			"mode": {"type": "string"}
		}
	}`)

	return &AnelSpec{
		Version:      Version,
		Command:      "agent",
		InputSchema:  inputSchema,
		OutputSchema: outputSchema,
		ErrorCodes: []ErrorCode{
			ErrorCodeSearchFailed,
			ErrorCodeIndexNotReady,
			ErrorCodeEmbeddingFailed,
		},
	}
}

// GetSpecForCommand returns the spec for a specific command
func GetSpecForCommand(command string) *AnelSpec {
	switch command {
	case "search":
		return SearchSpec()
	case "vsearch":
		return VSearchSpec()
	case "query":
		return QuerySpec()
	case "get":
		return GetSpec()
	case "collection":
		return CollectionSpec()
	case "embed":
		return EmbedSpec()
	case "update":
		return UpdateSpec()
	case "status":
		return StatusSpec()
	case "cleanup":
		return CleanupSpec()
	case "agent":
		return AgentSpec()
	default:
		return nil
	}
}
