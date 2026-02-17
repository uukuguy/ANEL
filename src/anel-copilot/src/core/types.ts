// ANEL Protocol Types

export interface AnelIssue {
  rule: AnelRule;
  status: "missing" | "non-compliant" | "present";
  severity: "high" | "medium" | "low";
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

export interface FileInfo {
  path: string;
  language: SupportedLanguage;
  framework?: string;
}

export type SupportedLanguage = "go" | "rust" | "python" | "typescript";

export type AnelRule =
  | "emit-spec"
  | "dry-run"
  | "output-format"
  | "error-format"
  | "ndjson-output"
  | "trace-id"
  | "env-vars";
