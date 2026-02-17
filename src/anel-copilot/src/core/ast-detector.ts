import type { AnelIssue, SupportedLanguage } from "./types.js";

// Tree-sitter types (dynamically imported)
interface TreeSitterParser {
  setLanguage(lang: unknown): void;
  parse(code: string): { rootNode: TreeSitterNode };
}

interface TreeSitterNode {
  type: string;
  text: string;
  children: TreeSitterNode[];
  namedChildren: TreeSitterNode[];
  childForFieldName(name: string): TreeSitterNode | null;
  descendantsOfType(type: string | string[]): TreeSitterNode[];
}

let treeSitterAvailable: boolean | null = null;

async function loadTreeSitter(): Promise<{
  Parser: new () => TreeSitterParser;
  languages: Record<string, unknown>;
} | null> {
  if (treeSitterAvailable === false) return null;

  try {
    const parserModule = await import(/* @vite-ignore */ "tree-sitter" + "");
    const Parser = parserModule.default;
    const languages: Record<string, unknown> = {};

    try { languages.go = (await import(/* @vite-ignore */ "tree-sitter-go" + "")).default; } catch {}
    try { languages.rust = (await import(/* @vite-ignore */ "tree-sitter-rust" + "")).default; } catch {}
    try { languages.python = (await import(/* @vite-ignore */ "tree-sitter-python" + "")).default; } catch {}
    try {
      const ts = await import(/* @vite-ignore */ "tree-sitter-typescript" + "");
      languages.typescript = ts.default.typescript;
    } catch {}

    treeSitterAvailable = true;
    return { Parser, languages };
  } catch {
    treeSitterAvailable = false;
    return null;
  }
}

export function isTreeSitterAvailable(): boolean {
  return treeSitterAvailable === true;
}

export async function detectWithAst(
  code: string,
  language: SupportedLanguage,
  framework?: string
): Promise<AnelIssue[] | null> {
  const ts = await loadTreeSitter();
  if (!ts || !ts.languages[language]) return null;

  const parser = new ts.Parser();
  parser.setLanguage(ts.languages[language]);
  const tree = parser.parse(code);
  const root = tree.rootNode;

  switch (language) {
    case "go":
      return detectGoAnel(root, code);
    case "rust":
      return detectRustAnel(root, code);
    case "python":
      return detectPythonAnel(root, code);
    case "typescript":
      return detectTypeScriptAnel(root, code);
    default:
      return null;
  }
}

function detectGoAnel(root: TreeSitterNode, code: string): AnelIssue[] {
  const issues: AnelIssue[] = [];
  const allText = code;

  // Check for --emit-spec: look for string literals containing "emit-spec"
  const stringLiterals = root.descendantsOfType("interpreted_string_literal");
  const hasEmitSpec = stringLiterals.some((n) => n.text.includes("emit-spec"));
  issues.push({
    rule: "emit-spec",
    status: hasEmitSpec ? "present" : "missing",
    severity: "high",
    suggestion: "Add --emit-spec flag handler that outputs JSON schema describing command parameters",
  });

  // Check for --dry-run
  const hasDryRun = stringLiterals.some((n) => n.text.includes("dry-run"));
  issues.push({
    rule: "dry-run",
    status: hasDryRun ? "present" : "missing",
    severity: "high",
    suggestion: "Add --dry-run flag to validate parameters without executing the operation",
  });

  // Check for error format: look for anel.Error, anel.NewError, recovery_hint patterns
  const hasErrorFormat = stringLiterals.some(
    (n) =>
      n.text.includes("recovery_hint") ||
      n.text.includes("RecoveryHint") ||
      n.text.includes("error_code")
  ) || allText.includes("anel.NewError") || allText.includes("WithRecoveryHint");
  issues.push({
    rule: "error-format",
    status: hasErrorFormat ? "present" : "non-compliant",
    severity: "high",
    suggestion: "Use ANEL error format with error_code, message, severity, recovery_hints[]",
  });

  // Check for NDJSON output: look for json.NewEncoder or json.Marshal calls
  const callExprs = root.descendantsOfType("call_expression");
  const selectorExprs = root.descendantsOfType("selector_expression");
  const hasNdjson =
    selectorExprs.some((n) => n.text.includes("json.NewEncoder") || n.text.includes("json.Marshal"));
  issues.push({
    rule: "ndjson-output",
    status: hasNdjson ? "present" : "missing",
    severity: "high",
    suggestion: "Use NDJSON format for stdout output (one JSON object per line)",
  });

  // Check for AGENT_TRACE_ID
  const hasTraceId = stringLiterals.some((n) => n.text.includes("AGENT_TRACE_ID"));
  issues.push({
    rule: "trace-id",
    status: hasTraceId ? "present" : "missing",
    severity: "low",
    suggestion: "Support AGENT_TRACE_ID environment variable for request correlation",
  });

  // Check for --output-format
  const hasOutputFormat = stringLiterals.some((n) => n.text.includes("output-format"));
  issues.push({
    rule: "output-format",
    status: hasOutputFormat ? "present" : "missing",
    severity: "medium",
    suggestion: 'Add --output-format flag supporting json/ndjson/text',
  });

  // Check for AGENT_IDENTITY_TOKEN
  const hasIdentityToken = stringLiterals.some((n) => n.text.includes("AGENT_IDENTITY_TOKEN"));
  issues.push({
    rule: "env-vars",
    status: hasIdentityToken ? "present" : "missing",
    severity: "medium",
    suggestion: "Support AGENT_IDENTITY_TOKEN environment variable for bearer auth",
  });

  return issues;
}

function detectRustAnel(root: TreeSitterNode, code: string): AnelIssue[] {
  const issues: AnelIssue[] = [];
  const allText = code;

  // Look for string literals
  const stringLiterals = root.descendantsOfType("string_literal");

  const hasEmitSpec =
    stringLiterals.some((n) => n.text.includes("emit-spec") || n.text.includes("emit_spec")) ||
    allText.includes("emit_spec");
  issues.push({
    rule: "emit-spec",
    status: hasEmitSpec ? "present" : "missing",
    severity: "high",
    suggestion: "Add --emit-spec flag handler that outputs JSON schema describing command parameters",
  });

  const hasDryRun =
    stringLiterals.some((n) => n.text.includes("dry-run") || n.text.includes("dry_run")) ||
    allText.includes("dry_run");
  issues.push({
    rule: "dry-run",
    status: hasDryRun ? "present" : "missing",
    severity: "high",
    suggestion: "Add --dry-run flag to validate parameters without executing the operation",
  });

  const hasErrorFormat =
    allText.includes("AnelError") ||
    allText.includes("recovery_hint") ||
    allText.includes("with_recovery_hint");
  issues.push({
    rule: "error-format",
    status: hasErrorFormat ? "present" : "non-compliant",
    severity: "high",
    suggestion: "Use ANEL error format with error_code, message, severity, recovery_hints[]",
  });

  const hasNdjson =
    allText.includes("serde_json::to_string") || allText.includes("serde_json::to_writer");
  issues.push({
    rule: "ndjson-output",
    status: hasNdjson ? "present" : "missing",
    severity: "high",
    suggestion: "Use NDJSON format for stdout output (one JSON object per line)",
  });

  const hasTraceId = stringLiterals.some((n) => n.text.includes("AGENT_TRACE_ID")) || allText.includes("trace_id");
  issues.push({
    rule: "trace-id",
    status: hasTraceId ? "present" : "missing",
    severity: "low",
    suggestion: "Support AGENT_TRACE_ID environment variable for request correlation",
  });

  const hasOutputFormat =
    stringLiterals.some((n) => n.text.includes("output-format") || n.text.includes("output_format")) ||
    allText.includes("output_format");
  issues.push({
    rule: "output-format",
    status: hasOutputFormat ? "present" : "missing",
    severity: "medium",
    suggestion: 'Add --output-format flag supporting json/ndjson/text',
  });

  const hasIdentityToken =
    stringLiterals.some((n) => n.text.includes("AGENT_IDENTITY_TOKEN")) ||
    allText.includes("identity_token");
  issues.push({
    rule: "env-vars",
    status: hasIdentityToken ? "present" : "missing",
    severity: "medium",
    suggestion: "Support AGENT_IDENTITY_TOKEN environment variable for bearer auth",
  });

  return issues;
}

function detectPythonAnel(root: TreeSitterNode, code: string): AnelIssue[] {
  const issues: AnelIssue[] = [];
  const allText = code;

  const stringNodes = root.descendantsOfType("string");

  const hasEmitSpec = stringNodes.some((n) => n.text.includes("emit-spec") || n.text.includes("emit_spec"));
  issues.push({
    rule: "emit-spec",
    status: hasEmitSpec ? "present" : "missing",
    severity: "high",
    suggestion: "Add --emit-spec flag handler that outputs JSON schema describing command parameters",
  });

  const hasDryRun = stringNodes.some((n) => n.text.includes("dry-run") || n.text.includes("dry_run"));
  issues.push({
    rule: "dry-run",
    status: hasDryRun ? "present" : "missing",
    severity: "high",
    suggestion: "Add --dry-run flag to validate parameters without executing the operation",
  });

  const hasErrorFormat =
    allText.includes("AnelError") ||
    allText.includes("recovery_hint") ||
    allText.includes("RecoveryHint");
  issues.push({
    rule: "error-format",
    status: hasErrorFormat ? "present" : "non-compliant",
    severity: "high",
    suggestion: "Use ANEL error format with error_code, message, severity, recovery_hints[]",
  });

  const hasNdjson = allText.includes("json.dumps") || allText.includes("json.dump");
  issues.push({
    rule: "ndjson-output",
    status: hasNdjson ? "present" : "missing",
    severity: "high",
    suggestion: "Use NDJSON format for stdout output (one JSON object per line)",
  });

  const hasTraceId = stringNodes.some((n) => n.text.includes("AGENT_TRACE_ID")) || allText.includes("trace_id");
  issues.push({
    rule: "trace-id",
    status: hasTraceId ? "present" : "missing",
    severity: "low",
    suggestion: "Support AGENT_TRACE_ID environment variable for request correlation",
  });

  const hasOutputFormat = stringNodes.some(
    (n) => n.text.includes("output-format") || n.text.includes("output_format")
  );
  issues.push({
    rule: "output-format",
    status: hasOutputFormat ? "present" : "missing",
    severity: "medium",
    suggestion: 'Add --output-format flag supporting json/ndjson/text',
  });

  const hasIdentityToken =
    stringNodes.some((n) => n.text.includes("AGENT_IDENTITY_TOKEN")) ||
    allText.includes("identity_token");
  issues.push({
    rule: "env-vars",
    status: hasIdentityToken ? "present" : "missing",
    severity: "medium",
    suggestion: "Support AGENT_IDENTITY_TOKEN environment variable for bearer auth",
  });

  return issues;
}

function detectTypeScriptAnel(root: TreeSitterNode, code: string): AnelIssue[] {
  const issues: AnelIssue[] = [];
  const allText = code;

  const stringNodes = root.descendantsOfType("string");

  const hasEmitSpec =
    stringNodes.some((n) => n.text.includes("emit-spec") || n.text.includes("emitSpec")) ||
    allText.includes("emit-spec");
  issues.push({
    rule: "emit-spec",
    status: hasEmitSpec ? "present" : "missing",
    severity: "high",
    suggestion: "Add --emit-spec flag handler that outputs JSON schema describing command parameters",
  });

  const hasDryRun =
    stringNodes.some((n) => n.text.includes("dry-run") || n.text.includes("dryRun")) ||
    allText.includes("dry-run");
  issues.push({
    rule: "dry-run",
    status: hasDryRun ? "present" : "missing",
    severity: "high",
    suggestion: "Add --dry-run flag to validate parameters without executing the operation",
  });

  const hasErrorFormat =
    allText.includes("AnelError") ||
    allText.includes("recovery_hint") ||
    allText.includes("RecoveryHint");
  issues.push({
    rule: "error-format",
    status: hasErrorFormat ? "present" : "non-compliant",
    severity: "high",
    suggestion: "Use ANEL error format with error_code, message, severity, recovery_hints[]",
  });

  const hasNdjson = allText.includes("JSON.stringify");
  issues.push({
    rule: "ndjson-output",
    status: hasNdjson ? "present" : "missing",
    severity: "high",
    suggestion: "Use NDJSON format for stdout output (one JSON object per line)",
  });

  const hasTraceId =
    stringNodes.some((n) => n.text.includes("AGENT_TRACE_ID")) || allText.includes("traceId");
  issues.push({
    rule: "trace-id",
    status: hasTraceId ? "present" : "missing",
    severity: "low",
    suggestion: "Support AGENT_TRACE_ID environment variable for request correlation",
  });

  const hasOutputFormat =
    stringNodes.some((n) => n.text.includes("output-format") || n.text.includes("outputFormat")) ||
    allText.includes("OutputFormat");
  issues.push({
    rule: "output-format",
    status: hasOutputFormat ? "present" : "missing",
    severity: "medium",
    suggestion: 'Add --output-format flag supporting json/ndjson/text',
  });

  const hasIdentityToken =
    stringNodes.some((n) => n.text.includes("AGENT_IDENTITY_TOKEN")) ||
    allText.includes("IdentityToken") ||
    allText.includes("identityToken");
  issues.push({
    rule: "env-vars",
    status: hasIdentityToken ? "present" : "missing",
    severity: "medium",
    suggestion: "Support AGENT_IDENTITY_TOKEN environment variable for bearer auth",
  });

  return issues;
}
