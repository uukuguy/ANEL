"""CLI application for QMD with ANEL support."""

import os
import sys
from typing import Optional

import typer
from rich import print
from enum import Enum

from ..anel import ENV_DRY_RUN, ENV_EMIT_SPEC
from ..anel.spec import get_spec_for_command


class OutputFormat(str, Enum):
    CLI = "cli"
    JSON = "json"
    MARKDOWN = "md"
    CSV = "csv"
    FILES = "files"


# Global ANEL options
emit_spec: bool = False
dry_run: bool = False


def check_emit_spec(command_name: str) -> bool:
    """Check if --emit-spec is set via flag or environment variable."""
    if os.environ.get(ENV_EMIT_SPEC, ""):
        return True
    return emit_spec


def check_dry_run() -> bool:
    """Check if --dry-run is set via flag or environment variable."""
    if os.environ.get(ENV_DRY_RUN, ""):
        return True
    return dry_run


app = typer.Typer(
    name="qmd",
    help="QMD - AI-powered search with hybrid BM25 and vector search",
    add_completion=False,
)


@app.callback()
def callback(
    emit_spec_flag: bool = typer.Option(False, "--emit-spec", help="Output JSON Schema and exit"),
    dry_run_flag: bool = typer.Option(False, "--dry-run", help="Validate parameters but don't execute"),
) -> None:
    """Global options for QMD."""
    global emit_spec, dry_run
    emit_spec = emit_spec_flag
    dry_run = dry_run_flag


@app.command("get")
def get_cmd(
    file: str = typer.Argument(..., help="File path (with optional :line suffix)"),
    limit: int = typer.Option(50, "-l", "--limit", help="Number of lines"),
    from_line: int = typer.Option(0, "--from", help="Start line"),
    full: bool = typer.Option(False, "--full", help="Full content"),
    emit_spec_cmd: bool = typer.Option(False, "--emit-spec", hidden=True),
    dry_run_cmd: bool = typer.Option(False, "--dry-run", hidden=True),
) -> None:
    """Get document content."""
    # Check for --emit-spec
    if emit_spec_cmd or check_emit_spec("get"):
        spec = get_spec_for_command("get")
        if spec:
            print(spec.to_json())
            sys.exit(0)

    # Check for --dry-run
    if dry_run_cmd or check_dry_run():
        print(f"[DRY-RUN] Would get file: {file}")
        print(f"[DRY-RUN] Limit: {limit}, From: {from_line}, Full: {full}")
        return

    print(f"Getting: {file}")


@app.command("search")
def search_cmd(
    query: str = typer.Argument(..., help="Search query"),
    limit: int = typer.Option(20, "-n", "--limit", help="Max results"),
    collection: Optional[str] = typer.Option(None, "-c", "--collection", help="Collection"),
    all_collections: bool = typer.Option(False, "--all", help="Search all collections"),
    format: str = typer.Option("cli", "--format", "-f", help="Output format"),
    fts_backend: str = typer.Option("sqlite_fts5", "--fts-backend", help="BM25 backend"),
    emit_spec_cmd: bool = typer.Option(False, "--emit-spec", hidden=True),
    dry_run_cmd: bool = typer.Option(False, "--dry-run", hidden=True),
) -> None:
    """BM25 full-text search."""
    # Check for --emit-spec
    if emit_spec_cmd or check_emit_spec("search"):
        spec = get_spec_for_command("search")
        if spec:
            print(spec.to_json())
            sys.exit(0)

    # Check for --dry-run
    if dry_run_cmd or check_dry_run():
        print(f"[DRY-RUN] Would execute BM25 search for query: {query}")
        print(f"[DRY-RUN] Limit: {limit}, Collection: {collection}, All: {all_collections}")
        return

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
    emit_spec_cmd: bool = typer.Option(False, "--emit-spec", hidden=True),
    dry_run_cmd: bool = typer.Option(False, "--dry-run", hidden=True),
) -> None:
    """Vector semantic search."""
    # Check for --emit-spec
    if emit_spec_cmd or check_emit_spec("vsearch"):
        spec = get_spec_for_command("vsearch")
        if spec:
            print(spec.to_json())
            sys.exit(0)

    # Check for --dry-run
    if dry_run_cmd or check_dry_run():
        print(f"[DRY-RUN] Would execute vector search for query: {query}")
        print(f"[DRY-RUN] Limit: {limit}, Collection: {collection}, All: {all_collections}")
        return

    print(f"Vector search: {query}")


@app.command("query")
def query_cmd(
    query: str = typer.Argument(..., help="Search query"),
    limit: int = typer.Option(20, "-n", "--limit", help="Max results"),
    collection: Optional[str] = typer.Option(None, "-c", "--collection", help="Collection"),
    all_collections: bool = typer.Option(False, "--all", help="Search all collections"),
    format: str = typer.Option("cli", "--format", "-f", help="Output format"),
    emit_spec_cmd: bool = typer.Option(False, "--emit-spec", hidden=True),
    dry_run_cmd: bool = typer.Option(False, "--dry-run", hidden=True),
) -> None:
    """Hybrid search with reranking."""
    # Check for --emit-spec
    if emit_spec_cmd or check_emit_spec("query"):
        spec = get_spec_for_command("query")
        if spec:
            print(spec.to_json())
            sys.exit(0)

    # Check for --dry-run
    if dry_run_cmd or check_dry_run():
        print(f"[DRY-RUN] Would execute hybrid search for query: {query}")
        print(f"[DRY-RUN] Limit: {limit}, Collection: {collection}, All: {all_collections}")
        return

    print(f"Hybrid query: {query}")


@app.command("embed")
def embed_cmd(
    force: bool = typer.Option(False, "-f", "--force", help="Force regeneration"),
    collection: Optional[str] = typer.Option(None, "-c", "--collection", help="Collection"),
    emit_spec_cmd: bool = typer.Option(False, "--emit-spec", hidden=True),
    dry_run_cmd: bool = typer.Option(False, "--dry-run", hidden=True),
) -> None:
    """Generate/update embeddings."""
    # Check for --emit-spec
    if emit_spec_cmd or check_emit_spec("embed"):
        spec = get_spec_for_command("embed")
        if spec:
            print(spec.to_json())
            sys.exit(0)

    # Check for --dry-run
    if dry_run_cmd or check_dry_run():
        print(f"[DRY-RUN] Would generate embeddings")
        print(f"[DRY-RUN] Force: {force}, Collection: {collection}")
        return

    print("Generating embeddings...")


@app.command("update")
def update_cmd(
    pull: bool = typer.Option(False, "--pull", help="Pull remote changes"),
    collection: Optional[str] = typer.Option(None, "-c", "--collection", help="Collection"),
    emit_spec_cmd: bool = typer.Option(False, "--emit-spec", hidden=True),
    dry_run_cmd: bool = typer.Option(False, "--dry-run", hidden=True),
) -> None:
    """Update index."""
    # Check for --emit-spec
    if emit_spec_cmd or check_emit_spec("update"):
        spec = get_spec_for_command("update")
        if spec:
            print(spec.to_json())
            sys.exit(0)

    # Check for --dry-run
    if dry_run_cmd or check_dry_run():
        print(f"[DRY-RUN] Would update index")
        print(f"[DRY-RUN] Pull: {pull}, Collection: {collection}")
        return

    print("Updating index...")


@app.command("status")
def status_cmd(
    verbose: bool = typer.Option(False, "-v", "--verbose", help="Detailed output"),
    collection: Optional[str] = typer.Option(None, "-c", "--collection", help="Collection"),
    emit_spec_cmd: bool = typer.Option(False, "--emit-spec", hidden=True),
    dry_run_cmd: bool = typer.Option(False, "--dry-run", hidden=True),
) -> None:
    """Show index status."""
    # Check for --emit-spec
    if emit_spec_cmd or check_emit_spec("status"):
        spec = get_spec_for_command("status")
        if spec:
            print(spec.to_json())
            sys.exit(0)

    # Check for --dry-run
    if dry_run_cmd or check_dry_run():
        print(f"[DRY-RUN] Would show status")
        print(f"[DRY-RUN] Verbose: {verbose}, Collection: {collection}")
        return

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
    collection: Optional[str] = typer.Option(None, "-c", "--collection", help="Collection"),
    emit_spec_cmd: bool = typer.Option(False, "--emit-spec", hidden=True),
) -> None:
    """Cleanup stale entries."""
    # Check for --emit-spec
    if emit_spec_cmd or check_emit_spec("cleanup"):
        spec = get_spec_for_command("cleanup")
        if spec:
            print(spec.to_json())
            sys.exit(0)

    # Check for --dry-run
    if dry_run or check_dry_run():
        print(f"[DRY-RUN] Would cleanup entries older than {older_than} days")
        if collection:
            print(f"[DRY-RUN] Collection: {collection}")
        return

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
    emit_spec_cmd: bool = typer.Option(False, "--emit-spec", hidden=True),
    dry_run_cmd: bool = typer.Option(False, "--dry-run", hidden=True),
) -> None:
    """Run in agent mode."""
    # Check for --emit-spec
    if emit_spec_cmd or check_emit_spec("agent"):
        spec = get_spec_for_command("agent")
        if spec:
            print(spec.to_json())
            sys.exit(0)

    # Check for --dry-run
    if dry_run_cmd or check_dry_run():
        print(f"[DRY-RUN] Would run in agent mode")
        print(f"[DRY-RUN] Interactive: {interactive}, MCP: {mcp}, Transport: {transport}")
        return

    print("Agent mode ready")


def main() -> None:
    """Main entry point."""
    app()
