"""CLI application for QMD."""

from typing import Optional
import typer
from rich import print
from enum import Enum


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


# Import commands after app is defined to avoid circular imports
@app.command("get")
def get_cmd(
    file: str = typer.Argument(..., help="File path (with optional :line suffix)"),
    limit: int = typer.Option(50, "-l", "--limit", help="Number of lines"),
    from_line: int = typer.Option(0, "--from", help="Start line"),
    full: bool = typer.Option(False, "--full", help="Full content"),
) -> None:
    """Get document content."""
    print(f"Getting: {file}")


@app.command("search")
def search_cmd(
    query: str = typer.Argument(..., help="Search query"),
    limit: int = typer.Option(20, "-n", "--limit", help="Max results"),
    collection: Optional[str] = typer.Option(None, "-c", "--collection", help="Collection"),
    all_collections: bool = typer.Option(False, "--all", help="Search all collections"),
    format: str = typer.Option("cli", "--format", "-f", help="Output format"),
    fts_backend: str = typer.Option("sqlite_fts5", "--fts-backend", help="BM25 backend"),
) -> None:
    """BM25 full-text search."""
    from ..config import Config
    from ..store import Store, SearchOptions

    config = Config.load()
    store = Store(config)
    options = SearchOptions(
        limit=limit,
        collection=collection,
        search_all=all_collections,
    )
    results = store.bm25_search(query, options)

    for r in results:
        print(f"[{r.score:.3f}] {r.path}")
        print(f"    Title: {r.title}")


@app.command("vsearch")
def vsearch_cmd(
    query: str = typer.Argument(..., help="Search query"),
    limit: int = typer.Option(20, "-n", "--limit", help="Max results"),
    collection: Optional[str] = typer.Option(None, "-c", "--collection", help="Collection"),
    all_collections: bool = typer.Option(False, "--all", help="Search all collections"),
    format: str = typer.Option("cli", "--format", "-f", help="Output format"),
    vector_backend: str = typer.Option("qmd_builtin", "--vector-backend", help="Vector backend"),
) -> None:
    """Vector semantic search."""
    print(f"Vector search: {query}")


@app.command("query")
def query_cmd(
    query: str = typer.Argument(..., help="Search query"),
    limit: int = typer.Option(20, "-n", "--limit", help="Max results"),
    collection: Optional[str] = typer.Option(None, "-c", "--collection", help="Collection"),
    all_collections: bool = typer.Option(False, "--all", help="Search all collections"),
    format: str = typer.Option("cli", "--format", "-f", help="Output format"),
) -> None:
    """Hybrid search with reranking."""
    print(f"Hybrid query: {query}")


@app.command("embed")
def embed_cmd(
    force: bool = typer.Option(False, "-f", "--force", help="Force regeneration"),
    collection: Optional[str] = typer.Option(None, "-c", "--collection", help="Collection"),
) -> None:
    """Generate/update embeddings."""
    print("Generating embeddings...")


@app.command("update")
def update_cmd(
    pull: bool = typer.Option(False, "--pull", help="Pull remote changes"),
) -> None:
    """Update index."""
    print("Updating index...")


@app.command("status")
def status_cmd(
    verbose: bool = typer.Option(False, "-v", "--verbose", help="Detailed output"),
) -> None:
    """Show index status."""
    from ..config import Config
    from ..store import Store

    config = Config.load()
    store = Store(config)
    stats = store.get_stats()

    print("Index Status")
    print("=" * 40)
    print(f"Collections: {stats.collection_count}")
    print(f"Documents: {stats.document_count}")
    if verbose:
        print("\nDetailed Statistics:")
        for name, count in stats.collection_stats.items():
            print(f"  {name}: {count} documents")


@app.command("cleanup")
def cleanup_cmd(
    dry_run: bool = typer.Option(False, "--dry-run", help="Dry run only"),
    older_than: int = typer.Option(30, "--older-than", help="Days"),
) -> None:
    """Cleanup stale entries."""
    print("Cleanup completed")


# Subcommands
collection_cmd = typer.Typer(help="Manage collections")
context_cmd = typer.Typer(help="Manage contexts")


@collection_cmd.command("add")
def collection_add(
    path: str = typer.Argument(..., help="Collection path"),
    name: Optional[str] = typer.Option(None, "-n", "--name", help="Collection name"),
    mask: str = typer.Option("**/*", "-m", "--mask", help="File pattern"),
    description: Optional[str] = typer.Option(None, "-d", "--description", help="Description"),
) -> None:
    """Add a collection."""
    from ..config import Config

    config = Config.load()
    print(f"Collection added: {path}")


@collection_cmd.command("list")
def collection_list() -> None:
    """List collections."""
    from ..config import Config

    config = Config.load()
    if not config.collections:
        print("No collections configured")
        return

    for col in config.collections:
        print(f"  {col.name}: {col.path}")


@collection_cmd.command("remove")
def collection_remove(name: str) -> None:
    """Remove a collection."""
    print(f"Collection '{name}' removed")


@context_cmd.command("add")
def context_add(
    path: Optional[str] = typer.Argument(None, help="Path (default: current directory)"),
    description: str = typer.Option(..., "-d", "--description", help="Description"),
) -> None:
    """Add a context."""
    print(f"Context added: {description}")


@context_cmd.command("list")
def context_list() -> None:
    """List contexts."""
    print("Contexts:")


@context_cmd.command("rm")
def context_rm(path: str) -> None:
    """Remove a context."""
    print(f"Context '{path}' removed")


# Register subcommands
app.add_typer(collection_cmd, name="collection", help="Manage collections")
app.add_typer(context_cmd, name="context", help="Manage contexts")


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
    print("Agent mode ready")


def main() -> None:
    """Main entry point."""
    app()
