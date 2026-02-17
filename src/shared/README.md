# QMD 多语言实现 - 共享资源

此目录包含四个语言实现共享的资源文件。

## 目录结构

```
shared/
├── index.yaml              # 基础配置文件
├── example-config.yaml     # 详细配置示例
├── README.md               # 本文档
└── scripts/                # 共享脚本
    ├── verify_qmd_compat.sh    # 验证单个实现兼容性
    └── compare_qmd_impls.sh   # 对比多个实现输出
```

## 配置说明

### 后端选择

```yaml
# BM25 后端
bm25:
  backend: "sqlite_fts5"   # 缺省，与原 QMD 一致
  # backend: "lancedb"     # 可选

# 向量后端
vector:
  backend: "qmd_builtin"   # 缺省，使用 sqlite-vec
  # backend: "lancedb"     # 可选
  # backend: "qdrant"      # 可选：Qdrant 向量数据库

  # embeddinggemma-300M 模型输出 768 维向量
  vector_size: 768
```

### Qdrant 配置

当使用 Qdrant 作为向量后端时，需要配置 Qdrant 连接：

```yaml
vector:
  backend: "qdrant"
  model: "embeddinggemma-300M"
  vector_size: 768
  qdrant:
    url: "http://localhost:6333"
    api_key: ""  # 可选，需要认证时填写
    collection: "qmd_documents"
```

### CLI 参数

```bash
# 使用 SQLite FTS5 (缺省)
qmd search "关键词"

# 使用 LanceDB FTS5
qmd search "关键词" --fts-backend lancedb

# 使用 QMD 内置向量 (缺省)
qmd vsearch "语义查询"

# 使用 LanceDB 向量
qmd vsearch "语义查询" --vector-backend lancedb
```

## 模型下载

本地模型应放置在 `~/.cache/qmd/models/` 目录：

```
~/.cache/qmd/models/
├── embeddinggemma-300M.gguf
├── qwen3-reranker-0.6b.gguf
└── qmd-query-expansion-1.7b.gguf
```

## 验证脚本

```bash
# 验证单个实现是否与原版 qmd 兼容
./scripts/verify_qmd_compat.sh <qmd-binary-path>

# 对比多个实现的输出
./scripts/compare_qmd_impls.sh
```

## 语言实现

| 语言 | 目录 | 构建命令 |
|------|------|----------|
| Rust | `qmd-rust/` | `cargo build --release` |
| Go | `qmd-go/` | `go build -o qmd ./cmd/qmd` |
| Python | `qmd-python/` | `pip install -e .` |
| TypeScript | `qmd-typescript/` | `bun install && bun link` |
