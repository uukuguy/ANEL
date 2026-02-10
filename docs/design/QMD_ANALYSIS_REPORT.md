# QMD 项目架构分析与跨语言移植评估报告

## 一、项目概述

**QMD (Query Markup Documents)** 是一个本地运行的个人知识库搜索引擎，专注于 Markdown 文件的全文检索和语义搜索。

| 属性 | 描述 |
|------|------|
| **项目地址** | github.com/tobi/qmd |
| **当前语言** | TypeScript (Bun 运行时) |
| **许可证** | MIT |
| **主要用途** | 个人笔记、会议记录、文档知识库的本地搜索 |

---

## 二、核心功能特性

### 2.1 三级搜索模式

1. **BM25 全文搜索 (`search`)** - 快速关键词匹配
2. **向量语义搜索 (`vsearch`)** - 基于嵌入的相似度检索
3. **混合搜索 + 重排序 (`query`)** - 最高质量搜索结果

### 2.2 混合搜索流程

```
用户查询 → 查询扩展 (LLM生成1个变体)
         ↓
    ┌────┴────┐
    ↓         ↓
 FTS5      Vector
 BM25     (cosine)
    ↓         ↓
    └────┬────┘
         ↓
    RRF 融合 (k=60, 原始查询×2权重)
         ↓
    Top 30 候选
         ↓
    LLM 重排序 (Yes/No + logprob)
         ↓
    位置感知混合:
    - Top 1-3:  75% RRF / 25% reranker
    - Top 4-10: 60% RRF / 40% reranker
    - Top 11+:  40% RRF / 60% reranker
         ↓
    最终结果
```

### 2.3 Agent/MCP 集成

- MCP (Model Context Protocol) 服务器
- 虚拟路径系统 (`qmd://collection/path`)
- JSON/CSV/XML/MD 多格式输出

---

## 三、架构设计分析

### 3.1 模块结构

```
qmd/
├── src/
│   ├── qmd.ts           # CLI 入口点，命令路由
│   ├── store.ts         # 核心数据库和搜索逻辑
│   ├── llm.ts           # LLM 抽象层，node-llama-cpp封装
│   ├── collections.ts   # YAML配置管理
│   ├── formatter.ts     # 输出格式化
│   ├── mcp.ts          # MCP服务器
│   └── *.test.ts       # 测试文件
├── finetune/            # 模型微调 (Python)
└── test/               # 评估测试框架
```

### 3.2 数据流架构

```
用户输入
    ↓
CLI解析 (qmd.ts)
    ↓
┌─────────────────────────────────────┐
│           Store 层 (store.ts)        │
│  ┌───────────┐    ┌─────────────┐   │
│  │  SQLite   │    │ sqlite-vec  │   │
│  │  FTS5     │    │  向量索引   │   │
│  └───────────┘    └─────────────┘   │
└─────────────────────────────────────┘
    ↓           ↓
┌─────────────────────────────────────┐
│           LLM 层 (llm.ts)            │
│  嵌入模型 → 重排序 → 查询扩展        │
│  (GGUF格式本地模型)                 │
└─────────────────────────────────────┘
    ↓
输出格式化 (formatter.ts)
```

### 3.3 依赖注入设计

- **StoreFactory**: 数据库连接工厂
- **LLMSession**: 带生命周期的 LLM 会话管理
- **配置层**: YAML 配置与运行时分离

---

## 四、核心技术栈

### 4.1 当前技术栈

| 层级 | 技术选型 | 说明 |
|------|---------|------|
| **运行时** | Bun >= 1.0.0 | 性能优于 Node.js，原生支持 TypeScript |
| **语言** | TypeScript 5.x | 静态类型检查 |
| **数据库** | SQLite + sqlite-vec | 嵌入式向量搜索 |
| **LLM框架** | node-llama-cpp | 本地 GGUF 模型推理 |
| **协议** | MCP SDK 1.25.1 | Claude Agent 集成 |
| **配置** | YAML | 用户配置 |
| **验证** | Zod | 运行时类型验证 |

### 4.2 外部依赖

```
@modelcontextprotocol/sdk  ^1.25.1  # MCP协议
node-llama-cpp            ^3.14.5  # LLM推理
sqlite-vec                ^0.1.7   # 向量搜索
yaml                       ^2.8.2   # YAML解析
zod                        ^4.2.1   # 验证
```

### 4.3 模型配置

| 用途 | 模型 | 大小 |
|------|------|------|
| 嵌入 | embeddinggemma-300M-Q8_0 | ~300MB |
| 重排序 | qwen3-reranker-0.6b-q8_0 | ~640MB |
| 查询扩展 | qmd-query-expansion-1.7B-q4_k_m | ~1.1GB |

---

## 五、实现亮点

### 5.1 数据库设计

```sql
-- 存储位置: ~/.cache/qmd/index.sqlite
collections           -- 索引目录配置
path_contexts        -- 路径上下文描述
documents            -- 文档元数据 + docid (6位hash)
documents_fts        -- FTS5 全文索引
content_vectors      -- 嵌入块 (800 tokens, 15% overlap)
vectors_vec          -- sqlite-vec 向量索引
llm_cache            -- LLM 响应缓存
```

**设计特点**:
- SQLite 嵌入式，无需独立数据库服务
- FTS5 实现 BM25 全文搜索
- sqlite-vec 实现向量相似度搜索
- 内容哈希去重 + docid 短标识符

### 5.2 文档分块策略

- **Token 分块**: 800 tokens/块
- **重叠**: 15% (120 tokens)
- **同步回退**: 字符级近似 (~4 chars/token)

### 5.3 路径处理

- Unix/Windows/Git Bash 路径兼容
- 虚拟路径系统 (`qmd://`)
- 上下文前缀匹配

---

## 六、跨语言移植评估

### 6.1 语言选择矩阵

| 语言 | 适合度 | 理由 | 挑战 |
|------|--------|------|------|
| **Rust** | ⭐⭐⭐⭐⭐ | 高性能、SQLite绑定成熟、async生态好 | LLM推理库较少 |
| **Go** | ⭐⭐⭐⭐ | 并发模型好、跨平台编译、WASM支持好 | 向量搜索库较少 |
| **Python** | ⭐⭐⭐⭐ | LLM生态最丰富、易于实现 | 性能较低、运行时依赖 |
| **C++** | ⭐⭐⭐⭐⭐ | 性能最佳、可复用llama.cpp | 开发效率低 |
| **Java** | ⭐⭐⭐ | 成熟生态、企业级 | 运行时较大、内存占用高 |

### 6.2 推荐移植方案

#### 方案 A: Rust (推荐)

**优势**:
- `rusqlite` + `sqlite-vec` 兼容
- `candle` 或 `llama.cpp` Rust绑定可用
- 性能与 Bun/TypeScript 相当或更优
- 静态编译，单二进制分发

**关键依赖**:
```toml
[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
sqlite-vec = "0.1"
candle-core = "0.6"  # 或 llama.cpp-rs
tokio = { version = "1", features = ["full"] }
clap = "4"  # CLI框架
```

**可行性**: 高。核心逻辑可直接移植，LLM推理可通过 llama.cpp 的 Rust FFI 或 candle 库实现。

#### 方案 B: Go (强烈推荐)

**优势**:
- 出色的跨平台编译 (静态链接)
- `modernc.org/sqlite` 纯Go实现
- 活跃的 AI/向量搜索社区
- WASM 支持良好 (可在浏览器/Claude Agent运行)
- 并发模型天然适合搜索任务

**关键依赖**:
```go
import (
  sqlite "modernc.org/sqlite"
  # 或 github.com/mattn/go-sqlite3 (CGO)
  "github.com/go-llama/llama.go"
  "github.com/charmbracelet/glow"
)
```

**可行性**: 非常高。`go-llama` 项目提供 llama.cpp 的 Go 绑定，sqlite-vec 可通过 CGO 或纯Go实现。

#### 方案 C: Python (备选)

**优势**:
- `llama-cpp-python` 直接可用
- `chromadb`, `faiss` 等向量搜索成熟
- 开发和迭代速度快

**劣势**:
- 性能约为 Rust/Go 的 1/5-1/10
- 依赖管理复杂
- 无法静态编译

**可行性**: 高，但生产环境不推荐。

### 6.3 核心模块可移植性分析

| 模块 | Rust | Go | Python | C++ |
|------|------|-------|--------|------|
| CLI/命令路由 | ✅ `clap` | ✅ `cobra` | ✅ `argparse` | ✅ |
| SQLite/FTS5 | ✅ `rusqlite` | ✅ sqlite | ✅ `sqlite3` | ✅ |
| 向量搜索 | ⚠️ 需要适配 | ⚠️ 需要适配 | ✅ `faiss` | ✅ |
| LLM推理 | ⚠️ `candle` | ⚠️ `llama.go` | ✅ `llama-cpp` | ✅ `llama.cpp` |
| MCP协议 | ⚠️ 需移植SDK | ⚠️ 需移植SDK | ✅ | ⚠️ |
| YAML配置 | ✅ `serde_yaml` | ✅ `go-yaml` | ✅ `pyyaml` | ⚠️ |
| 跨平台文件 | ✅ `walkdir` | ✅ `filepath` | ✅ `pathlib` | ⚠️ |

### 6.4 移植优先级

```
高优先级 (核心功能):
├── CLI 入口和命令解析
├── SQLite FTS5 全文搜索
├── 文档索引和管理
├── 集合配置管理

中优先级 (搜索增强):
├── 向量搜索 (sqlite-vec)
├── 嵌入生成
├── RRF 排名融合

低优先级 (AI 集成):
├── LLM 重排序
├── 查询扩展
├── MCP 服务器
```

---

## 七、结论与建议

### 7.1 评估总结

QMD 是一个设计良好、性能优化的本地搜索引擎。其核心价值在于:

1. **轻量级**: 单用户场景，无服务器依赖
2. **混合搜索**: BM25 + 向量 + 重排序的多级融合
3. **Agent Ready**: MCP 协议原生支持
4. **本地 LLM**: 隐私友好，无需云服务

### 7.2 移植建议

**首选语言**: **Go**

理由:
1. 与 TypeScript 相当的开发效率
2. 优秀的跨平台支持 (静态编译)
3. WASM 支持可扩展使用场景
4. `llama.go` 项目提供 llama.cpp 的完整 Go 绑定
5. 并发模型天然适合搜索任务

**备选语言**: **Rust**

理由:
1. 最高性能保证
2. 内存安全
3. 静态编译单二进制

**不推荐**: Python (除非原型验证)

### 7.3 移植风险

| 风险 | 等级 | 缓解措施 |
|------|------|----------|
| LLM 推理库成熟度 | 中 | 使用 llama.cpp CGO 绑定 |
| sqlite-vec Go 绑定 | 中 | 考虑使用 faiss/annoy 作为替代 |
| MCP 协议 Go 移植 | 低 | MCP SDK 为 TypeScript，Go需自行实现 |

---

## 八、附录: 文件清单

| 文件 | 行数估计 | 职责 |
|------|----------|------|
| `src/qmd.ts` | ~1500 | CLI 入口 |
| `src/store.ts` | ~3000+ | 核心存储和搜索 |
| `src/llm.ts` | ~2000+ | LLM 抽象层 |
| `src/mcp.ts` | ~1000 | MCP 服务器 |
| `src/collections.ts` | ~500 | 配置管理 |
| `src/formatter.ts` | ~500 | 输出格式化 |
| `src/qmd.ts` | ~500 | CLI 命令处理 |

---

*报告生成日期: 2026-02-11*
