# ANEL Copilot

ANEL Protocol compliance checker and auto-fixer for CLI tools. Works as an MCP Server for AI coding assistants (Claude Code, Cline, etc.) or as a standalone CLI.

## Install

```bash
npm install
npm run build
```

## CLI Usage

```bash
# Analyze a file for ANEL compliance
anel-copilot analyze <file>

# Batch analyze a directory
anel-copilot analyze-dir <dir> [--no-recursive]

# Auto-fix using template mode (default)
anel-copilot fix <file> [--dry-run]

# Auto-fix using LLM mode (requires ANTHROPIC_API_KEY)
anel-copilot fix <file> --llm [--dry-run]

# Verify runtime compliance of a compiled binary
anel-copilot verify <binary> <command>
```

## MCP Server

Register as an MCP server in your AI coding tool:

```json
{
  "mcpServers": {
    "anel-copilot": {
      "command": "node",
      "args": ["path/to/anel-copilot/dist/index.js"]
    }
  }
}
```

### Tools

| Tool | Description |
|------|-------------|
| `anel_analyze` | Analyze code for ANEL protocol compliance |
| `anel_analyze_dir` | Batch analyze all code files in a directory |
| `anel_fix` | Auto-fix code (template or LLM mode, with dryRun option) |
| `anel_verify` | Verify ANEL implementation at runtime |
| `anel_explain` | Explain ANEL protocol requirements |

## Rules

7 ANEL compliance rules are checked:

| Rule | Severity | Description |
|------|----------|-------------|
| `emit-spec` | high | CLI must support `--emit-spec` flag |
| `dry-run` | high | CLI must support `--dry-run` flag |
| `error-format` | high | Errors must follow RFC 7807 + recovery_hints |
| `ndjson-output` | high | STDOUT must output NDJSON format |
| `output-format` | medium | CLI must support `--output-format` flag |
| `env-vars` | medium | Support `AGENT_IDENTITY_TOKEN` env var |
| `trace-id` | low | Support `AGENT_TRACE_ID` env var |

## Supported Languages

| Language | Frameworks |
|----------|-----------|
| Go | cobra, urfave-cli |
| Rust | clap |
| Python | click, argparse, typer |
| TypeScript | commander, oclif, yargs |

## LLM Mode

Set `ANTHROPIC_API_KEY` to enable LLM-powered code fixes:

```bash
export ANTHROPIC_API_KEY=sk-ant-...
anel-copilot fix myfile.go --llm
```

Falls back to template mode automatically if no API key is set.

## AST Detection (Optional)

Install tree-sitter for more precise code analysis:

```bash
npm install tree-sitter tree-sitter-go tree-sitter-rust tree-sitter-python tree-sitter-typescript
```

When available, AST-based detection is used automatically. Falls back to string matching otherwise.

## Testing

```bash
npm test          # 82 tests
npm run build     # Type check + compile
```
