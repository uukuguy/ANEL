"""Agent mode - interactive search with intelligent routing"""

import sys

from enum import Enum

from qmd.formatter.formatter import Formatter
from qmd.store.store import SearchOptions


class QueryIntent(str, Enum):
    """Query intent types"""
    KEYWORD = "keyword"  # BM25
    SEMANTIC = "semantic"  # Vector
    HYBRID = "hybrid"  # Hybrid


def classify_intent(query: str) -> QueryIntent:
    """Classify query intent using rule-based approach"""
    query_lower = query.lower()

    # Check for natural language patterns
    nl_patterns = ["explain", "describe", "what is", "how does", "why", "meaning"]
    for p in nl_patterns:
        if p in query_lower:
            return QueryIntent.SEMANTIC

    # Check for technical terms (likely keyword search)
    tech_patterns = ["error", "exception", "api", "function", "class", "method"]
    for p in tech_patterns:
        if p in query_lower:
            return QueryIntent.KEYWORD

    # Default to hybrid
    return QueryIntent.HYBRID


def run_agent(ctx):
    """Run interactive agent"""
    from qmd.cli.commands import get_store

    store = get_store(ctx.obj.get("db"))

    print("QMD Agent - Interactive Search")
    print("Type 'quit' or 'exit' to exit")
    print("Type '/bm25', '/vector', or '/hybrid' to force search method")
    print()

    force_method = ""

    while True:
        try:
            query = input("> ").strip()
        except (EOFError, KeyboardInterrupt):
            break

        if not query:
            continue

        if query.lower() in ("quit", "exit"):
            break

        # Check for forced method
        if query.startswith("/bm25"):
            force_method = "bm25"
            query = query.replace("/bm25", "").strip()
            print("Forced BM25 search")
        elif query.startswith("/vector"):
            force_method = "vector"
            query = query.replace("/vector", "").strip()
            print("Forced Vector search")
        elif query.startswith("/hybrid"):
            force_method = "hybrid"
            query = query.replace("/hybrid", "").strip()
            print("Forced Hybrid search")

        if not query:
            continue

        # Classify intent
        intent = classify_intent(query)

        if force_method:
            if force_method == "bm25":
                intent = QueryIntent.KEYWORD
            elif force_method == "vector":
                intent = QueryIntent.SEMANTIC
            elif force_method == "hybrid":
                intent = QueryIntent.HYBRID
            force_method = ""

        opts = SearchOptions(
            limit=ctx.obj["limit"],
            min_score=ctx.obj["min_score"],
            all=True,
        )

        match intent:
            case QueryIntent.KEYWORD:
                results = store.bm25_search(query, opts)
            case QueryIntent.SEMANTIC:
                print("Vector search requires embedding model setup")
                continue
            case QueryIntent.HYBRID:
                bm25_results = store.bm25_search(query, opts)
                results = store.hybrid_search(bm25_results, [], 60)

        f = Formatter(ctx.obj["format"], ctx.obj["limit"])
        print(f.format_results(results, query))

    print("Goodbye!")
