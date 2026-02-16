# QMD 多语言实现 - 工作日志

## Session End - 2026-02-16

**Phase 1-5 完成状态**: ✅ 353+ 测试全部通过

---

## 2026-02-16 (Session 4) - TypeScript 功能移植

### 完成的工作

#### Phase 1: 虚拟路径系统 (store/path.rs)

1. **新建文件**: `src/qmd-rust/src/store/path.rs`
   - `normalize_virtual_path()` - 规范化虚拟路径格式
   - `parse_virtual_path()` - 解析 qmd:// URI 为集合和路径
   - `build_virtual_path()` - 构建虚拟路径
   - `is_virtual_path()` - 判断是否为虚拟路径
   - 15+ 单元测试覆盖

#### Phase 2: `ls` 命令 (cli/ls.rs)

2. **新建文件**: `src/qmd-rust/src/cli/ls.rs`
   - `qmd ls` - 列出所有集合及文件数量
   - `qmd ls collection` - 列出集合下的文件
   - `qmd ls qmd://collection/path` - 使用虚拟路径
   - 支持 `qmd://` 前缀格式

3. **修改文件**: `src/qmd-rust/src/cli/mod.rs`
   - 添加 `Ls` 命令枚举
   - 添加 `LsArgs` 参数结构

4. **修改文件**: `src/qmd-rust/src/main.rs`
   - 添加 `Commands::Ls` 处理分支

5. **修改文件**: `src/qmd-rust/src/anel/mod.rs`
   - 添加 `AnelSpec::ls()` 规范定义

#### Phase 3: `context check` 命令

6. **修改文件**: `src/qmd-rust/src/cli/mod.rs`
   - 添加 `Check` 子命令到 `ContextCommands`

7. **修改文件**: `src/qmd-rust/src/cli/context.rs`
   - 添加 `check_contexts()` 函数
   - 检查没有 context 的集合
   - 检查顶级目录缺少 context 的情况
   - 输出建议添加 context 的命令

#### Phase 4: MCP 资源 (基础设施)

8. **修改文件**: `src/qmd-rust/src/mcp/mod.rs`
   - 实现 `list_resource_templates()` 方法
   - 实现 `read_resource()` 方法 (返回 method_not_found，当前通过 get tool 访问文档)
   - 添加必要的 import

### 修改的文件

| 文件 | 更改类型 |
|------|----------|
| `src/qmd-rust/src/store/path.rs` | 新建 |
| `src/qmd-rust/src/cli/ls.rs` | 新建 |
| `src/qmd-rust/src/cli/mod.rs` | 修改 |
| `src/qmd-rust/src/cli/context.rs` | 修改 |
| `src/qmd-rust/src/mcp/mod.rs` | 修改 |
| `src/qmd-rust/src/main.rs` | 修改 |
| `src/qmd-rust/src/anel/mod.rs` | 修改 |

### 验证方法

```bash
# 构建
cd src/qmd-rust && cargo build --release

# 测试 ls 命令
./target/release/qmd ls
./target/release/qmd ls mycollection

# 测试 context check
./target/release/qmd context check
```

### 待完成

- MCP 资源完整实现 (需要更深入的 rmcp crate API 研究)
- 测试用例补充

---

## 2026-02-16 (Session 3)

### 完成的工作

#### P1: MCP Server 集成测试

1. **Go MCP 集成测试** (`src/qmd-go/internal/mcp/server_test.go`)
   - 17 个测试覆盖全部 5 个工具 (search, vsearch, query, get, status)
   - JSON-RPC 2.0 格式验证、ID 保留、错误处理
   - 含 inputSchema 验证、未知方法/工具处理

2. **Python MCP 集成测试** (`tests/test_mcp_server.py`)
   - 33 个测试，同等覆盖范围
   - 含 edge case: 空查询、缺失参数、不存在的文件

#### P3: 安全闭环 (三语言)

3. **Stream Tap 审计日志**
   - Rust: `src/qmd-rust/src/mcp/middleware.rs` — StreamTap NDJSON 审计到 stderr
   - Go: `src/qmd-go/internal/mcp/server.go` — StreamTap + AuditRecord struct
   - Python: `src/qmd-python/src/mcp/middleware.py` — AuditMiddleware 类

4. **Identity Propagation**
   - 三语言从 `AGENT_IDENTITY_TOKEN` 环境变量提取身份
   - 注入 MCP tool call 上下文，审计记录包含 identity 字段

5. **Dry-Run Interceptor**
   - 三语言检查 `AGENT_DRY_RUN` 环境变量
   - 返回 `[DRY-RUN] Would execute tool '...'` 预览，无副作用

#### P2: 端到端 Demo 场景

6. **E2E Demo 脚本** (`scripts/e2e-demo.py`)
   - 18/18 checks 全部通过
   - 5 阶段: Discovery → Rehearsal → Execution → Error Recovery → Identity
   - Python: MockStore 进程内测试
   - Go: `go test` 子进程验证 (含 audit + dry-run)

### 修改的文件

| 文件 | 更改类型 |
|------|----------|
| src/qmd-rust/src/mcp/middleware.rs | 新增 |
| src/qmd-go/internal/mcp/server.go | 修改 |
| src/qmd-go/internal/mcp/server_test.go | 新增 |
| src/qmd-python/src/mcp/server.py | 修改 |
| src/qmd-python/src/mcp/middleware.py | 新增 |
| scripts/e2e-demo.py | 新增 |
| scripts/demo-e2e.sh | 新增 |
| docs/dev/NEXT_SESSION_GUIDE.md | 修改 |

## 2026-02-13

### 完成的工作

#### Python 实现完善

1. **修复 Python 向量搜索 (Phase 1)**
   - 修改 `src/qmd-python/src/store/mod.py`
   - 实现 `_vector_search_qdrant()` 真正调用 QdrantBackend
   - 添加 QdrantClient 懒加载
   - 添加 `_get_line_count()` 辅助方法

2. **实现 Python LLM Embedding (Phase 2)**
   - 修改 `src/qmd-python/src/llm/router.py`
   - 实现 `_local_embed()` 使用 llama-cpp-python
   - 实现 `_remote_embed()` 使用 OpenAI 兼容 API
   - 添加 `_init_embedder()` 懒加载 GGUF 模型

3. **实现 Python LLM Reranking (Phase 3)**
   - 实现 `_local_rerank()` 本地重排序
   - 实现 `_remote_rerank()` 远程重排序（Cohere API 或 embedding 相似度）
   - 更新 `store/mod.py` 的 `_rerank()` 方法

4. **更新 CLI 命令**
   - 修改 `src/qmd-python/src/cli/commands.py`
   - 实现 `embed` 命令真正生成 embeddings
   - 实现 `vsearch` 命令调用向量搜索

5. **更新依赖**
   - 修改 `src/qmd-python/pyproject.toml`
   - 添加 `llama-cpp-python` 作为可选依赖
   - 添加 `httpx` 依赖

#### Go 实现完善

6. **实现 Go LLM Router (Phase 5)**
   - 修改 `src/qmd-go/internal/llm/router.go`
   - 实现 `llamaServerEmbed()` 调用 llama-server HTTP API
   - 实现 `remoteEmbed()` 使用 OpenAI 兼容 API
   - 实现 `llamaServerRerank()` 和 `remoteRerank()`

7. **更新 Go Store**
   - 修改 `src/qmd-go/internal/store/store.go`
   - 添加 LLM Router 和 Qdrant Backend 到 Store 结构体
   - 实现 `VectorSearchQdrant()` 真正调用 embedding + Qdrant 搜索
   - 实现 `VectorSearchSQLite()` 使用 embedding + sqlite-vec

8. **修复 Qdrant 客户端 API**
   - 修改 `src/qmd-go/internal/store/qdrant.go`
   - 使用正确的 `Query()` API 替代 `SearchPoints()`

### 修改的文件

| 文件 | 更改类型 |
|------|----------|
| src/qmd-python/src/store/mod.py | 修改 |
| src/qmd-python/src/llm/router.py | 修改 |
| src/qmd-python/src/cli/commands.py | 修改 |
| src/qmd-python/pyproject.toml | 修改 |
| src/qmd-go/internal/llm/router.go | 修改 |
| src/qmd-go/internal/store/store.go | 修改 |
| src/qmd-go/internal/store/qdrant.go | 修改 |
| src/qmd-go/internal/mcp/server.go | 修复 |

## 2026-02-11

### 完成的工作

1. **Rust 实现 (qmd-rust)** - 已完成
   - 项目结构和 Cargo.toml 配置
   - CLI 命令模块 (collection, context, get, multi-get, search, vsearch, query, embed, update, status, cleanup)
   - 配置管理模块 (config/mod.rs)
   - 存储后端 (store/mod.rs) - SQLite FTS5 Schema + sqlite-vec
   - LLM 路由层 (llm/mod.rs) - 本地/远程双模式
   - MCP Server (mcp/mod.rs)
   - 输出格式化 (formatter/mod.rs)

2. **Python 实现 (qmd-python)** - 已完成
   - 项目结构和 pyproject.toml 配置
   - Typer CLI 命令模块
   - 配置管理模块
   - 存储后端 (SQLite FTS5)
   - LLM 路由层
   - MCP Server

3. **Go 实现 (qmd-go)** - 已完成
   - 项目结构和 go.mod 配置
   - Cobra CLI 命令模块
   - 配置管理模块
   - 存储后端 (SQLite FTS5)
   - LLM 路由层
   - MCP Server

4. **共享资源** - 已完成
   - 示例配置文件 (index.yaml, example-config.yaml)
   - 共享 README 文档

### 文件统计

| 语言 | 文件数 | 主要模块 |
|------|--------|----------|
| Rust | ~15 | main.rs, cli/*, store/, llm/, mcp/, config/, formatter/ |
| Python | ~15 | app.py, commands.py, store/, llm/, mcp/, config/ |
| Go | ~12 | main.go, cli/*.go, store/, llm/, mcp/, config/ |

### 待完成的工作

1. ~~完善 RRF 融合算法实现~~ ✅ 已完成
2. 添加 LanceDB 后端支持
3. ~~实现查询扩展功能~~ ✅ 已完成
4. 实现 Agent 交互模式
5. ~~添加单元测试和集成测试~~ ✅ 已完成 (35 tests)
6. ~~验证编译和运行~~ ✅ 已完成

### 技术决策

1. **SQLite FTS5 作为缺省 BM25 后端**
   - 保持与原 QMD 工具的行为一致性
   - 使用 porter stemming + unicode61 tokenization

2. **QMD 内置 sqlite-vec 作为缺省向量后端**
   - 保持原有搜索特性
   - 384 维向量, cosine 距离

3. **本地/远程 LLM 双模式**
   - 本地: llama.cpp (GGUF 格式)
   - 远程: OpenAI/Anthropic 兼容 API
   - 自动路由, 本地优先
