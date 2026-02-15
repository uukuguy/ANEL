# Next Session Guide - ANEL Project

**Last Updated**: 2026-02-16
**Current Status**: 所有 16 个 Phase 已完成，P0/P1 收尾工作已完成
**Branch**: ANEL

## 本次完成的工作 (2026-02-16)

### P0: 提交未跟踪代码 ✅
- 提交 README.md (英文) + README_CN.md (中文) 项目主文档
- 提交 src/qmd-typescript/ 完整 TypeScript 实现（27 文件，14196 行）
- Commit: b744698

### P1: 多版本测试补全 ✅
- **Go**: 新增 142 个测试
  - `internal/anel/anel_test.go` — ErrorCode、AnelError、TraceContext、RecoveryHint、NDJSONRecord、AnelResult、from_error
  - `internal/anel/spec_test.go` — 12 个命令 spec 验证（JSON 有效性、schema 结构、error codes）
  - `internal/config/config_test.go` — Config 默认值、YAML 加载、后端常量
- **Python**: 新增 174 个测试
  - `tests/test_anel.py` — ANEL 核心类型（Pydantic 模型验证）
  - `tests/test_spec.py` — 12 个命令 spec 验证
  - `tests/test_config.py` — Config dataclass 验证
- **修复**: pyproject.toml 重复 `[project.optional-dependencies]` 段

### 测试总览

| 版本 | 测试数 | 框架 |
|------|--------|------|
| TypeScript | 700+ | Bun test |
| Python | 174 | pytest |
| Go | 142 | go test |
| Rust | 68 (inline) | cargo test |

## 下一步建议优先级

### P2: LanceDB 完整实现（中优先级）
- 当前是 stub（返回空结果）
- 核心问题：Arrow 版本冲突（LanceDB v0.23 依赖 arrow-array v56，qmd 使用 v57）
- 可选方案：
  - A: 将 LanceDB 作为外部服务运行（推荐）
  - B: 使用 PyO3 调用 Python 版 LanceDB
  - C: 在独立 crate 中使用 arrow-array v56

### P3: 统一三版本架构和配置（中优先级）
- 配置格式需要统一
- Go/Python MCP HTTP Server 尚未实现（仅 Rust 有）

### P4: 完善 README 和项目文档（低优先级）
- 根目录 README 已提交，内容完整
- 可选：添加 CONTRIBUTING.md、LICENSE 文件

### P5: 端到端 Demo 场景（低优先级）
- Phase 3 路线图要求的"故障排查"端到端演示脚本
- 串联：Agent 接收指令 → 生成脚本 → 调用工具 → 捕获错误 → 自动修正 → 执行成功

## 构建命令

```bash
# Rust
cd src/qmd-rust && cargo build --features sqlite-vec
cargo test --features sqlite-vec  # 169 tests

# Go
cd src/qmd-go && go build -o qmd ./cmd/qmd
go test ./internal/anel/ ./internal/config/ -v  # 142 tests

# Python
cd src/qmd-python && pip install -e .
python -m pytest tests/ -v  # 174 tests

# TypeScript
cd src/qmd-typescript && bun install
bun test  # 700+ tests
```

## 关键文件

### 新增文件 (本次)
- `src/qmd-go/internal/anel/anel_test.go`
- `src/qmd-go/internal/anel/spec_test.go`
- `src/qmd-go/internal/config/config_test.go`
- `src/qmd-python/tests/test_anel.py`
- `src/qmd-python/tests/test_spec.py`
- `src/qmd-python/tests/test_config.py`

### 修改文件 (本次)
- `src/qmd-python/pyproject.toml` — 修复重复 optional-dependencies
