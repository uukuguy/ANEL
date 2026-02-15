package anel

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"testing"
)

// --- ErrorCode ---

func TestErrorCodeToStatus(t *testing.T) {
	cases := []struct {
		code   ErrorCode
		status int
	}{
		{ErrorCodeUnknown, 500},
		{ErrorCodeInvalidInput, 400},
		{ErrorCodeNotFound, 404},
		{ErrorCodePermissionDenied, 403},
		{ErrorCodeSearchFailed, 500},
		{ErrorCodeIndexNotReady, 503},
		{ErrorCodeQueryParseError, 400},
		{ErrorCodeCollectionNotFound, 404},
		{ErrorCodeCollectionExists, 409},
		{ErrorCodeCollectionCorrupted, 500},
		{ErrorCodeEmbeddingFailed, 500},
		{ErrorCodeModelNotFound, 404},
		{ErrorCodeModelLoadFailed, 500},
		{ErrorCodeStorageError, 500},
		{ErrorCodeBackendUnavailable, 503},
		{ErrorCodeConfigError, 500},
		{ErrorCodeEnvironmentError, 500},
	}

	for _, tc := range cases {
		t.Run(string(tc.code), func(t *testing.T) {
			if got := tc.code.ToStatus(); got != tc.status {
				t.Errorf("ErrorCode(%s).ToStatus() = %d, want %d", tc.code, got, tc.status)
			}
		})
	}
}

func TestErrorCodeUnknownDefault(t *testing.T) {
	code := ErrorCode("NONEXISTENT_CODE")
	if got := code.ToStatus(); got != 500 {
		t.Errorf("unknown ErrorCode.ToStatus() = %d, want 500", got)
	}
}

// --- AnelError ---

func TestNewAnelError(t *testing.T) {
	err := NewAnelError(ErrorCodeInvalidInput, "Bad Request", "missing query parameter")

	if err.ErrorCode != ErrorCodeInvalidInput {
		t.Errorf("ErrorCode = %s, want INVALID_INPUT", err.ErrorCode)
	}
	if err.Status != 400 {
		t.Errorf("Status = %d, want 400", err.Status)
	}
	if err.Title != "Bad Request" {
		t.Errorf("Title = %s, want Bad Request", err.Title)
	}
	if err.Message != "missing query parameter" {
		t.Errorf("Message = %s, want missing query parameter", err.Message)
	}
	if err.Severity != SeverityError {
		t.Errorf("Severity = %s, want error", err.Severity)
	}
	if len(err.RecoveryHints) != 0 {
		t.Errorf("RecoveryHints should be empty, got %d", len(err.RecoveryHints))
	}
	if err.TraceID != nil {
		t.Error("TraceID should be nil")
	}
}

func TestAnelErrorWithHint(t *testing.T) {
	err := NewAnelError(ErrorCodeCollectionNotFound, "Not Found", "collection 'test' not found").
		WithHint(NewRecoveryHint("LIST_COLLECTIONS", "Run 'qmd collection list'"))

	if len(err.RecoveryHints) != 1 {
		t.Fatalf("RecoveryHints count = %d, want 1", len(err.RecoveryHints))
	}
	if err.RecoveryHints[0].Code != "LIST_COLLECTIONS" {
		t.Errorf("Hint code = %s, want LIST_COLLECTIONS", err.RecoveryHints[0].Code)
	}
}

func TestAnelErrorWithMultipleHints(t *testing.T) {
	err := NewAnelError(ErrorCodeBackendUnavailable, "Unavailable", "timeout").
		WithHint(NewRecoveryHint("RETRY", "Wait 5 seconds")).
		WithHint(NewRecoveryHint("INCREASE_TIMEOUT", "Use --timeout 15"))

	if len(err.RecoveryHints) != 2 {
		t.Errorf("RecoveryHints count = %d, want 2", len(err.RecoveryHints))
	}
}

func TestAnelErrorWithTraceID(t *testing.T) {
	err := NewAnelError(ErrorCodeSearchFailed, "Search Failed", "index corrupted").
		WithTraceID("trace-abc-123")

	if err.TraceID == nil || *err.TraceID != "trace-abc-123" {
		t.Errorf("TraceID = %v, want trace-abc-123", err.TraceID)
	}
}

func TestAnelErrorWithMetadata(t *testing.T) {
	err := NewAnelError(ErrorCodeInvalidInput, "Bad Input", "bad query").
		WithMetadata("field", "query").
		WithMetadata("value", "")

	if err.Metadata["field"] != "query" {
		t.Errorf("Metadata[field] = %v, want query", err.Metadata["field"])
	}
}

func TestAnelErrorInterface(t *testing.T) {
	var e error = NewAnelError(ErrorCodeNotFound, "Not Found", "file missing")
	msg := e.Error()

	if !strings.Contains(msg, "NOT_FOUND") {
		t.Errorf("Error() should contain error code, got: %s", msg)
	}
	if !strings.Contains(msg, "file missing") {
		t.Errorf("Error() should contain message, got: %s", msg)
	}
}

func TestAnelErrorToNDJSON(t *testing.T) {
	err := NewAnelError(ErrorCodeInvalidInput, "Bad Request", "missing query").
		WithHint(NewRecoveryHint("FIX", "add --query flag"))

	ndjson := err.ToNDJSON()

	var parsed map[string]interface{}
	if e := json.Unmarshal([]byte(ndjson), &parsed); e != nil {
		t.Fatalf("ToNDJSON() produced invalid JSON: %v", e)
	}

	if parsed["error_code"] != "INVALID_INPUT" {
		t.Errorf("error_code = %v, want INVALID_INPUT", parsed["error_code"])
	}
	if parsed["status"].(float64) != 400 {
		t.Errorf("status = %v, want 400", parsed["status"])
	}

	hints := parsed["recovery_hints"].([]interface{})
	if len(hints) != 1 {
		t.Errorf("recovery_hints count = %d, want 1", len(hints))
	}
}

// --- RecoveryHint ---

func TestRecoveryHintWithAction(t *testing.T) {
	hint := NewRecoveryHint("RETRY", "Wait and retry").WithAction("sleep 5 && qmd search test")

	if hint.Action != "sleep 5 && qmd search test" {
		t.Errorf("Action = %s, want sleep command", hint.Action)
	}
}

func TestRecoveryHintJSON(t *testing.T) {
	hint := NewRecoveryHint("FIX_URL", "URL must start with http://").WithAction("prepend https://")

	data, err := json.Marshal(hint)
	if err != nil {
		t.Fatalf("Marshal failed: %v", err)
	}

	var parsed map[string]string
	json.Unmarshal(data, &parsed)

	if parsed["code"] != "FIX_URL" {
		t.Errorf("code = %s, want FIX_URL", parsed["code"])
	}
	if parsed["action"] != "prepend https://" {
		t.Errorf("action = %s, want prepend https://", parsed["action"])
	}
}

// --- TraceContext ---

func TestNewTraceContextEmpty(t *testing.T) {
	os.Unsetenv(EnvTraceID)
	os.Unsetenv(EnvIdentityToken)

	ctx := NewTraceContext()

	if ctx.TraceID != nil {
		t.Errorf("TraceID should be nil when env not set, got %v", ctx.TraceID)
	}
	if ctx.IdentityToken != nil {
		t.Errorf("IdentityToken should be nil when env not set, got %v", ctx.IdentityToken)
	}
}

func TestNewTraceContextFromEnv(t *testing.T) {
	os.Setenv(EnvTraceID, "test-trace-id")
	os.Setenv(EnvIdentityToken, "test-token")
	defer os.Unsetenv(EnvTraceID)
	defer os.Unsetenv(EnvIdentityToken)

	ctx := NewTraceContext()

	if ctx.TraceID == nil || *ctx.TraceID != "test-trace-id" {
		t.Errorf("TraceID = %v, want test-trace-id", ctx.TraceID)
	}
	if ctx.IdentityToken == nil || *ctx.IdentityToken != "test-token" {
		t.Errorf("IdentityToken = %v, want test-token", ctx.IdentityToken)
	}
}

func TestGetOrGenerateTraceID_Existing(t *testing.T) {
	traceID := "existing-trace"
	ctx := TraceContext{TraceID: &traceID}

	got := ctx.GetOrGenerateTraceID()
	if got != "existing-trace" {
		t.Errorf("GetOrGenerateTraceID() = %s, want existing-trace", got)
	}
}

func TestGetOrGenerateTraceID_Generated(t *testing.T) {
	ctx := TraceContext{}

	got := ctx.GetOrGenerateTraceID()
	if !strings.HasPrefix(got, "qmd-") {
		t.Errorf("GetOrGenerateTraceID() = %s, should start with qmd-", got)
	}
}

// --- FromError ---

func TestFromErrorNil(t *testing.T) {
	if got := FromError(nil); got != nil {
		t.Errorf("FromError(nil) should return nil, got %v", got)
	}
}

func TestFromErrorMapping(t *testing.T) {
	cases := []struct {
		msg  string
		code ErrorCode
	}{
		{"file not found", ErrorCodeNotFound},
		{"permission denied", ErrorCodePermissionDenied},
		{"invalid argument", ErrorCodeInvalidInput},
		{"parse error in query", ErrorCodeQueryParseError},
		{"collection missing", ErrorCodeCollectionNotFound},
		{"embedding generation failed", ErrorCodeEmbeddingFailed},
		{"storage backend error", ErrorCodeStorageError},
		{"database connection lost", ErrorCodeStorageError},
		{"config file missing", ErrorCodeConfigError},
		{"something random happened", ErrorCodeUnknown},
	}

	for _, tc := range cases {
		t.Run(tc.msg, func(t *testing.T) {
			err := FromError(fmt.Errorf(tc.msg))
			if err.ErrorCode != tc.code {
				t.Errorf("FromError(%q).ErrorCode = %s, want %s", tc.msg, err.ErrorCode, tc.code)
			}
		})
	}
}

func TestFromErrorWithContext(t *testing.T) {
	traceID := "ctx-trace-123"
	ctx := &TraceContext{TraceID: &traceID}

	err := FromErrorWithContext(fmt.Errorf("not found"), ctx)
	if err.TraceID == nil || *err.TraceID != "ctx-trace-123" {
		t.Errorf("TraceID = %v, want ctx-trace-123", err.TraceID)
	}
}

// --- IsXxx helpers ---

func TestIsNotFound(t *testing.T) {
	anelErr := NewAnelError(ErrorCodeNotFound, "Not Found", "missing")
	if !IsNotFound(anelErr) {
		t.Error("IsNotFound should return true for NOT_FOUND AnelError")
	}

	plainErr := fmt.Errorf("resource not found")
	if !IsNotFound(plainErr) {
		t.Error("IsNotFound should return true for error containing 'not found'")
	}

	otherErr := fmt.Errorf("something else")
	if IsNotFound(otherErr) {
		t.Error("IsNotFound should return false for unrelated error")
	}
}

func TestIsInvalidInput(t *testing.T) {
	anelErr := NewAnelError(ErrorCodeInvalidInput, "Bad", "bad")
	if !IsInvalidInput(anelErr) {
		t.Error("IsInvalidInput should return true for INVALID_INPUT AnelError")
	}
}

func TestIsPermissionDenied(t *testing.T) {
	anelErr := NewAnelError(ErrorCodePermissionDenied, "Denied", "no access")
	if !IsPermissionDenied(anelErr) {
		t.Error("IsPermissionDenied should return true for PERMISSION_DENIED AnelError")
	}
}

// --- AnelSpec ---

func TestAnelSpecToJSON(t *testing.T) {
	spec := &AnelSpec{
		Version:      "1.0",
		Command:      "test",
		InputSchema:  json.RawMessage(`{"type":"object"}`),
		OutputSchema: json.RawMessage(`{"type":"object"}`),
		ErrorCodes:   []ErrorCode{ErrorCodeNotFound},
	}

	jsonStr := spec.ToJSON()

	var parsed map[string]interface{}
	if err := json.Unmarshal([]byte(jsonStr), &parsed); err != nil {
		t.Fatalf("ToJSON() produced invalid JSON: %v", err)
	}

	if parsed["version"] != "1.0" {
		t.Errorf("version = %v, want 1.0", parsed["version"])
	}
	if parsed["command"] != "test" {
		t.Errorf("command = %v, want test", parsed["command"])
	}
}

// --- NDJSONRecord ---

func TestNewNDJSONRecord(t *testing.T) {
	payload := map[string]string{"path": "test.md", "score": "0.95"}
	record := NewNDJSONRecord("result", 1, payload)

	if record.Type != "result" {
		t.Errorf("Type = %s, want result", record.Type)
	}
	if record.Seq != 1 {
		t.Errorf("Seq = %d, want 1", record.Seq)
	}

	ndjson := record.ToNDJSON()
	var parsed map[string]interface{}
	if err := json.Unmarshal([]byte(ndjson), &parsed); err != nil {
		t.Fatalf("ToNDJSON() produced invalid JSON: %v", err)
	}
}

// --- AnelResult ---

func TestNewSuccessResult(t *testing.T) {
	data := map[string]int{"count": 42}
	result := NewSuccessResult(data)

	if !result.Success {
		t.Error("Success should be true")
	}
	if result.Error != nil {
		t.Error("Error should be nil for success result")
	}

	ndjson := result.ToNDJSON()
	var parsed map[string]interface{}
	json.Unmarshal([]byte(ndjson), &parsed)
	if parsed["success"] != true {
		t.Error("JSON success should be true")
	}
}

func TestNewErrorResult(t *testing.T) {
	anelErr := NewAnelError(ErrorCodeSearchFailed, "Failed", "index error").
		WithTraceID("err-trace")
	result := NewErrorResult(anelErr)

	if result.Success {
		t.Error("Success should be false")
	}
	if result.Error == nil {
		t.Error("Error should not be nil")
	}
	if result.TraceID == nil || *result.TraceID != "err-trace" {
		t.Errorf("TraceID = %v, want err-trace", result.TraceID)
	}
}

func TestAnelResultWithTraceID(t *testing.T) {
	result := NewSuccessResult(nil).WithTraceID("my-trace")
	if result.TraceID == nil || *result.TraceID != "my-trace" {
		t.Errorf("TraceID = %v, want my-trace", result.TraceID)
	}
}

// --- Version ---

func TestVersion(t *testing.T) {
	if Version != "1.0" {
		t.Errorf("Version = %s, want 1.0", Version)
	}
}

// --- Environment variable names ---

func TestEnvVarNames(t *testing.T) {
	if EnvTraceID != "AGENT_TRACE_ID" {
		t.Errorf("EnvTraceID = %s, want AGENT_TRACE_ID", EnvTraceID)
	}
	if EnvIdentityToken != "AGENT_IDENTITY_TOKEN" {
		t.Errorf("EnvIdentityToken = %s, want AGENT_IDENTITY_TOKEN", EnvIdentityToken)
	}
	if EnvOutputFormat != "AGENT_OUTPUT_FORMAT" {
		t.Errorf("EnvOutputFormat = %s, want AGENT_OUTPUT_FORMAT", EnvOutputFormat)
	}
	if EnvDryRun != "AGENT_DRY_RUN" {
		t.Errorf("EnvDryRun = %s, want AGENT_DRY_RUN", EnvDryRun)
	}
	if EnvEmitSpec != "AGENT_EMIT_SPEC" {
		t.Errorf("EnvEmitSpec = %s, want AGENT_EMIT_SPEC", EnvEmitSpec)
	}
}
