# QMD 多语言实现 - 工作日志

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

1. 完善 RRF 融合算法实现
2. 添加 LanceDB 后端支持
3. 实现查询扩展功能
4. 实现 Agent 交互模式
5. 添加单元测试和集成测试
6. 验证编译和运行

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
