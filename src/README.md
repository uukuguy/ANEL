# QMD 多语言实现

使用 Rust、Go、Python 三种语言分别实现 QMD 的完整复刻项目，支持混合 BM25 和向量搜索。

## 项目结构

```
src/
├── qmd-rust/           # Rust 实现
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── cli/
│   │   ├── store/
│   │   ├── llm/
│   │   ├── mcp/
│   │   └── config/
│   └── tests/
│
├── qmd-go/             # Go 实现
│   ├── go.mod
│   ├── cmd/
│   │   └── qmd/
│   └── internal/
│       ├── cli/
│       ├── store/
│       ├── llm/
│       ├── mcp/
│       └── config/
│
├── qmd-python/        # Python 实现
│   ├── pyproject.toml
│   ├── src/
│   │   ├── __main__.py
│   │   ├── cli/
│   │   ├── store/
│   │   ├── llm/
│   │   ├── mcp/
│   │   └── config/
│   └── tests/
│
└── shared/             # 共享资源
    ├── index.yaml
    ├── example-config.yaml
    └── README.md
```

## 特性

### 后端支持

| 功能 | 后端选项 | 缺省选择 |
|------|----------|----------|
| BM25 全文搜索 | SQLite FTS5 / LanceDB | SQLite FTS5 |
| 向量语义搜索 | QMD 内置 sqlite-vec / LanceDB | QMD 内置 |

### LLM 支持

- **本地模型**: llama.cpp 格式 (GGUF)
- **远程 API**: OpenAI, Anthropic 兼容
- **自动路由**: 根据配置和可用性自动选择

### 运行模式

- **CLI 模式**: 命令行搜索
- **MCP Server**: 提供 MCP 工具
- **Agent 模式**: 自主搜索代理

## 快速开始

### Rust 实现

```bash
cd qmd-rust
cargo build --release
./target/release/qmd-rust --help
```

### Go 实现

```bash
cd qmd-go
go build -o qmd ./cmd/qmd
./qmd --help
```

### Python 实现

```bash
cd qmd-python
pip install -e .
python -m qmd_python --help
```

## CLI 命令

```bash
# 集合管理
qmd collection add <path> --name <name> --mask "**/*.md"
qmd collection list
qmd collection remove <name>

# 搜索
qmd search "关键词"          # BM25 全文搜索
qmd vsearch "语义查询"       # 向量搜索
qmd query "复杂查询"         # 混合搜索

# 索引管理
qmd embed                    # 生成向量索引
qmd update                   # 更新索引
qmd status                   # 查看状态

# 服务模式
qmd mcp --stdio             # MCP stdio 模式
qmd agent --interactive     # Agent 交互模式
```

## 配置

复制 `shared/example-config.yaml` 到 `~/.config/qmd/index.yaml`:

```yaml
bm25:
  backend: "sqlite_fts5"

vector:
  backend: "qmd_builtin"
  model: "embeddinggemma-300M"

collections:
  - name: "notes"
    path: "~/notes"
    pattern: "**/*.md"

models:
  embed:
    local: "embeddinggemma-300M"
```

## 开发计划

| 周次 | 内容 |
|------|------|
| Week 1 | 基础设施: CLI + SQLite FTS5 |
| Week 2 | 向量后端: QMD 内置 + LanceDB |
| Week 3 | LLM 集成: 本地 + 远程路由 |
| Week 4 | Agent + MCP Server |

## 参考

- 原项目: [github.com/tobi/qmd](https://github.com/tobi/qmd)
- LanceDB: [lancedb.github.io](https://lancedb.github.io/lancedb/)
- MCP: [modelcontextprotocol.io](https://modelcontextprotocol.io)
