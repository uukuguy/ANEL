/**
 * ANEL (Agent-Native Execution Layer) module for QMD.
 *
 * Provides ANEL protocol support:
 * - ANID error types with RFC 7807 extensions
 * - NDJSON streaming output
 * - Dry-run and spec emission capabilities
 * - Trace context propagation
 */

// ANEL protocol version
export const ANEL_VERSION = "1.0";

// Environment variable names
export const ENV_TRACE_ID = "AGENT_TRACE_ID";
export const ENV_IDENTITY_TOKEN = "AGENT_IDENTITY_TOKEN";
export const ENV_OUTPUT_FORMAT = "AGENT_OUTPUT_FORMAT";
export const ENV_DRY_RUN = "AGENT_DRY_RUN";
export const ENV_EMIT_SPEC = "AGENT_EMIT_SPEC";

// =============================================================================
// Severity
// =============================================================================

export type Severity = "debug" | "info" | "warning" | "error" | "critical";

// =============================================================================
// ErrorCode
// =============================================================================

export type ErrorCode =
  // Generic
  | "UNKNOWN"
  | "INVALID_INPUT"
  | "NOT_FOUND"
  | "PERMISSION_DENIED"
  // Search
  | "SEARCH_FAILED"
  | "INDEX_NOT_READY"
  | "QUERY_PARSE_ERROR"
  // Collection
  | "COLLECTION_NOT_FOUND"
  | "COLLECTION_EXISTS"
  | "COLLECTION_CORRUPTED"
  // Embedding
  | "EMBEDDING_FAILED"
  | "MODEL_NOT_FOUND"
  | "MODEL_LOAD_FAILED"
  // Storage
  | "STORAGE_ERROR"
  | "BACKEND_UNAVAILABLE"
  // Config
  | "CONFIG_ERROR"
  | "ENVIRONMENT_ERROR";

const ERROR_CODE_STATUS: Record<ErrorCode, number> = {
  UNKNOWN: 500,
  INVALID_INPUT: 400,
  NOT_FOUND: 404,
  PERMISSION_DENIED: 403,
  SEARCH_FAILED: 500,
  INDEX_NOT_READY: 503,
  QUERY_PARSE_ERROR: 400,
  COLLECTION_NOT_FOUND: 404,
  COLLECTION_EXISTS: 409,
  COLLECTION_CORRUPTED: 500,
  EMBEDDING_FAILED: 500,
  MODEL_NOT_FOUND: 404,
  MODEL_LOAD_FAILED: 500,
  STORAGE_ERROR: 500,
  BACKEND_UNAVAILABLE: 503,
  CONFIG_ERROR: 500,
  ENVIRONMENT_ERROR: 500,
};

export function errorCodeToStatus(code: ErrorCode): number {
  return ERROR_CODE_STATUS[code] ?? 500;
}

// =============================================================================
// RecoveryHint
// =============================================================================

export type RecoveryHint = {
  code: string;
  message: string;
  action?: string;
};

// =============================================================================
// AnelError (RFC 7807 + ANEL extensions)
// =============================================================================

export type AnelError = {
  error_code: ErrorCode;
  status: number;
  title: string;
  message: string;
  severity: Severity;
  recovery_hints: RecoveryHint[];
  trace_id?: string;
  metadata: Record<string, unknown>;
};

export function createAnelError(
  errorCode: ErrorCode,
  title: string,
  message: string,
): AnelError {
  return {
    error_code: errorCode,
    status: errorCodeToStatus(errorCode),
    title,
    message,
    severity: "error",
    recovery_hints: [],
    metadata: {},
  };
}

export function withHint(err: AnelError, hint: RecoveryHint): AnelError {
  err.recovery_hints.push(hint);
  return err;
}

export function withTraceId(err: AnelError, traceId: string): AnelError {
  err.trace_id = traceId;
  return err;
}

export function withMetadata(err: AnelError, key: string, value: unknown): AnelError {
  err.metadata[key] = value;
  return err;
}

export function anelErrorToNdjson(err: AnelError): string {
  const { metadata: _meta, ...rest } = err;
  return JSON.stringify(rest);
}

export function emitAnelErrorStderr(err: AnelError): void {
  process.stderr.write(anelErrorToNdjson(err) + "\n");
}

// =============================================================================
// TraceContext
// =============================================================================

export type TraceContext = {
  trace_id?: string;
  identity_token?: string;
  tags: Record<string, string>;
};

export function traceContextFromEnv(): TraceContext {
  return {
    trace_id: process.env[ENV_TRACE_ID] || undefined,
    identity_token: process.env[ENV_IDENTITY_TOKEN] || undefined,
    tags: {},
  };
}

export function getOrGenerateTraceId(ctx: TraceContext): string {
  if (ctx.trace_id) return ctx.trace_id;
  return `qmd-${Date.now().toString(16)}`;
}

// =============================================================================
// NdjsonRecord
// =============================================================================

export type NdjsonRecord<T> = {
  type: string; // "result" | "error" | "spec" | "metadata"
  seq: number;
  payload: T;
};

export function createNdjsonRecord<T>(type: string, seq: number, payload: T): NdjsonRecord<T> {
  return { type, seq, payload };
}

export function ndjsonRecordToString<T>(record: NdjsonRecord<T>): string {
  return JSON.stringify(record);
}

export function emitNdjsonRecord<T>(record: NdjsonRecord<T>): void {
  console.log(ndjsonRecordToString(record));
}

// =============================================================================
// AnelResult
// =============================================================================

export type AnelResult = {
  success: boolean;
  data: unknown;
  error?: AnelError;
  trace_id?: string;
};

export function successResult(data: unknown): AnelResult {
  return { success: true, data };
}

export function errorResult(err: AnelError): AnelResult {
  return { success: false, data: null, error: err, trace_id: err.trace_id };
}

// =============================================================================
// fromError - convert JS Error to AnelError
// =============================================================================

export function fromError(err: Error, ctx?: TraceContext): AnelError {
  const message = err.message.toLowerCase();

  let errorCode: ErrorCode = "UNKNOWN";
  if (message.includes("not found")) errorCode = "NOT_FOUND";
  else if (message.includes("permission")) errorCode = "PERMISSION_DENIED";
  else if (message.includes("invalid")) errorCode = "INVALID_INPUT";
  else if (message.includes("parse")) errorCode = "QUERY_PARSE_ERROR";
  else if (message.includes("collection")) errorCode = "COLLECTION_NOT_FOUND";
  else if (message.includes("embedding") || message.includes("embed")) errorCode = "EMBEDDING_FAILED";
  else if (message.includes("storage") || message.includes("database")) errorCode = "STORAGE_ERROR";
  else if (message.includes("config")) errorCode = "CONFIG_ERROR";

  const anelErr = createAnelError(errorCode, "Operation Failed", err.message);
  if (ctx) {
    anelErr.trace_id = getOrGenerateTraceId(ctx);
  }
  return anelErr;
}
