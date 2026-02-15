# Next Session Guide - ANEL Project

**Last Updated**: 2026-02-16
**Current Status**: Phase 1 测试对齐完成 — Rust ANEL 协议层 88 个测试全部通过
**Branch**: ANEL

## 本次完成的工作 (2026-02-16 Session 4)

### Phase 1: Rust ANEL 协议层测试 ✅ (88 tests)

**扩展 `src/qmd-rust/src/anel/mod.rs`**:
- `AnelSpec::for_command(&str)` — 按命令名获取 spec（对标 Go/Python）
- `AnelSpec::to_json()` — JSON 序列化
- `AnelResult::to_ndjson()` — NDJSON 序列化
- `impl Display for AnelError` — `[{ErrorCode:?}] {message}` 格式
- `impl std::error::Error for AnelError`

**新增 `tests/anel_protocol.rs` (47 tests)**:
- ErrorCode: to_status 全 17 种、default、serde、deserialize、debug、clone
- Severity: 5 级别、default
- RecoveryHint: basic、with_action、json 序列化（含 None action）
- AnelError: new、with_hint(单/多)、with_trace_id、with_metadata、display、to_ndjson、status_auto_set、implements_std_error
- From<anyhow::Error>: 10 种错误消息映射（not_found/permission/invalid/query_parse/collection/embedding/storage/database/config/unknown）
- TraceContext: from_env(空/有值)、get_or_generate(已有/新生成)、tags_default、default
- NdjsonRecord: basic、to_ndjson、complex_payload
- AnelResult: success、error、with_trace_id、to_ndjson(成功/失败)
- Constants: version、env_var_names

**新增 `tests/anel_spec.rs` (37 tests)**:
- for_command: 全 13 命令返回 spec、unknown 返回 None
- 全局验证: JSON 有效性、input_schema 是 object、output_schema 是 object、都有 error_codes
- 各命令独立验证: search(4)、vsearch(3)、query(3)、get(3)、collection(2)、context(2)、embed(2)、update(2)、status(1)、cleanup(2)、agent(2)、mcp(3)、multi_get(2)

## 下一步: Phase 2 — Rust Store 核心 + 搜索管线测试 (~130 个)

### 重点
1. 对标 TypeScript `store.test.ts`（最完整的参考实现）
2. 覆盖 CRUD、搜索管线（BM25/Vector/Hybrid）、snippet 提取、collection 过滤、去重、统计
3. 使用 `tempfile` crate 创建临时 SQLite 数据库
4. 注意 Rust Store 使用 `rusqlite` + `sqlite-vec`，需要 `--features sqlite-vec`

### 注意事项
- Rust Store 的 `vector_search_sqlite_vec` 方法有编译警告（dead code），可能需要 feature gate
- `dev-dependencies` 已有 `tempfile = "3.10"` 和 `tokio = { features = ["test-util"] }`
- 参考 Go `store_test.go` 和 Python `test_store.py`（如果存在）

## 全局测试对齐计划

| Phase | 内容 | 目标数量 | 状态 |
|-------|------|---------|------|
| 1 | Rust ANEL 协议层 | 88 | ✅ 完成 |
| 2 | Rust Store 核心 + 搜索管线 | ~130 | 📋 下一步 |
| 3 | Rust MCP Server | ~50 | 待做 |
| 4 | Rust CLI 端到端 | ~50 | 待做 |
| 5 | Rust LLM + Eval + 路径 | ~45 | 待做 |
| 6 | Rust 独有功能 | ~65 | 待做 |
| 7 | Python 补充 Store + CLI | ~35 | 待做 |
| 8 | Go 补充 Store + CLI | ~28 | 待做 |

## 构建命令

```bash
# Rust — 运行 ANEL 协议测试
cd src/qmd-rust && cargo test --test anel_protocol --test anel_spec

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

### 新增文件 (Session 4)
- `src/qmd-rust/tests/anel_protocol.rs` — 47 个 ANEL 协议测试
- `src/qmd-rust/tests/anel_spec.rs` — 37 个命令 spec 测试

### 修改文件 (Session 4)
- `src/qmd-rust/src/anel/mod.rs` — 添加 for_command, to_json, to_ndjson, Display, Error impl
