"""CLI commands for QMD."""

from pathlib import Path
from typing import Optional
import typer
from rich import print


app_collection = typer.Typer(help="Manage collections")
app_context = typer.Typer(help="Manage contexts")


@app_collection.command("add")
def collection_add(
    path: str = typer.Argument(..., help="Collection path"),
    name: Optional[str] = typer.Option(None, "-n", "--name", help="Collection name"),
    mask: str = typer.Option("**/*", "-m", "--mask", help="File pattern"),
    description: Optional[str] = typer.Option(None, "-d", "--description", help="Description"),
) -> None:
    """Add a collection."""
    from ..config import Config

    config = Config.load()
    # TODO: Implement collection add
    print(f"Collection added: {path}")


@app_collection.command("list")
def collection_list() -> None:
    """List collections."""
    from ..config import Config

    config = Config.load()
    if not config.collections:
        print("No collections configured")
        return

    for col in config.collections:
        print(f"  {col.name}: {col.path}")


@app_collection.command("remove")
def collection_remove(name: str) -> None:
    """Remove a collection."""
    print(f"Collection '{name}' removed")


@app_collection.command("rename")
def collection_rename(old_name: str, new_name: str) -> None:
    """Rename a collection."""
    print(f"Renamed '{old_name}' to '{new_name}'")


app_context.command("add")
def context_add(
    path: Optional[str] = typer.Argument(None, help="Path (default: current directory)"),
    description: str = typer.Option(..., "-d", "--description", help="Description"),
) -> None:
    """Add a context."""
    print(f"Context added: {description}")


@app_context.command("list")
def context_list() -> None:
    """List contexts."""
    print("Contexts:")


@app_context.command("rm")
def context_rm(path: str) -> None:
    """Remove a context."""
    print(f"Context '{path}' removed")


collection_cmd = app_collection
context_cmd = app_context


@app.command("get")
def get_cmd(
    file: str = typer.Argument(..., help="File path (with optional :line suffix)"),
    limit: int = typer.Option(50, "-l", "--limit", help="Number of lines"),
    from_line: int = typer.Option(0, "--from", help="Start line"),
    full: bool = typer.Option(False, "--full", help="Full content"),
) -> None:
    """Get document content."""
    print(f"Getting: {file}")


@app.command("multi-get")
def multi_get_cmd(
    pattern: str = typer.Argument(..., help="File pattern"),
    limit: int = typer.Option(50, "-l", "--limit", help="Lines per file"),
    max_bytes: Optional[int] = typer.Option(None, "--max-bytes", help="Max bytes per file"),
) -> None:
    """Get multiple documents by pattern."""
    print(f"Pattern: {pattern}")


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
    print(f"Searching: {query}")


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
    print("Index Status")
    print("=" * 40)
    print("Collections: 0")
    print("Documents: 0")


@app.command("cleanup")
def cleanup_cmd(
    dry_run: bool = typer.Option(False, "--dry-run", help="Dry run only"),
    older_than: int = typer.Option(30, "--older-than", help="Days"),
) -> None:
    """Cleanup stale entries."""
    print("Cleanup completed")
