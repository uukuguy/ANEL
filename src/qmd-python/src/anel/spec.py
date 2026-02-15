"""ANEL spec generators for QMD commands."""

from . import VERSION, AnelSpec, ErrorCode


def search_spec() -> AnelSpec:
    """Return the ANEL spec for the search command."""
    return AnelSpec(
        version=VERSION,
        command="search",
        input_schema={
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "limit": {"type": "integer", "default": 20},
                "min_score": {"type": "number", "default": 0.0},
                "collection": {"type": "string"},
                "all": {"type": "boolean", "default": False},
            },
            "required": ["query"],
        },
        output_schema={
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
                        },
                    },
                },
                "total": {"type": "integer"},
            },
        },
        error_codes=[
            ErrorCode.SEARCH_FAILED,
            ErrorCode.INDEX_NOT_READY,
            ErrorCode.QUERY_PARSE_ERROR,
        ],
    )


def vsearch_spec() -> AnelSpec:
    """Return the ANEL spec for the vsearch command."""
    return AnelSpec(
        version=VERSION,
        command="vsearch",
        input_schema={
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "limit": {"type": "integer", "default": 20},
                "collection": {"type": "string"},
                "all": {"type": "boolean", "default": False},
            },
            "required": ["query"],
        },
        output_schema={
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
                        },
                    },
                },
                "total": {"type": "integer"},
            },
        },
        error_codes=[
            ErrorCode.SEARCH_FAILED,
            ErrorCode.INDEX_NOT_READY,
            ErrorCode.EMBEDDING_FAILED,
            ErrorCode.MODEL_NOT_FOUND,
        ],
    )


def query_spec() -> AnelSpec:
    """Return the ANEL spec for the query (hybrid search) command."""
    return AnelSpec(
        version=VERSION,
        command="query",
        input_schema={
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "limit": {"type": "integer", "default": 20},
                "collection": {"type": "string"},
                "all": {"type": "boolean", "default": False},
            },
            "required": ["query"],
        },
        output_schema={
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
                            "reranked": {"type": "boolean"},
                        },
                    },
                },
                "total": {"type": "integer"},
            },
        },
        error_codes=[
            ErrorCode.SEARCH_FAILED,
            ErrorCode.INDEX_NOT_READY,
            ErrorCode.EMBEDDING_FAILED,
            ErrorCode.MODEL_NOT_FOUND,
            ErrorCode.QUERY_PARSE_ERROR,
        ],
    )


def get_spec() -> AnelSpec:
    """Return the ANEL spec for the get command."""
    return AnelSpec(
        version=VERSION,
        command="get",
        input_schema={
            "type": "object",
            "properties": {
                "file": {"type": "string", "description": "File path with optional :line suffix"},
                "limit": {"type": "integer", "default": 50},
                "from": {"type": "integer", "default": 0},
                "full": {"type": "boolean", "default": False},
            },
            "required": ["file"],
        },
        output_schema={
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "lines": {"type": "array", "items": {"type": "string"}},
                "total_lines": {"type": "integer"},
            },
        },
        error_codes=[
            ErrorCode.NOT_FOUND,
            ErrorCode.INVALID_INPUT,
        ],
    )


def collection_spec() -> AnelSpec:
    """Return the ANEL spec for the collection command."""
    return AnelSpec(
        version=VERSION,
        command="collection",
        input_schema={
            "type": "object",
            "properties": {
                "action": {"type": "string", "enum": ["add", "list", "remove", "rename"]},
                "name": {"type": "string"},
                "path": {"type": "string"},
                "mask": {"type": "string", "default": "**/*"},
                "description": {"type": "string"},
                "new_name": {"type": "string"},
            },
        },
        output_schema={
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
                            "description": {"type": "string"},
                        },
                    },
                },
                "action": {"type": "string"},
            },
        },
        error_codes=[
            ErrorCode.COLLECTION_NOT_FOUND,
            ErrorCode.COLLECTION_EXISTS,
            ErrorCode.INVALID_INPUT,
        ],
    )


def embed_spec() -> AnelSpec:
    """Return the ANEL spec for the embed command."""
    return AnelSpec(
        version=VERSION,
        command="embed",
        input_schema={
            "type": "object",
            "properties": {
                "force": {"type": "boolean", "default": False},
                "collection": {"type": "string"},
            },
        },
        output_schema={
            "type": "object",
            "properties": {
                "collections_processed": {"type": "integer"},
                "documents_embedded": {"type": "integer"},
                "chunks_embedded": {"type": "integer"},
                "model": {"type": "string"},
            },
        },
        error_codes=[
            ErrorCode.EMBEDDING_FAILED,
            ErrorCode.MODEL_NOT_FOUND,
            ErrorCode.MODEL_LOAD_FAILED,
            ErrorCode.COLLECTION_NOT_FOUND,
        ],
    )


def update_spec() -> AnelSpec:
    """Return the ANEL spec for the update command."""
    return AnelSpec(
        version=VERSION,
        command="update",
        input_schema={
            "type": "object",
            "properties": {
                "pull": {"type": "boolean", "default": False},
                "collection": {"type": "string"},
            },
        },
        output_schema={
            "type": "object",
            "properties": {
                "collections_updated": {"type": "integer"},
                "documents_indexed": {"type": "integer"},
                "documents_removed": {"type": "integer"},
            },
        },
        error_codes=[
            ErrorCode.INDEX_NOT_READY,
            ErrorCode.COLLECTION_NOT_FOUND,
            ErrorCode.STORAGE_ERROR,
        ],
    )


def status_spec() -> AnelSpec:
    """Return the ANEL spec for the status command."""
    return AnelSpec(
        version=VERSION,
        command="status",
        input_schema={
            "type": "object",
            "properties": {
                "verbose": {"type": "boolean", "default": False},
                "collection": {"type": "string"},
            },
        },
        output_schema={
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
                            "last_updated": {"type": "string"},
                        },
                    },
                },
            },
        },
        error_codes=[
            ErrorCode.COLLECTION_NOT_FOUND,
            ErrorCode.INDEX_NOT_READY,
        ],
    )


def cleanup_spec() -> AnelSpec:
    """Return the ANEL spec for the cleanup command."""
    return AnelSpec(
        version=VERSION,
        command="cleanup",
        input_schema={
            "type": "object",
            "properties": {
                "dry_run": {"type": "boolean", "default": False},
                "older_than": {"type": "integer", "default": 30},
                "collection": {"type": "string"},
            },
        },
        output_schema={
            "type": "object",
            "properties": {
                "entries_removed": {"type": "integer"},
                "dry_run": {"type": "boolean"},
            },
        },
        error_codes=[
            ErrorCode.COLLECTION_NOT_FOUND,
            ErrorCode.STORAGE_ERROR,
        ],
    )


def agent_spec() -> AnelSpec:
    """Return the ANEL spec for the agent command."""
    return AnelSpec(
        version=VERSION,
        command="agent",
        input_schema={
            "type": "object",
            "properties": {
                "interactive": {"type": "boolean", "default": False},
                "query": {"type": "string"},
                "mcp": {"type": "boolean", "default": False},
                "transport": {"type": "string", "default": "stdio"},
            },
        },
        output_schema={
            "type": "object",
            "properties": {
                "intent": {"type": "string"},
                "results": {"type": "array"},
                "mode": {"type": "string"},
            },
        },
        error_codes=[
            ErrorCode.SEARCH_FAILED,
            ErrorCode.INDEX_NOT_READY,
            ErrorCode.EMBEDDING_FAILED,
        ],
    )


def context_spec() -> AnelSpec:
    """Return the ANEL spec for the context command."""
    return AnelSpec(
        version=VERSION,
        command="context",
        input_schema={
            "type": "object",
            "properties": {
                "action": {"type": "string", "enum": ["add", "list", "rm"]},
                "path": {"type": "string"},
                "description": {"type": "string"},
            },
            "required": ["action"],
        },
        output_schema={
            "type": "object",
            "properties": {
                "contexts": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "path": {"type": "string"},
                            "description": {"type": "string"},
                        },
                    },
                },
                "action": {"type": "string"},
            },
        },
        error_codes=[
            ErrorCode.NOT_FOUND,
            ErrorCode.INVALID_INPUT,
        ],
    )


def mcp_spec() -> AnelSpec:
    """Return the ANEL spec for the mcp command."""
    return AnelSpec(
        version=VERSION,
        command="mcp",
        input_schema={
            "type": "object",
            "properties": {
                "transport": {"type": "string", "default": "stdio"},
                "port": {"type": "integer", "default": 8080},
            },
        },
        output_schema={
            "type": "object",
            "properties": {
                "status": {"type": "string"},
                "transport": {"type": "string"},
                "port": {"type": "integer"},
            },
        },
        error_codes=[
            ErrorCode.CONFIG_ERROR,
            ErrorCode.BACKEND_UNAVAILABLE,
        ],
    )


SPEC_GETTERS = {
    "search": search_spec,
    "vsearch": vsearch_spec,
    "query": query_spec,
    "get": get_spec,
    "collection": collection_spec,
    "context": context_spec,
    "embed": embed_spec,
    "update": update_spec,
    "status": status_spec,
    "cleanup": cleanup_spec,
    "agent": agent_spec,
    "mcp": mcp_spec,
}


def get_spec_for_command(command: str) -> AnelSpec | None:
    """Get the spec for a specific command."""
    getter = SPEC_GETTERS.get(command)
    if getter:
        return getter()
    return None
