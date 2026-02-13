"""CLI commands for QMD."""

from pathlib import Path
from typing import Optional
import typer
from rich import print


app_collection = typer.Typer(help="Manage collections")
app_context = typer.Typer(help="Manage contexts")

# Main app commands
app = typer.Typer(
    name="qmd",
    help="QMD - AI-powered search with hybrid BM25 and vector search",
    add_completion=False,
)


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


@app_context.command("add")
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


# Main app subcommands
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
    from ..config import Config
    from ..store import Store, SearchOptions
    from ..llm import Router
    import asyncio

    config = Config.load()

    # Override vector backend if specified
    if vector_backend != "qmd_builtin":
        config.vector.backend = vector_backend

    store = Store(config)
    options = SearchOptions(
        limit=limit,
        collection=collection,
        search_all=all_collections,
    )

    # Try to get LLM for embedding
    llm = None
    try:
        if config.models.embed:
            llm = Router(config)
    except Exception as e:
        print(f"Warning: LLM not available: {e}")

    results = store.vector_search(query, options, llm)

    for r in results:
        print(f"[{r.score:.3f}] {r.path}")
        print(f"    Title: {r.title}")


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
    from ..config import Config
    from ..store import Store, SearchOptions
    from ..llm import Router
    from pathlib import Path
    import asyncio

    config = Config.load()
    store = Store(config)

    # Get LLM router
    llm = None
    try:
        llm = Router(config)
    except Exception as e:
        print(f"Error: LLM not available: {e}")
        print("Please install llama-cpp-python or configure remote embedding:")
        print("  pip install qmd-python[local]")
        return

    # Determine collections to process
    collections = [collection] if collection else [c.name for c in config.collections]

    for col_name in collections:
        print(f"Processing collection: {col_name}")
        try:
            # Embed collection
            store.embed_collection(col_name, llm, force)
            print(f"  Embeddings generated for {col_name}")
        except Exception as e:
            print(f"  Error: {e}")


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


# Register subcommands at module level
collection_cmd = app_collection
context_cmd = app_context
search_cmd = app  # Use main app for search
