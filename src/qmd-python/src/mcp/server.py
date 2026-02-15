"""MCP server for QMD."""

import json
import sys
from pathlib import Path
from typing import Optional


class McpServer:
    """MCP server with Store integration."""

    def __init__(self, store=None, config=None):
        self.store = store
        self.config = config

    def handle_message(self, message: dict) -> Optional[dict]:
        """Handle incoming MCP message."""
        method = message.get("method", "")
        msg_id = message.get("id")

        if method == "initialize":
            return self._handle_initialize(msg_id)
        elif method == "tools/list":
            return self._handle_tools_list(msg_id)
        elif method == "tools/call":
            return self._handle_tools_call(msg_id, message)
        return None

    def _handle_initialize(self, msg_id) -> dict:
        return {
            "jsonrpc": "2.0",
            "id": msg_id,
            "result": {
                "name": "qmd-python",
                "version": "0.1.0",
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {},
                },
            },
        }

    def _handle_tools_list(self, msg_id) -> dict:
        return {
            "jsonrpc": "2.0",
            "id": msg_id,
            "result": {
                "tools": [
                    {
                        "name": "search",
                        "description": "BM25 full-text search",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {"type": "string", "description": "Search query"},
                                "limit": {"type": "integer", "description": "Max results"},
                                "collection": {"type": "string", "description": "Collection name"},
                            },
                            "required": ["query"],
                        },
                    },
                    {
                        "name": "vsearch",
                        "description": "Vector semantic search",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {"type": "string", "description": "Search query"},
                                "limit": {"type": "integer", "description": "Max results"},
                                "collection": {"type": "string", "description": "Collection name"},
                            },
                            "required": ["query"],
                        },
                    },
                    {
                        "name": "query",
                        "description": "Hybrid search with reranking",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {"type": "string", "description": "Search query"},
                                "limit": {"type": "integer", "description": "Max results"},
                                "collection": {"type": "string", "description": "Collection name"},
                            },
                            "required": ["query"],
                        },
                    },
                    {
                        "name": "get",
                        "description": "Get document content",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "path": {"type": "string", "description": "File path"},
                                "from": {"type": "integer", "description": "Start line"},
                                "limit": {"type": "integer", "description": "Max lines"},
                            },
                            "required": ["path"],
                        },
                    },
                    {
                        "name": "status",
                        "description": "Show index status",
                    },
                ],
            },
        }

    def _handle_tools_call(self, msg_id, message: dict) -> dict:
        params = message.get("params", {})
        tool_name = params.get("name", "")
        args = params.get("arguments", {})

        content = ""
        is_error = False

        try:
            if tool_name == "search":
                content = self._tool_search(args)
            elif tool_name == "vsearch":
                content = self._tool_vsearch(args)
            elif tool_name == "query":
                content = self._tool_query(args)
            elif tool_name == "get":
                content = self._tool_get(args)
            elif tool_name == "status":
                content = self._tool_status()
            else:
                content = f"Unknown tool: {tool_name}"
                is_error = True
        except Exception as e:
            content = f"Error: {e}"
            is_error = True

        return {
            "jsonrpc": "2.0",
            "id": msg_id,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": content,
                    }
                ],
                "isError": is_error,
            },
        }

    def _parse_search_args(self, args: dict):
        from ..store import SearchOptions

        query = args.get("query", "")
        limit = int(args.get("limit", 20))
        collection = args.get("collection")

        options = SearchOptions(
            limit=limit,
            collection=collection,
            search_all=collection is None,
        )
        return query, options

    def _tool_search(self, args: dict) -> str:
        query, options = self._parse_search_args(args)
        if not query:
            raise ValueError("query is required")

        results = self.store.bm25_search(query, options)
        return self._format_search_results(results)

    def _tool_vsearch(self, args: dict) -> str:
        query, options = self._parse_search_args(args)
        if not query:
            raise ValueError("query is required")

        results = self.store.vector_search(query, options)
        return self._format_search_results(results)

    def _tool_query(self, args: dict) -> str:
        query, options = self._parse_search_args(args)
        if not query:
            raise ValueError("query is required")

        results = self.store.hybrid_search(query, options)
        return self._format_search_results(results)

    def _tool_get(self, args: dict) -> str:
        path = args.get("path", "")
        if not path:
            raise ValueError("path is required")

        from_line = int(args.get("from", 0))
        limit = int(args.get("limit", 0))

        file_path = Path(path)
        if not file_path.exists():
            raise FileNotFoundError(f"File not found: {path}")

        content = file_path.read_text(encoding="utf-8", errors="replace")
        lines = content.split("\n")

        if from_line > 0:
            lines = lines[from_line:]
        if limit > 0:
            lines = lines[:limit]

        return "\n".join(lines)

    def _tool_status(self) -> str:
        stats = self.store.get_stats()

        parts = [
            "Index Status",
            "============",
            f"Collections: {stats.collection_count}",
            f"Documents:   {stats.document_count}",
            f"Indexed:     {stats.indexed_count}",
            f"Pending:     {stats.pending_count}",
        ]

        if stats.collection_stats:
            parts.append("\nPer-collection:")
            for name, count in stats.collection_stats.items():
                parts.append(f"  {name}: {count} documents")

        return "\n".join(parts)

    def _format_search_results(self, results: list) -> str:
        if not results:
            return "No results found."

        parts = [f"Found {len(results)} results:\n"]
        for i, r in enumerate(results, 1):
            parts.append(f"{i}. [{r.score:.3f}] {r.path}")
            if r.title:
                parts.append(f"   Title: {r.title}")
            if r.collection:
                parts.append(f"   Collection: {r.collection}")

        return "\n".join(parts)


async def run_server(transport: str = "stdio", port: int = 8080) -> None:
    """Run MCP server."""
    if transport == "stdio":
        await run_stdio_server()
    elif transport == "sse":
        await run_sse_server(port)
    else:
        raise ValueError(f"Unknown transport: {transport}")


async def run_stdio_server() -> None:
    """Run MCP server with stdio transport."""
    from ..config import Config
    from ..store import Store

    print("Starting MCP server (stdio)", file=sys.stderr)

    config = Config.load()
    store = Store(config)
    server = McpServer(store=store, config=config)

    for line in sys.stdin:
        if line.strip():
            try:
                message = json.loads(line)
                response = server.handle_message(message)
                if response:
                    print(json.dumps(response), flush=True)
            except json.JSONDecodeError:
                pass


async def run_sse_server(port: int) -> None:
    """Run MCP server with SSE transport."""
    print(f"SSE transport not yet implemented, port {port}")
