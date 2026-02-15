"""Tests for MCP Server: handle_message dispatching, tool calls, JSON-RPC format."""

import json
import sys
import pytest
from unittest.mock import MagicMock

sys.path.insert(0, str(__import__("pathlib").Path(__file__).parent.parent / "src"))

from mcp.server import McpServer
from store import SearchResult, SearchOptions, IndexStats


# ---------------------------------------------------------------------------
# Fix relative import: _parse_search_args uses ``from ..store import SearchOptions``
# which fails when ``mcp`` is imported as a top-level package via sys.path.
# We monkeypatch the method to use the already-imported SearchOptions.
# ---------------------------------------------------------------------------

_original_parse = McpServer._parse_search_args


def _patched_parse_search_args(self, args: dict):
    query = args.get("query", "")
    limit = int(args.get("limit", 20))
    collection = args.get("collection")
    options = SearchOptions(
        limit=limit,
        collection=collection,
        search_all=collection is None,
    )
    return query, options


McpServer._parse_search_args = _patched_parse_search_args


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

@pytest.fixture
def mock_store():
    """Create a MagicMock store with sensible defaults."""
    store = MagicMock()
    store.bm25_search.return_value = [
        SearchResult(path="/a.md", collection="docs", score=1.5, lines=10, title="Doc A", hash="aaa"),
    ]
    store.vector_search.return_value = [
        SearchResult(path="/b.md", collection="docs", score=0.92, lines=20, title="Doc B", hash="bbb"),
    ]
    store.hybrid_search.return_value = [
        SearchResult(path="/c.md", collection="notes", score=0.88, lines=5, title="Doc C", hash="ccc"),
    ]
    store.get_stats.return_value = IndexStats(
        collection_count=2,
        document_count=42,
        indexed_count=40,
        pending_count=2,
        collection_stats={"docs": 30, "notes": 12},
    )
    return store


@pytest.fixture
def server(mock_store):
    return McpServer(store=mock_store)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _make_request(method: str, msg_id: int = 1, params: dict | None = None) -> dict:
    req = {"jsonrpc": "2.0", "id": msg_id, "method": method}
    if params is not None:
        req["params"] = params
    return req


def _assert_jsonrpc(response: dict, msg_id: int):
    """Assert standard JSON-RPC 2.0 envelope."""
    assert response["jsonrpc"] == "2.0"
    assert response["id"] == msg_id
    assert "result" in response


# ---------------------------------------------------------------------------
# 1. handle_message dispatching
# ---------------------------------------------------------------------------

class TestHandleMessageDispatch:
    def test_dispatch_initialize(self, server):
        resp = server.handle_message(_make_request("initialize", msg_id=10))
        _assert_jsonrpc(resp, 10)
        assert resp["result"]["name"] == "qmd-python"

    def test_dispatch_tools_list(self, server):
        resp = server.handle_message(_make_request("tools/list", msg_id=20))
        _assert_jsonrpc(resp, 20)
        assert "tools" in resp["result"]

    def test_dispatch_tools_call(self, server):
        req = _make_request("tools/call", msg_id=30, params={
            "name": "status",
            "arguments": {},
        })
        resp = server.handle_message(req)
        _assert_jsonrpc(resp, 30)

    def test_dispatch_unknown_method_returns_none(self, server):
        resp = server.handle_message(_make_request("unknown/method", msg_id=99))
        assert resp is None


# ---------------------------------------------------------------------------
# 2. _handle_initialize
# ---------------------------------------------------------------------------

class TestHandleInitialize:
    def test_initialize_fields(self, server):
        resp = server.handle_message(_make_request("initialize", msg_id=1))
        result = resp["result"]
        assert result["name"] == "qmd-python"
        assert result["version"] == "0.1.0"
        assert result["protocolVersion"] == "2024-11-05"
        assert "tools" in result["capabilities"]
        assert "resources" in result["capabilities"]


# ---------------------------------------------------------------------------
# 3. _handle_tools_list
# ---------------------------------------------------------------------------

class TestHandleToolsList:
    def test_returns_five_tools(self, server):
        resp = server.handle_message(_make_request("tools/list", msg_id=2))
        tools = resp["result"]["tools"]
        assert len(tools) == 5

    def test_tool_names(self, server):
        resp = server.handle_message(_make_request("tools/list", msg_id=2))
        names = [t["name"] for t in resp["result"]["tools"]]
        assert names == ["search", "vsearch", "query", "get", "status"]

    @pytest.mark.parametrize("tool_name", ["search", "vsearch", "query", "get"])
    def test_tools_have_input_schema(self, server, tool_name):
        resp = server.handle_message(_make_request("tools/list", msg_id=2))
        tool = next(t for t in resp["result"]["tools"] if t["name"] == tool_name)
        schema = tool["inputSchema"]
        assert schema["type"] == "object"
        assert "properties" in schema

    @pytest.mark.parametrize("tool_name", ["search", "vsearch", "query"])
    def test_search_tools_require_query(self, server, tool_name):
        resp = server.handle_message(_make_request("tools/list", msg_id=2))
        tool = next(t for t in resp["result"]["tools"] if t["name"] == tool_name)
        assert "query" in tool["inputSchema"]["required"]

    def test_get_tool_requires_path(self, server):
        resp = server.handle_message(_make_request("tools/list", msg_id=2))
        tool = next(t for t in resp["result"]["tools"] if t["name"] == "get")
        assert "path" in tool["inputSchema"]["required"]

    def test_status_tool_has_no_input_schema(self, server):
        resp = server.handle_message(_make_request("tools/list", msg_id=2))
        tool = next(t for t in resp["result"]["tools"] if t["name"] == "status")
        assert "inputSchema" not in tool


# ---------------------------------------------------------------------------
# 4. _handle_tools_call — search
# ---------------------------------------------------------------------------

class TestToolSearch:
    def test_search_valid_query(self, server, mock_store):
        req = _make_request("tools/call", msg_id=3, params={
            "name": "search",
            "arguments": {"query": "hello"},
        })
        resp = server.handle_message(req)
        _assert_jsonrpc(resp, 3)
        text = resp["result"]["content"][0]["text"]
        assert "Found 1 results" in text
        assert "/a.md" in text
        assert resp["result"]["isError"] is False
        mock_store.bm25_search.assert_called_once()

    def test_search_missing_query(self, server):
        req = _make_request("tools/call", msg_id=4, params={
            "name": "search",
            "arguments": {},
        })
        resp = server.handle_message(req)
        _assert_jsonrpc(resp, 4)
        assert resp["result"]["isError"] is True
        assert "query is required" in resp["result"]["content"][0]["text"]


# ---------------------------------------------------------------------------
# 4. _handle_tools_call — vsearch
# ---------------------------------------------------------------------------

class TestToolVsearch:
    def test_vsearch_valid_query(self, server, mock_store):
        req = _make_request("tools/call", msg_id=5, params={
            "name": "vsearch",
            "arguments": {"query": "semantic"},
        })
        resp = server.handle_message(req)
        _assert_jsonrpc(resp, 5)
        text = resp["result"]["content"][0]["text"]
        assert "Found 1 results" in text
        assert "/b.md" in text
        assert resp["result"]["isError"] is False
        mock_store.vector_search.assert_called_once()

    def test_vsearch_missing_query(self, server):
        req = _make_request("tools/call", msg_id=6, params={
            "name": "vsearch",
            "arguments": {},
        })
        resp = server.handle_message(req)
        _assert_jsonrpc(resp, 6)
        assert resp["result"]["isError"] is True
        assert "query is required" in resp["result"]["content"][0]["text"]


# ---------------------------------------------------------------------------
# 4. _handle_tools_call — query (hybrid)
# ---------------------------------------------------------------------------

class TestToolQuery:
    def test_query_valid(self, server, mock_store):
        req = _make_request("tools/call", msg_id=7, params={
            "name": "query",
            "arguments": {"query": "hybrid"},
        })
        resp = server.handle_message(req)
        _assert_jsonrpc(resp, 7)
        text = resp["result"]["content"][0]["text"]
        assert "Found 1 results" in text
        assert "/c.md" in text
        assert resp["result"]["isError"] is False
        mock_store.hybrid_search.assert_called_once()

    def test_query_missing_query(self, server):
        req = _make_request("tools/call", msg_id=8, params={
            "name": "query",
            "arguments": {},
        })
        resp = server.handle_message(req)
        _assert_jsonrpc(resp, 8)
        assert resp["result"]["isError"] is True
        assert "query is required" in resp["result"]["content"][0]["text"]


# ---------------------------------------------------------------------------
# 4. _handle_tools_call — get
# ---------------------------------------------------------------------------

class TestToolGet:
    def test_get_valid_path(self, server, tmp_path):
        f = tmp_path / "sample.txt"
        f.write_text("line0\nline1\nline2\nline3\n", encoding="utf-8")

        req = _make_request("tools/call", msg_id=9, params={
            "name": "get",
            "arguments": {"path": str(f)},
        })
        resp = server.handle_message(req)
        _assert_jsonrpc(resp, 9)
        text = resp["result"]["content"][0]["text"]
        assert "line0" in text
        assert "line3" in text
        assert resp["result"]["isError"] is False

    def test_get_missing_path(self, server):
        req = _make_request("tools/call", msg_id=10, params={
            "name": "get",
            "arguments": {},
        })
        resp = server.handle_message(req)
        _assert_jsonrpc(resp, 10)
        assert resp["result"]["isError"] is True
        assert "path is required" in resp["result"]["content"][0]["text"]

    def test_get_nonexistent_file(self, server, tmp_path):
        req = _make_request("tools/call", msg_id=11, params={
            "name": "get",
            "arguments": {"path": str(tmp_path / "nope.txt")},
        })
        resp = server.handle_message(req)
        _assert_jsonrpc(resp, 11)
        assert resp["result"]["isError"] is True
        assert "File not found" in resp["result"]["content"][0]["text"]

    def test_get_with_from_and_limit(self, server, tmp_path):
        f = tmp_path / "lines.txt"
        f.write_text("\n".join(f"L{i}" for i in range(10)), encoding="utf-8")

        req = _make_request("tools/call", msg_id=12, params={
            "name": "get",
            "arguments": {"path": str(f), "from": 2, "limit": 3},
        })
        resp = server.handle_message(req)
        _assert_jsonrpc(resp, 12)
        text = resp["result"]["content"][0]["text"]
        assert "L2" in text
        assert "L4" in text
        # L1 should have been skipped
        assert "L1" not in text
        # L5 should be beyond the limit
        assert "L5" not in text
        assert resp["result"]["isError"] is False


# ---------------------------------------------------------------------------
# 4. _handle_tools_call — status
# ---------------------------------------------------------------------------

class TestToolStatus:
    def test_status_returns_stats(self, server, mock_store):
        req = _make_request("tools/call", msg_id=13, params={
            "name": "status",
            "arguments": {},
        })
        resp = server.handle_message(req)
        _assert_jsonrpc(resp, 13)
        text = resp["result"]["content"][0]["text"]
        assert "Collections: 2" in text
        assert "Documents:   42" in text
        assert "Indexed:     40" in text
        assert "Pending:     2" in text
        assert "docs: 30 documents" in text
        assert "notes: 12 documents" in text
        assert resp["result"]["isError"] is False
        mock_store.get_stats.assert_called_once()


# ---------------------------------------------------------------------------
# 4. _handle_tools_call — unknown tool
# ---------------------------------------------------------------------------

class TestToolUnknown:
    def test_unknown_tool_returns_error(self, server):
        req = _make_request("tools/call", msg_id=14, params={
            "name": "nonexistent",
            "arguments": {},
        })
        resp = server.handle_message(req)
        _assert_jsonrpc(resp, 14)
        assert resp["result"]["isError"] is True
        assert "Unknown tool" in resp["result"]["content"][0]["text"]


# ---------------------------------------------------------------------------
# 5. JSON-RPC format validation across all endpoints
# ---------------------------------------------------------------------------

class TestJsonRpcFormat:
    """Every response must carry jsonrpc=2.0, matching id, and a result field."""

    @pytest.mark.parametrize("method,params", [
        ("initialize", None),
        ("tools/list", None),
        ("tools/call", {"name": "status", "arguments": {}}),
        ("tools/call", {"name": "search", "arguments": {"query": "x"}}),
    ])
    def test_envelope(self, server, method, params):
        req = _make_request(method, msg_id=77, params=params)
        resp = server.handle_message(req)
        assert resp is not None
        assert resp["jsonrpc"] == "2.0"
        assert resp["id"] == 77
        assert "result" in resp

    def test_response_is_json_serializable(self, server):
        """Ensure every response can round-trip through json.dumps/loads."""
        for method, params in [
            ("initialize", None),
            ("tools/list", None),
            ("tools/call", {"name": "status", "arguments": {}}),
        ]:
            req = _make_request(method, msg_id=88, params=params)
            resp = server.handle_message(req)
            roundtrip = json.loads(json.dumps(resp))
            assert roundtrip == resp
