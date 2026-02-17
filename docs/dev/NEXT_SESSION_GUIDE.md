# Next Session Guide - ANEL Project

**Last Updated**: 2026-02-18
**Current Status**: Phase 1-6 完成，Phase 9 (ANEL Copilot) 完成 + 增强 Phase A/B/C 完成，**anel-copilot MCP Server + CLI + 82 tests**
**Branch**: dev

## 当前状态

### Phase 1-5 完成 ✅

| Phase | 内容 | 测试数量 |
|-------|------|---------|
| 1 | Rust ANEL 协议层 | 88 |
| 2 | Rust Store 核心 + 搜索管线 | 127 |
| 3 | Rust MCP Server | 22 |
| 4 | Rust CLI 端到端 | 83 |
| 5 | Rust LLM + Eval + 路径 | 33 |

**总计**: 353+ tests 全部通过

### Phase 6: 存储层兼容性修复 ✅

**Rust 版本**: ✅ 完成
- 新增 `content` 表（与原版一致）
- 删除 `documents.doc` 字段（改用外键关联）
- 删除 `collections` 表（改用 YAML 配置）
- 删除 `path_contexts` 表（改用 YAML 配置）
- 添加 llm_cache 表
- 测试: 242+ 全部通过

**Go 版本**: ✅ 完成
- 添加 llm_cache 表
- 修复 indexes

**Python 版本**: ✅ 完成
- 重写 schema 与原版一致
- 测试: 174 全部通过

### 三版本 Schema 对齐 ✅

| 表 | 原版 | Rust | Go | Python |
|---|------|------|-----|--------|
| content | ✅ | ✅ | ✅ | ✅ |
| documents | ✅ | ✅ | ✅ | ✅ |
| vectors_vec | ✅ | ✅ | ✅ | ✅ |
| content_vectors | ✅ | ✅ | ✅ | ✅ |
| documents_fts | ✅ | ✅ | ✅ | ✅ |
| llm_cache | ✅ | ✅ | ✅ | ✅ |

### TypeScript 功能移植 ✅

| 功能 | 文件 | 状态 |
|------|------|------|
| 虚拟路径系统 | store/path.rs | ✅ 完成 |
| ls 命令 | cli/ls.rs | ✅ 完成 |
| context check | cli/context.rs | ✅ 完成 |
| MCP 资源基础设施 | mcp/mod.rs | ✅ 完成 |

### 配置文件一致性调整 ✅

**修复的问题**:
- Embedding 维度: 从 384 改为 768（匹配 embeddinggemma-300M）
- Qdrant 配置: 添加 Qdrant 后端配置模板

**修改的文件**:
- `src/shared/index.yaml`
- `src/shared/example-config.yaml`
- `src/shared/README.md`
- `src/CLAUDE.md`

### LanceDB 后端实现 ✅ (2026-02-17)

**完成内容**:
- 编译验证: `cargo build --features lancedb` 成功
- 测试验证: 26 tests passed
- 配置支持: 添加 `LanceDbConfig` (embedding_dim:384)
- 文档同步: `sync_to_lance`, `sync_from_sqlite`
- 索引管理: `ensure_lance_indexes`

**修改的文件**:
- `src/qmd-rust/src/config/mod.rs` - LanceDbConfig
- `src/qmd-rust/src/store/lance_backend/lance_backend.rs` - sync_from_sqlite
- `src/qmd-rust/src/store/mod.rs` - sync_to_lance, ensure_lance_indexes

**配置文件示例**:
```yaml
bm25:
  backend: lancedb
vector:
  backend: lancedb
  lancedb:
    embedding_dim: 384
```

**待完成**:
- 运行时验证（需要实际 embedder）
- 集成测试

### 文档对齐 ✅ (2026-02-17)

**完成内容**:
- 分析了 src/ 目录下 README.md 和 CLAUDE.md 与实际代码的不一致
- 更新 src/README.md: 添加 TypeScript 版本、完整 CLI 命令列表、Rust 所有模块
- 更新 src/CLAUDE.md: 添加 TypeScript 命名约定、完整 CLI 命令、ANEL 协议支持
- 更新 src/shared/README.md: 修正语言数量、更新脚本列表
- 创建验证脚本: verify_qmd_compat.sh, compare_qmd_impls.sh

**发现的不一致**:
- README 提到"三种语言"，实际是四种（Rust/Go/Python/TypeScript）
- CLI 命令不完整，缺少 context, get, multi-get, ls, cleanup 等
- Rust 模块缺少 anel, formatter, plugin, server
- shared/ 目录结构与文档不符

**修改的文件**:
- `src/README.md`
- `src/CLAUDE.md`
- `src/shared/README.md`
- `src/shared/scripts/verify_qmd_compat.sh` (新增)
- `src/shared/scripts/compare_qmd_impls.sh` (新增)

### 架构文档更新 ✅

**新增内容**:
- 添加 ANEL 架构蓝图图片（`imgs/ANEL-en.jpeg`, `imgs/ANEL-zh.jpeg`）
- 在 README.md 和 README_CN.md 顶部添加架构蓝图和架构宣言链接
- 新增英文版架构宣言 `docs/ANEL/ANEL-Architecture-Manifesto-v1.0.md`

### ANEL Copilot 方案设计 ✅ (2026-02-18)

**完成内容**:
- 设计了无侵入式 ANEL 协议助手方案
- 命名为 **anel-copilot**
- 技术栈: TypeScript/Node.js
- 核心功能: MCP Server、CLI、代码分析、自动修复、运行时验证

**设计文档**:
- `docs/plans/2026-02-18-anel-copilot.md`

**方案要点**:
- 四种形态: MCP Server、Skill、CLI、工具库
- 四个 MCP 工具: anel_analyze, anel_fix, anel_verify, anel_explain
- 自动修复: AI 分析后可直接修改代码，无需用户手动复制
- 支持语言: Go (cobra/urfave), Rust (clap), Python (click), TypeScript

**实施计划** (9 个任务):
1. 项目搭建
2. 测试基础设施
3. 代码检测实现
4. 规则分析器实现
5. 代码生成器实现
6. 自动修复集成
7. 运行时验证器
8. CLI 封装
9. 文档和发布

### Phase 9: ANEL Copilot 实施 ✅ (2026-02-18)

**完成内容**:
- 创建 `src/anel-copilot/` TypeScript/Node.js 项目
- MCP Server: 使用最新 `McpServer` + `registerTool()` + Zod schema API
- 4 个 MCP 工具: `anel_analyze`, `anel_fix`, `anel_verify`, `anel_explain`
- 核心模块: detector（语言/框架检测）、rules（5条ANEL规则+加权评分）、analyzer（合规分析）、generator（四语言自动修复）、verifier（运行时验证）
- CLI: `anel-copilot analyze|fix|verify` 子命令
- 测试: 4 个测试文件，43 tests 全部通过（vitest）

**技术决策**:
- MCP SDK API 已从旧版 `Server` + `setRequestHandler` 迁移到 `McpServer` + `registerTool()` + Zod schema
- 依赖: `@modelcontextprotocol/sdk ^1.12.0`, `zod ^3.25.0`
- 规则检测使用字符串匹配（非 AST），后续可升级

**创建的文件**:
- `src/anel-copilot/package.json`
- `src/anel-copilot/tsconfig.json`
- `src/anel-copilot/src/index.ts` (MCP Server)
- `src/anel-copilot/src/cli.ts` (CLI)
- `src/anel-copilot/src/core/types.ts`
- `src/anel-copilot/src/core/detector.ts`
- `src/anel-copilot/src/core/rules.ts`
- `src/anel-copilot/src/core/analyzer.ts`
- `src/anel-copilot/src/core/generator.ts`
- `src/anel-copilot/src/core/verifier.ts`
- `src/anel-copilot/tests/` (4 test files + 2 fixture files)

### Phase 9 增强: anel-copilot Phase A/B/C ✅ (2026-02-18)

**Phase A: 补全缺失规则 + 功能完善**
- 新增 `output-format` (medium) 和 `env-vars` (medium) 规则，现共 7 条规则
- MCP `anel_fix` 添加 `dryRun` 参数（预览修改不写入文件）
- 新增 `batch.ts` 实现 `analyzeDirectory()`，支持递归扫描和扩展名过滤
- 注册 `anel_analyze_dir` MCP 工具，CLI 添加 `analyze-dir` 命令
- 更新 fixtures 添加 `AGENT_IDENTITY_TOKEN` 模式
- 更新 generator 四种语言模板均添加 `AGENT_IDENTITY_TOKEN` env var 读取

**Phase B: LLM 智能代码修改模式**
- 新增 `llm.ts`: `LlmProvider` 接口 + `TemplateLlmProvider` + `AnthropicLlmProvider`
- `AnthropicLlmProvider` 使用 fetch 调用 Anthropic Messages API，构造 ANEL 协议 system prompt
- `createLlmProvider()` 工厂函数：llm 模式需要 `ANTHROPIC_API_KEY`，无 key 自动 fallback
- MCP `anel_fix` 添加 `mode: "template" | "llm"` 参数
- CLI `fix` 命令添加 `--llm` flag

**Phase C: AST 解析升级**
- tree-sitter 系列为 `optionalDependencies`（编译失败不影响功能）
- 新增 `ast-detector.ts`: 动态 import 加载 tree-sitter，实现 Go/Rust/Python/TypeScript AST 检测
- `analyzer.ts` 新增 `analyzeCodeWithAst()`: 优先 AST 检测，fallback 到字符串匹配

**新增文件**:
- `src/anel-copilot/src/core/batch.ts`
- `src/anel-copilot/src/core/llm.ts`
- `src/anel-copilot/src/core/ast-detector.ts`
- `src/anel-copilot/tests/batch.test.ts`
- `src/anel-copilot/tests/llm.test.ts`
- `src/anel-copilot/tests/ast-detector.test.ts`

**测试**: 7 个测试文件，82 tests 全部通过

## 待完成

- Phase 7: Python 补充测试
- Phase 8: Go 补充
- anel-copilot 后续增强:
  - Claude Code Skill 形态
  - npm 发布
  - tree-sitter 在 CI 环境中的集成测试
  - LLM 模式端到端测试（需要 API key）

## 构建命令

```bash
# Rust
cd src/qmd-rust && cargo test

# Python
cd src/qmd-python && python -m pytest tests/ -v

# Go
cd src/qmd-go && go test ./internal/... -v

# TypeScript (原版 QMD)
cd src/qmd-typescript && bun test

# ANEL Copilot
cd src/anel-copilot && npm test        # 运行测试
cd src/anel-copilot && npm run build   # 构建
cd src/anel-copilot && node dist/cli.js analyze <file>  # CLI 分析
```
