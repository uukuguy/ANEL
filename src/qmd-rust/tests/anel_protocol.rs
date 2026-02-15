//! ANEL protocol tests — aligned with Go anel_test.go and Python test_anel.py
//!
//! Covers: ErrorCode, Severity, RecoveryHint, AnelError, TraceContext,
//!         AnelSpec, NdjsonRecord, AnelResult, From<anyhow::Error>, constants.

use qmd_rust::anel::*;
use std::collections::HashMap;

// ============================================================
// ErrorCode
// ============================================================

#[test]
fn error_code_to_status_all_17_codes() {
    let cases: Vec<(AnelErrorCode, u16)> = vec![
        (AnelErrorCode::Unknown, 500),
        (AnelErrorCode::InvalidInput, 400),
        (AnelErrorCode::NotFound, 404),
        (AnelErrorCode::PermissionDenied, 403),
        (AnelErrorCode::SearchFailed, 500),
        (AnelErrorCode::IndexNotReady, 503),
        (AnelErrorCode::QueryParseError, 400),
        (AnelErrorCode::CollectionNotFound, 404),
        (AnelErrorCode::CollectionExists, 409),
        (AnelErrorCode::CollectionCorrupted, 500),
        (AnelErrorCode::EmbeddingFailed, 500),
        (AnelErrorCode::ModelNotFound, 404),
        (AnelErrorCode::ModelLoadFailed, 500),
        (AnelErrorCode::StorageError, 500),
        (AnelErrorCode::BackendUnavailable, 503),
        (AnelErrorCode::ConfigError, 500),
        (AnelErrorCode::EnvironmentError, 500),
    ];
    for (code, expected) in cases {
        assert_eq!(code.to_status(), expected, "{:?} -> {}", code, expected);
    }
}

#[test]
fn error_code_default_is_unknown() {
    assert_eq!(AnelErrorCode::default(), AnelErrorCode::Unknown);
}

#[test]
fn error_code_serde_screaming_snake() {
    let json = serde_json::to_string(&AnelErrorCode::NotFound).unwrap();
    assert_eq!(json, "\"NOT_FOUND\"");

    let json2 = serde_json::to_string(&AnelErrorCode::SearchFailed).unwrap();
    assert_eq!(json2, "\"SEARCH_FAILED\"");
}

#[test]
fn error_code_deserialize_from_string() {
    let code: AnelErrorCode = serde_json::from_str("\"INVALID_INPUT\"").unwrap();
    assert_eq!(code, AnelErrorCode::InvalidInput);
}

#[test]
fn error_code_debug_format() {
    let s = format!("{:?}", AnelErrorCode::BackendUnavailable);
    assert!(s.contains("BackendUnavailable"));
}

#[test]
fn error_code_clone_eq() {
    let a = AnelErrorCode::StorageError;
    let b = a;
    assert_eq!(a, b);
}

// ============================================================
// Severity
// ============================================================

#[test]
fn severity_all_levels() {
    let cases = vec![
        (Severity::Debug, "debug"),
        (Severity::Info, "info"),
        (Severity::Warning, "warning"),
        (Severity::Error, "error"),
        (Severity::Critical, "critical"),
    ];
    for (sev, expected) in cases {
        let json = serde_json::to_string(&sev).unwrap();
        assert_eq!(json, format!("\"{}\"", expected));
    }
}

#[test]
fn severity_default_is_error() {
    assert_eq!(Severity::default(), Severity::Error);
}

// ============================================================
// RecoveryHint
// ============================================================

#[test]
fn recovery_hint_basic() {
    let hint = RecoveryHint::new("RETRY", "Wait and retry");
    assert_eq!(hint.code, "RETRY");
    assert_eq!(hint.message, "Wait and retry");
    assert!(hint.action.is_none());
}

#[test]
fn recovery_hint_with_action() {
    let hint = RecoveryHint::new("FIX", "Fix it").with_action("run fix cmd");
    assert_eq!(hint.action.as_deref(), Some("run fix cmd"));
}

#[test]
fn recovery_hint_json_serialization() {
    let hint = RecoveryHint::new("A", "B").with_action("C");
    let json = serde_json::to_value(&hint).unwrap();
    assert_eq!(json["code"], "A");
    assert_eq!(json["message"], "B");
    assert_eq!(json["action"], "C");
}

#[test]
fn recovery_hint_json_omits_none_action() {
    let hint = RecoveryHint::new("X", "Y");
    let json = serde_json::to_value(&hint).unwrap();
    assert_eq!(json["code"], "X");
    // action is None -> serialized as null
    assert!(json["action"].is_null());
}

// ============================================================
// AnelError
// ============================================================

#[test]
fn anel_error_new() {
    let err = AnelError::new(AnelErrorCode::InvalidInput, "Bad Request", "missing query");
    assert_eq!(err.error_code, AnelErrorCode::InvalidInput);
    assert_eq!(err.status, 400);
    assert_eq!(err.title, "Bad Request");
    assert_eq!(err.message, "missing query");
    assert_eq!(err.severity, Severity::Error);
    assert!(err.recovery_hints.is_empty());
    assert!(err.trace_id.is_none());
    assert!(err.metadata.is_empty());
}

#[test]
fn anel_error_with_hint() {
    let err = AnelError::new(AnelErrorCode::NotFound, "Not Found", "missing")
        .with_hint(RecoveryHint::new("LIST", "List items"));
    assert_eq!(err.recovery_hints.len(), 1);
    assert_eq!(err.recovery_hints[0].code, "LIST");
}

#[test]
fn anel_error_with_multiple_hints() {
    let err = AnelError::new(AnelErrorCode::BackendUnavailable, "Down", "timeout")
        .with_hint(RecoveryHint::new("RETRY", "Retry"))
        .with_hint(RecoveryHint::new("TIMEOUT", "Increase timeout"));
    assert_eq!(err.recovery_hints.len(), 2);
}

#[test]
fn anel_error_with_trace_id() {
    let err = AnelError::new(AnelErrorCode::SearchFailed, "Failed", "err")
        .with_trace_id("t-123");
    assert_eq!(err.trace_id.as_deref(), Some("t-123"));
}

#[test]
fn anel_error_with_metadata() {
    let err = AnelError::new(AnelErrorCode::InvalidInput, "Bad", "bad")
        .with_metadata("field", "query")
        .with_metadata("value", "");
    assert_eq!(err.metadata["field"], "query");
    assert_eq!(err.metadata["value"], "");
}

#[test]
fn anel_error_display_contains_code_and_message() {
    let err = AnelError::new(AnelErrorCode::NotFound, "Not Found", "file missing");
    let s = err.to_string();
    assert!(s.contains("NotFound"), "Display should contain error code: {}", s);
    assert!(s.contains("file missing"), "Display should contain message: {}", s);
}

#[test]
fn anel_error_to_ndjson_valid_json() {
    let err = AnelError::new(AnelErrorCode::InvalidInput, "Bad", "missing")
        .with_hint(RecoveryHint::new("FIX", "add flag"));
    let ndjson = err.to_ndjson();
    let parsed: serde_json::Value = serde_json::from_str(&ndjson).unwrap();
    assert_eq!(parsed["error_code"], "INVALID_INPUT");
    assert_eq!(parsed["status"], 400);
    let hints = parsed["recovery_hints"].as_array().unwrap();
    assert_eq!(hints.len(), 1);
}

#[test]
fn anel_error_status_auto_set_for_all_codes() {
    let codes = vec![
        AnelErrorCode::Unknown,
        AnelErrorCode::InvalidInput,
        AnelErrorCode::NotFound,
        AnelErrorCode::PermissionDenied,
        AnelErrorCode::SearchFailed,
        AnelErrorCode::IndexNotReady,
        AnelErrorCode::QueryParseError,
        AnelErrorCode::CollectionNotFound,
        AnelErrorCode::CollectionExists,
        AnelErrorCode::CollectionCorrupted,
        AnelErrorCode::EmbeddingFailed,
        AnelErrorCode::ModelNotFound,
        AnelErrorCode::ModelLoadFailed,
        AnelErrorCode::StorageError,
        AnelErrorCode::BackendUnavailable,
        AnelErrorCode::ConfigError,
        AnelErrorCode::EnvironmentError,
    ];
    for code in codes {
        let err = AnelError::new(code, "T", "M");
        assert_eq!(err.status, code.to_status(), "{:?}", code);
    }
}

#[test]
fn anel_error_implements_std_error() {
    let err = AnelError::new(AnelErrorCode::NotFound, "Not Found", "missing");
    let _: &dyn std::error::Error = &err;
}

// ============================================================
// From<anyhow::Error> for AnelError
// ============================================================

#[test]
fn from_error_not_found() {
    let err: AnelError = anyhow::anyhow!("file not found").into();
    assert_eq!(err.error_code, AnelErrorCode::NotFound);
}

#[test]
fn from_error_permission_denied() {
    let err: AnelError = anyhow::anyhow!("permission denied").into();
    assert_eq!(err.error_code, AnelErrorCode::PermissionDenied);
}

#[test]
fn from_error_invalid_input() {
    let err: AnelError = anyhow::anyhow!("invalid argument").into();
    assert_eq!(err.error_code, AnelErrorCode::InvalidInput);
}

#[test]
fn from_error_query_parse() {
    let err: AnelError = anyhow::anyhow!("parse error in query").into();
    assert_eq!(err.error_code, AnelErrorCode::QueryParseError);
}

#[test]
fn from_error_collection() {
    let err: AnelError = anyhow::anyhow!("collection missing").into();
    assert_eq!(err.error_code, AnelErrorCode::CollectionNotFound);
}

#[test]
fn from_error_embedding() {
    let err: AnelError = anyhow::anyhow!("embedding generation failed").into();
    assert_eq!(err.error_code, AnelErrorCode::EmbeddingFailed);
}

#[test]
fn from_error_storage() {
    let err: AnelError = anyhow::anyhow!("storage backend error").into();
    assert_eq!(err.error_code, AnelErrorCode::StorageError);
}

#[test]
fn from_error_database() {
    let err: AnelError = anyhow::anyhow!("database connection lost").into();
    assert_eq!(err.error_code, AnelErrorCode::StorageError);
}

#[test]
fn from_error_config() {
    let err: AnelError = anyhow::anyhow!("config file missing").into();
    assert_eq!(err.error_code, AnelErrorCode::ConfigError);
}

#[test]
fn from_error_unknown_fallback() {
    let err: AnelError = anyhow::anyhow!("something random happened").into();
    assert_eq!(err.error_code, AnelErrorCode::Unknown);
}

// ============================================================
// TraceContext
// ============================================================

#[test]
fn trace_context_from_env_empty() {
    // Clear env vars to ensure clean state
    std::env::remove_var(env::TRACE_ID);
    std::env::remove_var(env::IDENTITY_TOKEN);
    let ctx = TraceContext::from_env();
    assert!(ctx.trace_id.is_none());
    assert!(ctx.identity_token.is_none());
}

#[test]
fn trace_context_from_env_set() {
    std::env::set_var(env::TRACE_ID, "test-trace-id");
    std::env::set_var(env::IDENTITY_TOKEN, "test-token");
    let ctx = TraceContext::from_env();
    assert_eq!(ctx.trace_id.as_deref(), Some("test-trace-id"));
    assert_eq!(ctx.identity_token.as_deref(), Some("test-token"));
    std::env::remove_var(env::TRACE_ID);
    std::env::remove_var(env::IDENTITY_TOKEN);
}

#[test]
fn trace_context_get_or_generate_existing() {
    let ctx = TraceContext {
        trace_id: Some("existing-trace".to_string()),
        ..Default::default()
    };
    assert_eq!(ctx.get_or_generate_trace_id(), "existing-trace");
}

#[test]
fn trace_context_get_or_generate_new() {
    let ctx = TraceContext::default();
    let tid = ctx.get_or_generate_trace_id();
    assert!(tid.starts_with("qmd-"), "generated trace_id should start with qmd-: {}", tid);
}

#[test]
fn trace_context_tags_default_empty() {
    let ctx = TraceContext::default();
    assert!(ctx.tags.is_empty());
}

#[test]
fn trace_context_default() {
    let ctx = TraceContext::default();
    assert!(ctx.trace_id.is_none());
    assert!(ctx.identity_token.is_none());
    assert!(ctx.tags.is_empty());
}

// ============================================================
// NdjsonRecord
// ============================================================

#[test]
fn ndjson_record_basic() {
    let record = NdjsonRecord::new("result", 1, serde_json::json!({"path": "test.md"}));
    assert_eq!(record.record_type, "result");
    assert_eq!(record.seq, 1);
}

#[test]
fn ndjson_record_to_ndjson_valid_json() {
    let record = NdjsonRecord::new("error", 0, serde_json::json!({"msg": "fail"}));
    let ndjson = record.to_ndjson();
    let parsed: serde_json::Value = serde_json::from_str(&ndjson).unwrap();
    assert_eq!(parsed["type"], "error");
    assert_eq!(parsed["seq"], 0);
}

#[test]
fn ndjson_record_with_complex_payload() {
    let payload = serde_json::json!({
        "path": "test.md",
        "score": 0.95,
        "lines": [1, 2, 3]
    });
    let record = NdjsonRecord::new("result", 42, payload);
    let ndjson = record.to_ndjson();
    let parsed: serde_json::Value = serde_json::from_str(&ndjson).unwrap();
    assert_eq!(parsed["seq"], 42);
    assert_eq!(parsed["payload"]["score"], 0.95);
}

// ============================================================
// AnelResult
// ============================================================

#[test]
fn anel_result_success() {
    let result = AnelResult::success(serde_json::json!({"count": 42}));
    assert!(result.success);
    assert_eq!(result.data["count"], 42);
    assert!(result.error.is_none());
    assert!(result.trace_id.is_none());
}

#[test]
fn anel_result_error() {
    let err = AnelError::new(AnelErrorCode::SearchFailed, "Failed", "err")
        .with_trace_id("err-trace");
    let result = AnelResult::error(err);
    assert!(!result.success);
    assert!(result.error.is_some());
    assert_eq!(result.trace_id.as_deref(), Some("err-trace"));
}

#[test]
fn anel_result_with_trace_id() {
    let result = AnelResult::success(serde_json::json!({})).with_trace_id("my-trace");
    assert_eq!(result.trace_id.as_deref(), Some("my-trace"));
}

#[test]
fn anel_result_to_ndjson() {
    let result = AnelResult::success(serde_json::json!({"x": 1}));
    let ndjson = result.to_ndjson();
    let parsed: serde_json::Value = serde_json::from_str(&ndjson).unwrap();
    assert_eq!(parsed["success"], true);
}

#[test]
fn anel_result_error_to_ndjson() {
    let err = AnelError::new(AnelErrorCode::NotFound, "Not Found", "missing");
    let result = AnelResult::error(err);
    let parsed: serde_json::Value = serde_json::from_str(&result.to_ndjson()).unwrap();
    assert_eq!(parsed["success"], false);
    assert!(parsed["error"].is_object());
}

// ============================================================
// Constants / env vars
// ============================================================

#[test]
fn version_is_1_0() {
    assert_eq!(ANEL_VERSION, "1.0");
}

#[test]
fn env_var_names() {
    assert_eq!(env::TRACE_ID, "AGENT_TRACE_ID");
    assert_eq!(env::IDENTITY_TOKEN, "AGENT_IDENTITY_TOKEN");
    assert_eq!(env::OUTPUT_FORMAT, "AGENT_OUTPUT_FORMAT");
    assert_eq!(env::DRY_RUN, "AGENT_DRY_RUN");
    assert_eq!(env::EMIT_SPEC, "AGENT_EMIT_SPEC");
}
