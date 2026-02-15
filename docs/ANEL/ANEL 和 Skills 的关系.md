# ANEL 和 Skills 的关系

你这里 对接Agent框架, 我们是打算用 Agent Skills 来构建业务技能库，两者之间在落地实现上是什么样具体的关系？如何实现？还有，“当企业里有 1000 个 Agent 同时跑时，单机版的 CLI 就不够用了。”不太理解，ANEL是新的企业级AI工具，是被智能体通过Skills编排调用的，不太理解你描述的架构，或者理解上有误？

这是一个非常敏锐且关键的问题。你触及到了架构设计的**“连接层（Integration Layer）”**和**“部署层（Deployment Layer）”**。

让我们把这两个问题拆解开，用最直观的**“代码级”**和**“架构级”**视角来澄清。

---

### **第一部分：Agent Skills 与 ANEL (Hyper-Shell) 的落地关系**

**你的理解是正确的：** ANEL 是被 Skills 调用的工具。
但在落地代码中，它们的关系不仅仅是“调用”，而是**“封装（Wrap）”与“驱动（Drive）”**的关系。

我们可以把这个关系比作：**Skills 是“Python 驱动程序”，而 ANEL 是“底层硬件接口”**。

### **1. 概念分界线**

- **Agent Skills (业务技能库)**：
    - **载体**：通常是 Python 类（如 LangChain `BaseTool`）或 YAML 流程定义。
    - **内容**：包含业务逻辑、Prompt 模板、参数校验、重试策略。
    - **职责**：它告诉 Agent **“什么情况下用这个能力”** 以及 **“如何准备参数”**。
- **ANEL / Hyper-Shell (执行层)**：
    - **载体**：Rust 编译出的二进制文件 (`hyper-shell`) 或 Wasm 插件。
    - **内容**：原子操作的实现（如发 HTTP 请求、查库）。
    - **职责**：它负责 **“执行动作”** 并 **“返回结构化结果”**。

### **2. 代码级实现：如何“胶合”在一起？**

在企业落地时，你需要编写一个中间层（SDK），把 ANEL 的原子能力映射为 Agent 框架能理解的 Skill。

**场景**：你有一个 ANEL 原子能力 `svc-check` (我们之前定义的)。
**目标**：在 LangChain 中使用它。

**落地代码示意（Python SDK 层）：**

**Python**

`# 这就是属于 "Agent Skills" 层面的代码
from langchain.tools import BaseTool
import subprocess
import json

class ServiceHealthCheckSkill(BaseTool):
    name = "service_health_check"
    description = "查询服务健康状态。输入参数为服务URL。"

    def _run(self, target_url: str) -> dict:
        # === 核心交互点 ===
        # Skill (Python) 驱动 ANEL (Rust CLI)
        # 1. 构造命令
        cmd = [
            "./hyper-shell",      # ANEL 内核
            "svc-check",          # 原子能力
            "--target", target_url,
            "--output-format", "ndjson" # 强制要求结构化输出
        ]
        
        # 2. 执行 (Process Execution)
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        # 3. 处理 ANEL 的反馈 (ANID 协议生效处)
        if result.returncode != 0:
            # 解析 Stderr 中的结构化错误 (JSON)
            error_obj = json.loads(result.stderr)
            return f"执行失败: {error_obj['message']}. 建议: {error_obj['recovery_hints']}"
            
        # 4. 返回业务数据
        # 解析 Stdout 中的 NDJSON 流
        return [json.loads(line) for line in result.stdout.splitlines()]`

**结论**：

- **Agent Skills** 是 ANEL 的**上层封装（Wrapper）**。
- Agent 只看到 Python 的 `ServiceHealthCheckSkill`。
- 当 Agent 决定使用该 Skill 时，Skill 内部启动 `hyper-shell` 进程完成脏活累活。

---

### **第二部分：关于“1000 个 Agent 并发与 CLI 不够用”的解释**

这里的“CLI 不够用”**不是指命令行界面（Interface）不好用**，而是指**单机进程模型（Process Model）在分布式环境下的局限性**。

如果不引入企业级架构，单纯把 `hyper-shell` 当作一个本地工具，当并发量上来时会遇到物理瓶颈。

### **误区：所有 Agent 都在一台服务器上跑 CLI**

如果你有 1000 个 Agent 实例（比如 1000 个用户同时在使用客服机器人），且它们都运行在同一台物理机或同一个容器里：

1. **进程爆炸**：每个 Agent 调用一次 Skill，就要 fork 一个 `hyper-shell` 子进程。1000 个并发 = 瞬间 fork 1000 个进程。操作系统会因为 PID 耗尽或上下文切换过高而崩溃。
2. **网络连接数耗尽**：1000 个 `hyper-shell` 同时对外发起 TCP 连接，端口瞬间耗尽。

### **架构真相：分布式部署 (Sidecar 模式)**

在企业级落地（如 Kubernetes 环境）中，ANEL 不再是简单地“装在笔记本里的工具”，而是变成了一种**基础设施服务**。

我们通常采用 **Sidecar（边车）模式** 或 **DaemonSet（守护进程）模式**。

**场景**：

- **Pod A (Agent)**: 运行 Python 代码，是大脑。
- **Pod B (ANEL Sidecar)**: 运行 `hyper-shell server` 模式（这是一个长驻进程，不是一次性 CLI）。

**交互流程变成了这样：**

1. **Agent (Pod A)** 发送一个极其轻量的指令（通过 gRPC 或 HTTP）：
    - `POST /exec { "cmd": "svc-check", "args": [...] }`
2. **ANEL Sidecar (Pod B)** 接收请求：
    - 它**不需要** fork 新进程，而是在内部通过 Rust 的异步线程（Tokio Task）执行 Wasm 插件。
    - **优势**：1000 个请求只需要 1 个进程和极少的内存，因为 Wasm 是轻量级线程。
3. **结果返回**：Sidecar 把 NDJSON 流回传给 Agent。

### **所谓的“单机版 CLI 不够用”是指：**

当规模扩大时，你不能再像在自己电脑上那样，简单粗暴地用 `subprocess.run("hyper-shell ...")` 了。你需要将 ANEL 升级为**“服务化运行”**。

**这正是 Phase 4+ 要做的事情：**

- **CLI 模式**：适合开发调试、单体应用（Agent < 50 个）。
- **Server/Sidecar 模式**：适合生产环境（Agent > 1000 个）。
    - Agent 不再直接调 CLI，而是调用 SDK。
    - SDK 底层自动判断：是起本地进程（CLI），还是发 RPC 请求给 Sidecar（Server）。

### **总结**

1. **Skills 与 ANEL 的关系**：
    - Skills 是 **“皮”**（Python/LangChain 定义），ANEL 是 **“骨”**（Rust/Wasm 实现）。
    - Skills 负责**决策**（什么时候用），ANEL 负责**执行**（怎么干）。
2. **1000 Agent 的挑战**：
    - 是**并发模型**的挑战。
    - ANEL 架构的终局不仅仅是一个命令行工具，而是一个**可以被 API 调用的高性能执行微服务**。但它保留了 CLI 的协议标准（ANID），确保无论怎么部署，交互契约是不变的。