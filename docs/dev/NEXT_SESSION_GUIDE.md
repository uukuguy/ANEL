# 下一阶段开发指南

## 当前进度

### 已完成 ✅

1. **Rust 实现 (qmd-rust)** - 基础框架
   - CLI 命令模块 (collection, context, get, multi-get, search, vsearch, query, embed, update, status, cleanup, mcp, agent)
   - 配置管理模块 (YAML 配置加载/保存)
   - SQLite FTS5 存储后端 (Schema, BM25 搜索)
   - LLM 路由层 (本地/远程双模式 placeholder)
   - MCP Server 框架
   - 输出格式化

2. **Python 实现 (qmd-python)** - 基础框架
   - Typer CLI 命令
   - 配置管理
   - SQLite FTS5 存储后端
   - LLM 路由层
   - MCP Server

3. **Go 实现 (qmd-go)** - 基础框架
   - Cobra CLI 命令
   - 配置管理
   - SQLite FTS5 存储后端
   - LLM 路由层
   - MCP Server

4. **共享资源**
   - 配置文件模板
   - 项目文档

### 待完成 ❌

1. **RRF 融合算法** - 当前仅返回 BM25 结果
2. **LanceDB 后端** - 未实现
3. **查询扩展** - 未实现
4. **Agent 交互模式** - 仅框架
5. **单元测试** - 无测试用例
6. **sqlite-vec 集成** - 向量搜索未完成

---

## 下阶段重点任务

### 1. 完善 Store 模块 (优先级: 高)

#### RRF 融合算法

文件: `src/qmd-rust/src/store/mod.rs`

```rust
fn rrf_fusion(
    result_lists: &[Vec<SearchResult>],
    weights: Option<Vec<f32>>,
    k: u32,
) -> Vec<SearchResult> {
    // 当前实现是 placeholder
    // 需要实现完整的 RRF 算法
}
```

#### sqlite-vec 向量搜索

需要在 Rust 实现中添加：

```rust
fn vector_sqlite_search(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> {
    // 1. 生成查询嵌入
    let embedding = self.llm.embed(&[query]).await?;
    // 2. 使用 sqlite-vec 搜索
    // 3. 返回结果
}
```

### 2. 添加 LanceDB 后端 (优先级: 中)

三个实现都需要添加 LanceDB 支持：

| 实现 | 需要添加的模块 |
|------|---------------|
| Rust | `src/qmd-rust/src/store/lancedb_fts.rs` |
| Go | `internal/store/lancedb.go` |
| Python | `src/store/lancedb.py` |

### 3. 实现查询扩展 (优先级: 中)

LLM 模块需要实现 `expand_query` 方法：

```rust
pub fn expand_query(&self, query: &str) -> Result<Vec<String>> {
    // 使用 LLM 生成查询变体
    // 返回原始查询 + 2-3 个变体
}
```

### 4. Agent 交互模式 (优先级: 低)

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

### 4. 错误处理
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
