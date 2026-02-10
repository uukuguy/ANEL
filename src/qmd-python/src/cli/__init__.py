"""CLI module for QMD Python."""

from .app import app, main
from .commands import (
    collection_cmd,
    context_cmd,
    get_cmd,
    multi_get_cmd,
    search_cmd,
    vsearch_cmd,
    query_cmd,
    embed_cmd,
    update_cmd,
    status_cmd,
    cleanup_cmd,
)
from .agent import run_agent

__all__ = [
    "app",
    "main",
    "collection_cmd",
    "context_cmd",
    "get_cmd",
    "multi_get_cmd",
    "search_cmd",
    "vsearch_cmd",
    "query_cmd",
    "embed_cmd",
    "update_cmd",
    "status_cmd",
    "cleanup_cmd",
    "run_agent",
]
