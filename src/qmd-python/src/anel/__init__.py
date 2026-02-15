"""ANEL (Agent-Native Execution Layer) module for QMD.

This module provides ANEL protocol support for QMD, enabling:
- ANID (Agent-Native ID) error types with RFC 7807 extensions
- NDJSON streaming output
- Dry-run and spec emission capabilities
- Trace context propagation
"""

import json
import os
import time
from enum import Enum
from typing import Any, Generic, Optional, TypeVar

import pydantic


class Severity(str, Enum):
    """Error severity levels."""

    DEBUG = "debug"
    INFO = "info"
    WARNING = "warning"
    ERROR = "error"
    CRITICAL = "critical"


class ErrorCode(str, Enum):
    """Error codes for ANEL operations."""

    # Generic errors
    UNKNOWN = "UNKNOWN"
    INVALID_INPUT = "INVALID_INPUT"
    NOT_FOUND = "NOT_FOUND"
    PERMISSION_DENIED = "PERMISSION_DENIED"

    # Search-related errors
    SEARCH_FAILED = "SEARCH_FAILED"
    INDEX_NOT_READY = "INDEX_NOT_READY"
    QUERY_PARSE_ERROR = "QUERY_PARSE_ERROR"

    # Collection errors
    COLLECTION_NOT_FOUND = "COLLECTION_NOT_FOUND"
    COLLECTION_EXISTS = "COLLECTION_EXISTS"
    COLLECTION_CORRUPTED = "COLLECTION_CORRUPTED"

    # Embedding errors
    EMBEDDING_FAILED = "EMBEDDING_FAILED"
    MODEL_NOT_FOUND = "MODEL_NOT_FOUND"
    MODEL_LOAD_FAILED = "MODEL_LOAD_FAILED"

    # Storage errors
    STORAGE_ERROR = "STORAGE_ERROR"
    BACKEND_UNAVAILABLE = "BACKEND_UNAVAILABLE"

    # Configuration errors
    CONFIG_ERROR = "CONFIG_ERROR"
    ENVIRONMENT_ERROR = "ENVIRONMENT_ERROR"

    def to_status(self) -> int:
        """Convert error code to HTTP-style status."""
        mapping = {
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
        return mapping.get(self, 500)


# ANEL protocol version
VERSION = "1.0"

# Environment variable names
ENV_TRACE_ID = "AGENT_TRACE_ID"
ENV_IDENTITY_TOKEN = "AGENT_IDENTITY_TOKEN"
ENV_OUTPUT_FORMAT = "AGENT_OUTPUT_FORMAT"
ENV_DRY_RUN = "AGENT_DRY_RUN"
ENV_EMIT_SPEC = "AGENT_EMIT_SPEC"


class RecoveryHint(pydantic.BaseModel):
    """Recovery hint for error resolution."""

    code: str
    message: str
    action: Optional[str] = None

    def with_action(self, action: str) -> "RecoveryHint":
        """Add an action to the recovery hint."""
        self.action = action
        return self


class AnelError(pydantic.BaseModel):
    """ANID Error type (Agent-Native ID Error).

    Implements RFC 7807 Problem Details with ANEL extensions.
    """

    error_code: ErrorCode
    status: int
    title: str
    message: str
    severity: Severity = Severity.ERROR
    recovery_hints: list[RecoveryHint] = pydantic.Field(default_factory=list)
    trace_id: Optional[str] = None
    metadata: dict[str, Any] = pydantic.Field(default_factory=dict)

    @classmethod
    def new(
        cls,
        error_code: ErrorCode,
        title: str,
        message: str,
    ) -> "AnelError":
        """Create a new ANEL error."""
        return cls(
            error_code=error_code,
            status=error_code.to_status(),
            title=title,
            message=message,
            severity=Severity.ERROR,
            recovery_hints=[],
            metadata={},
        )

    def with_hint(self, hint: RecoveryHint) -> "AnelError":
        """Add a recovery hint."""
        self.recovery_hints.append(hint)
        return self

    def with_trace_id(self, trace_id: str) -> "AnelError":
        """Add trace ID."""
        self.trace_id = trace_id
        return self

    def with_metadata(self, key: str, value: Any) -> "AnelError":
        """Add metadata."""
        self.metadata[key] = value
        return self

    def to_ndjson(self) -> str:
        """Serialize to NDJSON line."""
        return self.model_dump_json(exclude={"metadata"})

    def emit_stderr(self) -> None:
        """Print to stderr in NDJSON format."""
        print(self.to_ndjson(), file=__import__("sys").stderr)

    def __str__(self) -> str:
        return f"[{self.error_code.value}] {self.title}: {self.message}"


class TraceContext(pydantic.BaseModel):
    """Trace context for request correlation."""

    trace_id: Optional[str] = None
    identity_token: Optional[str] = None
    tags: dict[str, str] = pydantic.Field(default_factory=dict)

    @classmethod
    def from_env(cls) -> "TraceContext":
        """Create from environment variables."""
        trace_id = os.environ.get(ENV_TRACE_ID)
        identity_token = os.environ.get(ENV_IDENTITY_TOKEN)

        return cls(
            trace_id=trace_id if trace_id else None,
            identity_token=identity_token if identity_token else None,
            tags={},
        )

    def get_or_generate_trace_id(self) -> str:
        """Get trace ID or generate a new one."""
        if self.trace_id:
            return self.trace_id
        return f"qmd-{int(time.time() * 1e9)}"


class AnelSpec(pydantic.BaseModel):
    """ANEL specification for a command."""

    version: str
    command: str
    input_schema: dict[str, Any]
    output_schema: dict[str, Any]
    error_codes: list[ErrorCode]

    def to_json(self) -> str:
        """Convert spec to JSON string."""
        return self.model_dump_json(indent=2)


# Type variable for generic NDJSON records
T = TypeVar("T")


class NDJSONRecord(pydantic.BaseModel, Generic[T]):
    """NDJSON output wrapper for streaming."""

    type: str  # "result", "error", "spec", "metadata"
    seq: int
    payload: T

    def to_ndjson(self) -> str:
        """Serialize to NDJSON line."""
        return self.model_dump_json()

    def emit(self) -> None:
        """Print to stdout in NDJSON format."""
        print(self.to_ndjson())


class AnelResult(pydantic.BaseModel):
    """ANEL command result."""

    success: bool
    data: Optional[dict[str, Any]] = None
    error: Optional[AnelError] = None
    trace_id: Optional[str] = None

    @classmethod
    def success_result(cls, data: dict[str, Any]) -> "AnelResult":
        """Create a success result."""
        return cls(success=True, data=data, error=None)

    @classmethod
    def error_result(cls, err: AnelError) -> "AnelResult":
        """Create an error result."""
        return cls(success=False, data=None, error=err, trace_id=err.trace_id)

    def with_trace_id(self, trace_id: str) -> "AnelResult":
        """Add trace ID."""
        self.trace_id = trace_id
        return self

    def to_ndjson(self) -> str:
        """Serialize to NDJSON."""
        return self.model_dump_json()


def from_error(err: Exception, ctx: Optional[TraceContext] = None) -> AnelError:
    """Convert a standard error to AnelError."""
    message = str(err)

    # Try to extract error code from error message
    if "not found" in message.lower():
        error_code = ErrorCode.NOT_FOUND
    elif "permission" in message.lower():
        error_code = ErrorCode.PERMISSION_DENIED
    elif "invalid" in message.lower():
        error_code = ErrorCode.INVALID_INPUT
    elif "parse" in message.lower():
        error_code = ErrorCode.QUERY_PARSE_ERROR
    elif "collection" in message.lower():
        error_code = ErrorCode.COLLECTION_NOT_FOUND
    elif "embedding" in message.lower() or "embed" in message.lower():
        error_code = ErrorCode.EMBEDDING_FAILED
    elif "storage" in message.lower() or "database" in message.lower():
        error_code = ErrorCode.STORAGE_ERROR
    elif "config" in message.lower():
        error_code = ErrorCode.CONFIG_ERROR
    else:
        error_code = ErrorCode.UNKNOWN

    anel_err = AnelError.new(error_code, "Operation Failed", message)

    if ctx:
        anel_err.with_trace_id(ctx.get_or_generate_trace_id())

    return anel_err
