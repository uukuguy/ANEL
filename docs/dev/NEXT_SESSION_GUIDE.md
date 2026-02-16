# Next Session Guide - ANEL Project

**Last Updated**: 2026-02-16
**Current Status**: Phase 5 完成 — Rust LLM 集成 33 个测试 + Phase 4 83 + Phase 3 22 + Phase 2 127 + Phase 1 88 = 353+ 测试全部通过
**Branch**: ANEL

## 本次完成的工作 (2026-02-16 Session 8)

### Phase 5: Rust LLM + Eval + 路径测试 ✅ (33 tests)

**新增 `tests/llm_integration.rs` 测试**:

**Router Embedding Tests (8 tests)**:
- test_router_embed_local_fallback, test_router_embed_multiple_texts
- test_router_embed_returns_valid_dimensions, test_router_embed_different_texts_different_embeddings
- test_router_embed_empty_array, test_router_embed_no_embedder
- test_embedder_produces_normalized_vectors

**Router Reranking Tests (6 tests)**:
- test_router_rerank_local_fallback, test_router_rerank_single_document
- test_router_rerank_empty_documents, test_router_rerank_no_reranker
- test_router_rerank_uses_title_and_path, test_local_reranker_new

**Query Expansion Tests (9 tests)**:
- test_query_expansion_always_includes_original, test_query_expansion_max_5
- test_query_expansion_no_duplicates, test_query_expansion_keyword_how
- test_query_expansion_keyword_what, test_query_expansion_phrase_based
- test_query_expansion_single_word, test_query_expansion_multiple_keywords
- test_query_expansion_deduplication

**Path Resolution and SearchResult Tests (8 tests)**:
- test_search_result_fields, test_search_result_default_values
- test_search_result_equality, test_search_result_clone
- test_search_result_debug, test_store_make_docid
- test_store_make_docid_format, test_store_search_options_custom
- test_store_search_options_default

**Integration Tests (2 tests)**:
- test_store_with_embedder_integration, test_hybrid_search_integration

## 上次完成的工作

### Phase 4: Rust CLI 端到端测试 ✅ (83 tests)

**新增 `tests/cli_e2e_integration.rs` 测试**:

**Search Command Tests (17 tests)**:
- test_search_basic, test_search_with_limit, test_search_with_limit_long
- test_search_with_collection, test_search_with_collection_long, test_search_all_collections
- test_search_with_min_score, test_search_json_format, test_search_ndjson_format
- test_search_md_format, test_search_csv_format, test_search_files_format
- test_search_dry_run, test_search_emit_spec, test_search_empty_query
- test_search_invalid_format

**VSearch Command Tests (11 tests)**:
- test_vsearch_basic, test_vsearch_with_limit, test_vsearch_with_collection
- test_vsearch_all_collections, test_vsearch_json_format, test_vsearch_dry_run
- test_vsearch_emit_spec, test_vsearch_fts_backend_option, test_vsearch_vector_backend_option
- test_vsearch_min_score

**Query Command Tests (11 tests)**:
- test_query_basic, test_query_with_limit, test_query_with_collection
- test_query_all_collections, test_query_json_format, test_query_ndjson_format
- test_query_md_format, test_query_dry_run, test_query_emit_spec
- test_query_with_fts_backend

**Get Command Tests (10 tests)**:
- test_get_basic, test_get_with_line_number, test_get_with_limit
- test_get_with_from, test_get_full_content, test_get_json_format
- test_get_dry_run, test_get_emit_spec, test_get_nonexistent_file
- test_get_help

**Status Command Tests (5 tests)**:
- test_status_basic, test_status_verbose, test_status_with_collection
- test_status_json_format, test_status_dry_run

**Collection Command Tests (5 tests)**:
- test_collection_list, test_collection_add, test_collection_add_with_name
- test_collection_remove, test_collection_rename

**Update Command Tests (3 tests)**:
- test_update_basic, test_update_with_pull, test_update_with_collection

**Embed Command Tests (3 tests)**:
- test_embed_basic, test_embed_with_force, test_embed_with_collection

**Context Command Tests (3 tests)**:
- test_context_list, test_context_add, test_context_rm

**Cleanup Command Tests (2 tests)**:
- test_cleanup_dry_run, test_cleanup_with_older_than

**MultiGet Command Tests (2 tests)**:
- test_multiget_basic, test_multiget_with_limit

**MCP Server Command Tests (2 tests)**:
- test_mcp_stdio_transport, test_mcp_sse_transport

**Agent Command Tests (2 tests)**:
- test_agent_query_mode, test_agent_json_format

**Plugin Command Tests (3 tests)**:
- test_plugin_list, test_plugin_dir, test_plugin_info

**Server Command Tests (2 tests)**:
- test_server_start, test_server_with_workers

**Error Handling Tests (4 tests)**:
- test_invalid_subcommand, test_search_invalid_format
- test_status_nonexistent_collection, test_get_nonexistent_file

**CLI Help Tests (4 tests)**:
- test_search_help, test_vsearch_help, test_query_help, test_get_help

## 下一步: Phase 5 — Rust LLM + Eval + 路径测试 (~45 个)

### 重点
1. 对标 Python/TypeScript LLM 集成
2. 测试 Embedding 和 Reranking 流程
3. 测试路径解析和上下文获取

## 全局测试对齐计划

| Phase | 内容 | 目标数量 | 状态 |
|-------|------|---------|------|
| 1 | Rust ANEL 协议层 | 88 | ✅ 完成 |
| 2 | Rust Store 核心 + 搜索管线 | ~130 | ✅ 完成 (127) |
| 3 | Rust MCP Server | ~50 | ✅ 完成 (22) |
| 4 | Rust CLI 端到端 | ~50 | ✅ 完成 (83) |
| 5 | Rust LLM + Eval + 路径 | ~45 | ✅ 完成 (33) |
| 6 | Rust 独有功能 | ~65 | 📋 下一步 |
| 7 | Python 补充 Store + CLI | ~35 | 待做 |
| 8 | Go 补充 Store + CLI | ~28 | 待做 |

## 构建命令

```bash
# Rust — 运行 LLM 集成测试
cd src/qmd-rust && cargo test --test llm_integration

# Rust — 运行 CLI E2E 测试
cd src/qmd-rust && cargo test --test cli_e2e_integration

# Rust — 运行 MCP Server 测试
cd src/qmd-rust && cargo test --test mcp_server_integration

# Rust — 运行 Store 集成测试
cd src/qmd-rust && cargo test --test store_integration

# Rust — 全部测试
cd src/qmd-rust && cargo test

# Go
cd src/qmd-go && go test ./internal/... -v

# Python
cd src/qmd-python && python -m pytest tests/ -v

# TypeScript
cd src/qmd-typescript && bun test

# E2E Demo
python3 scripts/e2e-demo.py
```

## 关键文件

### 新增文件 (Session 8)
- `src/qmd-rust/tests/llm_integration.rs` — 新增 33 个 LLM 集成测试

### 测试统计
- Phase 1: 88 tests
- Phase 2: 127 tests
- Phase 3: 22 tests
- Phase 4: 83 tests
- Phase 5: 33 tests
- **总计**: 353+ tests
