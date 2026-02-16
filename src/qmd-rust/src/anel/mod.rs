//! ANEL (Agent-Native Execution Layer) module
//!
//! This module provides ANEL protocol support for QMD, enabling:
//! - ANID (Agent-Native ID) error types with RFC 7807 extensions
//! - NDJSON streaming output
//! - Dry-run and spec emission capabilities
//! - Trace context propagation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ANEL protocol version
pub const ANEL_VERSION: &str = "1.0";

/// Environment variable names
pub mod env {
    /// Agent trace ID for request correlation
    pub const TRACE_ID: &str = "AGENT_TRACE_ID";
    /// Agent identity token for authentication
    pub const IDENTITY_TOKEN: &str = "AGENT_IDENTITY_TOKEN";
    /// Output format override
    pub const OUTPUT_FORMAT: &str = "AGENT_OUTPUT_FORMAT";
    /// Dry-run mode override
    pub const DRY_RUN: &str = "AGENT_DRY_RUN";
    /// Emit spec mode
    pub const EMIT_SPEC: &str = "AGENT_EMIT_SPEC";
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Debug/trace level
    Debug,
    /// Informational
    Info,
    /// Warning - operation may have issues
    Warning,
    /// Error - operation failed
    Error,
    /// Critical - system-level failure
    Critical,
}

impl Default for Severity {
    fn default() -> Self {
        Self::Error
    }
}

/// Error codes for ANEL operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnelErrorCode {
    // Generic errors
    Unknown,
    InvalidInput,
    NotFound,
    PermissionDenied,

    // Search-related errors
    SearchFailed,
    IndexNotReady,
    QueryParseError,

    // Collection errors
    CollectionNotFound,
    CollectionExists,
    CollectionCorrupted,

    // Embedding errors
    EmbeddingFailed,
    ModelNotFound,
    ModelLoadFailed,

    // Storage errors
    StorageError,
    BackendUnavailable,

    // Configuration errors
    ConfigError,
    EnvironmentError,
}

impl Default for AnelErrorCode {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Recovery hint for error resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryHint {
    /// Hint code
    pub code: String,
    /// Human-readable description
    pub message: String,
    /// Suggested action
    pub action: Option<String>,
}

impl RecoveryHint {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            action: None,
        }
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }
}

/// ANID Error type (Agent-Native ID Error)
///
/// Implements RFC 7807 Problem Details with ANEL extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnelError {
    /// Error type identifier (ANEL error code)
    #[serde(rename = "error_code")]
    pub error_code: AnelErrorCode,

    /// HTTP-style status code (for compatibility)
    pub status: u16,

    /// Error title
    pub title: String,

    /// Detailed error message
    pub message: String,

    /// Severity level
    pub severity: Severity,

    /// Recovery hints (ANEL extension)
    #[serde(rename = "recovery_hints")]
    pub recovery_hints: Vec<RecoveryHint>,

    /// Request trace ID
    #[serde(rename = "trace_id")]
    pub trace_id: Option<String>,

    /// Additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AnelError {
    /// Create a new ANEL error
    pub fn new(
        error_code: AnelErrorCode,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let status = error_code.to_status();
        Self {
            error_code,
            status,
            title: title.into(),
            message: message.into(),
            severity: Severity::Error,
            recovery_hints: Vec::new(),
            trace_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Add a recovery hint
    pub fn with_hint(mut self, hint: RecoveryHint) -> Self {
        self.recovery_hints.push(hint);
        self
    }

    /// Add trace ID
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Serialize to NDJSON line
    pub fn to_ndjson(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Print to stderr in NDJSON format
    pub fn emit_stderr(&self) {
        eprintln!("{}", self.to_ndjson());
    }
}

impl std::fmt::Display for AnelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}", self.error_code, self.message)
    }
}

impl std::error::Error for AnelError {}

impl AnelErrorCode {
    /// Convert error code to HTTP status
    pub fn to_status(&self) -> u16 {
        match self {
            Self::Unknown => 500,
            Self::InvalidInput => 400,
            Self::NotFound => 404,
            Self::PermissionDenied => 403,
            Self::SearchFailed => 500,
            Self::IndexNotReady => 503,
            Self::QueryParseError => 400,
            Self::CollectionNotFound => 404,
            Self::CollectionExists => 409,
            Self::CollectionCorrupted => 500,
            Self::EmbeddingFailed => 500,
            Self::ModelNotFound => 404,
            Self::ModelLoadFailed => 500,
            Self::StorageError => 500,
            Self::BackendUnavailable => 503,
            Self::ConfigError => 500,
            Self::EnvironmentError => 500,
        }
    }
}

impl From<anyhow::Error> for AnelError {
    fn from(err: anyhow::Error) -> Self {
        let message = err.to_string();

        // Try to extract error code from error chain
        let error_code = if message.contains("not found") {
            AnelErrorCode::NotFound
        } else if message.contains("permission") {
            AnelErrorCode::PermissionDenied
        } else if message.contains("invalid") {
            AnelErrorCode::InvalidInput
        } else if message.contains("parse") || message.contains("Parse") {
            AnelErrorCode::QueryParseError
        } else if message.contains("collection") {
            AnelErrorCode::CollectionNotFound
        } else if message.contains("embedding") || message.contains("embed") {
            AnelErrorCode::EmbeddingFailed
        } else if message.contains("storage") || message.contains("database") {
            AnelErrorCode::StorageError
        } else if message.contains("config") || message.contains("Config") {
            AnelErrorCode::ConfigError
        } else {
            AnelErrorCode::Unknown
        };

        Self::new(error_code, "Operation Failed", message)
    }
}

/// Trace context for request correlation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TraceContext {
    /// Trace ID for correlation
    pub trace_id: Option<String>,
    /// Identity token
    pub identity_token: Option<String>,
    /// Additional tags
    pub tags: HashMap<String, String>,
}

impl TraceContext {
    /// Create from environment variables
    pub fn from_env() -> Self {
        Self {
            trace_id: std::env::var(env::TRACE_ID).ok(),
            identity_token: std::env::var(env::IDENTITY_TOKEN).ok(),
            tags: HashMap::new(),
        }
    }

    /// Get trace ID or generate a new one
    pub fn get_or_generate_trace_id(&self) -> String {
        self.trace_id.clone().unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            format!("qmd-{:x}", timestamp)
        })
    }
}

/// ANEL specification for a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnelSpec {
    /// Specification version
    pub version: String,
    /// Command name
    pub command: String,
    /// Input parameters schema
    pub input_schema: serde_json::Value,
    /// Output result schema
    pub output_schema: serde_json::Value,
    /// Error codes
    pub error_codes: Vec<AnelErrorCode>,
}

impl AnelSpec {
    /// Serialize to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Get spec for a command by name
    pub fn for_command(command: &str) -> Option<Self> {
        match command {
            "search" => Some(Self::search()),
            "vsearch" => Some(Self::vsearch()),
            "query" => Some(Self::query()),
            "get" => Some(Self::get()),
            "multi_get" => Some(Self::multi_get()),
            "collection" => Some(Self::collection()),
            "embed" => Some(Self::embed()),
            "update" => Some(Self::update()),
            "status" => Some(Self::status()),
            "cleanup" => Some(Self::cleanup()),
            "agent" => Some(Self::agent()),
            "context" => Some(Self::context()),
            "mcp" => Some(Self::mcp()),
            _ => None,
        }
    }

    /// Get spec for search command
    pub fn search() -> Self {
        Self {
            version: ANEL_VERSION.to_string(),
            command: "search".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "limit": {"type": "integer", "default": 20},
                    "min_score": {"type": "number", "default": 0.0},
                    "collection": {"type": "string"},
                    "all": {"type": "boolean", "default": false}
                },
                "required": ["query"]
            }),
            output_schema: serde_json::json!({
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
            }),
            error_codes: vec![
                AnelErrorCode::SearchFailed,
                AnelErrorCode::IndexNotReady,
                AnelErrorCode::QueryParseError,
            ],
        }
    }

    /// Get spec for vsearch command
    pub fn vsearch() -> Self {
        Self {
            version: ANEL_VERSION.to_string(),
            command: "vsearch".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "limit": {"type": "integer", "default": 20},
                    "collection": {"type": "string"},
                    "all": {"type": "boolean", "default": false}
                },
                "required": ["query"]
            }),
            output_schema: serde_json::json!({
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
            }),
            error_codes: vec![
                AnelErrorCode::SearchFailed,
                AnelErrorCode::IndexNotReady,
                AnelErrorCode::EmbeddingFailed,
                AnelErrorCode::ModelNotFound,
            ],
        }
    }

    /// Get spec for query (hybrid search) command
    pub fn query() -> Self {
        Self {
            version: ANEL_VERSION.to_string(),
            command: "query".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "limit": {"type": "integer", "default": 20},
                    "collection": {"type": "string"},
                    "all": {"type": "boolean", "default": false}
                },
                "required": ["query"]
            }),
            output_schema: serde_json::json!({
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
            }),
            error_codes: vec![
                AnelErrorCode::SearchFailed,
                AnelErrorCode::IndexNotReady,
                AnelErrorCode::EmbeddingFailed,
                AnelErrorCode::ModelNotFound,
                AnelErrorCode::QueryParseError,
            ],
        }
    }

    /// Get spec for get command
    pub fn get() -> Self {
        Self {
            version: ANEL_VERSION.to_string(),
            command: "get".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file": {"type": "string", "description": "File path with optional :line suffix"},
                    "limit": {"type": "integer", "default": 50},
                    "from": {"type": "integer", "default": 0},
                    "full": {"type": "boolean", "default": false}
                },
                "required": ["file"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "lines": {"type": "array", "items": {"type": "string"}},
                    "total_lines": {"type": "integer"}
                }
            }),
            error_codes: vec![
                AnelErrorCode::NotFound,
                AnelErrorCode::InvalidInput,
            ],
        }
    }

    /// Get spec for multi-get command
    pub fn multi_get() -> Self {
        Self {
            version: ANEL_VERSION.to_string(),
            command: "multi_get".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {"type": "string", "description": "Glob pattern for files"},
                    "limit": {"type": "integer", "default": 50},
                    "max_bytes": {"type": "integer"}
                },
                "required": ["pattern"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "files": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "path": {"type": "string"},
                                "lines": {"type": "array", "items": {"type": "string"}},
                                "truncated": {"type": "boolean"}
                            }
                        }
                    },
                    "total_files": {"type": "integer"},
                    "errors": {"type": "integer"}
                }
            }),
            error_codes: vec![
                AnelErrorCode::InvalidInput,
                AnelErrorCode::NotFound,
            ],
        }
    }

    /// Get spec for collection command
    pub fn collection() -> Self {
        Self {
            version: ANEL_VERSION.to_string(),
            command: "collection".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {"type": "string", "enum": ["add", "list", "remove", "rename"]},
                    "name": {"type": "string"},
                    "path": {"type": "string"},
                    "mask": {"type": "string", "default": "**/*"},
                    "description": {"type": "string"},
                    "new_name": {"type": "string"}
                }
            }),
            output_schema: serde_json::json!({
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
            }),
            error_codes: vec![
                AnelErrorCode::CollectionNotFound,
                AnelErrorCode::CollectionExists,
                AnelErrorCode::InvalidInput,
            ],
        }
    }

    /// Get spec for embed command
    pub fn embed() -> Self {
        Self {
            version: ANEL_VERSION.to_string(),
            command: "embed".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "force": {"type": "boolean", "default": false},
                    "collection": {"type": "string"}
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "collections_processed": {"type": "integer"},
                    "documents_embedded": {"type": "integer"},
                    "chunks_embedded": {"type": "integer"},
                    "model": {"type": "string"}
                }
            }),
            error_codes: vec![
                AnelErrorCode::EmbeddingFailed,
                AnelErrorCode::ModelNotFound,
                AnelErrorCode::ModelLoadFailed,
                AnelErrorCode::CollectionNotFound,
            ],
        }
    }

    /// Get spec for update command
    pub fn update() -> Self {
        Self {
            version: ANEL_VERSION.to_string(),
            command: "update".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pull": {"type": "boolean", "default": false},
                    "collection": {"type": "string"}
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "collections_updated": {"type": "integer"},
                    "documents_indexed": {"type": "integer"},
                    "documents_removed": {"type": "integer"}
                }
            }),
            error_codes: vec![
                AnelErrorCode::IndexNotReady,
                AnelErrorCode::CollectionNotFound,
                AnelErrorCode::StorageError,
            ],
        }
    }

    /// Get spec for status command
    pub fn status() -> Self {
        Self {
            version: ANEL_VERSION.to_string(),
            command: "status".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "verbose": {"type": "boolean", "default": false},
                    "collection": {"type": "string"}
                }
            }),
            output_schema: serde_json::json!({
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
            }),
            error_codes: vec![
                AnelErrorCode::CollectionNotFound,
                AnelErrorCode::IndexNotReady,
            ],
        }
    }

    /// Get spec for cleanup command
    pub fn cleanup() -> Self {
        Self {
            version: ANEL_VERSION.to_string(),
            command: "cleanup".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "dry_run": {"type": "boolean", "default": false},
                    "older_than": {"type": "integer", "default": 30},
                    "collection": {"type": "string"}
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "entries_removed": {"type": "integer"},
                    "dry_run": {"type": "boolean"}
                }
            }),
            error_codes: vec![
                AnelErrorCode::CollectionNotFound,
                AnelErrorCode::StorageError,
            ],
        }
    }

    /// Get spec for agent command
    pub fn agent() -> Self {
        Self {
            version: ANEL_VERSION.to_string(),
            command: "agent".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "interactive": {"type": "boolean", "default": false},
                    "query": {"type": "string"},
                    "mcp": {"type": "boolean", "default": false},
                    "transport": {"type": "string", "default": "stdio"}
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "intent": {"type": "string"},
                    "results": {"type": "array"},
                    "mode": {"type": "string"}
                }
            }),
            error_codes: vec![
                AnelErrorCode::SearchFailed,
                AnelErrorCode::IndexNotReady,
                AnelErrorCode::EmbeddingFailed,
            ],
        }
    }

    /// Get spec for ls command
    pub fn ls() -> Self {
        Self {
            version: ANEL_VERSION.to_string(),
            command: "ls".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Optional path: collection or collection/path. Supports qmd:// prefix"}
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "collections": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": {"type": "string"},
                                "path": {"type": "string"},
                                "file_count": {"type": "integer"}
                            }
                        }
                    },
                    "files": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "path": {"type": "string"},
                                "title": {"type": "string"},
                                "size": {"type": "integer"},
                                "modified_at": {"type": "string"}
                            }
                        }
                    }
                }
            }),
            error_codes: vec![
                AnelErrorCode::CollectionNotFound,
                AnelErrorCode::InvalidInput,
            ],
        }
    }

    /// Get spec for context command
    pub fn context() -> Self {
        Self {
            version: ANEL_VERSION.to_string(),
            command: "context".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {"type": "string", "enum": ["add", "list", "rm"]},
                    "path": {"type": "string"},
                    "description": {"type": "string"}
                },
                "required": ["action"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "contexts": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "path": {"type": "string"},
                                "description": {"type": "string"}
                            }
                        }
                    },
                    "action": {"type": "string"}
                }
            }),
            error_codes: vec![
                AnelErrorCode::NotFound,
                AnelErrorCode::InvalidInput,
            ],
        }
    }

    /// Get spec for mcp command
    pub fn mcp() -> Self {
        Self {
            version: ANEL_VERSION.to_string(),
            command: "mcp".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "transport": {"type": "string", "default": "stdio"},
                    "port": {"type": "integer", "default": 8080}
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "status": {"type": "string"},
                    "transport": {"type": "string"},
                    "port": {"type": "integer"}
                }
            }),
            error_codes: vec![
                AnelErrorCode::ConfigError,
                AnelErrorCode::BackendUnavailable,
            ],
        }
    }
}

/// NDJSON output wrapper for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdjsonRecord<T: Serialize> {
    /// Record type: "result", "error", "spec", "metadata"
    #[serde(rename = "type")]
    pub record_type: String,
    /// Sequence number for ordering
    pub seq: u64,
    /// Payload
    pub payload: T,
}

impl<T: Serialize> NdjsonRecord<T> {
    /// Create a new NDJSON record
    pub fn new(record_type: impl Into<String>, seq: u64, payload: T) -> Self {
        Self {
            record_type: record_type.into(),
            seq,
            payload,
        }
    }

    /// Serialize to NDJSON line
    pub fn to_ndjson(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Print to stdout in NDJSON format
    pub fn emit(&self) {
        println!("{}", self.to_ndjson());
    }
}

/// ANEL command result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnelResult {
    /// Success flag
    pub success: bool,
    /// Result data
    pub data: serde_json::Value,
    /// Error (if failed)
    pub error: Option<AnelError>,
    /// Trace ID
    #[serde(rename = "trace_id")]
    pub trace_id: Option<String>,
}

impl AnelResult {
    /// Create a success result
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data,
            error: None,
            trace_id: None,
        }
    }

    /// Create an error result
    pub fn error(error: AnelError) -> Self {
        let trace_id = error.trace_id.clone();
        Self {
            success: false,
            data: serde_json::Value::Null,
            error: Some(error),
            trace_id,
        }
    }

    /// Add trace ID
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Serialize to NDJSON line
    pub fn to_ndjson(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anel_error_creation() {
        let error = AnelError::new(
            AnelErrorCode::NotFound,
            "Document Not Found",
            "The requested document was not found in the index",
        )
        .with_hint(RecoveryHint::new("REINDEX", "Try running qmd update to refresh the index"))
        .with_trace_id("test-trace-123");

        assert_eq!(error.error_code, AnelErrorCode::NotFound);
        assert_eq!(error.status, 404);
        assert_eq!(error.recovery_hints.len(), 1);
        assert_eq!(error.trace_id, Some("test-trace-123".to_string()));
    }

    #[test]
    fn test_ndjson_record() {
        let record = NdjsonRecord::new("result", 1, serde_json::json!({"key": "value"}));
        let json = record.to_ndjson();
        assert!(json.contains("\"type\":\"result\""));
        assert!(json.contains("\"seq\":1"));
    }

    #[test]
    fn test_anel_spec() {
        let spec = AnelSpec::search();
        assert_eq!(spec.command, "search");
        assert_eq!(spec.version, ANEL_VERSION);
    }

    #[test]
    fn test_trace_context() {
        let ctx = TraceContext::from_env();
        let trace_id = ctx.get_or_generate_trace_id();
        assert!(trace_id.starts_with("qmd-"));
    }
}
