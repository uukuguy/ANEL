# 下一阶段开发指南

## 当前进度

### 已完成 ✅

1. **Rust 实现 (qmd-rust)** - 基础框架
   - CLI 命令模块 (collection, context, get, multi-get, search, vsearch, query, embed, update, status, cleanup, agent)
   - 配置管理模块 (YAML 配置加载/保存)
   - SQLite FTS5 存储后端 (Schema, BM25 搜索)
   - LLM 路由层 (本地/远程双模式)
   - **RRF 融合算法** - 已实现
   - **查询扩展** - 已实现（基于规则 + LLM placeholder）
   - **输出格式化**
   - **向量搜索框架** - embed_sync/rerank_sync 包装器已实现
   - **Hybrid Search 修复** - LLM reranking 功能已启用

2. **Python 实现 (qmd-python)** - 基础框架
   - Typer CLI 命令
   - 配置管理
   - SQLite FTS5 存储后端
   - LLM 路由层

3. **Go 实现 (qmd-go)** - 基础框架
   - Cobra CLI 命令
   - 配置管理
   - SQLite FTS5 存储后端
   - LLM 路由层

4. **共享资源**
   - 配置文件模板
   - 项目文档

### 待完成 ❌

1. **LanceDB 后端** - 未实现
2. **sqlite-vec 向量搜索** - 框架已实现，需启用 feature 测试
3. **Agent 交互模式** - 仅框架
4. **单元测试** - 无测试用例
5. **MCP Server** - 暂时禁用（SDK API 不稳定）
6. **Rust 警告清理** - 移除未使用的变量和导入

---

## 下阶段重点任务

### 1. 完善向量搜索 (优先级: 高) ✅

**已修复** (2026-02-11):

- ✅ 实现 `embed_sync` 同步包装器 (`src/llm/mod.rs:149`)
- ✅ 实现 `rerank_sync` 同步 reranking 包装器 (`src/llm/mod.rs:159`)
- ✅ 修复 `hybrid_search` 使用 LLM reranking (`src/store/mod.rs`)
- ✅ 实现 `vector_search_with_embedder` 方法 (`src/store/mod.rs`)
- ✅ 实现 sqlite-vec 搜索框架 (`src/store/mod.rs:vector_search_sqlite_vec`)

**启用方式**: 构建时添加 `--features sqlite-vec`

**待完成**:
- 启用 sqlite-vec feature 进行实际测试
- 集成真正的 llama.cpp embedding 模型

### 2. 添加 LanceDB 后端 (优先级: 中)

三个实现都需要添加 LanceDB 支持：

| 实现 | 需要添加的模块 |
|------|---------------|
| Rust | `src/qmd-rust/src/store/lancedb.rs` |
| Go | `internal/store/lancedb.go` |
| Python | `src/store/lancedb.py` |

### 3. Agent 交互模式 (优先级: 中)

完善 `cli/agent.rs`：

```rust
fn run_interactive_agent(&self) -> Result<()> {
    loop {
        let query = self.read_user_input()?;
        let intent = self.classify_intent(&query)?;

        match intent {
            Intent::Keyword => self.bm25_search(&query)?,
            Intent::Semantic => self.vector_search(&query)?,
            Intent::Complex => self.hybrid_search(&query)?,
        }
    }
}
```

### 4. MCP Server (优先级: 低)

重新启用 MCP 模块，需要：
1. 更新 MCP SDK API 调用（当前 0.0.3 版本 API 有变化）
2. 添加正确的 ServerBuilder 用法

### 5. 测试 (优先级: 高)

添加单元测试：

```
tests/
├── test_rrf.py          # RRF 融合测试
├── test_search.py       # 搜索一致性测试
├── test_backends.py     # 后端一致性测试
└── test_formatters.py   # 输出格式化测试
```

---

## 快速开始

```bash
# Rust 构建测试
cd src/qmd-rust
cargo build --release

# Python 安装测试
cd src/qmd-python
pip install -e .

# Go 构建测试
cd src/qmd-go
go build -o qmd ./cmd/qmd
```

---

## 检查清单

### 代码质量
- [ ] Rust: `cargo clippy` 无警告
- [ ] Python: `ruff check .` 无错误
- [ ] Go: `go vet ./...` 无错误

### 功能验证
- [ ] CLI help 输出正确
- [ ] 配置文件加载成功
- [ ] SQLite FTS5 搜索返回结果
- [ ] RRF 融合排序正确
- [ ] 查询扩展生成变体

### 文档
- [ ] API 文档更新
- [ ] CLI 用法示例
- [ ] 配置文件说明

---

## 注意事项

### 1. Schema 兼容性
所有实现必须使用相同的 SQLite Schema：

```sql
CREATE VIRTUAL TABLE documents_fts USING fts5(
    filepath, title, body,
    tokenize='porter unicode61'
);
```

### 2. CLI 参数兼容性
必须与原 QMD 工具保持一致：

```bash
qmd search <query> [-n <num>] [-c <collection>] [--all]
qmd vsearch <query> [-n <num>] [-c <collection>] [--all]
qmd query <query> [-n <num>] [-c <collection>] [--all]
```

### 3. 路径处理
使用 `shellexpand` 处理 `~` 路径：

```rust
let path = shellexpand::tilde("~/notes").parse::<PathBuf>()?;
```

### 4. 异步处理
如果需要在同步函数中调用异步代码，使用：

```rust
let result = tokio::runtime::Handle::current().block_on(async {
    llm.embed(&[query]).await
})?;
```

### 5. 错误处理
使用 `anyhow` 简化错误传播：

```rust
fn search(&self) -> Result<Vec<SearchResult>> {
    // ... 实现
    Ok(results)
}
```

---

## 参考链接

- [sqlite-vec](https://github.com/asg017/sqlite-vec)
- [LanceDB Python](https://lancedb.github.io/lancedb/)
- [RRF 融合算法](https://plg.uwaterloo.ca/~gvcormac/cormacksph04-rrf.pdf)
- [MCP SDK](https://github.com/modelcontextprotocol/spec)

---

## 2026-02-11 代码变更

### 本次会话修复

| 文件 | 变更 |
|------|------|
| `src/llm/mod.rs` | 添加 `embed_sync()` 和 `rerank_sync()` 同步包装器 |
| `src/store/mod.rs` | 修复 `hybrid_search` 使用 LLM reranking；添加向量搜索框架 |
| `src/config/mod.rs` | 修复 `ModelsConfig` Default 实现 |
| `src/cli/mod.rs` | 添加子模块声明 |
| `src/main.rs` | 修复 CLI 解析；暂时禁用 MCP 模块 |

### 验证命令

```bash
cd src/qmd-rust
cargo build --release
./target/release/qmd-rust --help
```
