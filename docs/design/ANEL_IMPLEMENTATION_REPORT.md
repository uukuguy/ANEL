# ANEL 设计实现报告

## 概述

ANEL (Agent-Native Execution Layer，智能体原生执行层) 是 QMD 项目的智能体集成方案，使 QMD 能够被 AI Agent 高效调用。

## 实现状态

| 功能 | 状态 | 说明 |
|------|------|------|
| search 命令 | ✅ | --emit-spec, --dry-run, NDJSON |
| vsearch 命令 | ✅ | --emit-spec, --dry-run, NDJSON |
| query 命令 | ✅ | --emit-spec, --dry-run, NDJSON |
| get 命令 | ✅ | --emit-spec, --dry-run |
| multi_get 命令 | ✅ | --emit-spec, --dry-run |
| collection 命令 | ✅ | --emit-spec, --dry-run |
| embed 命令 | ✅ | --emit-spec, --dry-run |
| update 命令 | ✅ | --emit-spec, --dry-run |
| status 命令 | ✅ | --emit-spec, --dry-run |
| cleanup 命令 | ✅ | --emit-spec |
| agent 命令 | ✅ | --emit-spec, --dry-run, --query |
| 契约测试 | ✅ | JSON Schema 文件 |

## 核心组件

### 1. ANEL 错误类型 (AnelError)
- 定义了 17 种错误码
- 支持 RFC 7807 Problem Details 格式
- 提供 recovery_hints 恢复提示

### 2. 跟踪上下文 (TraceContext)
- 从环境变量自动加载 trace_id
- 支持 identity_token 身份认证

### 3. 命令规范 (AnelSpec)
- 每个命令输出 JSON Schema
- 包含 input_schema 和 output_schema
- 列出错误码

### 4. NDJSON 流式输出
- 支持 metadata/result/error 类型
- 适合智能体流式处理

## CLI 参数

所有命令支持以下 ANEL 参数：

| 参数 | 说明 |
|------|------|
| --emit-spec | 输出 JSON Schema 规范并退出 |
| --dry-run | 验证参数但不执行 |
| --format | 输出格式 (cli/json/ndjson) |

环境变量覆盖：
- AGENT_TRACE_ID
- AGENT_EMIT_SPEC
- AGENT_DRY_RUN

## 使用示例

```bash
# 查看命令规范
qmd search --emit-spec "query"
qmd embed --emit-spec
qmd collection --emit-spec list

# 干运行验证
qmd get --dry-run test.txt
qmd embed --dry-run --collection my_col
qmd cleanup --dry-run --older-than 7

# 非交互式 agent 查询
qmd agent --query "how to configure embedding"

# NDJSON 流式输出
qmd search "error handling" --format ndjson | jq -c '.payload'
```

## 涉及文件

### 核心模块
- `src/anel/mod.rs` - ANEL 核心实现

### CLI 命令
- `src/cli/search.rs`
- `src/cli/vsearch.rs`
- `src/cli/query.rs`
- `src/cli/get.rs`
- `src/cli/multi_get.rs`
- `src/cli/collection.rs`
- `src/cli/embed.rs`
- `src/cli/update.rs`
- `src/cli/status.rs`
- `src/cli/cleanup.rs`
- `src/cli/agent.rs`
- `src/cli/mod.rs` - 通用参数定义

### 契约测试
- `tests/fixtures/anel/*.schema.json`

### 文档
- `docs/design/QMD_ANEL_IMPLEMENTATION.md`
- `docs/ANEL/` - 架构设计文档

## 技术细节

### 错误码映射
```rust
AnelErrorCode::SearchFailed -> 500
AnelErrorCode::NotFound -> 404
AnelErrorCode::InvalidInput -> 400
AnelErrorCode::CollectionNotFound -> 404
// ... 共 17 种错误码
```

### NDJSON 格式
```json
{"type":"metadata","seq":0,"payload":{"query":"test","total":2}}
{"type":"result","seq":1,"payload":{"docid":"...","path":"...","score":0.85}}
{"type":"error","seq":2,"payload":{"error_code":"NOT_FOUND","message":"..."}}
```

## 后续计划

### Phase 3: Hyper-Shell 模式 (待实现)
- Wasm 插件系统
- 远程执行协议
- 标准工具库

### 可选增强
- Go/Python 版本 ANEL 支持
- MCP HTTP Server 集成
- Qdrant 向量后端
