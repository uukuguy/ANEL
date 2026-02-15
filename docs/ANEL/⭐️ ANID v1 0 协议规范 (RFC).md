# ⭐️ ANID v1.0 协议规范 (RFC)

**文件路径**: `docs/spec/ANID_v1.0_RFC.md`

**版本**: 1.0.0

**状态**: APPROVED

### 1. 协议元语 (Protocol Primitives)

本规范定义了 **Hyper-Shell** 运行时与 **原子能力 (Tools)** 之间的通讯标准。

### 1.1 核心原则

- **无状态 (Stateless)**: 每次调用都是独立的进程或 Wasm 实例。
- **双流分离 (Dual-Stream Separation)**:
    - `STDOUT`: 仅传输**业务数据** (Payload)。
    - `STDERR`: 仅传输**控制信号** (Control Plane)，包含日志、错误、进度、心跳。
- **零歧义 (Zero-Ambiguity)**: 禁止输出“人类可读”的模糊文本，必须输出结构化数据。

---

### 2. 标准输入/输出定义 (I/O Specification)

### 2.1 标准输出 (STDOUT)

- **格式**: 必须支持 **NDJSON** (Newline Delimited JSON)。
- **MIME Type**: `application/x-ndjson`
- **约束**:
    1. 每一行必须是一个合法的 JSON 对象。
    2. 行与行之间不得有逗号连接。
    3. 禁止包含任何 Markdown 格式（如 ````json`）。
- **示例**:JSON
    
    `{"id": 1, "status": "active"}
    {"id": 2, "status": "pending"}`
    

### 2.2 标准错误 (STDERR)

- **格式**: 单个 JSON 对象 (RFC 7807 扩展)。
- **约束**: 当且仅当 `exit_code != 0` 或需要发送带外信息（如 Log）时写入。
- **Schema**:TypeScript
    
    `type AnidError = {
      error_code: string;       // 机器码，如 "E_INVALID_ARG"
      message: string;          // 人类可读描述
      severity: "fatal" | "retryable" | "warning";
      context?: Record<string, any>; // 上下文数据
      recovery_hints: string[];      // [关键] 指导 Agent 自愈的建议
    }`
    

---

### 3. 控制面定义 (Control Plane)

### 3.1 必需参数 (Mandatory Flags)

所有 Tool 必须实现以下参数：

| **参数** | **描述** | **预期行为** |
| --- | --- | --- |
| `--emit-spec` | 自省 | 向 STDOUT 输出工具的元数据（Definition Schema）。 |
| `--dry-run` | 预演 | 执行检查但不产生副作用。向 STDOUT 输出 Impact Report。 |
| `--output-format` | 格式 | 接受 `json`, `ndjson`, `text`。默认应为 `ndjson`。 |

### 3.2 环境变量 (Environment Context)

Runtime 会向 Tool 注入以下环境变量，Tool **必须**透传：

| **变量名** | **描述** | **用途** |
| --- | --- | --- |
| `AGENT_TRACE_ID` | 全链路 ID | 用于分布式追踪与审计。 |
| `AGENT_IDENTITY_TOKEN` | 身份凭证 | 用于下游 API 鉴权 (Bearer Token)。 |