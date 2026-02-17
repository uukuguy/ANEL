import { readFile } from "fs/promises";
import { detectFileInfo, detectFrameworkFromCode } from "./detector.js";
import { anelRules, calculateScore } from "./rules.js";
import { detectWithAst } from "./ast-detector.js";
import type { AnelAnalysisResult, AnelIssue, SupportedLanguage } from "./types.js";

export async function analyze(
  filePath: string,
  language?: SupportedLanguage
): Promise<AnelAnalysisResult> {
  const code = await readFile(filePath, "utf-8");
  return analyzeCode(code, filePath, language);
}

export function analyzeCode(
  code: string,
  filePath: string,
  language?: SupportedLanguage
): AnelAnalysisResult {
  const fileInfo = language
    ? { path: filePath, language }
    : detectFileInfo(filePath);

  const framework = detectFrameworkFromCode(code, fileInfo.language) ?? fileInfo.framework;

  const issues: AnelIssue[] = anelRules.map((rule) =>
    rule.check(code, fileInfo.language, framework)
  );

  return {
    file: filePath,
    language: fileInfo.language,
    framework,
    complianceScore: calculateScore(issues),
    issues,
  };
}

/**
 * Analyze code using AST when tree-sitter is available, falling back to string matching.
 */
export async function analyzeCodeWithAst(
  code: string,
  filePath: string,
  language?: SupportedLanguage
): Promise<AnelAnalysisResult> {
  const fileInfo = language
    ? { path: filePath, language }
    : detectFileInfo(filePath);

  const framework = detectFrameworkFromCode(code, fileInfo.language) ?? fileInfo.framework;

  // Try AST-based detection first
  const astIssues = await detectWithAst(code, fileInfo.language, framework);

  // Fall back to string matching if AST is unavailable
  const issues = astIssues ?? anelRules.map((rule) =>
    rule.check(code, fileInfo.language, framework)
  );

  return {
    file: filePath,
    language: fileInfo.language,
    framework,
    complianceScore: calculateScore(issues),
    issues,
  };
}
