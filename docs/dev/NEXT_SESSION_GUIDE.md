# Next Session Guide - ANEL Project

**Last Updated**: 2026-02-16
**Current Status**: P2 (Rust LanceDB) + P3 (Go/Python MCP Server) 已完成
**Branch**: ANEL

## 本次完成的工作 (2026-02-16 Session 2)

### P2: Rust LanceDB 完整实现 ✅
- 替换 stub 为完整 LanceDB 后端实现
- `lance_backend.rs`: connect, fts_search, vector_search, insert_documents, ensure_fts_index, ensure_vector_index
- 添加 `lance-index = "1.0"` 依赖（FullTextSearchQuery 类型来自 lance_index::scalar）
- Arrow FixedSizeListArray 使用 `new(field, size, values, nulls)` 构造
- `cargo check --features lancedb` 编译通过
- `cargo test --features sqlite-vec` 17 个测试全部通过

### P3: Go/Python MCP Server tools/call 实现 ✅
- **Go** (`internal/mcp/server.go`): 新增 Server struct，实现 tools/call 分发
  - 5 个工具: search → BM25Search, vsearch → VectorSearch, query → HybridSearch, get → ReadFile, status → GetStats
  - CLI (`internal/cli/mcp.go`): 改用 RunE 调用 mcp.RunServer()
  - `go build ./...` 通过，所有测试通过
- **Python** (`src/mcp/server.py`): 新增 McpServer 类，实现 tools/call 分发
  - 同样 5 个工具，通过 Store 集成
  - 174 个测试全部通过

### 配置统一 ✅
- 三语言 tool inputSchema 已统一:
  - search/vsearch/query: `{query: string, limit?: integer, collection?: string}`
  - get: `{path: string, from?: integer, limit?: integer}`
  - status: 无参数

## 下一步建议优先级

### P4: 完善 README 和项目文档（低优先级）
- 根目录 README 已提交，内容完整
- 可选：添加 CONTRIBUTING.md、LICENSE 文件

### P5: 端到端 Demo 场景（低优先级）
- Phase 3 路线图要求的"故障排查"端到端演示脚本
- 串联：Agent 接收指令 → 生成脚本 → 调用工具 → 捕获错误 → 自动修正 → 执行成功

### P6: MCP Server 集成测试
- 为 Go/Python MCP Server 添加 tools/call 单元测试
- 验证 JSON-RPC 请求/响应格式正确性

## 构建命令

```bash
# Rust
cd src/qmd-rust && cargo build --features sqlite-vec
cargo test --features sqlite-vec
cargo check --features lancedb  # LanceDB 编译验证

# Go
cd src/qmd-go && go build ./...
go test ./internal/... -v

# Python
cd src/qmd-python && pip install -e .
python -m pytest tests/ -v  # 174 tests

# TypeScript
cd src/qmd-typescript && bun install
bun test  # 700+ tests
```

## 关键文件

### 修改文件 (本次 Session 2)
- `src/qmd-rust/src/store/lance_backend/lance_backend.rs` — LanceDB 完整实现
- `src/qmd-rust/Cargo.toml` — 添加 lance-index 依赖
- `src/qmd-go/internal/mcp/server.go` — Go MCP Server tools/call
- `src/qmd-go/internal/cli/mcp.go` — CLI 集成 mcp.RunServer()
- `src/qmd-python/src/mcp/server.py` — Python MCP Server tools/call
