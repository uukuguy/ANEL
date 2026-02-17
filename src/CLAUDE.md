# CLAUDE.md

此文件指导 Claude Code 处理 QMD 多语言实现项目。

## 项目概述

QMD 是一个 AI 驱动的搜索工具，支持混合 BM25 和向量搜索。本项目使用 Rust、Go、Python、TypeScript 四种语言实现完整复刻。

## 关键约束

### 后端选择

- **BM25 后端**: SQLite FTS5 (缺省) / LanceDB
- **向量后端**: QMD 内置 sqlite-vec (缺省) / LanceDB / Qdrant
- CLI 参数: `--fts-backend`, `--vector-backend`

### 命名约定

- Rust: `qmd-rust`, `Cargo.toml`, `src/main.rs`, `src/cli/`
- Go: `qmd-go`, `go.mod`, `cmd/qmd/main.go`, `internal/cli/`
- Python: `qmd-python`, `pyproject.toml`, `src/__main__.py`, `src/cli/`
- TypeScript: `qmd-typescript`, `package.json`, `src/qmd.ts`, `bun install && bun link`

### Schema 兼容性

所有实现必须使用相同的 SQLite schema:

```sql
-- FTS5 全文索引
CREATE VIRTUAL TABLE documents_fts USING fts5(
    filepath, title, body,
    tokenize='porter unicode61'
);

-- sqlite-vec 向量
CREATE VIRTUAL TABLE vectors_vec USING vec0(
    hash_seq TEXT PRIMARY KEY,
    embedding float[768] distance_metric=cosine
);
```

### CLI 兼容性

命令参数必须与原版 qmd 工具保持一致:

```
qmd search <query> [-n <num>] [-c <collection>] [--all] [--format json|csv|md|files]
qmd vsearch <query> [-n <num>] [-c <collection>] [--all]
qmd query <query> [-n <num>] [-c <collection>] [--all]
qmd get <file>[:line] [-l <num>] [--full]
qmd multi-get <pattern> [--max-bytes <num>]
qmd collection add <path> --name <name> --mask "**/*.md"
qmd collection list
qmd collection remove <name>
qmd collection rename <old> <new>
qmd context add [path] "description"
qmd context list
qmd context check
qmd context rm <path>
qmd ls [collection[/path]]
qmd embed [--force] [--collection <name>]
qmd update [--pull] [--collection <name>]
qmd status [--verbose]
qmd cleanup [--dry-run] [--older-than <days>]
qmd mcp [--transport stdio|sse] [--port <port>]
qmd agent [--interactive] [--query <query>]
```

### ANEL 协议支持

所有实现支持 ANEL (Agent-Native Exchange Language) 协议:

- `--emit_spec`: 输出 JSON Schema 规范而非执行
- `--dry_run`: 验证参数但不执行

## 开发流程

1. **先实现 Rust 版本** - 作为参考实现
2. **确保所有实现共享相同行为**
3. **保持 CLI 参数一致**
4. **使用验证脚本测试兼容性**

## 常用命令

```bash
# Rust 构建
cd qmd-rust && cargo build --release

# Go 构建
cd qmd-go && go build -o qmd ./cmd/qmd

# Python 安装
cd qmd-python && pip install -e .

# TypeScript 安装
cd qmd-typescript && bun install && bun link

# 验证兼容性
./shared/scripts/verify_qmd_compat.sh <binary-path>
./shared/scripts/compare_qmd_impls.sh

# 运行测试 (Rust)
cd qmd-rust && cargo test

# 运行测试 (Python)
cd qmd-python && pytest
```

## 配置位置

- 全局配置: `~/.config/qmd/index.yaml`
- 缓存目录: `~/.cache/qmd/`
- 模型目录: `~/.cache/qmd/models/`

## 模块说明

### Rust 版本

| 模块 | 说明 |
|------|------|
| `cli/` | 命令行接口 |
| `store/` | 存储层 (SQLite, LanceDB) |
| `llm/` | LLM 集成 (本地/远程) |
| `mcp/` | MCP 服务器 |
| `config/` | 配置管理 |
| `anel/` | ANEL 协议实现 |
| `formatter/` | 输出格式化 |
| `plugin/` | 插件系统 |
| `server/` | HTTP 服务器 |

### TypeScript 版本

基于原版 qmd (github.com/tobi/qmd)，是项目的原始实现：
- `src/qmd.ts` - 主入口
- `src/store.ts` - SQLite 存储
- `src/llm.ts` - LLM 集成
- `src/mcp.ts` - MCP 服务器
