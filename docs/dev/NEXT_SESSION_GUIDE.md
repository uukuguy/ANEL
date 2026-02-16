# Next Session Guide - ANEL Project

**Last Updated**: 2026-02-16
**Current Status**: Phase 3 测试完成 — Rust MCP Server 22 个测试 + Phase 2 127 + Phase 1 88 = 237+ 测试全部通过
**Branch**: ANEL

## 本次完成的工作 (2026-02-16 Session 6)

### Phase 3: Rust MCP Server 测试 ✅ (22 tests)

**新增 `tests/mcp_server_integration.rs` 测试**:

**Server Info Tests (3 tests)**:
- test_server_info_name: 验证服务器名称包含 "qmd"
- test_server_info_has_instructions: 验证指令包含 search/vsearch/query
- test_server_info_has_capabilities: 验证 capabilities 字段存在

**Store-backed Search Tests (4 tests)**:
- test_bm25_search_returns_results: BM25 搜索返回结果
- test_bm25_search_no_results: 无结果时优雅处理
- test_bm25_search_with_limit: limit 参数生效
- test_bm25_search_with_collection_filter: 集合过滤生效

**Get Tool Tests (2 tests)**:
- test_get_file_content: 读取文件内容
- test_get_file_with_line_range: 支持行范围读取

**Status Tool Tests (1 test)**:
- test_status_returns_stats: 返回索引统计信息

**Parameter Type Tests (4 tests)**:
- test_search_options_default_limit: 默认 limit=20
- test_search_options_with_limit: 自定义 limit
- test_search_options_with_collection: 集合过滤参数
- test_search_options_without_collection: 全局搜索参数

**SearchResult Type Tests (2 tests)**:
- test_search_result_fields: 验证结果字段完整性
- test_search_result_query_optional: query 字段可选

**RRF Fusion Tests (4 tests)**:
- test_rrf_fusion_empty_lists: 空列表处理
- test_rrf_fusion_single_list: 单列表融合
- test_rrf_fusion_multiple_lists: 多列表融合（重复结果排名提升）
- test_rrf_fusion_with_weights: 加权融合

**Error Handling Tests (2 tests)**:
- test_store_invalid_collection: 无效集合处理
- test_get_nonexistent_file: 文件不存在处理

## 下一步: Phase 4 — Rust CLI 端到端测试 (~50 个)

### 重点
1. 对标 Python/TypeScript CLI 实现
2. 测试完整的 search/vsearch/query/get 命令流程
3. 使用 test config 和 mock LLM

## 全局测试对齐计划

| Phase | 内容 | 目标数量 | 状态 |
|-------|------|---------|------|
| 1 | Rust ANEL 协议层 | 88 | ✅ 完成 |
| 2 | Rust Store 核心 + 搜索管线 | ~130 | ✅ 完成 (127) |
| 3 | Rust MCP Server | ~50 | ✅ 完成 (22) |
| 4 | Rust CLI 端到端 | ~50 | 📋 下一步 |
| 5 | Rust LLM + Eval + 路径 | ~45 | 待做 |
| 6 | Rust 独有功能 | ~65 | 待做 |
| 7 | Python 补充 Store + CLI | ~35 | 待做 |
| 8 | Go 补充 Store + CLI | ~28 | 待做 |

## 构建命令

```bash
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

### 新增文件 (Session 6)
- `src/qmd-rust/tests/mcp_server_integration.rs` — 新增 22 个 MCP Server 测试

### 测试统计
- Phase 1: 88 tests
- Phase 2: 127 tests
- Phase 3: 22 tests
- **总计**: 237+ tests
