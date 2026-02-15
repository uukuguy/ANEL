// Package anel provides ANEL (Agent-Native Execution Layer) protocol support for QMD.
//
// This package enables:
//   - ANID (Agent-Native ID) error types with RFC 7807 extensions
//   - NDJSON streaming output
//   - Dry-run and spec emission capabilities
//   - Trace context propagation
package anel

import (
	"encoding/json"
	"fmt"
	"os"
	"time"
)

// ANEL protocol version
const Version = "1.0"

// Environment variable names
var (
	// TRACE_ID is the agent trace ID for request correlation
	EnvTraceID = "AGENT_TRACE_ID"
	// IDENTITY_TOKEN is the agent identity token for authentication
	EnvIdentityToken = "AGENT_IDENTITY_TOKEN"
	// OUTPUT_FORMAT is the output format override
	EnvOutputFormat = "AGENT_OUTPUT_FORMAT"
	// DRY_RUN is the dry-run mode override
	EnvDryRun = "AGENT_DRY_RUN"
	// EMIT_SPEC is the emit spec mode
	EnvEmitSpec = "AGENT_EMIT_SPEC"
)

// Severity levels for errors
type Severity string

const (
	SeverityDebug    Severity = "debug"
	SeverityInfo    Severity = "info"
	SeverityWarning Severity = "warning"
	SeverityError   Severity = "error"
	SeverityCritical Severity = "critical"
)

// Error codes for ANEL operations
type ErrorCode string

const (
	// Generic errors
	ErrorCodeUnknown          ErrorCode = "UNKNOWN"
	ErrorCodeInvalidInput     ErrorCode = "INVALID_INPUT"
	ErrorCodeNotFound         ErrorCode = "NOT_FOUND"
	ErrorCodePermissionDenied ErrorCode = "PERMISSION_DENIED"

	// Search-related errors
	ErrorCodeSearchFailed    ErrorCode = "SEARCH_FAILED"
	ErrorCodeIndexNotReady   ErrorCode = "INDEX_NOT_READY"
	ErrorCodeQueryParseError ErrorCode = "QUERY_PARSE_ERROR"

	// Collection errors
	ErrorCodeCollectionNotFound  ErrorCode = "COLLECTION_NOT_FOUND"
	ErrorCodeCollectionExists    ErrorCode = "COLLECTION_EXISTS"
	ErrorCodeCollectionCorrupted ErrorCode = "COLLECTION_CORRUPTED"

	// Embedding errors
	ErrorCodeEmbeddingFailed ErrorCode = "EMBEDDING_FAILED"
	ErrorCodeModelNotFound   ErrorCode = "MODEL_NOT_FOUND"
	ErrorCodeModelLoadFailed ErrorCode = "MODEL_LOAD_FAILED"

	// Storage errors
	ErrorCodeStorageError       ErrorCode = "STORAGE_ERROR"
	ErrorCodeBackendUnavailable ErrorCode = "BACKEND_UNAVAILABLE"

	// Configuration errors
	ErrorCodeConfigError      ErrorCode = "CONFIG_ERROR"
	ErrorCodeEnvironmentError ErrorCode = "ENVIRONMENT_ERROR"
)

// ErrorCodeToStatus converts error code to HTTP-style status
func (e ErrorCode) ToStatus() int {
	switch e {
	case ErrorCodeUnknown:
		return 500
	case ErrorCodeInvalidInput:
		return 400
	case ErrorCodeNotFound:
		return 404
	case ErrorCodePermissionDenied:
		return 403
	case ErrorCodeSearchFailed:
		return 500
	case ErrorCodeIndexNotReady:
		return 503
	case ErrorCodeQueryParseError:
		return 400
	case ErrorCodeCollectionNotFound:
		return 404
	case ErrorCodeCollectionExists:
		return 409
	case ErrorCodeCollectionCorrupted:
		return 500
	case ErrorCodeEmbeddingFailed:
		return 500
	case ErrorCodeModelNotFound:
		return 404
	case ErrorCodeModelLoadFailed:
		return 500
	case ErrorCodeStorageError:
		return 500
	case ErrorCodeBackendUnavailable:
		return 503
	case ErrorCodeConfigError:
		return 500
	case ErrorCodeEnvironmentError:
		return 500
	default:
		return 500
	}
}

// RecoveryHint provides hints for error resolution
type RecoveryHint struct {
	Code    string `json:"code"`
	Message string `json:"message"`
	Action  string `json:"action,omitempty"`
}

// NewRecoveryHint creates a new recovery hint
func NewRecoveryHint(code, message string) RecoveryHint {
	return RecoveryHint{
		Code:    code,
		Message: message,
	}
}

// WithAction adds an action to the recovery hint
func (h RecoveryHint) WithAction(action string) RecoveryHint {
	h.Action = action
	return h
}

// AnelError is the ANID Error type (Agent-Native ID Error)
// Implements RFC 7807 Problem Details with ANEL extensions
type AnelError struct {
	ErrorCode     ErrorCode        `json:"error_code"`
	Status        int              `json:"status"`
	Title         string           `json:"title"`
	Message       string           `json:"message"`
	Severity      Severity         `json:"severity"`
	RecoveryHints []RecoveryHint   `json:"recovery_hints"`
	TraceID       *string          `json:"trace_id,omitempty"`
	Metadata      map[string]any   `json:"-"`
}

// NewAnelError creates a new ANEL error
func NewAnelError(errorCode ErrorCode, title, message string) *AnelError {
	return &AnelError{
		ErrorCode:     errorCode,
		Status:        errorCode.ToStatus(),
		Title:         title,
		Message:       message,
		Severity:      SeverityError,
		RecoveryHints: []RecoveryHint{},
		TraceID:       nil,
		Metadata:      map[string]any{},
	}
}

// WithHint adds a recovery hint
func (e *AnelError) WithHint(hint RecoveryHint) *AnelError {
	e.RecoveryHints = append(e.RecoveryHints, hint)
	return e
}

// WithTraceID adds trace ID
func (e *AnelError) WithTraceID(traceID string) *AnelError {
	e.TraceID = &traceID
	return e
}

// WithMetadata adds metadata
func (e *AnelError) WithMetadata(key string, value any) *AnelError {
	e.Metadata[key] = value
	return e
}

// ToNDJSON serializes to NDJSON line
func (e *AnelError) ToNDJSON() string {
	data, err := json.Marshal(e)
	if err != nil {
		return "{}"
	}
	return string(data)
}

// EmitStderr prints to stderr in NDJSON format
func (e *AnelError) EmitStderr() {
	fmt.Fprintln(os.Stderr, e.ToNDJSON())
}

// Error implements error interface
func (e *AnelError) Error() string {
	return fmt.Sprintf("[%s] %s: %s", e.ErrorCode, e.Title, e.Message)
}

// TraceContext for request correlation
type TraceContext struct {
	TraceID       *string        `json:"trace_id,omitempty"`
	IdentityToken *string        `json:"identity_token,omitempty"`
	Tags          map[string]string `json:"tags,omitempty"`
}

// NewTraceContext creates a new trace context from environment variables
func NewTraceContext() TraceContext {
	traceID := os.Getenv(EnvTraceID)
	identityToken := os.Getenv(EnvIdentityToken)

	ctx := TraceContext{
		Tags: map[string]string{},
	}

	if traceID != "" {
		ctx.TraceID = &traceID
	}
	if identityToken != "" {
		ctx.IdentityToken = &identityToken
	}

	return ctx
}

// GetOrGenerateTraceID returns existing trace ID or generates a new one
func (t *TraceContext) GetOrGenerateTraceID() string {
	if t.TraceID != nil && *t.TraceID != "" {
		return *t.TraceID
	}
	id := fmt.Sprintf("qmd-%d", time.Now().UnixNano())
	return id
}

// AnelSpec represents the ANEL specification for a command
type AnelSpec struct {
	Version      string          `json:"version"`
	Command      string          `json:"command"`
	InputSchema  json.RawMessage `json:"input_schema"`
	OutputSchema json.RawMessage `json:"output_schema"`
	ErrorCodes   []ErrorCode    `json:"error_codes"`
}

// ToJSON converts spec to JSON string
func (s *AnelSpec) ToJSON() string {
	data, err := json.MarshalIndent(s, "", "  ")
	if err != nil {
		return "{}"
	}
	return string(data)
}

// NDJSONRecord is a wrapper for NDJSON output
type NDJSONRecord struct {
	Type    string          `json:"type"` // "result", "error", "spec", "metadata"
	Seq     uint64          `json:"seq"`
	Payload json.RawMessage `json:"payload"`
}

// NewNDJSONRecord creates a new NDJSON record
func NewNDJSONRecord(recordType string, seq uint64, payload any) *NDJSONRecord {
	data, _ := json.Marshal(payload)
	return &NDJSONRecord{
		Type:    recordType,
		Seq:     seq,
		Payload: data,
	}
}

// ToNDJSON serializes to NDJSON line
func (r *NDJSONRecord) ToNDJSON() string {
	data, err := json.Marshal(r)
	if err != nil {
		return "{}"
	}
	return string(data)
}

// Emit prints to stdout in NDJSON format
func (r *NDJSONRecord) Emit() {
	fmt.Println(r.ToNDJSON())
}

// AnelResult represents the result of an ANEL command
type AnelResult struct {
	Success  bool            `json:"success"`
	Data     json.RawMessage `json:"data,omitempty"`
	Error    *AnelError     `json:"error,omitempty"`
	TraceID  *string        `json:"trace_id,omitempty"`
}

// NewSuccessResult creates a success result
func NewSuccessResult(data any) *AnelResult {
	jsonData, _ := json.Marshal(data)
	return &AnelResult{
		Success:  true,
		Data:     jsonData,
		Error:    nil,
		TraceID:  nil,
	}
}

// NewErrorResult creates an error result
func NewErrorResult(err *AnelError) *AnelResult {
	return &AnelResult{
		Success:  false,
		Data:     nil,
		Error:    err,
		TraceID:  err.TraceID,
	}
}

// WithTraceID adds trace ID
func (r *AnelResult) WithTraceID(traceID string) *AnelResult {
	r.TraceID = &traceID
	return r
}

// ToNDJSON serializes to NDJSON
func (r *AnelResult) ToNDJSON() string {
	data, err := json.Marshal(r)
	if err != nil {
		return "{}"
	}
	return string(data)
}
