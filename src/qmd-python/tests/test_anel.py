"""Tests for ANEL core types: ErrorCode, AnelError, TraceContext, AnelResult, NDJSONRecord."""

import json
import os
import pytest

import sys
sys.path.insert(0, str(__import__("pathlib").Path(__file__).parent.parent / "src"))

from anel import (
    VERSION,
    ENV_TRACE_ID,
    ENV_IDENTITY_TOKEN,
    ENV_OUTPUT_FORMAT,
    ENV_DRY_RUN,
    ENV_EMIT_SPEC,
    Severity,
    ErrorCode,
    RecoveryHint,
    AnelError,
    TraceContext,
    AnelSpec,
    NDJSONRecord,
    AnelResult,
    from_error,
)


# --- ErrorCode ---


class TestErrorCode:
    def test_all_codes_exist(self):
        expected = [
            "UNKNOWN", "INVALID_INPUT", "NOT_FOUND", "PERMISSION_DENIED",
            "SEARCH_FAILED", "INDEX_NOT_READY", "QUERY_PARSE_ERROR",
            "COLLECTION_NOT_FOUND", "COLLECTION_EXISTS", "COLLECTION_CORRUPTED",
            "EMBEDDING_FAILED", "MODEL_NOT_FOUND", "MODEL_LOAD_FAILED",
            "STORAGE_ERROR", "BACKEND_UNAVAILABLE",
            "CONFIG_ERROR", "ENVIRONMENT_ERROR",
        ]
        for name in expected:
            assert hasattr(ErrorCode, name), f"ErrorCode.{name} missing"

    def test_to_status_mapping(self):
        cases = {
            ErrorCode.UNKNOWN: 500,
            ErrorCode.INVALID_INPUT: 400,
            ErrorCode.NOT_FOUND: 404,
            ErrorCode.PERMISSION_DENIED: 403,
            ErrorCode.SEARCH_FAILED: 500,
            ErrorCode.INDEX_NOT_READY: 503,
            ErrorCode.QUERY_PARSE_ERROR: 400,
            ErrorCode.COLLECTION_NOT_FOUND: 404,
            ErrorCode.COLLECTION_EXISTS: 409,
            ErrorCode.COLLECTION_CORRUPTED: 500,
            ErrorCode.EMBEDDING_FAILED: 500,
            ErrorCode.MODEL_NOT_FOUND: 404,
            ErrorCode.MODEL_LOAD_FAILED: 500,
            ErrorCode.STORAGE_ERROR: 500,
            ErrorCode.BACKEND_UNAVAILABLE: 503,
            ErrorCode.CONFIG_ERROR: 500,
            ErrorCode.ENVIRONMENT_ERROR: 500,
        }
        for code, status in cases.items():
            assert code.to_status() == status, f"{code} -> {code.to_status()}, want {status}"

    def test_string_values(self):
        assert ErrorCode.NOT_FOUND.value == "NOT_FOUND"
        assert ErrorCode.SEARCH_FAILED.value == "SEARCH_FAILED"

    def test_is_str_enum(self):
        assert isinstance(ErrorCode.UNKNOWN, str)


# --- Severity ---


class TestSeverity:
    def test_all_levels(self):
        assert Severity.DEBUG.value == "debug"
        assert Severity.INFO.value == "info"
        assert Severity.WARNING.value == "warning"
        assert Severity.ERROR.value == "error"
        assert Severity.CRITICAL.value == "critical"


# --- RecoveryHint ---


class TestRecoveryHint:
    def test_basic(self):
        hint = RecoveryHint(code="RETRY", message="Wait and retry")
        assert hint.code == "RETRY"
        assert hint.message == "Wait and retry"
        assert hint.action is None

    def test_with_action(self):
        hint = RecoveryHint(code="FIX", message="Fix it").with_action("run fix cmd")
        assert hint.action == "run fix cmd"

    def test_json_serialization(self):
        hint = RecoveryHint(code="A", message="B", action="C")
        data = hint.model_dump()
        assert data["code"] == "A"
        assert data["message"] == "B"
        assert data["action"] == "C"

    def test_json_omits_none_action(self):
        hint = RecoveryHint(code="X", message="Y")
        data = hint.model_dump()
        assert data["action"] is None


# --- AnelError ---


class TestAnelError:
    def test_new(self):
        err = AnelError.new(ErrorCode.INVALID_INPUT, "Bad Request", "missing query")
        assert err.error_code == ErrorCode.INVALID_INPUT
        assert err.status == 400
        assert err.title == "Bad Request"
        assert err.message == "missing query"
        assert err.severity == Severity.ERROR
        assert err.recovery_hints == []
        assert err.trace_id is None
        assert err.metadata == {}

    def test_with_hint(self):
        err = AnelError.new(ErrorCode.NOT_FOUND, "Not Found", "missing").with_hint(
            RecoveryHint(code="LIST", message="List items")
        )
        assert len(err.recovery_hints) == 1
        assert err.recovery_hints[0].code == "LIST"

    def test_with_multiple_hints(self):
        err = (
            AnelError.new(ErrorCode.BACKEND_UNAVAILABLE, "Down", "timeout")
            .with_hint(RecoveryHint(code="RETRY", message="Retry"))
            .with_hint(RecoveryHint(code="TIMEOUT", message="Increase timeout"))
        )
        assert len(err.recovery_hints) == 2

    def test_with_trace_id(self):
        err = AnelError.new(ErrorCode.SEARCH_FAILED, "Failed", "err").with_trace_id("t-123")
        assert err.trace_id == "t-123"

    def test_with_metadata(self):
        err = (
            AnelError.new(ErrorCode.INVALID_INPUT, "Bad", "bad")
            .with_metadata("field", "query")
            .with_metadata("value", "")
        )
        assert err.metadata["field"] == "query"
        assert err.metadata["value"] == ""

    def test_str(self):
        err = AnelError.new(ErrorCode.NOT_FOUND, "Not Found", "file missing")
        s = str(err)
        assert "NOT_FOUND" in s
        assert "file missing" in s

    def test_to_ndjson_valid_json(self):
        err = AnelError.new(ErrorCode.INVALID_INPUT, "Bad", "missing").with_hint(
            RecoveryHint(code="FIX", message="add flag")
        )
        ndjson = err.to_ndjson()
        parsed = json.loads(ndjson)
        assert parsed["error_code"] == "INVALID_INPUT"
        assert parsed["status"] == 400
        assert len(parsed["recovery_hints"]) == 1

    def test_status_auto_set(self):
        for code in ErrorCode:
            err = AnelError.new(code, "T", "M")
            assert err.status == code.to_status()


# --- TraceContext ---


class TestTraceContext:
    def test_from_env_empty(self):
        os.environ.pop(ENV_TRACE_ID, None)
        os.environ.pop(ENV_IDENTITY_TOKEN, None)
        ctx = TraceContext.from_env()
        assert ctx.trace_id is None
        assert ctx.identity_token is None

    def test_from_env_set(self):
        os.environ[ENV_TRACE_ID] = "test-trace"
        os.environ[ENV_IDENTITY_TOKEN] = "test-token"
        try:
            ctx = TraceContext.from_env()
            assert ctx.trace_id == "test-trace"
            assert ctx.identity_token == "test-token"
        finally:
            del os.environ[ENV_TRACE_ID]
            del os.environ[ENV_IDENTITY_TOKEN]

    def test_get_or_generate_existing(self):
        ctx = TraceContext(trace_id="existing")
        assert ctx.get_or_generate_trace_id() == "existing"

    def test_get_or_generate_new(self):
        ctx = TraceContext()
        tid = ctx.get_or_generate_trace_id()
        assert tid.startswith("qmd-")

    def test_tags_default_empty(self):
        ctx = TraceContext()
        assert ctx.tags == {}


# --- AnelSpec ---


class TestAnelSpec:
    def test_to_json(self):
        spec = AnelSpec(
            version="1.0",
            command="test",
            input_schema={"type": "object"},
            output_schema={"type": "object"},
            error_codes=[ErrorCode.NOT_FOUND],
        )
        parsed = json.loads(spec.to_json())
        assert parsed["version"] == "1.0"
        assert parsed["command"] == "test"
        assert parsed["error_codes"] == ["NOT_FOUND"]


# --- NDJSONRecord ---


class TestNDJSONRecord:
    def test_basic(self):
        record = NDJSONRecord(type="result", seq=1, payload={"path": "test.md"})
        assert record.type == "result"
        assert record.seq == 1

    def test_to_ndjson(self):
        record = NDJSONRecord(type="error", seq=0, payload={"msg": "fail"})
        parsed = json.loads(record.to_ndjson())
        assert parsed["type"] == "error"
        assert parsed["seq"] == 0


# --- AnelResult ---


class TestAnelResult:
    def test_success(self):
        result = AnelResult.success_result({"count": 42})
        assert result.success is True
        assert result.data == {"count": 42}
        assert result.error is None

    def test_error(self):
        err = AnelError.new(ErrorCode.SEARCH_FAILED, "Failed", "err").with_trace_id("t")
        result = AnelResult.error_result(err)
        assert result.success is False
        assert result.error is not None
        assert result.trace_id == "t"

    def test_with_trace_id(self):
        result = AnelResult.success_result({}).with_trace_id("my-trace")
        assert result.trace_id == "my-trace"

    def test_to_ndjson(self):
        result = AnelResult.success_result({"x": 1})
        parsed = json.loads(result.to_ndjson())
        assert parsed["success"] is True


# --- from_error ---


class TestFromError:
    @pytest.mark.parametrize(
        "msg,expected_code",
        [
            ("file not found", ErrorCode.NOT_FOUND),
            ("permission denied", ErrorCode.PERMISSION_DENIED),
            ("invalid argument", ErrorCode.INVALID_INPUT),
            ("parse error in query", ErrorCode.QUERY_PARSE_ERROR),
            ("collection missing", ErrorCode.COLLECTION_NOT_FOUND),
            ("embedding generation failed", ErrorCode.EMBEDDING_FAILED),
            ("storage backend error", ErrorCode.STORAGE_ERROR),
            ("database connection lost", ErrorCode.STORAGE_ERROR),
            ("config file missing", ErrorCode.CONFIG_ERROR),
            ("something random", ErrorCode.UNKNOWN),
        ],
    )
    def test_mapping(self, msg, expected_code):
        err = from_error(Exception(msg))
        assert err.error_code == expected_code

    def test_with_trace_context(self):
        ctx = TraceContext(trace_id="ctx-123")
        err = from_error(Exception("not found"), ctx)
        assert err.trace_id == "ctx-123"

    def test_without_context(self):
        err = from_error(Exception("unknown issue"))
        assert err.trace_id is None


# --- Constants ---


class TestConstants:
    def test_version(self):
        assert VERSION == "1.0"

    def test_env_vars(self):
        assert ENV_TRACE_ID == "AGENT_TRACE_ID"
        assert ENV_IDENTITY_TOKEN == "AGENT_IDENTITY_TOKEN"
        assert ENV_OUTPUT_FORMAT == "AGENT_OUTPUT_FORMAT"
        assert ENV_DRY_RUN == "AGENT_DRY_RUN"
        assert ENV_EMIT_SPEC == "AGENT_EMIT_SPEC"
