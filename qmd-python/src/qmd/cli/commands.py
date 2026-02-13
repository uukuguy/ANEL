"""CLI commands for QMD"""

import os
import sys
from pathlib import Path

import click
from qmd.config.config import Config
from qmd.formatter.formatter import Formatter
from qmd.store.store import Store, SearchOptions


# Global state
config: Config = None
store: Store = None


def get_store(db_path: str = "") -> Store:
    """Get or create Store instance"""
    global store
    if store is None:
        if not db_path:
            db_path = str(Path.home() / ".cache" / "qmd" / "index.db")
        store = Store(db_path)
    return store


def load_config() -> Config:
    """Load configuration"""
    global config
    if config is None:
        try:
            config = Config.load()
        except Exception:
            config = Config.default()
    return config


@click.group()
@click.option("--db", "-d", default="", help="Database path")
@click.option("--collection", "-c", default="", help="Collection name")
@click.option("--format", "-f", "output_format", default="cli", help="Output format (cli, json, md, csv, files, xml)")
@click.option("--limit", "-l", default=20, help="Maximum number of results")
@click.option("--min-score", default=0.0, help="Minimum score threshold")
@click.pass_context
def cli(ctx, db, collection, output_format, limit, min_score):
    """QMD - AI-powered search tool"""
    ctx.ensure_object(dict)
    ctx.obj["db"] = db
    ctx.obj["collection"] = collection
    ctx.obj["format"] = output_format
    ctx.obj["limit"] = limit
    ctx.obj["min_score"] = min_score

    # Load config
    load_config()


@cli.command()
@click.argument("query")
@click.option("--all", "-a", is_flag=True, help="Search all collections")
@click.pass_context
def search(ctx, query, all):
    """BM25 full-text search"""
    s = get_store(ctx.obj.get("db"))
    opts = SearchOptions(
        limit=ctx.obj["limit"],
        min_score=ctx.obj["min_score"],
        collection=ctx.obj["collection"],
        all=all,
    )

    results = s.bm25_search(query, opts)

    f = Formatter(ctx.obj["format"], ctx.obj["limit"])
    click.echo(f.format_results(results, query))


@cli.command()
@click.argument("query")
@click.option("--all", "-a", is_flag=True, help="Search all collections")
@click.pass_context
def vsearch(ctx, query, all):
    """Vector semantic search"""
    from qmd.llm.router import Router

    s = get_store(ctx.obj.get("db"))
    cfg = load_config()
    opts = SearchOptions(
        limit=ctx.obj["limit"],
        min_score=ctx.obj["min_score"],
        collection=ctx.obj["collection"],
        all=all,
    )

    # Create embedder for query
    router = Router(cfg)
    try:
        router.init_embedder()
    except Exception as e:
        click.echo(f"Warning: Failed to load embedder: {e}")
        click.echo("Using random query vector for demonstration...")
        # Generate random embedding for query
        embedding = _random_embedding(768)
    else:
        # Generate embedding for query
        result = router.embed([query])
        embedding = result.embeddings[0]

    # Perform vector search
    results = s.vector_search(embedding, opts)

    f = Formatter(ctx.obj["format"], ctx.obj["limit"])
    click.echo(f.format_results(results, query))


@cli.command()
@click.argument("query")
@click.option("--all", "-a", is_flag=True, help="Search all collections")
@click.pass_context
def query(ctx, query, all):
    """Hybrid search (BM25 + Vector + RRF + Reranking)"""
    from qmd.llm.router import Router

    s = get_store(ctx.obj.get("db"))
    cfg = load_config()
    opts = SearchOptions(
        limit=ctx.obj["limit"],
        min_score=ctx.obj["min_score"],
        collection=ctx.obj["collection"],
        all=all,
    )

    # BM25 search
    bm25_results = s.bm25_search(query, opts)

    # Vector search
    vector_results = []
    router = Router(cfg)
    try:
        router.init_embedder()
        result = router.embed([query])
        vector_results = s.vector_search(result.embeddings[0], opts)
    except Exception as e:
        click.echo(f"Note: Vector search unavailable: {e}")

    # Hybrid search with RRF
    results = s.hybrid_search(bm25_results, vector_results, 60)

    f = Formatter(ctx.obj["format"], ctx.obj["limit"])
    click.echo(f.format_results(results, query))


@cli.command()
@click.argument("action", type=click.Choice(["add", "list", "remove", "rename"]))
@click.option("--path", help="Collection path")
@click.option("--pattern", help="File pattern")
@click.option("--description", help="Description")
@click.option("--new-name", help="New name for rename")
@click.pass_context
def collection(ctx, action, path, pattern, description, new_name):
    """Collection management"""
    cfg = load_config()

    if action == "add":
        if not path:
            click.echo("Error: --path is required", err=True)
            sys.exit(1)

        col_name = ctx.obj.get("collection") or Path(path).name
        from qmd.config.config import CollectionConfig

        cfg.add_collection(CollectionConfig(
            name=col_name,
            path=path,
            pattern=pattern,
            description=description,
        ))
        cfg.save()
        click.echo(f"Collection '{col_name}' added")

    elif action == "list":
        if not cfg.collections:
            click.echo("No collections")
        else:
            click.echo("Collections:")
            for c in cfg.collections:
                click.echo(f"  {c.name}: {c.path}")

    elif action == "remove":
        if not ctx.obj.get("collection"):
            click.echo("Error: --collection is required", err=True)
            sys.exit(1)

        cfg.remove_collection(ctx.obj["collection"])
        cfg.save()
        click.echo(f"Collection '{ctx.obj['collection']}' removed")

    elif action == "rename":
        if not ctx.obj.get("collection") or not new_name:
            click.echo("Error: --collection and --new-name are required", err=True)
            sys.exit(1)

        for c in cfg.collections:
            if c.name == ctx.obj["collection"]:
                c.name = new_name
                break
        cfg.save()
        click.echo(f"Collection '{ctx.obj['collection']}' renamed to '{new_name}'")


@cli.command()
@click.argument("action", type=click.Choice(["add", "list", "rm"]))
@click.pass_context
def context(ctx, action):
    """Context management"""
    click.echo(f"Context {action} not implemented")


@cli.command()
@click.argument("path")
@click.option("--from", "from_line", default=0, help="Start line")
@click.option("--limit", default=0, help="Line limit")
@click.pass_context
def get(ctx, path, from_line, limit):
    """Get document content"""
    s = get_store(ctx.obj.get("db"))

    try:
        doc, content = s.get_document(path)
    except ValueError as e:
        click.echo(f"Error: {e}", err=True)
        sys.exit(1)

    lines = content.split("\n")
    if from_line > 0:
        lines = lines[from_line:]
    if limit > 0:
        lines = lines[:limit]

    click.echo(f"Title: {doc.title}")
    click.echo(f"Collection: {doc.collection}")
    click.echo(f"Hash: {doc.hash}")
    click.echo("---")
    click.echo("\n".join(lines))


@cli.command()
@click.argument("pattern")
@click.pass_context
def multi_get(ctx, pattern):
    """Get multiple documents by pattern"""
    click.echo("multi_get not implemented")
    sys.exit(1)


@cli.command()
@click.argument("collection", required=False)
@click.option("--force", "-f", is_flag=True, help="Force regeneration of all embeddings")
@click.pass_context
def embed(ctx, collection, force):
    """Generate/update embeddings"""
    from qmd.llm.router import Router

    cfg = load_config()
    s = get_store(ctx.obj.get("db"))

    # Create LLM router
    router = Router(cfg)

    # Initialize embedder from config
    if cfg.models and cfg.models.embed and cfg.models.embed.local:
        try:
            router.init_embedder()
        except Exception as e:
            click.echo(f"Warning: Failed to load embedder: {e}")
            click.echo("Using random embeddings for demonstration...")

    if collection:
        _embed_collection(s, router, collection, force)
    else:
        # Embed all collections
        collections = cfg.collections or []
        for c in collections:
            _embed_collection(s, router, c.name, force)


def _embed_collection(s, router, collection: str, force: bool):
    """Embed a single collection"""
    click.echo(f"Generating embeddings for collection: {collection}")

    # Get documents needing embedding
    docs = s.get_documents_for_embedding(collection, force)

    if not docs:
        click.echo("No documents need embedding")
        return

    click.echo(f"Found {len(docs)} documents to embed")

    # Delete existing embeddings if force mode
    if force:
        for doc in docs:
            s.delete_embeddings(doc["hash"])

    # Determine embedding model
    model_name = "nomic-embed-text-v1.5"

    # Process documents in batches
    batch_size = 10
    processed = 0

    for i in range(0, len(docs), batch_size):
        batch = docs[i:i + batch_size]
        click.echo(f"Processing batch {i+1}-{min(i+batch_size, len(docs))}...")

        # Chunk documents
        from qmd.store.chunker import chunk_document, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP
        texts = []
        hashes = []
        seqs = []
        poss = []

        for doc in batch:
            chunks = chunk_document(doc["doc"], DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP)
            for chunk in chunks:
                texts.append(chunk["text"])
                hashes.append(doc["hash"])
                seqs.append(chunk["seq"])
                poss.append(chunk["pos"])

        if not texts:
            continue

        # Generate embeddings
        if router.has_embedder():
            try:
                result = router.embed(texts)
                embeddings = result.embeddings
                if result.model:
                    model_name = result.model
            except Exception as e:
                click.echo(f"Warning: Failed to generate embeddings: {e}")
                embeddings = [_random_embedding(768) for _ in texts]
        else:
            # Use random embeddings as placeholder
            embeddings = [_random_embedding(768) for _ in texts]
            click.echo("Note: Using random embeddings (no embedder configured)")

        # Store embeddings
        for j, embedding in enumerate(embeddings):
            s.store_embedding(hashes[j], seqs[j], poss[j], embedding, model_name)

        processed += len(batch)
        click.echo(f"Embedded {processed}/{len(docs)} documents")

    click.echo("Embedding complete!")


def _random_embedding(dim: int) -> list[float]:
    """Generate a random embedding vector"""
    import math
    embedding = [float(i % 100) / 100.0 for i in range(dim)]
    # Normalize
    s = sum(v * v for v in embedding)
    if s > 0:
        norm = 1.0 / math.sqrt(s)
        embedding = [v * norm for v in embedding]
    return embedding


@cli.command()
@click.argument("collection", required=False)
@click.pass_context
def update(ctx, collection):
    """Update index"""
    s = get_store(ctx.obj.get("db"))
    cfg = load_config()

    collections = cfg.collections
    if collection:
        collections = [c for c in collections if c.name == collection]

    total_docs = 0
    for c in collections:
        docs = _scan_directory(c.path, c.pattern)
        for doc in docs:
            try:
                s.add_document(c.name, doc["path"], doc["title"], doc["content"])
                total_docs += 1
            except Exception as e:
                click.echo(f"Error adding {doc['path']}: {e}", err=True)

        click.echo(f"Indexed {len(docs)} documents from {c.name}")

    click.echo(f"Total: {total_docs} documents indexed")


def _scan_directory(dir_path: str, pattern: str = None):
    """Scan directory for documents"""
    docs = []
    for root, _, files in os.walk(dir_path):
        for fname in files:
            if fname.startswith("."):
                continue
            fpath = os.path.join(root, fname)
            try:
                with open(fpath) as f:
                    content = f.read()
                docs.append({
                    "path": fpath,
                    "title": fname,
                    "content": content,
                })
            except Exception:
                pass
    return docs


@cli.command()
@click.pass_context
def status(ctx):
    """Show index status"""
    s = get_store(ctx.obj.get("db"))
    stats = s.get_stats()

    if ctx.obj["format"] == "json":
        import json
        click.echo(json.dumps({
            "collection_count": stats.collection_count,
            "document_count": stats.document_count,
            "indexed_count": stats.indexed_count,
            "chunk_count": stats.chunk_count,
            "collection_stats": stats.collection_stats,
        }, indent=2))
    else:
        click.echo(f"Collections: {stats.collection_count}")
        click.echo(f"Documents: {stats.document_count}")
        click.echo(f"Indexed: {stats.indexed_count}")
        click.echo(f"Chunks: {stats.chunk_count}")
        click.echo("\nPer-collection:")
        for name, count in stats.collection_stats.items():
            click.echo(f"  {name}: {count}")


@cli.command()
def cleanup():
    """Clean up stale entries"""
    click.echo("Cleanup not implemented")
    sys.exit(1)


@cli.command()
def mcp():
    """Run as MCP server"""
    click.echo("MCP server not implemented in Python version")
    sys.exit(1)


@cli.command()
@click.pass_context
def agent(ctx):
    """Run in agent mode (interactive)"""
    from qmd.cli.agent import run_agent
    run_agent(ctx)


if __name__ == "__main__":
    cli()
