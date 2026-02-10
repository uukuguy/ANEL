# QMD 下一阶段开发指南

## 一、当前项目状态

### 已完成工作

| 里程碑 | 状态 | 说明 |
|--------|------|------|
| TypeScript/Bun 架构分析 | ✅ 完成 | 核心模块职责和数据流已明确 |
| 跨语言移植评估 | ✅ 完成 | Go 被选为主要目标语言 |
| 三级搜索系统理解 | ✅ 完成 | BM25 + 向量 + RRF 融合 + LLM 重排序 |

### 待开始工作

- Go 语言移植实现
- MCP 协议支持
- 性能优化和验证

---

## 二、Go 移植路线图

### 阶段一：项目初始化与基础设施

```
目标: 创建 Go 项目脚手架，搭建基础框架

任务清单:
├── [ ] 1.1 创建 Go 模块 (go.mod)
│   ├── 模块名: github.com/tobi/qmd-go
│   └── Go 版本: 1.21+
│
├── [ ] 1.2 配置管理
│   ├── YAML 解析: gopkg.in/yaml.v3
│   ├── 配置验证: go-playground/validator
│   └── 目录: cmd/, internal/, pkg/
│
├── [ ] 1.3 日志系统
│   └── zerolog (结构化日志)
│
└── [ ] 1.4 错误处理
    └── 定义自定义错误类型
```

**目录结构建议**:
```
qmd-go/
├── cmd/
│   └── qmd/
│       └── main.go           # CLI 入口
├── internal/
│   ├── config/               # 配置解析
│   ├── store/                # 数据库层
│   ├── search/               # 搜索算法
│   ├── llm/                  # LLM 集成
│   └── mcp/                  # MCP 协议
├── pkg/
│   ├── collections/          # 集合管理
│   └── formatter/            # 输出格式化
├── test/                      # 测试数据
├── qmd.yaml                  # 配置文件
└── go.mod
```

### 阶段二：SQLite 基础与 FTS5 搜索

```
目标: 实现 SQLite 集成和 BM25 全文搜索

技术选型:
├── 数据库驱动: mattn/go-sqlite3 (CGO) 或 modernc.org/sqlite (纯Go)
├── 全文搜索: SQLite FTS5
└── 向量扩展: sqlite-vec (需验证 Go 绑定)

核心接口:
├── OpenDB(path) -> *sql.DB
├── IndexDocument(doc) error
├── SearchBM25(query) -> []Result
└── ListCollections() []Collection
```

### 阶段三：向量搜索与 RRF 融合

```
目标: 实现语义搜索和排名融合

依赖验证:
├── sqlite-vec Go 绑定状态
├── 嵌入模型调用接口
└── 余弦相似度计算

RRF 实现:
公式: RRF(d) = Σ 1/(k + r(d))
参数: k = 60, 原始查询权重 2x
```

### 阶段四：LLM 集成

```
目标: 实现查询扩展和 LLM 重排序

集成选项:
├── llama.go: https://github.com/go-skllama/llama.go
├── go-llama: https://github.com/go-skynet/go-llama.cpp
└── candle: Rust 绑定 (通过 cgo)

LLM 功能:
├── 查询扩展: 1 个变体生成
├── 重排序: Yes/No + logprob 评分
└── 嵌入: 300M 参数模型
```

### 阶段五：MCP 协议支持

```
目标: 实现 MCP 服务器

评估选项:
├── 检查现有 Go MCP 实现
├── 参考 TypeScript SDK 设计
└── 自定义实现必要部分

工具定义:
├── qmd_search: 混合搜索
├── qmd_vsearch: 向量搜索
├── qmd_index: 文档索引
└── qmd_list: 列出集合
```

---

## 三、技术决策要点

### 3.1 数据库引擎选择

| 方案 | 优点 | 缺点 | 推荐场景 |
|------|------|------|----------|
| mattn/go-sqlite3 | 功能完整、稳定 | 需要 CGO | 生产环境 |
| modernc.org/sqlite | 纯 Go、无 CGO | 功能受限 | 跨平台部署 |

**建议**: 初期使用 `mattn/go-sqlite3`，后期评估纯 Go 替代方案

### 3.2 向量搜索方案

| 方案 | 优点 | 缺点 | 状态 |
|------|------|------|------|
| sqlite-vec | 与 FTS5 集成好 | Go 绑定待验证 | 待测试 |
| faiss | 成熟、高性能 | 体积大 | 备选 |
| annoy | 内存占用低 | 仅 ANN | 备选 |

**建议**: 优先 sqlite-vec，faiss 作为备选

### 3.3 LLM 推理方案

| 方案 | 优点 | 缺点 |
|------|------|------|
| llama.go | 纯 Go 实现 | 活跃度一般 |
| llama-cpp-python | 成熟 | 需要 Python |
| 外部 API | 无本地依赖 | 隐私/网络 |

**建议**: llama.go 或外部 API

### 3.4 部署模式

| 模式 | 描述 | 适用场景 |
|------|------|----------|
| 嵌入式 | CLI + 本地 LLM | 个人使用 |
| 服务化 | REST/gRPC API | 多用户/共享 |
| MCP Server | Claude Agent 集成 | Agent 场景 |

---

## 四、关键注意事项

### 4.1 性能考量

- **GC 影响**: Go GC 可能影响 LLM 推理延迟
- **连接池**: SQLite 需配置合理的 busy_timeout
- **预编译**: 常用查询使用 prepared statement
- **内存映射**: 大型索引考虑 memory-mapped 文件

### 4.2 兼容性保证

| 方面 | 当前格式 | 迁移策略 |
|------|----------|----------|
| YAML 配置 | 标准 YAML | 完全兼容 |
| 数据库索引 | SQLite + sqlite-vec | 复用现有索引 |
| API 接口 | JSON 输出 | 保持一致 |

### 4.3 测试策略

```
测试金字塔:
         ┌─────────┐
        /   E2E    \      <- 端到端测试 (CLI 完整流程)
       /────────────\
      /  集成测试    \    <- 模块间交互
     /────────────────\
    /    单元测试      \  <- 核心算法 (BM25, RRF)
   /────────────────────\

优先级:
1. 核心算法单元测试 (BM25, RRF, 余弦相似度)
2. SQLite 集成测试
3. 搜索质量评估测试
4. 与 TypeScript 版本性能对比
```

---

## 五、开发资源

### 5.1 参考项目

| 项目 | 用途 | URL |
|------|------|-----|
| llama.go | Go LLM 推理 | github.com/go-skllama/llama.go |
| modernc.org/sqlite | 纯 Go SQLite | pkg.go.dev/modernc.org/sqlite |
| sqlite-vec | 向量搜索 | github.com/asg017/sqlite-vec |
| zerolog | 日志框架 | github.com/rs/zerolog |
| cobra | CLI 框架 | github.com/spf13/cobra |

### 5.2 原版代码位置

```
原始 TypeScript 实现:
├── src/store.ts      <- 核心存储和搜索逻辑
├── src/llm.ts        <- LLM 抽象层
├── src/qmd.ts        <- CLI 入口
├── src/collections.ts <- 配置管理
└── src/mcp.ts        <- MCP 服务器
```

### 5.3 模型配置参考

| 用途 | 模型 | 大小 | 说明 |
|------|------|------|------|
| 嵌入 | embeddinggemma-300M-Q8_0 | ~300MB | 内容向量化 |
| 重排序 | qwen3-reranker-0.6b-q8_0 | ~640MB | 结果重排序 |
| 查询扩展 | qmd-query-expansion-1.7B-q4_k_m | ~1.1GB | 查询增强 |

---

## 六、下一步行动

### 立即执行

1. **创建 Go 项目脚手架**
   ```bash
   mkdir qmd-go && cd qmd-go
   go mod init github.com/tobi/qmd-go
   ```

2. **验证 sqlite-vec Go 绑定可行性**
   - 检查 mattn/go-sqlite3 是否支持 sqlite-vec
   - 测试向量存储和检索

3. **实现最小可行产品 (MVP)**
   - 最小功能: CLI + YAML 配置 + FTS5 搜索

### 短期目标 (1-2 周)

- [ ] 完成 SQLite FTS5 搜索实现
- [ ] 验证向量搜索可行性
- [ ] 通过基准测试验证性能

### 中期目标 (1 个月)

- [ ] 完成完整搜索管道
- [ ] 实现 RRF 融合
- [ ] 集成 LLM 重排序

---

## 七、风险与应对

| 风险 | 可能性 | 影响 | 应对措施 |
|------|--------|------|----------|
| sqlite-vec Go 绑定不支持 | 中 | 高 | 切换到 faiss/annoy |
| llama.go 不稳定 | 中 | 中 | 使用外部 API 作为备选 |
| 性能不达预期 | 低 | 中 | 优化算法或使用 Rust |
| MCP Go SDK 缺失 | 高 | 低 | 自定义实现核心功能 |

---

*文档更新日期: 2026-02-11*
