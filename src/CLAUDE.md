# CLAUDE.md

此文件指导 Claude Code 处理 QMD 多语言实现项目。

## 项目概述

QMD 是一个 AI 驱动的搜索工具，支持混合 BM25 和向量搜索。本项目使用 Rust、Go、Python 三种语言实现完整复刻。

## 关键约束

### 后端选择

- **BM25 后端**: SQLite FTS5 (缺省) / LanceDB
- **向量后端**: QMD 内置 sqlite-vec (缺省) / LanceDB
- CLI 参数: `--fts-backend`, `--vector-backend`

### 命名约定

- Rust: `qmd-rust`, `Cargo.toml`, `src/main.rs`
- Go: `qmd-go`, `go.mod`, `cmd/qmd/main.go`
- Python: `qmd-python`, `pyproject.toml`, `src/__main__.py`

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
    embedding float[384] distance_metric=cosine
);
```

### CLI 兼容性

命令参数必须与原 QMD 工具保持一致:

```
qmd search <query> [-n <num>] [-c <collection>] [--all]
qmd vsearch <query> [-n <num>] [-c <collection>] [--all]
qmd query <query> [-n <num>] [-c <collection>] [--all]
```

## 开发流程

1. **先实现 Rust 版本** - 作为参考实现
2. **确保所有实现共享相同行为**
3. **保持 CLI 参数一致**
4. **使用共享测试数据**

## 常用命令

```bash
# Rust 构建
cd qmd-rust && cargo build --release

# Go 构建
cd qmd-go && go build -o qmd ./cmd/qmd

# Python 安装
cd qmd-python && pip install -e .

# 运行测试
./shared/scripts/test.sh
```

## 配置位置

- 全局配置: `~/.config/qmd/index.yaml`
- 缓存目录: `~/.cache/qmd/`
- 模型目录: `~/.cache/qmd/models/`
