"""Tests for ANEL command specs."""

import json
import pytest

import sys
sys.path.insert(0, str(__import__("pathlib").Path(__file__).parent.parent / "src"))

from anel import VERSION, ErrorCode
from anel.spec import (
    search_spec,
    vsearch_spec,
    query_spec,
    get_spec,
    collection_spec,
    context_spec,
    embed_spec,
    update_spec,
    status_spec,
    cleanup_spec,
    agent_spec,
    mcp_spec,
    get_spec_for_command,
    SPEC_GETTERS,
)

ALL_COMMANDS = [
    "search", "vsearch", "query", "get", "collection",
    "context", "embed", "update", "status", "cleanup",
    "agent", "mcp",
]


class TestGetSpecForCommand:
    @pytest.mark.parametrize("cmd", ALL_COMMANDS)
    def test_returns_spec(self, cmd):
        spec = get_spec_for_command(cmd)
        assert spec is not None
        assert spec.command == cmd
        assert spec.version == VERSION

    def test_unknown_returns_none(self):
        assert get_spec_for_command("nonexistent") is None

    def test_spec_getters_dict_complete(self):
        for cmd in ALL_COMMANDS:
            assert cmd in SPEC_GETTERS


class TestAllSpecsValidJSON:
    @pytest.mark.parametrize("cmd", ALL_COMMANDS)
    def test_to_json(self, cmd):
        spec = get_spec_for_command(cmd)
        parsed = json.loads(spec.to_json())
        for field in ["version", "command", "input_schema", "output_schema", "error_codes"]:
            assert field in parsed, f"Spec for {cmd} missing field {field}"


class TestAllSpecsInputSchema:
    @pytest.mark.parametrize("cmd", ALL_COMMANDS)
    def test_is_object(self, cmd):
        spec = get_spec_for_command(cmd)
        assert spec.input_schema["type"] == "object"
        assert "properties" in spec.input_schema


class TestAllSpecsOutputSchema:
    @pytest.mark.parametrize("cmd", ALL_COMMANDS)
    def test_is_object(self, cmd):
        spec = get_spec_for_command(cmd)
        assert spec.output_schema["type"] == "object"


class TestAllSpecsErrorCodes:
    @pytest.mark.parametrize("cmd", ALL_COMMANDS)
    def test_has_error_codes(self, cmd):
        spec = get_spec_for_command(cmd)
        assert len(spec.error_codes) > 0

    @pytest.mark.parametrize("cmd", ALL_COMMANDS)
    def test_error_codes_are_valid(self, cmd):
        spec = get_spec_for_command(cmd)
        for code in spec.error_codes:
            assert isinstance(code, ErrorCode)


# --- Individual spec validation ---


class TestSearchSpec:
    def test_requires_query(self):
        spec = search_spec()
        assert "query" in spec.input_schema.get("required", [])

    def test_error_codes(self):
        spec = search_spec()
        codes = set(spec.error_codes)
        assert ErrorCode.SEARCH_FAILED in codes
        assert ErrorCode.INDEX_NOT_READY in codes
        assert ErrorCode.QUERY_PARSE_ERROR in codes

    def test_has_limit_property(self):
        spec = search_spec()
        assert "limit" in spec.input_schema["properties"]

    def test_has_min_score_property(self):
        spec = search_spec()
        assert "min_score" in spec.input_schema["properties"]


class TestVSearchSpec:
    def test_requires_query(self):
        spec = vsearch_spec()
        assert "query" in spec.input_schema.get("required", [])

    def test_has_embedding_error(self):
        spec = vsearch_spec()
        assert ErrorCode.EMBEDDING_FAILED in spec.error_codes

    def test_has_model_not_found(self):
        spec = vsearch_spec()
        assert ErrorCode.MODEL_NOT_FOUND in spec.error_codes


class TestQuerySpec:
    def test_requires_query(self):
        spec = query_spec()
        assert "query" in spec.input_schema.get("required", [])

    def test_output_has_reranked(self):
        spec = query_spec()
        items = spec.output_schema["properties"]["results"]["items"]
        assert "reranked" in items["properties"]

    def test_has_all_search_errors(self):
        spec = query_spec()
        codes = set(spec.error_codes)
        assert ErrorCode.SEARCH_FAILED in codes
        assert ErrorCode.EMBEDDING_FAILED in codes
        assert ErrorCode.QUERY_PARSE_ERROR in codes


class TestGetSpec:
    def test_requires_file(self):
        spec = get_spec()
        assert "file" in spec.input_schema.get("required", [])

    def test_output_has_lines(self):
        spec = get_spec()
        assert "lines" in spec.output_schema["properties"]

    def test_error_codes(self):
        spec = get_spec()
        codes = set(spec.error_codes)
        assert ErrorCode.NOT_FOUND in codes
        assert ErrorCode.INVALID_INPUT in codes


class TestCollectionSpec:
    def test_has_actions(self):
        spec = collection_spec()
        action = spec.input_schema["properties"]["action"]
        expected = {"add", "list", "remove", "rename"}
        assert set(action["enum"]) == expected

    def test_error_codes(self):
        spec = collection_spec()
        codes = set(spec.error_codes)
        assert ErrorCode.COLLECTION_NOT_FOUND in codes
        assert ErrorCode.COLLECTION_EXISTS in codes
        assert ErrorCode.INVALID_INPUT in codes


class TestContextSpec:
    def test_requires_action(self):
        spec = context_spec()
        assert "action" in spec.input_schema.get("required", [])

    def test_has_actions(self):
        spec = context_spec()
        action = spec.input_schema["properties"]["action"]
        assert set(action["enum"]) == {"add", "list", "rm"}


class TestEmbedSpec:
    def test_has_force(self):
        spec = embed_spec()
        assert "force" in spec.input_schema["properties"]

    def test_error_codes(self):
        spec = embed_spec()
        codes = set(spec.error_codes)
        assert ErrorCode.EMBEDDING_FAILED in codes
        assert ErrorCode.MODEL_NOT_FOUND in codes
        assert ErrorCode.MODEL_LOAD_FAILED in codes
        assert ErrorCode.COLLECTION_NOT_FOUND in codes


class TestUpdateSpec:
    def test_has_pull(self):
        spec = update_spec()
        assert "pull" in spec.input_schema["properties"]

    def test_error_codes(self):
        spec = update_spec()
        codes = set(spec.error_codes)
        assert ErrorCode.INDEX_NOT_READY in codes
        assert ErrorCode.COLLECTION_NOT_FOUND in codes
        assert ErrorCode.STORAGE_ERROR in codes


class TestStatusSpec:
    def test_has_verbose(self):
        spec = status_spec()
        assert "verbose" in spec.input_schema["properties"]


class TestCleanupSpec:
    def test_has_dry_run(self):
        spec = cleanup_spec()
        assert "dry_run" in spec.input_schema["properties"]

    def test_has_older_than(self):
        spec = cleanup_spec()
        assert "older_than" in spec.input_schema["properties"]


class TestAgentSpec:
    def test_has_interactive(self):
        spec = agent_spec()
        assert "interactive" in spec.input_schema["properties"]

    def test_has_mcp_option(self):
        spec = agent_spec()
        assert "mcp" in spec.input_schema["properties"]


class TestMcpSpec:
    def test_has_transport(self):
        spec = mcp_spec()
        assert "transport" in spec.input_schema["properties"]

    def test_has_port(self):
        spec = mcp_spec()
        assert "port" in spec.input_schema["properties"]

    def test_error_codes(self):
        spec = mcp_spec()
        codes = set(spec.error_codes)
        assert ErrorCode.CONFIG_ERROR in codes
        assert ErrorCode.BACKEND_UNAVAILABLE in codes
