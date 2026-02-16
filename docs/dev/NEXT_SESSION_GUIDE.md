# Next Session Guide - ANEL Project

**Last Updated**: 2026-02-16
**Current Status**: Phase 1-5 完成，Phase 6 探索完成
**Branch**: ANEL

## 当前状态

### Phase 1-5 完成 ✅

| Phase | 内容 | 测试数量 |
|-------|------|---------|
| 1 | Rust ANEL 协议层 | 88 |
| 2 | Rust Store 核心 + 搜索管线 | 127 |
| 3 | Rust MCP Server | 22 |
| 4 | Rust CLI 端到端 | 83 |
| 5 | Rust LLM + Eval + 路径 | 33 |

**总计**: 353+ tests 全部通过

### Phase 6: 存储层兼容性修复

**Rust 版本**: ✅ 完成
- 新增 `content` 表（与原版一致）
- 删除 `documents.doc` 字段（改用外键关联）
- 删除 `collections` 表（改用 YAML 配置）
- 删除 `path_contexts` 表（改用 YAML 配置）
- 添加自动迁移机制
- 测试: 242+ 全部通过

**Go 版本**: 待同步
**Python 版本**: 待同步

### TypeScript 功能移植 ✅

| 功能 | 文件 | 状态 |
|------|------|------|
| 虚拟路径系统 | store/path.rs | ✅ 完成 |
| ls 命令 | cli/ls.rs | ✅ 完成 |
| context check | cli/context.rs | ✅ 完成 |
| MCP 资源基础设施 | mcp/mod.rs | ✅ 完成 |

## 待完成

- Phase 7: Python 补充测试
- Phase 8: Go 补充

## 构建命令

```bash
# Rust
cd src/qmd-rust && cargo test

# Python
cd src/qmd-python && python -m pytest tests/ -v

# Go
cd src/qmd-go && go test ./internal/... -v

# TypeScript
cd src/qmd-typescript && bun test
```
