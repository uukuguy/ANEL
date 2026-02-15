# ANEL 工程落地路线图 (Project Roadmap)

### 总体目标

构建 **Hyper-Shell v0.5 (Alpha)** 原型，实现核心运行时、Wasm 插件加载、双模态 I/O 以及 ANID 协议的完整支持，并通过一个实际业务场景验证闭环。

---

### Phase 0: 协议立法与标准定义 (Protocol & Standardization)

**周期估计**: 2 周
**核心维度**: 协议标准化 (Standardization)

在这个阶段，不写一行 Rust 代码，只写 Markdown 和 JSON。目标是确立“法律条款”，避免后续返工。

- **里程碑 0.1: 发布 ANID v1.0 RFC**
    
    [关于 RFC 规范的理解](https://www.notion.so/RFC-306fc249f8b080508fdeccc29c5590bc?pvs=21) 
    
    - **动作**: 细化《文档二：ANID 接口定义标准》，确定所有保留字段（`error_code`, `severity`, `recovery_hints`）的最终 JSON Schema。
    - **关键决策**: 确定 NDJSON 的流式分隔符标准；定义 `-dry-run` 的标准返回结构。
    - **交付物**:
        - `ANID_Spec_v1.md`: 完整的协议文档 。
        - `schemas/error.json`: 错误对象的 JSON Schema 校验文件。
        - `schemas/dry_run.json`: 预演报告的 Schema。
- **里程碑 0.2: 设计“原子能力”样例 (Mockup)**
    
    [**Phase 0.2 —— 定义“原子能力”**。](https://www.notion.so/Phase-0-2-306fc249f8b0809bad2ef9c29ef5f0b5?pvs=21)
    
    - **动作**: 选取**“服务健康检查” (svc-check)** 作为高频原子能力。手写其预期的 Input/Output 样例。
    - **交付物**:
        - `svc-check.yaml`: 该工具的 ANID 定义文件（自省元数据）。
        - `mock_stdout.ndjson`: 模拟的业务输出流样例。
        - `mock_stderr.json`: 模拟的结构化错误样例。

<aside>
💡

现在，你的工作流应该是：

1. **User**: 制定 RFC，创建 JSON Fixtures (Golden Data)。
2. **Claude Code**: "/add RFC", "/add Fixtures" -> "实现代码以通过测试"。
3. **User**: 运行 `cargo test`。
    - 🔴 挂了 -> 把错误日志甩给 Claude -> "Fix it"。
    - 🟢 过了 -> 进入下一个 Milestone。

**现在，请打开你的 Claude Code，执行步骤 1 的 Prompt 吧！** 这是你作为架构师的高光时刻。

</aside>

---

### Phase 1: 内核骨架与原生实现 (Kernel & Native Impl)

[⭐️ AI Coding 完成 Phase 1](https://www.notion.so/AI-Coding-Phase-1-306fc249f8b080d09cf6fd4352cb7b8c?pvs=21)

**周期估计**: 3-4 周
**核心维度**: 核心运行时 (Core Runtime)

构建 Rust 主程序 `hyper-shell`，暂不引入 Wasm，专注于 I/O 管道和协议解析。

- **里程碑 1.1: 搭建 Rust Kernel 骨架**
    - **动作**: 初始化 Rust 项目，引入 `tokio` (异步运行时) 和 `clap` (CLI 解析) 。
    - **核心功能**: 实现 `main` 函数，能够解析 `-output-format` 参数。
    - **交付物**:
        - `anel-kernel` 二进制文件：能打印 "Hello ANEL" 并响应 `-version`。
- **里程碑 1.2: 实现双模态 I/O (Dual-Mode I/O)**
    - **动作**: 实现 `StreamManager` 模块。
    - **逻辑**: 检测 `std::io::stdout().is_terminal()`。如果是 TTY，打印彩色文本；如果是管道，强制输出 NDJSON 。
    - **交付物**:
        - I/O 模块单元测试：验证在管道模式下自动切换为 JSON 输出。
- **里程碑 1.3: 重构原子能力 "svc-check" (Native版)**
    - **动作**: 不调用外部 curl，而是用 Rust 的 `reqwest` 库硬编码实现一个内置命令 `svc-check`。
    - **功能**: 接受 `-url`，返回 HTTP 状态。
    - **验证**:
        - 执行 `svc-check --url google.com` -> 输出漂亮的表格。
        - 执行 `svc-check --url google.com | cat` -> 输出 NDJSON 。
        - 执行 `svc-check --url invalid-url` -> Stderr 输出包含 `recovery_hints` 的 JSON 。

---

### Phase 2: Wasm 引擎与插件系统 (Wasm Engine & Plugins)

[⭐️ AI Coding 完成 Phase 2](https://www.notion.so/AI-Coding-Phase-2-306fc249f8b08095afd2f508a16cd8b7?pvs=21)

**周期估计**: 4 周
**核心维度**: 基础设施 (Infrastructure) & 开发者体验 (DX)

引入 `wasmtime`，让运行时支持动态加载外部能力。

- **里程碑 2.1: 集成 Wasmtime 运行时**
    - **动作**: 在 Kernel 中集成 `wasmtime` crate。配置 WASI 上下文（文件系统沙箱、网络白名单）。
    - **交付物**:
        - `PluginHost` 模块：能加载磁盘上的 `.wasm` 文件并实例化。
- **里程碑 2.2: 定义宿主绑定 (Host Bindings)**
    - **动作**: 使用 `wit-bindgen` 定义 `world.wit` 文件，描述 Kernel 暴露给插件的能力（如：打印日志、读配置）。
    - **交付物**:
        - `anel-pdk` (Plugin Development Kit)：提供给开发者的 Rust SDK crate。
- **里程碑 2.3: 迁移 "svc-check" 为 Wasm 插件**
    - **动作**: 将 Phase 1 中的 `svc-check` 逻辑剥离，使用 `anel-pdk` 重写为独立的 Rust 项目，编译为 `svc_check.wasm`。
    - **验证**: Kernel 加载 `svc_check.wasm`，执行效果与 Native 版完全一致。验证冷启动时间是否 < 10ms 。

---

### Phase 3: 安全闭环与集成验证 (Security & Integration)

[⭐️ AI Coding 完成 Phase 3](https://www.notion.so/AI-Coding-Phase-3-306fc249f8b080f0a5d0e33b3bd9ca33?pvs=21)

**周期估计**: 3 周
**核心维度**: 安全 (Security) & 试点验证 (Pilot)

这一阶段将原型提升为“企业可用”状态。

- **里程碑 3.1: 实现流式审计 (Stream Tap)**
    - **动作**: 在 Kernel 层实现 `T-Tap` 分流器。使用 `tokio::io::copy_bidirectional` 捕获所有 Stdout/Stderr 。
    - **交付物**:
        - 审计日志文件 `audit.log`：记录了每次执行的完整输入输出和 `TRACE_ID`。
- **里程碑 3.2: 身份传播与 Dry-Run 强制**
    - **动作**:
        - 实现环境变量注入：自动将 `AGENT_IDENTITY_TOKEN` 注入 Wasm 实例 。
        - 实现 Dry-Run 拦截器：如果用户输入 `-dry-run`，Kernel 拦截所有 Write 操作（需 WASI 支持或插件配合）。
    - **交付物**:
        - 集成测试：验证 Token 是否成功传递到了模拟的后端 API。
- **里程碑 3.3: 最终验收 - "故障排查" 场景**
    - **动作**: 串联整个链路。
    - **场景**: Agent 接收指令 -> 生成脚本 -> 调用 Wasm 插件 -> 捕获 JSON 错误 -> 根据 Hint 自动修正参数 -> 执行成功。
    - **交付物**:
        - 一个端到端的 Demo 视频或演示脚本。

---

### 📅 资源与交付物清单汇总

为了方便你操作，我整理了你需要维护的**核心代码仓库结构**：

```
anel-project/
├── docs/                       # [Phase 0]
│   ├── ANID_SPEC.md            # 协议规范
│   └── rfcs/001-error-handling.md
├── crates/
│   ├── anel-kernel/            # [Phase 1] Rust 主程序 (Hyper-Shell)
│   ├── anel-core/              # 核心 Trait 定义 (AgentCommand)
│   └── anel-pdk/               # [Phase 2] 插件开发 SDK
├── plugins/                    # [Phase 2]
│   ├── svc-check/              # 原子能力源码 (Rust -> Wasm)
│   └── log-filter/             # 另一个测试插件
└── examples/                   # [Phase 3]
    └── demo-scenario/          # 演示脚本
```