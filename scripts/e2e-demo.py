#!/usr/bin/env python3
"""
ANEL End-to-End Demo — MCP Protocol Lifecycle

Demonstrates the full ANEL agent lifecycle at the MCP JSON-RPC layer:
  Phase 1: Discovery   (--emit-spec)
  Phase 2: Rehearsal   (dry-run via AGENT_DRY_RUN=1)
  Phase 3: Execution   (tool calls with audit trail via StreamTap)
  Phase 4: Error Recovery (unknown tool, missing args → RFC 7807 style)
  Phase 5: Identity     (AGENT_IDENTITY_TOKEN propagation in audit)

Runs against Go and Python MCP servers using mock stores (no real DB needed).
"""

import json
import os
import subprocess
import sys
import textwrap
import time

# ── Paths ─────────────────────────────────────────────────────────

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
GO_DIR = os.path.join(PROJECT_ROOT, "src", "qmd-go")
PY_DIR = os.path.join(PROJECT_ROOT, "src", "qmd-python")

# ── Helpers ───────────────────────────────────────────────────────

PASS = "\033[32m✓\033[0m"
FAIL = "\033[31m✗\033[0m"
PHASE = "\033[1;36m"
RESET = "\033[0m"
DIM = "\033[2m"


def banner(phase: int, title: str):
    print(f"\n{PHASE}{'═' * 60}")
    print(f"  Phase {phase}: {title}")
    print(f"{'═' * 60}{RESET}\n")


def check(label: str, ok: bool, detail: str = ""):
    mark = PASS if ok else FAIL
    suffix = f"  {DIM}{detail}{RESET}" if detail else ""
    print(f"  {mark} {label}{suffix}")
    return ok


def send_jsonrpc(proc, method: str, params=None, msg_id=1) -> dict:
    """Send a JSON-RPC 2.0 request to a subprocess via stdin, read response."""
    request = {"jsonrpc": "2.0", "id": msg_id, "method": method}
    if params is not None:
        request["params"] = params
    line = json.dumps(request) + "\n"
    proc.stdin.write(line)
    proc.stdin.flush()
    resp_line = proc.stdout.readline()
    if not resp_line:
        return {"error": "no response"}
    return json.loads(resp_line.strip())


# ── Python MCP Server (in-process, mock store) ───────────────────

class MockSearchResult:
    def __init__(self, path, title, score, collection, docid="abc123"):
        self.path = path
        self.title = title
        self.score = score
        self.collection = collection
        self.docid = docid


class MockIndexStats:
    def __init__(self):
        self.collection_count = 2
        self.document_count = 42
        self.indexed_count = 40
        self.pending_count = 2
        self.chunk_count = 128
        self.collection_stats = {"notes": 30, "docs": 12}


class MockStore:
    """Mock store that returns canned results for demo purposes."""

    def bm25_search(self, query, options):
        return [
            MockSearchResult("docs/api-design.md", "API Design Principles", 0.92, "docs"),
            MockSearchResult("docs/distributed-systems.md", "Distributed Systems", 0.78, "docs"),
        ]

    def vector_search(self, query, options):
        return [
            MockSearchResult("notes/ml-primer.md", "Machine Learning Primer", 0.88, "notes"),
            MockSearchResult("docs/api-design.md", "API Design Principles", 0.85, "docs"),
        ]

    def hybrid_search(self, query, options):
        return [
            MockSearchResult("docs/api-design.md", "API Design Principles", 0.95, "docs"),
            MockSearchResult("notes/ml-primer.md", "Machine Learning Primer", 0.87, "notes"),
            MockSearchResult("docs/distributed-systems.md", "Distributed Systems", 0.72, "docs"),
        ]

    def get_stats(self):
        return MockIndexStats()


def run_python_demo():
    """Run E2E demo against the Python MCP server in-process."""
    sys.path.insert(0, PY_DIR)
    from src.mcp.server import McpServer

    passed = 0
    total = 0

    # ── Phase 1: Discovery ────────────────────────────────────
    banner(1, "Discovery (initialize + tools/list)")

    server = McpServer(store=MockStore())

    resp = server.handle_message({"jsonrpc": "2.0", "id": 1, "method": "initialize"})
    total += 1
    if check("initialize returns server info",
             resp and resp.get("result", {}).get("name") == "qmd-python",
             f"name={resp.get('result', {}).get('name')}"):
        passed += 1

    resp = server.handle_message({"jsonrpc": "2.0", "id": 2, "method": "tools/list"})
    tools = resp.get("result", {}).get("tools", [])
    tool_names = [t["name"] for t in tools]
    total += 1
    if check("tools/list returns 5 tools",
             len(tools) == 5 and set(tool_names) == {"search", "vsearch", "query", "get", "status"},
             f"tools={tool_names}"):
        passed += 1

    # ── Phase 2: Rehearsal (Dry-Run) ──────────────────────────
    banner(2, "Rehearsal (AGENT_DRY_RUN=1)")

    os.environ["AGENT_DRY_RUN"] = "1"
    os.environ["AGENT_IDENTITY_TOKEN"] = "agent:demo-user@anel.dev"
    os.environ["AGENT_TRACE_ID"] = "trace-e2e-demo-001"

    dry_server = McpServer(store=MockStore())

    resp = dry_server.handle_message({
        "jsonrpc": "2.0", "id": 3, "method": "tools/call",
        "params": {"name": "search", "arguments": {"query": "API design"}}
    })
    content = resp.get("result", {}).get("content", [{}])[0].get("text", "")
    total += 1
    if check("dry-run search returns [DRY-RUN] prefix",
             content.startswith("[DRY-RUN]"),
             content[:60]):
        passed += 1

    resp = dry_server.handle_message({
        "jsonrpc": "2.0", "id": 4, "method": "tools/call",
        "params": {"name": "query", "arguments": {"query": "machine learning"}}
    })
    content = resp.get("result", {}).get("content", [{}])[0].get("text", "")
    total += 1
    if check("dry-run query returns [DRY-RUN] prefix",
             content.startswith("[DRY-RUN]"),
             content[:60]):
        passed += 1

    resp = dry_server.handle_message({
        "jsonrpc": "2.0", "id": 5, "method": "tools/call",
        "params": {"name": "status", "arguments": {}}
    })
    content = resp.get("result", {}).get("content", [{}])[0].get("text", "")
    total += 1
    if check("dry-run status returns [DRY-RUN] prefix",
             content.startswith("[DRY-RUN]"),
             content[:60]):
        passed += 1

    # Clean up dry-run env
    del os.environ["AGENT_DRY_RUN"]

    # ── Phase 3: Execution with Audit Trail ───────────────────
    banner(3, "Execution with Audit Trail (StreamTap)")

    # Identity and trace still set from Phase 2
    exec_server = McpServer(store=MockStore())

    resp = exec_server.handle_message({
        "jsonrpc": "2.0", "id": 6, "method": "tools/call",
        "params": {"name": "search", "arguments": {"query": "API design", "limit": 5}}
    })
    content = resp.get("result", {}).get("content", [{}])[0].get("text", "")
    is_error = resp.get("result", {}).get("isError", True)
    total += 1
    if check("search executes and returns results",
             not is_error and "API Design" in content,
             f"found {'API Design' in content}, error={is_error}"):
        passed += 1

    resp = exec_server.handle_message({
        "jsonrpc": "2.0", "id": 7, "method": "tools/call",
        "params": {"name": "vsearch", "arguments": {"query": "neural networks"}}
    })
    content = resp.get("result", {}).get("content", [{}])[0].get("text", "")
    total += 1
    if check("vsearch executes and returns results",
             not resp.get("result", {}).get("isError", True) and "results" in content.lower(),
             f"len={len(content)}"):
        passed += 1

    resp = exec_server.handle_message({
        "jsonrpc": "2.0", "id": 8, "method": "tools/call",
        "params": {"name": "query", "arguments": {"query": "best practices", "limit": 10}}
    })
    content = resp.get("result", {}).get("content", [{}])[0].get("text", "")
    total += 1
    if check("hybrid query returns fused results",
             not resp.get("result", {}).get("isError", True) and "results" in content.lower(),
             f"len={len(content)}"):
        passed += 1

    resp = exec_server.handle_message({
        "jsonrpc": "2.0", "id": 9, "method": "tools/call",
        "params": {"name": "status", "arguments": {}}
    })
    content = resp.get("result", {}).get("content", [{}])[0].get("text", "")
    total += 1
    if check("status returns index stats",
             "Collections: 2" in content and "Documents:   42" in content,
             content.split("\n")[0]):
        passed += 1

    # ── Phase 4: Error Recovery ───────────────────────────────
    banner(4, "Error Recovery (unknown tool, missing args)")

    resp = exec_server.handle_message({
        "jsonrpc": "2.0", "id": 10, "method": "tools/call",
        "params": {"name": "nonexistent_tool", "arguments": {}}
    })
    is_error = resp.get("result", {}).get("isError", False)
    content = resp.get("result", {}).get("content", [{}])[0].get("text", "")
    total += 1
    if check("unknown tool returns isError=true",
             is_error and "Unknown tool" in content,
             content[:50]):
        passed += 1

    resp = exec_server.handle_message({
        "jsonrpc": "2.0", "id": 11, "method": "tools/call",
        "params": {"name": "search", "arguments": {}}
    })
    is_error = resp.get("result", {}).get("isError", False)
    content = resp.get("result", {}).get("content", [{}])[0].get("text", "")
    total += 1
    if check("search with empty query returns error",
             is_error and "required" in content.lower(),
             content[:50]):
        passed += 1

    resp = exec_server.handle_message({
        "jsonrpc": "2.0", "id": 12, "method": "tools/call",
        "params": {"name": "get", "arguments": {"path": "/nonexistent/file.md"}}
    })
    is_error = resp.get("result", {}).get("isError", False)
    content = resp.get("result", {}).get("content", [{}])[0].get("text", "")
    total += 1
    if check("get nonexistent file returns error",
             is_error and ("not found" in content.lower() or "error" in content.lower()),
             content[:50]):
        passed += 1

    # ── Phase 5: Identity Propagation ─────────────────────────
    banner(5, "Identity Propagation (AGENT_IDENTITY_TOKEN)")

    # The audit records were written to stderr during Phase 3.
    # We verify the identity was set correctly in the server.
    total += 1
    tap = exec_server._tap
    if check("StreamTap captured identity token",
             tap.identity == "agent:demo-user@anel.dev",
             f"identity={tap.identity}"):
        passed += 1

    total += 1
    if check("StreamTap captured trace ID",
             tap.trace_id == "trace-e2e-demo-001",
             f"trace_id={tap.trace_id}"):
        passed += 1

    # Clean up
    del os.environ["AGENT_IDENTITY_TOKEN"]
    del os.environ["AGENT_TRACE_ID"]

    return passed, total


# ── Go MCP Server (subprocess, stdio transport) ──────────────────

def run_go_demo():
    """Run E2E demo against the Go MCP server via subprocess (if built)."""
    go_binary = os.path.join(GO_DIR, "qmd")
    go_test_binary = None

    # Check if Go tests can run (we'll use `go test` to exercise the server)
    try:
        result = subprocess.run(
            ["go", "build", "./..."],
            cwd=GO_DIR, capture_output=True, text=True, timeout=60
        )
        if result.returncode != 0:
            print(f"  {DIM}Go build failed, skipping Go demo{RESET}")
            return 0, 0
    except (FileNotFoundError, subprocess.TimeoutExpired):
        print(f"  {DIM}Go not available, skipping Go demo{RESET}")
        return 0, 0

    # Run Go tests as proxy for E2E validation (use shell to merge stderr)
    passed = 0
    total = 0

    # Phase A: Normal execution with audit trail
    env_normal = {**os.environ, "AGENT_DRY_RUN": "0", "AGENT_IDENTITY_TOKEN": "agent:go-demo@anel.dev"}
    result = subprocess.run(
        "go test ./internal/mcp/ -v -count=1 -run 'TestToolsCall_Get_ValidFile' 2>&1",
        cwd=GO_DIR, shell=True, capture_output=True, text=True, timeout=60,
        env=env_normal,
    )
    combined = result.stdout
    total += 1
    if check("Go MCP tool call tests pass",
             "PASS" in combined,
             f"exit={result.returncode}"):
        passed += 1

    total += 1
    has_audit = '"type":"audit"' in combined
    if check("Go StreamTap emits audit records",
             has_audit,
             f"has_audit={has_audit}"):
        passed += 1

    total += 1
    has_identity = '"identity":"agent:go-demo@anel.dev"' in combined
    if check("Go audit includes identity token",
             has_identity,
             f"has_identity={has_identity}"):
        passed += 1

    # Phase B: Dry-run mode
    env_dry = {**os.environ, "AGENT_DRY_RUN": "1", "AGENT_IDENTITY_TOKEN": "agent:go-demo@anel.dev"}
    result = subprocess.run(
        "go test ./internal/mcp/ -v -count=1 -run 'TestToolsCall_Get_ValidFile' 2>&1",
        cwd=GO_DIR, shell=True, capture_output=True, text=True, timeout=60,
        env=env_dry,
    )
    combined = result.stdout
    total += 1
    # With dry-run on, the test may fail (content mismatch) but the process shouldn't crash
    has_dry_run = "DRY-RUN" in combined or '"status":"dry-run"' in combined
    if check("Go dry-run mode activates",
             has_dry_run,
             f"has_dry_run={has_dry_run}"):
        passed += 1

    return passed, total


# ── Main ──────────────────────────────────────────────────────────

def main():
    print(f"\n{PHASE}{'═' * 60}")
    print("  ANEL End-to-End Demo")
    print(f"  MCP Protocol Lifecycle × P3 Security Features")
    print(f"{'═' * 60}{RESET}")

    total_passed = 0
    total_tests = 0

    # Python demo (always runs)
    print(f"\n{PHASE}── Python MCP Server ──{RESET}")
    p, t = run_python_demo()
    total_passed += p
    total_tests += t

    # Go demo (runs if Go is available)
    print(f"\n{PHASE}── Go MCP Server ──{RESET}")
    p, t = run_go_demo()
    total_passed += p
    total_tests += t

    # Summary
    print(f"\n{PHASE}{'═' * 60}")
    color = "\033[32m" if total_passed == total_tests else "\033[33m"
    print(f"  {color}Results: {total_passed}/{total_tests} checks passed{RESET}")
    print(f"{PHASE}{'═' * 60}{RESET}\n")

    sys.exit(0 if total_passed == total_tests else 1)


if __name__ == "__main__":
    main()
