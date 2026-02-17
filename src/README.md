# QMD 多语言实现

使用 Rust、Go、Python、TypeScript 四种语言分别实现 QMD 的完整复刻项目，支持混合 BM25 和向量搜索。

## 项目结构

```
src/
├── qmd-rust/           # Rust 实现
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── cli/        # CLI 命令 (collection, context, search, get, embed, etc.)
│   │   ├── store/     # 存储层 (SQLite FTS5, LanceDB, sqlite-vec)
│   │   ├── llm/       # LLM 集成 (本地 GGUF, 远程 API)
│   │   ├── mcp/       # MCP 服务器
│   │   ├── config/    # 配置管理
│   │   ├── anel/      # ANEL 协议实现
│   │   ├── formatter/ # 输出格式化
│   │   ├── plugin/    # 插件系统
│   │   └── server/    # HTTP 服务器
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
│       ├── config/
│       └── anel/
│
├── qmd-python/        # Python 实现
│   ├── pyproject.toml
│   ├── src/
│   │   ├── __main__.py
│   │   ├── cli/
│   │   ├── store/
│   │   ├── llm/
│   │   ├── mcp/
│   │   ├── config/
│   │   └── anel/
│   └── tests/
│
├── qmd-typescript/    # TypeScript 实现 (基于原版 qmd)
│   ├── package.json
│   ├── src/
│   │   ├── qmd.ts     # 主入口
│   │   ├── cli.test.ts
│   │   ├── store.ts   # SQLite 存储
│   │   ├── llm.ts     # LLM 集成
│   │   ├── mcp.ts     # MCP 服务器
│   │   └── anel/      # ANEL 协议
│   └── tests/
│
└── shared/            # 共享资源
    ├── index.yaml
    ├── example-config.yaml
    ├── README.md
    └── scripts/
        ├── verify_qmd_compat.sh
        └── compare_qmd_impls.sh

├── anel-copilot/      # ANEL 协议合规助手 (MCP Server + CLI)
│   ├── package.json
│   ├── src/
│   │   ├── index.ts       # MCP Server (6 tools)
│   │   ├── cli.ts         # CLI 入口
│   │   └── core/
│   │       ├── types.ts       # 类型定义
│   │       ├── rules.ts       # 7 条 ANEL 合规规则
│   │       ├── analyzer.ts    # 代码分析引擎
│   │       ├── detector.ts    # 语言/框架检测
│   │       ├── generator.ts   # 自动修复代码生成
│   │       ├── verifier.ts    # 运行时验证
│   │       ├── batch.ts       # 批量目录分析
│   │       ├── llm.ts         # LLM 智能修复 (Anthropic API)
│   │       └── ast-detector.ts # AST 检测 (tree-sitter, optional)
│   └── tests/             # 82 tests (vitest)
```

## 特性

### 后端支持

| 功能 | 后端选项 | 缺省选择 |
|------|----------|----------|
| BM25 全文搜索 | SQLite FTS5 / LanceDB | SQLite FTS5 |
| 向量语义搜索 | QMD 内置 sqlite-vec / LanceDB | QMD 内置 |
| 向量语义搜索 | Qdrant | - |

### LLM 支持

- **本地模型**: llama.cpp 格式 (GGUF)
- **远程 API**: OpenAI, Anthropic 兼容
- **自动路由**: 根据配置和可用性自动选择

### 运行模式

- **CLI 模式**: 命令行搜索
- **MCP Server**: 提供 MCP 工具 (stdio / SSE)
- **Agent 模式**: 自主搜索代理
- **HTTP Server**: REST API 服务

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

### TypeScript 实现

```bash
cd qmd-typescript
bun install
bun link  # 链接为全局命令 qmd
bun src/qmd.ts --help
```

### ANEL Copilot

```bash
cd anel-copilot
npm install
npm run build
node dist/cli.js analyze <file>          # 分析 ANEL 合规性
node dist/cli.js analyze-dir <dir>       # 批量分析目录
node dist/cli.js fix <file> [--dry-run]  # 自动修复
node dist/cli.js fix <file> --llm        # LLM 智能修复 (需要 ANTHROPIC_API_KEY)
```

## CLI 命令

与原版 qmd 完全兼容的命令列表：

```bash
# 集合管理
qmd collection add <path> --name <name> --mask "**/*.md"
qmd collection list
qmd collection remove <name>
qmd collection rename <old> <new>

# 上下文管理
qmd context add [path] "description"
qmd context list
qmd context check
qmd context rm <path>

# 文件浏览
qmd ls [collection[/path]]

# 文档获取
qmd get <file>[:line]
qmd multi-get <pattern>

# 搜索
qmd search <query>              # BM25 全文搜索
qmd vsearch <query>             # 向量语义搜索
qmd query <query>               # 混合搜索 + 重排序

# 索引管理
qmd embed [--force] [--collection <name>]
qmd update [--pull] [--collection <name>]
qmd status [--verbose] [--collection <name>]
qmd cleanup [--dry-run] [--older-than <days>]

# 服务模式
qmd mcp [--transport stdio|sse] [--port <port>]
qmd server [--host <host>] [--port <port>] [--workers <num>]
qmd agent [--interactive] [--query <query>]

# 插件管理
qmd plugin list
qmd plugin install <path> [--name <name>]
qmd plugin remove <name>
qmd plugin info <name>
qmd plugin dir
```

### 通用参数

```bash
# 输出格式
--format cli|json|ndjson|csv|md|xml|files

# 搜索参数
-n <num>                # 结果数量 (default: 20)
-c, --collection <name> # 限定集合
--all                   # 返回所有匹配
--min-score <num>       # 最低分数阈值
--full                  # 显示完整文档内容
--line-numbers          # 显示行号
```

## 配置

复制 `shared/example-config.yaml` 到 `~/.config/qmd/index.yaml`:

```yaml
bm25:
  backend: "sqlite_fts5"  # 或 "lancedb"

vector:
  backend: "qmd_builtin"  # 或 "lancedb", "qdrant"
  model: "embeddinggemma-300M"

collections:
  - name: "notes"
    path: "~/notes"
    pattern: "**/*.md"

models:
  embed:
    local: "embeddinggemma-300M"
  rerank:
    local: "qwen3-reranker-0.6b"
```

## 验证脚本

```bash
# 验证单个实现
./shared/scripts/verify_qmd_compat.sh <qmd-binary-path>

# 对比多个实现
./shared/scripts/compare_qmd_impls.sh
```

## 架构要点

### 存储层

- SQLite FTS5: BM25 全文搜索
- sqlite-vec: 向量相似度搜索 (Rust/Python 内置)
- LanceDB: 可选的向量+全文搜索后端
- Qdrant: 向量数据库后端

### ANEL 协议

所有实现支持 ANEL (Agent-Native Exchange Language) 协议规范:

- `emit_spec`: 输出 JSON Schema 规范
- `dry_run`: 验证参数但不执行

**ANEL Copilot** (`src/anel-copilot/`) 提供自动化合规检测和修复:
- 7 条规则: emit-spec, dry-run, output-format, error-format, ndjson-output, trace-id, env-vars
- 6 个 MCP 工具: anel_analyze, anel_analyze_dir, anel_fix, anel_verify, anel_explain
- 支持 template 和 LLM 两种修复模式
- 可选 tree-sitter AST 精确检测

### 兼容性

- 数据文件兼容: 各实现使用相同的 SQLite schema
- CLI 兼容: 与原版 qmd 命令行参数一致
- MCP 兼容: 提供相同的 MCP 工具

## 参考

- 原项目: [github.com/tobi/qmd](https://github.com/tobi/qmd)
- LanceDB: [lancedb.github.io](https://lancedb.github.io/lancedb/)
- MCP: [modelcontextprotocol.io](https://modelcontextprotocol.io)
