import type { AnelIssue, AnelRule } from "./types.js";

export interface AnelRuleDefinition {
  name: AnelRule;
  description: string;
  severity: "high" | "medium" | "low";
  check: (code: string, language: string, framework?: string) => AnelIssue;
}

export const anelRules: AnelRuleDefinition[] = [
  {
    name: "emit-spec",
    description: "CLI must support --emit-spec flag to output JSON schema",
    severity: "high",
    check: (code) => ({
      rule: "emit-spec",
      status: hasPattern(code, ["--emit-spec", "emit-spec", "emit_spec"]) ? "present" : "missing",
      severity: "high",
      suggestion: "Add --emit-spec flag handler that outputs JSON schema describing command parameters",
    }),
  },
  {
    name: "dry-run",
    description: "CLI must support --dry-run flag for validation without execution",
    severity: "high",
    check: (code) => ({
      rule: "dry-run",
      status: hasPattern(code, ["--dry-run", "dry-run", "dry_run"]) ? "present" : "missing",
      severity: "high",
      suggestion: "Add --dry-run flag to validate parameters without executing the operation",
    }),
  },
  {
    name: "error-format",
    description: "Errors must follow RFC 7807 + recovery_hints",
    severity: "high",
    check: (code) => {
      const hasAnelError = hasPattern(code, [
        "anel.Error",
        "anel.NewError",
        "AnelError",
        "recovery_hints",
        "recovery_hint",
        "RecoveryHint",
        "WithRecoveryHint",
      ]);
      return {
        rule: "error-format",
        status: hasAnelError ? "present" : "non-compliant",
        severity: "high",
        suggestion:
          "Use ANEL error format with error_code, message, severity, recovery_hints[]",
      };
    },
  },
  {
    name: "ndjson-output",
    description: "STDOUT must output NDJSON format",
    severity: "high",
    check: (code, language) => {
      const patterns: string[] = [];
      switch (language) {
        case "go":
          patterns.push("json.NewEncoder(os.Stdout)", "json.Marshal");
          break;
        case "rust":
          patterns.push("serde_json::to_string", "serde_json::to_writer");
          break;
        case "python":
          patterns.push("json.dumps", "json.dump");
          break;
        case "typescript":
          patterns.push("JSON.stringify");
          break;
      }
      return {
        rule: "ndjson-output",
        status: hasPattern(code, patterns) ? "present" : "missing",
        severity: "high",
        suggestion: "Use NDJSON format for stdout output (one JSON object per line)",
      };
    },
  },
  {
    name: "trace-id",
    description: "Support AGENT_TRACE_ID environment variable",
    severity: "low",
    check: (code) => ({
      rule: "trace-id",
      status: hasPattern(code, ["AGENT_TRACE_ID", "trace_id", "traceId"]) ? "present" : "missing",
      severity: "low",
      suggestion: "Support AGENT_TRACE_ID environment variable for request correlation",
    }),
  },
];

function hasPattern(code: string, patterns: string[]): boolean {
  return patterns.some((p) => code.includes(p));
}

export function calculateScore(issues: AnelIssue[]): number {
  const weights: Record<string, number> = { high: 25, medium: 10, low: 5 };
  let deduction = 0;
  for (const issue of issues) {
    if (issue.status !== "present") {
      deduction += weights[issue.severity] ?? 0;
    }
  }
  return Math.max(0, 100 - deduction);
}
