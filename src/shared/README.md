# QMD 多语言实现 - 共享资源

此目录包含三个语言实现共享的资源文件。

## 目录结构

```
shared/
├── index.yaml              # 示例配置文件
├── example-config.yaml     # 详细配置示例
├── README.md               # 本文档
├── test-data/              # 测试数据
│   ├── sample-docs/        # 示例文档
│   └── expected-results/   # 预期搜索结果
└── scripts/                # 共享脚本
    ├── build.sh           # 构建脚本
    └── test.sh             # 测试脚本
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

## 测试

```bash
# 运行所有测试
./scripts/test.sh

# 构建所有项目
./scripts/build.sh
```
