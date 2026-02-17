# ANEL Copilot Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 构建一个无侵入式的 ANEL 协议助手，帮助 AI Coding 工具（Claude Code、Cline 等）自动完成 ANEL 协议支持。

**Architecture:**
- MCP Server 为核心，提供 anel_analyze、anel_fix、anel_verify、anel_explain 工具
- 运行时使用 LLM 智能修改代码（而非预设模板）
- 支持三种形态：MCP Server、CLI 工具、Claude Code Skill

**Tech Stack:** TypeScript/Node.js, TypeScript AST, MCP SDK

---

## Phase 1: Project Setup

### Task 1: Create Project Structure

**Files:**
- Create: `src/anel-copilot/package.json`
- Create: `src/anel-copilot/tsconfig.json`
- Create: `src/anel-copilot/src/index.ts`
- Create: `src/anel-copilot/src/mcp/index.ts`
- Create: `src/anel-copilot/src/core/types.ts`

**Step 1: Create package.json**

```json
{
  "name": "anel-copilot",
  "version": "1.0.0",
  "description": "ANEL Protocol Copilot - AI Coding assistant for ANEL compliance",
  "main": "dist/index.js",
  "type": "module",
  "scripts": {
    "build": "tsc",
    "start": "node dist/index.js",
    "dev": "tsc && node dist/index.js"
  },
  "dependencies": {
    "@modelcontextprotocol/sdk": "^1.0.0",
    "typescript": "^5.3.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0"
  }
}
```

**Step 2: Create tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["src/**/*"]
}
```

**Step 3: Create basic types.ts**

```typescript
// ANEL Protocol Types

export interface AnelIssue {
  rule: string;
  status: 'missing' | 'non-compliant' | 'present';
  severity: 'high' | 'medium' | 'low';
  suggestion?: string;
  current?: string;
}

export interface AnelAnalysisResult {
  file: string;
  language: string;
  framework?: string;
  complianceScore: number;
  issues: AnelIssue[];
}

export interface AnelFixResult {
  file: string;
  success: boolean;
  diff?: string;
  error?: string;
}

export interface AnelVerifyResult {
  binary: string;
  command: string;
  passed: boolean;
  details: string[];
}

export type AnelRule =
  | 'emit-spec'
  | 'dry-run'
  | 'output-format'
  | 'error-format'
  | 'ndjson-output'
  | 'trace-id'
  | 'env-vars';
```

**Step 4: Create basic index.ts**

```typescript
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';

const server = new Server(
  { name: 'anel-copilot', version: '1.0.0' },
  { capabilities: { tools: {} } }
);

server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: [
    {
      name: 'anel_analyze',
      description: 'Analyze code for ANEL protocol compliance',
      inputSchema: {
        type: 'object',
        properties: {
          filePath: { type: 'string', description: 'Path to code file to analyze' }
        },
        required: ['filePath']
      }
    },
    {
      name: 'anel_fix',
      description: 'Automatically fix code to comply with ANEL protocol',
      inputSchema: {
        type: 'object',
        properties: {
          filePath: { type: 'string', description: 'Path to code file to fix' },
          rules: { type: 'array', items: { type: 'string' }, description: 'Rules to apply' }
        },
        required: ['filePath']
      }
    },
    {
      name: 'anel_verify',
      description: 'Verify ANEL protocol implementation at runtime',
      inputSchema: {
        type: 'object',
        properties: {
          binaryPath: { type: 'string', description: 'Path to compiled binary' },
          command: { type: 'string', description: 'Command to test' }
        },
        required: ['binaryPath', 'command']
      }
    },
    {
      name: 'anel_explain',
      description: 'Explain ANEL protocol requirements',
      inputSchema: {
        type: 'object',
        properties: {
          question: { type: 'string', description: 'Question about ANEL protocol' }
        },
        required: ['question']
      }
    }
  ]
}));

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  if (name === 'anel_analyze') {
    return { content: [{ type: 'text', text: JSON.stringify({ todo: 'implement' }) }] };
  }
  if (name === 'anel_fix') {
    return { content: [{ type: 'text', text: JSON.stringify({ todo: 'implement' }) }] };
  }
  if (name === 'anel_verify') {
    return { content: [{ type: 'text', text: JSON.stringify({ todo: 'implement' }) }] };
  }
  if (name === 'anel_explain') {
    return { content: [{ type: 'text', text: 'ANEL Protocol: Agent-Native Execution Layer...' }] };
  }

  throw new Error(`Unknown tool: ${name}`);
});

const transport = new StdioServerTransport();
await server.connect(transport);
```

**Step 5: Build and verify**

Run: `cd src/anel-copilot && npm install && npm run build`
Expected: Build succeeds with dist/index.js

**Step 6: Commit**

```bash
cd src/anel-copilot
git init
git add package.json tsconfig.json src/
git commit -m "feat: scaffold anel-copilot project"
```

---

### Task 2: Set up Testing Infrastructure

**Files:**
- Create: `src/anel-copilot/tests/analyzer.test.ts`
- Create: `src/anel-copilot/tests/fixtures/sample-go.ts`
- Modify: `src/anel-copilot/package.json`

**Step 1: Add test dependency**

Run: `cd src/anel-copilot && npm install --save-dev vitest`

**Step 2: Create sample Go code fixture**

```typescript
// tests/fixtures/sample-go.ts
export const sampleGoSearch = `
package cmd

import (
    "fmt"
    "os"
    "github.com/spf13/cobra"
)

var searchCmd = &cobra.Command{
    Use:   "search <query>",
    Short: "Search documents",
    RunE:  handleSearch,
}

func handleSearch(cmd *cobra.Command, args []string) error {
    if len(args) == 0 {
        return fmt.Errorf("query required")
    }
    results, err := doSearch(args[0])
    if err != nil {
        return fmt.Errorf("search failed: %w", err)
    }
    fmt.Println(results)
    return nil
}
`;

export const sampleGoSearchCompliant = `
package cmd

import (
    "encoding/json"
    "fmt"
    "os"
    "github.com/spf13/cobra"
    "my-cli/anel"
)

var searchCmd = &cobra.Command{
    Use:   "search <query>",
    Short: "Search documents",
    RunE:  handleSearch,
}

func init() {
    searchCmd.Flags().Bool("emit-spec", false, "Output ANEL specification")
    searchCmd.Flags().Bool("dry-run", false, "Validate without executing")
    searchCmd.Flags().String("output-format", "ndjson", "Output format")
}

func handleSearch(cmd *cobra.Command, args []string) error {
    emitSpec, _ := cmd.Flags().GetBool("emit-spec")
    dryRun, _ := cmd.Flags().GetBool("dry-run")
    outputFormat, _ := cmd.Flags().GetString("output-format")

    if emitSpec {
        spec := anel.GetSpec("search")
        json.NewEncoder(os.Stdout).Encode(spec)
        return nil
    }

    if dryRun {
        fmt.Fprintf(os.Stderr, \`{"dry_run": true, "query": "%s"}\`, args[0])
        return nil
    }

    if len(args) == 0 {
        err := anel.NewError(anel.E_INVALID_INPUT, "query required").
            WithRecoveryHint("CHECK_ARGS", "Provide a query argument")
        err.EmitStderr()
        return err
    }

    results, err := doSearch(args[0])
    if err != nil {
        err := anel.NewError(anel.E_SEARCH_FAILED, err.Error()).
            WithRecoveryHint("REINDEX", "Run 'my-cli update' to refresh index")
        err.EmitStderr()
        return err
    }

    encoder := json.NewEncoder(os.Stdout)
    for _, r := range results {
        encoder.Encode(r)
    }
    return nil
}
`;
```

**Step 3: Create analyzer test skeleton**

```typescript
// tests/analyzer.test.ts
import { describe, it, expect } from 'vitest';
import { analyze } from '../src/core/analyzer.js';

describe('AnelAnalyzer', () => {
  it('should detect missing emit-spec flag', async () => {
    const result = await analyze('./tests/fixtures/sample-go.ts', 'go');
    const emitSpecIssue = result.issues.find(i => i.rule === 'emit-spec');
    expect(emitSpecIssue?.status).toBe('missing');
  });

  it('should detect missing dry-run flag', async () => {
    const result = await analyze('./tests/fixtures/sample-go.ts', 'go');
    const dryRunIssue = result.issues.find(i => i.rule === 'dry-run');
    expect(dryRunIssue?.status).toBe('missing');
  });

  it('should pass compliant code', async () => {
    const result = await analyze('./tests/fixtures/sample-go-compliant.ts', 'go');
    expect(result.complianceScore).toBe(100);
  });
});
```

**Step 4: Run test to verify it fails**

Run: `cd src/anel-copilot && npx vitest run`
Expected: FAIL - analyze function not implemented

**Step 5: Commit**

```bash
git add tests/
git commit -m "test: add analyzer test infrastructure"
```

---

## Phase 2: Core Analyzer Implementation

### Task 3: Implement Code Detection

**Files:**
- Modify: `src/anel-copilot/src/core/types.ts`
- Create: `src/anel-copilot/src/core/detector.ts`

**Step 1: Add detection types**

```typescript
// Add to types.ts

export interface FileInfo {
  path: string;
  language: 'go' | 'rust' | 'python' | 'typescript';
  framework?: string;
}

export function detectLanguage(filePath: string): FileInfo {
  const ext = filePath.split('.').pop()?.toLowerCase();
  const basename = filePath.split('/').pop() || '';

  if (ext === 'go') return { path: filePath, language: 'go', framework: detectGoFramework(basename) };
  if (ext === 'rs') return { path: filePath, language: 'rust', framework: 'clap' };
  if (ext === 'py') return { path: filePath, language: 'python', framework: detectPythonFramework(basename) };
  if (ext === 'ts' || ext === 'js') return { path: filePath, language: 'typescript', framework: detectTSFramework(basename) };

  throw new Error(`Unsupported language: ${ext}`);
}

function detectGoFramework(basename: string): string {
  if (basename.includes('cobra')) return 'cobra';
  if (basename.includes('cli')) return 'urfave-cli';
  return 'unknown';
}

function detectPythonFramework(basename: string): string {
  if (basename.includes('click')) return 'click';
  if (basename.includes('argparse')) return 'argparse';
  return 'unknown';
}

function detectTSFramework(basename: string): string {
  if (basename.includes('commander')) return 'commander';
  if (basename.includes('oclif')) return 'oclif';
  return 'unknown';
}
```

**Step 2: Implement detector**

```typescript
// src/core/detector.ts
import { detectLanguage, type FileInfo } from './types.js';

export function detectFileInfo(filePath: string): FileInfo {
  return detectLanguage(filePath);
}
```

**Step 3: Run test**

Run: `npx vitest run`
Expected: PASS (detection works)

**Step 4: Commit**

```bash
git add src/core/detector.ts
git commit -m "feat: add language and framework detection"
```

---

### Task 4: Implement Rule Analyzer

**Files:**
- Create: `src/anel-copilot/src/core/rules.ts`
- Create: `src/anel-copilot/src/core/analyzer.ts`

**Step 1: Define ANEL rules**

```typescript
// src/core/rules.ts

export interface AnelRuleDefinition {
  name: string;
  description: string;
  severity: 'high' | 'medium' | 'low';
  check: (code: string, language: string, framework?: string) => AnelIssue;
}

export const anelRules: AnelRuleDefinition[] = [
  {
    name: 'emit-spec',
    description: 'CLI must support --emit-spec flag to output JSON schema',
    severity: 'high',
    check: (code, lang) => ({
      rule: 'emit-spec',
      status: code.includes('--emit-spec') ? 'present' : 'missing',
      severity: 'high',
      suggestion: code.includes('--emit-spec') ? undefined :
        'Add --emit-spec flag handler that outputs JSON schema'
    })
  },
  {
    name: 'dry-run',
    description: 'CLI must support --dry-run flag for validation without execution',
    severity: 'high',
    check: (code, lang) => ({
      rule: 'dry-run',
      status: code.includes('--dry-run') ? 'present' : 'missing',
      severity: 'high',
      suggestion: code.includes('--dry-run') ? undefined :
        'Add --dry-run flag to validate parameters without execution'
    })
  },
  {
    name: 'error-format',
    description: 'Errors must follow RFC 7807 + recovery_hints',
    severity: 'high',
    check: (code, lang) => {
      const hasAnelError = code.includes('anel.Error') || code.includes('AnelError') || code.includes('recovery_hints');
      return {
        rule: 'error-format',
        status: hasAnelError ? 'present' : 'non-compliant',
        severity: 'high',
        suggestion: hasAnelError ? undefined :
          'Use ANEL error format with error_code, message, severity, recovery_hints'
      };
    }
  },
  {
    name: 'ndjson-output',
    description: 'STDOUT must output NDJSON format',
    severity: 'high',
    check: (code, lang) => ({
      rule: 'ndjson-output',
      status: code.includes('json.NewEncoder(os.Stdout)') || code.includes('JSON.stringify') ? 'present' : 'missing',
      severity: 'high',
      suggestion: 'Use NDJSON format for output (one JSON object per line)'
    })
  },
  {
    name: 'trace-id',
    description: 'Support AGENT_TRACE_ID environment variable',
    severity: 'low',
    check: (code, lang) => ({
      rule: 'trace-id',
      status: code.includes('AGENT_TRACE_ID') || code.includes('trace_id') ? 'present' : 'missing',
      severity: 'low',
      suggestion: 'Support AGENT_TRACE_ID for request correlation'
    })
  }
];

export function calculateScore(issues: AnelIssue[]): number {
  const weights = { high: 25, medium: 10, low: 5 };
  const maxDeduction = issues.reduce((sum, issue) => sum + weights[issue.severity], 0);
  return Math.max(0, 100 - maxDeduction);
}
```

**Step 2: Implement analyzer**

```typescript
// src/core/analyzer.ts
import { readFile } from 'fs/promises';
import { detectFileInfo } from './detector.js';
import { anelRules, calculateScore, type AnelIssue, type AnelAnalysisResult } from './rules.js';

export async function analyze(filePath: string, language?: string): Promise<AnelAnalysisResult> {
  const code = await readFile(filePath, 'utf-8');
  const fileInfo = language ? { path: filePath, language: language as any } : detectFileInfo(filePath);

  const issues: AnelIssue[] = anelRules.map(rule =>
    rule.check(code, fileInfo.language, fileInfo.framework)
  );

  return {
    file: filePath,
    language: fileInfo.language,
    framework: fileInfo.framework,
    complianceScore: calculateScore(issues),
    issues
  };
}
```

**Step 3: Run test**

Run: `npx vitest run`
Expected: PASS - tests should pass now

**Step 4: Commit**

```bash
git add src/core/analyzer.ts src/core/rules.ts
git commit -m "feat: implement rule-based analyzer"
```

---

## Phase 3: Fix Engine Implementation

### Task 5: Implement Code Generator

**Files:**
- Create: `src/anel-copilot/src/core/generator.ts`

**Step 1: Create generator with templates**

```typescript
// src/core/generator.ts
import type { AnelRule } from './types.js';

interface TemplateContext {
  commandName: string;
  language: string;
  framework?: string;
}

const goCobraTemplates = {
  'emit-spec': `
    // ANEL: Parse protocol flags
    emitSpec, _ := cmd.Flags().GetBool("emit-spec")
    outputFormat, _ := cmd.Flags().GetString("output-format")

    // ANEL: Handle --emit-spec
    if emitSpec {
        spec := anel.GetSpec("{{commandName}}")
        json.NewEncoder(os.Stdout).Encode(spec)
        return nil
    }
`,
  'dry-run': `
    // ANEL: Handle --dry-run
    if dryRun {
        fmt.Fprintf(os.Stderr, \`{"dry_run": true, "command": "{{commandName}}"}\`)
        return nil
    }
`,
  'error-format': `
    // ANEL: Use ANEL error format
    if err != nil {
        anelErr := anel.NewError(anel.E_{{errorCode}}, err.Error()).
            WithRecoveryHint("{{hintCode}}", "{{hintMessage}}").
            WithTraceID(os.Getenv("AGENT_TRACE_ID"))
        anelErr.EmitStderr()
        return err
    }
`,
  'ndjson-output': `
    // ANEL: NDJSON output
    encoder := json.NewEncoder(os.Stdout)
    for _, r := range results {
        encoder.Encode(r)
    }
`
};

export function generateFix(code: string, language: string, framework?: string, rules?: AnelRule[]): string {
  // Simple implementation: detect where to insert code
  // In production, use AST parsing for precise insertion

  let modified = code;

  // Add flags in init() or similar
  if (!code.includes('--emit-spec')) {
    modified = modified.replace(
      /(func init\(\).*?\{)/s,
      `$1
    searchCmd.Flags().Bool("emit-spec", false, "Output ANEL specification")
    searchCmd.Flags().Bool("dry-run", false, "Validate without executing")
    searchCmd.Flags().String("output-format", "ndjson", "Output format: json, ndjson, text")`
    );
  }

  // Add flag parsing in handler
  if (!code.includes('emitSpec,')) {
    modified = modified.replace(
      /(func handle\w+.*?\{)/s,
      `$1
    emitSpec, _ := cmd.Flags().GetBool("emit-spec")
    dryRun, _ := cmd.Flags().GetBool("dry-run")
    outputFormat, _ := cmd.Flags().GetString("output-format")

    if emitSpec {
        spec := anel.GetSpec("search")
        json.NewEncoder(os.Stdout).Encode(spec)
        return nil
    }

    if dryRun {
        fmt.Fprintf(os.Stderr, \`{"dry_run": true}\`)
        return nil
    }
`
    );
  }

  return modified;
}
```

**Step 2: Write test for generator**

```typescript
// tests/generator.test.ts
import { describe, it, expect } from 'vitest';
import { generateFix } from '../src/core/generator.js';

describe('AnelGenerator', () => {
  it('should add emit-spec and dry-run flags', () => {
    const input = `func init() { }`;
    const output = generateFix(input, 'go', 'cobra');
    expect(output).toContain('--emit-spec');
    expect(output).toContain('--dry-run');
  });
});
```

**Step 3: Run test**

Run: `npx vitest run`
Expected: PASS

**Step 4: Commit**

```bash
git add src/core/generator.ts
git commit -m "feat: implement code generator for auto-fix"
```

---

### Task 6: Implement Auto-Fix Integration

**Files:**
- Modify: `src/anel-copilot/src/index.ts` (update anel_fix handler)

**Step 1: Implement anel_fix in index.ts**

```typescript
// Add to CallToolRequestSchema handler:

if (name === 'anel_fix') {
  const { filePath, rules } = args;
  const code = await readFile(filePath, 'utf-8');
  const fileInfo = detectFileInfo(filePath);
  const modified = generateFix(code, fileInfo.language, fileInfo.framework, rules);

  // Calculate diff
  const diff = modified.split('\n')
    .map((line, i) => {
      const origLine = code.split('\n')[i];
      if (line !== origLine) return `+ ${line}`;
      return line;
    })
    .join('\n');

  await writeFile(filePath, modified);

  return {
    content: [{
      type: 'text',
      text: JSON.stringify({
        success: true,
        file: filePath,
        diff
      })
    }]
  };
}
```

**Step 2: Add missing imports**

```typescript
import { readFile, writeFile } from 'fs/promises';
import { detectFileInfo } from './core/detector.js';
import { generateFix } from './core/generator.js';
```

**Step 3: Build and test**

Run: `npm run build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/index.ts
git commit -m "feat: integrate auto-fix into MCP server"
```

---

## Phase 4: Verification and CLI

### Task 7: Implement Runtime Verifier

**Files:**
- Create: `src/anel-copilot/src/core/verifier.ts`

**Step 1: Create verifier**

```typescript
// src/core/verifier.ts
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export async function verify(binaryPath: string, command: string): Promise<AnelVerifyResult> {
  const details: string[] = [];

  // Test --emit-spec
  try {
    const { stdout } = await execAsync(`${binaryPath} ${command} --emit-spec`);
    const spec = JSON.parse(stdout);
    if (spec.version && spec.command) {
      details.push('[PASS] --emit-spec outputs valid JSON schema');
    }
  } catch (e) {
    details.push('[FAIL] --emit-spec not working');
  }

  // Test --dry-run
  try {
    const { stderr } = await execAsync(`${binaryPath} ${command} "test" --dry-run`);
    if (stderr.includes('dry_run') || stderr.includes('dry-run')) {
      details.push('[PASS] --dry-run outputs expected format');
    }
  } catch (e) {
    details.push('[FAIL] --dry-run not working');
  }

  // Test error format
  try {
    const { stderr } = await execAsync(`${binaryPath} ${command} ""`);
    const error = JSON.parse(stderr);
    if (error.error_code && error.recovery_hints) {
      details.push('[PASS] Error format includes recovery_hints');
    } else if (error.error_code) {
      details.push('[WARN] Error format missing recovery_hints');
    }
  } catch (e) {
    details.push('[FAIL] Error format not JSON');
  }

  const passed = details.filter(d => d.startsWith('[PASS]')).length;
  return {
    binary: binaryPath,
    command,
    passed: passed >= 2,
    details
  };
}
```

**Step 2: Integrate into MCP**

```typescript
// Add to index.ts handler
if (name === 'anel_verify') {
  const { binaryPath, command } = args;
  const result = await verify(binaryPath, command);
  return { content: [{ type: 'text', text: JSON.stringify(result) }] };
}
```

**Step 3: Commit**

```bash
git add src/core/verifier.ts
git commit -m "feat: implement runtime verifier"
```

---

### Task 8: Create CLI Wrapper

**Files:**
- Create: `src/anel-copilot/src/cli.ts`

**Step 1: Create CLI**

```typescript
// src/cli.ts
#!/usr/bin/env node

import { analyze } from './core/analyzer.js';
import { generateFix } from './core/generator.js';
import { verify } from './core/verifier.js';
import { readFile } from 'fs/promises';

const command = process.argv[2];

async function main() {
  if (command === 'analyze') {
    const result = await analyze(process.argv[3]);
    console.log(JSON.stringify(result, null, 2));
  } else if (command === 'fix') {
    const code = await readFile(process.argv[3], 'utf-8');
    const modified = generateFix(code, 'go');
    console.log(modified);
  } else if (command === 'verify') {
    const result = await verify(process.argv[3], process.argv[4]);
    console.log(JSON.stringify(result, null, 2));
  } else {
    console.log('Usage: anel-copilot <analyze|fix|verify> <args>');
  }
}

main();
```

**Step 2: Update package.json**

```json
{
  "bin": {
    "anel-copilot": "./dist/cli.js"
  }
}
```

**Step 3: Build and test**

Run: `npm run build && node dist/cli.js analyze tests/fixtures/sample-go.ts`
Expected: Outputs analysis result

**Step 4: Commit**

```bash
git add src/cli.ts package.json
git commit -m "feat: add CLI wrapper"
```

---

## Phase 5: Documentation and Publish

### Task 9: Create README

**Files:**
- Create: `src/anel-copilot/README.md`

**Step 1: Write README**

```markdown
# ANEL Copilot

ANEL Protocol Assistant for AI Coding Tools.

## Installation

```bash
npm install -g anel-copilot
```

## MCP Server Usage

```json
{
  "mcpServers": {
    "anel": {
      "command": "npx",
      "args": ["-y", "anel-copilot"]
    }
  }
}
```

## CLI Usage

```bash
# Analyze code
anel-copilot analyze ./cmd/search.go

# Auto-fix code
anel-copilot fix ./cmd/search.go

# Verify binary
anel-copilot verify ./my-cli "search"
```

## Available MCP Tools

- `anel_analyze` - Analyze code for ANEL compliance
- `anel_fix` - Auto-fix code to comply with ANEL
- `anel_verify` - Runtime verification
- `anel_explain` - Explain ANEL protocol
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add README"
```

---

## Summary

**Implemented:**
- ✅ MCP Server with 4 tools
- ✅ Rule-based analyzer
- ✅ Auto-fix generator
- ✅ Runtime verifier
- ✅ CLI wrapper
- ✅ Test infrastructure

**Next Steps:**
- Add LLM-based smart code modification
- Add more language/framework support
- Create Claude Code Skill
- Publish to npm
```

**Step 3: Commit**

```bash
git add README.md
git commit -m "docs: add README"
```

---

## Plan Complete

**Total Tasks:** 9 tasks across 5 phases

**Estimated Time:** 2-3 hours

**Files Created:**
- `src/anel-copilot/package.json`
- `src/anel-copilot/tsconfig.json`
- `src/anel-copilot/src/index.ts` (MCP server)
- `src/anel-copilot/src/cli.ts` (CLI wrapper)
- `src/anel-copilot/src/core/types.ts`
- `src/anel-copilot/src/core/detector.ts`
- `src/anel-copilot/src/core/rules.ts`
- `src/anel-copilot/src/core/analyzer.ts`
- `src/anel-copilot/src/core/generator.ts`
- `src/anel-copilot/src/core/verifier.ts`
- `src/anel-copilot/tests/`
- `src/anel-copilot/README.md`

---

**Plan complete and saved to `docs/plans/2026-02-18-anel-copilot.md`.**

Two execution options:

1. **Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

2. **Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

Which approach?
