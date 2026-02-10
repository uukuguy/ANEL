"""MCP server for QMD."""

from typing import Dict
import asyncio
import json


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
    import sys

    # TODO: Implement proper MCP stdio server
    print("Starting MCP server (stdio)", file=sys.stderr)

    # Read JSON-RPC messages from stdin
    for line in sys.stdin:
        if line.strip():
            try:
                message = json.loads(line)
                response = handle_message(message)
                if response:
                    print(json.dumps(response))
            except json.JSONDecodeError:
                pass


async def run_sse_server(port: int) -> None:
    """Run MCP server with SSE transport."""
    # TODO: Implement SSE transport
    print(f"SSE transport not yet implemented, port {port}")


def handle_message(message: dict) -> dict:
    """Handle incoming MCP message."""
    # TODO: Implement MCP message handling
    method = message.get("method", "")

    if method == "initialize":
        return {
            "jsonrpc": "2.0",
            "id": message.get("id"),
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

    if method == "tools/list":
        return {
            "jsonrpc": "2.0",
            "id": message.get("id"),
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

    return None
