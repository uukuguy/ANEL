# Hyper-Shell 模式架构设计

## 概述

本文档描述 QMD 项目的 Hyper-Shell 模式架构设计方案，旨在实现长期运行的 Server 模式和可扩展的插件系统。

**文档版本**: 1.0
**创建日期**: 2026-02-15
**项目阶段**: Phase 3

---

## 1. 当前架构分析

### 1.1 QMD 现有 Server 能力

| 模式 | 说明 | 特点 |
|------|------|------|
| **CLI 模式** | 每次调用加载模型 | 轻便但延迟高 (~3s) |
| **MCP Stdio** | 标准输入/输出 | 适合临时调用 |
| **MCP HTTP** | HTTP Server (8080) | 模型预加载，复用连接 |

### 1.2 现有 MCP HTTP 实现

```rust
// src/qmd-rust/src/mcp/mod.rs
pub fn run_server(args: &McpArgs, config: &Config) -> Result<()> {
    match args.transport.as_str() {
        "stdio" => run_stdio_server(config),
        "http" | "sse" => run_http_server        _ => anyhow(args, config),
::bail!("Unknown transport: {}", args.transport),
    }
}
```

**当前能力**:
- 5 个 MCP 工具: search, vsearch, query, get, status
- 模型缓存: embedding 模型加载一次，复用上下文
- 连接管理: 每个请求独立处理

### 1.3 性能对比

| 模式 | 模型加载 | 查询延迟 | 适用场景 |
|------|----------|----------|----------|
| MCP Stdio | ~3s (每次) | ~100ms | 临时/移动使用 |
| **MCP HTTP** | ~3s (首次) | ~10ms | **AI 高频调用** ✅ |
| CLI | ~3s (每次) | ~100ms | 偶尔使用 |

---

## 2. Wasm 插件系统评估

### 2.1 插件系统需求分析

**QMD 需要插件系统扩展的功能**:
1. **自定义评分器** - 用户定义的文档相关性算法
2. **预处理管道** - 自定义文本处理/分词
3. **后处理过滤器** - 结果过滤/增强
4. **新搜索后端** - 连接外部搜索引擎
5. **自定义输出格式** - 特定业务格式

### 2.2 Rust Wasm 运行时对比

| 特性 | wasmtime | wasmer |
|------|----------|--------|
| **开发者** | Bytecode Alliance | Wasmer |
| **编译策略** | Cranelift JIT | JIT/AOT |
| **WASI 支持** | ✅ 原生 | ✅ 完整 |
| **安全沙箱** | ✅ 严格 | ✅ 严格 |
| **文档质量** | 丰富 | 良好 |
| **crates.io 下载** | ~1800万 | ~900万 |
| **许可** | Apache 2.0 / MIT | MIT |

**推荐选择**: **wasmtime**
- Bytecode Alliance 背书，长期维护有保障
- Cranelift 编译器，性能优异
- 文档最丰富，社区活跃

### 2.3 插件系统架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                      QMD Server                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐   │
│  │   MCP/HTTP  │  │  Plugin     │  │  Model         │   │
│  │   Server    │  │  Manager    │  │  Cache         │   │
│  └─────────────┘  └─────────────┘  └─────────────────┘   │
│         │                │                                  │
│         └────────────────┼──────────────────────────────  │
│                          ▼                                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Wasmtime Runtime                        │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐        │   │
│  │  │ Plugin 1  │ │ Plugin 2  │ │ Plugin 3  │        │   │
│  │  │ (scorer)  │ │ (filter)  │ │ (backend) │        │   │
│  │  └───────────┘ └───────────┘ └───────────┘        │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 2.4 WIT 接口定义

```wit
// plugin-api/wit/plugin.wit
package qmd:plugin

world plugin {
    export scorer: func(query: string, doc: string) -> f32
    export filter: func(doc: string) -> bool
    export transform: func(input: string) -> string
}
```

---

## 3. Server 模式架构设计

### 3.1 设计目标

1. **高性能** - 毫秒级查询延迟
2. **可扩展** - Wasm 插件支持
3. **可观测** - 日志/指标/追踪
4. **可配置** - 灵活的配置选项

### 3.2 架构组件

```
┌─────────────────────────────────────────────────────────────────┐
│                         QMD Server                              │
├─────────────────────────────────────────────────────────────────┤
│  HTTP Server (Axum)                                            │
│  ├── /health     - 健康检查                                     │
│  ├── /search     - BM25 搜索                                   │
│  ├── /vsearch    - 向量搜索                                     │
│  ├── /query      - 混合搜索                                     │
│  ├── /mcp        - MCP 协议 (JSON-RPC)                         │
│  └── /plugins    - 插件管理                                     │
├─────────────────────────────────────────────────────────────────┤
│  Middleware Layer                                              │
│  ├── Rate Limiter    - 请求限流                                 │
│  ├── Auth            - 认证授权                                 │
│  ├── Cache           - 结果缓存                                 │
│  └── Tracing         - 分布式追踪                               │
├─────────────────────────────────────────────────────────────────┤
│  Core Services                                                 │
│  ├── Search Engine    - BM25 + Vector + Hybrid                │
│  ├── LLM Router       - Embedding + Reranking                 │
│  ├── Plugin Manager   - Wasm 插件生命周期                       │
│  └── Config Manager   - 运行时配置                              │
├─────────────────────────────────────────────────────────────────┤
│  Model Layer                                                   │
│  ├── Embedding Model   - nomic-embed-text-v1.5                │
│  └── Reranker Model   - bge-reranker-v2-m3                   │
└─────────────────────────────────────────────────────────────────┘
```

### 3.3 配置选项

```yaml
# server.yaml
server:
  host: "0.0.0.0"
  port: 8080
  workers: 4              # 工作线程数

models:
  embedding:
    model: "nomic-embed-text-v1.5"
    gpu: true
    preload: true
  reranker:
    model: "bge-reranker-v2-m3"
    gpu: true
    preload: true

plugins:
  directory: "~/.qmd/plugins"
  enabled: true

rate_limit:
  requests_per_minute: 60
  burst: 10

cache:
  enabled: true
  ttl_seconds: 3600
  max_entries: 1000
```

### 3.4 API 设计

**REST API**:
```
GET  /health              - 健康检查
GET  /collections        - 列出集合
POST /search             - BM25 搜索
POST /vsearch            - 向量搜索
POST /query              - 混合搜索
GET  /documents/:path    - 获取文档
GET  /stats              - 索引统计
```

**MCP 协议** (JSON-RPC 2.0):
```
POST /mcp                - MCP 工具调用
```

---

## 4. 实现路线图

### Phase 3.1: Server 基础架构 (2-3 周)

1. **HTTP Server 增强**
   - 迁移到独立的 HTTP Server (脱离 MCP)
   - 添加 REST API 端点
   - 实现中间件层 (rate limit, auth)

2. **模型管理**
   - 模型预加载机制
   - GPU 内存管理
   - 模型热更新

3. **连接池**
   - SQLite 连接池
   - Qdrant 连接池 (可选)

### Phase 3.2: Wasm 插件系统 (3-4 周)

1. **插件 API 设计**
   - WIT 接口定义
   - 宿主/插件边界

2. **Wasmtime 集成**
   - 插件加载/卸载
   - 资源限制
   - 错误隔离

3. **插件管理 CLI**
   - `qmd plugin install <path>`
   - `qmd plugin list`
   - `qmd plugin remove <name>`

### Phase 3.3: 可观测性 (1-2 周)

1. **日志系统**
   - 结构化日志
   - 日志级别控制
   - 日志轮转

2. **指标收集**
   - 请求延迟
   - 模型推理时间
   - 缓存命中率

3. **分布式追踪**
   - OpenTelemetry 集成
   - Span 传播

---

## 5. 技术选型

| 组件 | 选型 | 理由 |
|------|------|------|
| HTTP 框架 | Axum | 成熟，生态丰富，与 Tower 集成 |
| 中间件 | Tower | 统一接口，易扩展 |
| Wasm 运行时 | Wasmtime | Bytecode Alliance，文档丰富 |
| 插件接口 | WASI + WIT | 标准化的组件模型 |
| 指标 | metrics + prometheus | Rust 生态标准 |
| 追踪 | opentelemetry | 厂商中立 |
| 日志 | tracing | async 友好 |

---

## 6. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| Wasm 插件沙箱逃逸 | 高 | 严格资源限制，定期更新 wasmtime |
| 模型内存爆炸 | 高 | 显存/内存监控，自动降级 |
| HTTP 连接耗尽 | 中 | 连接池，超时控制 |
| 插件依赖冲突 | 低 | 版本锁定，隔离环境 |

---

## 7. 后续工作

1. **Go/Python Server 实现** - 统一三版本架构
2. **Qdrant 集成完善** - 向量后端增强
3. **多租户支持** - 隔离用户数据

---

*文档结束*
