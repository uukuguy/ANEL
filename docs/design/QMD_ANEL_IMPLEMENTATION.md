# QMD ANEL 级别架构设计实施方案

## 概述

本文档描述了如何将 ANEL（智能体原生执行层）架构与 QMD 项目结合，实现真正的 Agent-Native CLI。ANEL 架构的核心原则是原子化、确定性和流式交互，通过引入 ANID 协议和三层动态模型，将 QMD 从一个本地知识库工具升级为企业级智能体基础设施。

## 架构设计

### 1. ANEL 核心模块

ANEL 模块位于 `src/qmd-rust/src/anel/mod.rs`，提供以下核心功能：

#### 1.1 ANID 错误类型

ANEL 定义了一套标准的错误码体系，用于智能体精确识别和处理错误情况：

```rust
pub enum AnelErrorCode {
    Unknown,
    InvalidInput,
    NotFound,
    PermissionDenied,
    SearchFailed,
    IndexNotReady,
    QueryParseError,
    CollectionNotFound,
    CollectionExists,
    CollectionCorrupted,
    EmbeddingFailed,
    ModelNotFound,
    ModelLoadFailed,
    StorageError,
    BackendUnavailable,
    ConfigError,
    EnvironmentError,
}
```

每个错误码都映射到标准的 HTTP 状态码，便于与现有 Web 基础设施集成。错误类型还包含 `recovery_hints` 字段，提供 RFC 7807 扩展的错误恢复提示。

#### 1.2 跟踪上下文

跟踪上下文用于在分布式系统中追踪请求：

```rust
pub struct TraceContext {
    pub trace_id: Option<String>,
    pub identity_token: Option<String>,
    pub tags: HashMap<String, String>,
}
```

跟踪上下文从环境变量 `AGENT_TRACE_ID` 和 `AGENT_IDENTITY_TOKEN` 自动加载。

#### 1.3 ANEL 规范

每个命令都可以输出其 JSON Schema 规范，用于智能体动态发现和验证参数：

```rust
pub struct AnelSpec {
    pub version: String,
    pub command: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub error_codes: Vec<AnelErrorCode>,
}
```

### 2. NDJSON 流式输出

QMD 现在支持 NDJSON（Newline-Delimited JSON）格式的流式输出，非常适合智能体处理：

```bash
$ qmd search "test query" --format ndjson --limit 2
{"type":"metadata","seq":0,"payload":{"query":"test","total":2,"trace_id":"qmd-1894194b6475aa10","version":"1.0"}}
{"type":"result","seq":1,"payload":{"docid":"...","path":"...","score":...}}
{"type":"result","seq":2,"payload":{"docid":"...","path":"...","score":...}}
```

每行是一个独立的 JSON 对象，便于流式处理和错误恢复。

### 3. ANEL CLI 参数

所有搜索命令现在都支持以下 ANEL 参数：

| 参数 | 说明 |
|------|------|
| `--emit-spec` | 输出 ANEL 规范（JSON Schema）并退出，不执行命令 |
| `--dry-run` | 验证参数但不执行命令 |

### 4. 环境变量支持

| 环境变量 | 说明 |
|----------|------|
| `AGENT_TRACE_ID` | 请求跟踪 ID，用于日志关联 |
| `AGENT_IDENTITY_TOKEN` | 智能体身份令牌 |
| `AGENT_OUTPUT_FORMAT` | 覆盖默认输出格式 |
| `AGENT_DRY_RUN` | 启用干运行模式 |
| `AGENT_EMIT_SPEC` | 启用规范输出模式 |

## 实现细节

### 模块结构

```
src/qmd-rust/src/
├── anel/mod.rs          # ANEL 核心模块
├── formatter/mod.rs     # 输出格式化（包含 NDJSON）
├── cli/
│   ├── mod.rs           # CLI 定义（包含 ANEL 参数）
│   ├── search.rs       # BM25 搜索（含 ANEL 支持）
│   ├── vsearch.rs      # 向量搜索（含 ANEL 支持）
│   └── query.rs        # 混合搜索（含 ANEL 支持）
└── main.rs             # 入口点
```

### 关键类型

#### AnelError

```rust
pub struct AnelError {
    pub error_code: AnelErrorCode,
    pub status: u16,
    pub title: String,
    pub message: String,
    pub severity: Severity,
    pub recovery_hints: Vec<RecoveryHint>,
    pub trace_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

#### NdjsonRecord<T>

```rust
pub struct NdjsonRecord<T: Serialize> {
    pub record_type: String,  // "metadata", "result", "error"
    pub seq: u64,
    pub payload: T,
}
```

## 使用示例

### 1. 查看命令规范

```bash
# 输出 search 命令的 ANEL 规范
$ qmd search --emit-spec "query"
{
  "version": "1.0",
  "command": "search",
  "input_schema": {...},
  "output_schema": {...},
  "error_codes": ["SEARCH_FAILED", "INDEX_NOT_READY", "QUERY_PARSE_ERROR"]
}
```

### 2. 干运行模式

```bash
# 验证参数但不执行
$ qmd search --dry-run "test query"
[DRY-RUN] Would execute search with:
  query: test query
  limit: 20
  min_score: 0
  collection: None
  search_all: false
```

### 3. NDJSON 流式输出

```bash
# 流式输出，适合智能体处理
$ qmd search "error handling" --format ndjson --limit 5
{"type":"metadata","seq":0,"payload":{"query":"error handling",...}}
{"type":"result","seq":1,"payload":{"docid":...}}
...
```

### 4. 集成智能体工作流

```bash
#!/bin/bash
# 智能体工作流示例

export AGENT_TRACE_ID="agent-workflow-$$"
export AGENT_OUTPUT_FORMAT="ndjson"

# 步骤 1: 发现可用命令
qmd search --emit-spec "test" > /tmp/search-spec.json
qmd vsearch --emit-spec "test" > /tmp/vsearch-spec.json

# 步骤 2: 验证搜索参数
qmd search --dry-run "configuration"

# 步骤 3: 执行搜索并流式处理结果
qmd search "configuration" --format ndjson | jq -c '.payload | select(.score > 0.5)'
```

## 契约测试

契约文件位于 `tests/fixtures/anel/` 目录：

| 文件 | 说明 |
|------|------|
| `search-input.schema.json` | search 命令输入规范 |
| `search-output.schema.json` | search 命令输出规范 |
| `vsearch-input.schema.json` | vsearch 命令输入规范 |
| `vsearch-output.schema.json` | vsearch 命令输出规范 |
| `query-input.schema.json` | query 命令输入规范 |
| `query-output.schema.json` | query 命令输出规范 |
| `error.schema.json` | ANEL 错误格式规范 |

这些 JSON Schema 文件可用于：
- 智能体动态验证输入输出
- 自动化契约测试
- API 文档生成

## QMD 到 ANEL 能力映射

| QMD 能力 | ANEL 原子能力 | 实现状态 |
|----------|---------------|---------|
| `search` | BM25_SEARCH | ✅ 已完成 |
| `vsearch` | VECTOR_SEARCH | ✅ 已完成 |
| `query` | HYBRID_SEARCH | ✅ 已完成 |
| `get` | GET_DOCUMENT | ✅ 已完成 |
| `multi_get` | LIST_DOCUMENTS | ✅ 已完成 |
| `collection add/list/remove` | MANAGE_COLLECTION | ✅ 已完成 |
| `embed` | GENERATE_EMBEDDINGS | ✅ 已完成 |
| `update` | UPDATE_INDEX | ✅ 已完成 |
| `status` | GET_INDEX_STATUS | ✅ 已完成 |
| `cleanup` | CLEANUP_INDEX | ✅ 已完成 |
| `agent` | CLASSIFY_QUERY + ROUTE | ✅ 已完成 |
| `--emit-spec` | 自省能力 | ✅ 已完成 |
| `--dry-run` | 预演模式 | ✅ 已完成 |
| NDJSON 流式输出 | 流式数据 | ✅ 已完成 |

## Phase 2 完成内容 (2026-02-15)

### 完成的工作

1. **为所有命令添加 ANEL 支持**
   - `get` - --emit-spec, --dry-run
   - `multi_get` - --emit-spec, --dry-run
   - `collection` - --emit-spec, --dry-run
   - `embed` - --emit-spec, --dry-run
   - `update` - --emit-spec, --dry-run
   - `status` - --emit-spec, --dry-run
   - `cleanup` - --emit-spec (已有 --dry-run)
   - `agent` - --emit-spec, --dry-run

2. **ANEL 规范扩展**
   - 为每个命令添加了完整的 input_schema 和 output_schema
   - 添加了对应的错误码列表

3. **CLI 参数标准化**
   - 为所有命令添加了 --format, --emit-spec, --dry-run 参数
   - 支持环境变量覆盖 (AGENT_EMIT_SPEC, AGENT_DRY_RUN)

### 使用示例

```bash
# 查看命令规范
qmd get --emit-spec test.txt
qmd embed --emit-spec

# 干运行模式
qmd get --dry-run test.txt
qmd embed --dry-run --collection my_collection
qmd update --dry-run
qmd cleanup --dry-run --older-than 7

# 非交互式 agent 查询
qmd agent --query "how to configure embedding"
```

## 后续计划

### Phase 3: Hyper-Shell 模式探索（P2）

1. 评估 Wasm 插件系统需求
2. 设计 Server 模式架构
3. 编写架构建议文档

## 技术栈

- **语言**: Rust
- **CLI 框架**: clap 4.4
- **错误处理**: anyhow + thiserror
- **序列化**: serde + serde_json

## 注意事项

1. NDJSON 输出在流式场景下非常有用，但不适合人类直接阅读
2. `--emit-spec` 输出的是 JSON Schema，可用于自动化验证
3. 错误恢复提示 (`recovery_hints`) 是 ANEL 的重要扩展，帮助智能体自动恢复
4. 跟踪上下文 (`trace_id`) 对于调试分布式智能体工作流至关重要

## 参考资料

- [ANEL 架构宣言](./ANEL/⭐️%20智能体原生执行层%20(ANEL)%20架构宣言%20v1%200——%20AGI%20时代的"数字躯体"重构.md)
- [ANID 协议规范](./ANEL/⭐️%20ANID%20v1%200%20协议规范%20(RFC).md)
- [原子能力契约文件](./ANEL/⭐️%20原子能力契约文件的具体形式.md)
