"""CLI application for QMD."""

from typing import Optional
import typer
from rich import print
from enum import Enum

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


class OutputFormat(str, Enum):
    CLI = "cli"
    JSON = "json"
    MARKDOWN = "md"
    CSV = "csv"
    FILES = "files"


app = typer.Typer(
    name="qmd",
    help="QMD - AI-powered search with hybrid BM25 and vector search",
    add_completion=False,
)

# Register subcommands
app.add_typer(collection_cmd, name="collection", help="Manage collections")
app.add_typer(context_cmd, name="context", help="Manage contexts")
app.add_typer(get_cmd, name="get", help="Get document content")
app.add_typer(multi_get_cmd, name="multi-get", help="Get multiple documents")
app.add_typer(search_cmd, name="search", help="BM25 full-text search")
app.add_typer(vsearch_cmd, name="vsearch", help="Vector semantic search")
app.add_typer(query_cmd, name="query", help="Hybrid search with reranking")
app.add_typer(embed_cmd, name="embed", help="Generate/update embeddings")
app.add_typer(update_cmd, name="update", help="Update index")
app.add_typer(status_cmd, name="status", help="Show index status")
app.add_typer(cleanup_cmd, name="cleanup", help="Cleanup stale entries")


@app.command("mcp")
def mcp_server(
    transport: str = typer.Option("stdio", "--transport", "-t", help="Transport: stdio, sse"),
    port: int = typer.Option(8080, "--port", "-p", help="Port for SSE transport"),
) -> None:
    """Run as MCP server."""
    from ..mcp.server import run_server
    import asyncio

    asyncio.run(run_server(transport, port))


@app.command("agent")
def agent_mode(
    interactive: bool = typer.Option(False, "--interactive", "-i", help="Interactive mode"),
    mcp: bool = typer.Option(False, "--mcp", help="Also run MCP server"),
    transport: str = typer.Option("stdio", "--transport", "-t", help="MCP transport"),
) -> None:
    """Run in agent mode."""
    from ..cli.agent import run_agent
    run_agent(interactive, mcp, transport)


def main() -> None:
    """Main entry point."""
    app()
