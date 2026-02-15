package anel

import (
	"strings"
)

// FromError converts a standard error to AnelError
func FromError(err error) *AnelError {
	if err == nil {
		return nil
	}

	message := err.Error()

	// Try to extract error code from error message
	var errorCode ErrorCode
	switch {
	case strings.Contains(message, "not found"):
		errorCode = ErrorCodeNotFound
	case strings.Contains(message, "permission"):
		errorCode = ErrorCodePermissionDenied
	case strings.Contains(message, "invalid"):
		errorCode = ErrorCodeInvalidInput
	case strings.Contains(message, "parse"), strings.Contains(message, "Parse"):
		errorCode = ErrorCodeQueryParseError
	case strings.Contains(message, "collection"):
		errorCode = ErrorCodeCollectionNotFound
	case strings.Contains(message, "embedding"), strings.Contains(message, "embed"):
		errorCode = ErrorCodeEmbeddingFailed
	case strings.Contains(message, "storage"), strings.Contains(message, "database"):
		errorCode = ErrorCodeStorageError
	case strings.Contains(message, "config"), strings.Contains(message, "Config"):
		errorCode = ErrorCodeConfigError
	default:
		errorCode = ErrorCodeUnknown
	}

	return NewAnelError(errorCode, "Operation Failed", message)
}

// FromErrorWithContext converts error with trace context
func FromErrorWithContext(err error, ctx *TraceContext) *AnelError {
	anelErr := FromError(err)
	if ctx != nil {
		traceID := ctx.GetOrGenerateTraceID()
		anelErr.WithTraceID(traceID)
	}
	return anelErr
}

// IsNotFound checks if error is a not found error
func IsNotFound(err error) bool {
	if aerr, ok := err.(*AnelError); ok {
		return aerr.ErrorCode == ErrorCodeNotFound
	}
	return strings.Contains(err.Error(), "not found")
}

// IsInvalidInput checks if error is an invalid input error
func IsInvalidInput(err error) bool {
	if aerr, ok := err.(*AnelError); ok {
		return aerr.ErrorCode == ErrorCodeInvalidInput
	}
	return strings.Contains(err.Error(), "invalid")
}

// IsPermissionDenied checks if error is a permission denied error
func IsPermissionDenied(err error) bool {
	if aerr, ok := err.(*AnelError); ok {
		return aerr.ErrorCode == ErrorCodePermissionDenied
	}
	return strings.Contains(err.Error(), "permission")
}
