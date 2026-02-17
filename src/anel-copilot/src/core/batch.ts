import { readdir, stat } from "fs/promises";
import { join, extname } from "path";
import { analyze } from "./analyzer.js";
import type { AnelAnalysisResult } from "./types.js";

const DEFAULT_EXTENSIONS = [".go", ".rs", ".py", ".ts", ".js"];

export interface BatchAnalysisResult {
  files: AnelAnalysisResult[];
  summary: {
    total: number;
    compliant: number;
    nonCompliant: number;
    averageScore: number;
  };
}

export async function analyzeDirectory(
  dirPath: string,
  options?: { recursive?: boolean; extensions?: string[] }
): Promise<BatchAnalysisResult> {
  const recursive = options?.recursive ?? true;
  const extensions = options?.extensions ?? DEFAULT_EXTENSIONS;

  const filePaths = await collectFiles(dirPath, extensions, recursive);
  const files: AnelAnalysisResult[] = [];

  for (const filePath of filePaths) {
    try {
      const result = await analyze(filePath);
      files.push(result);
    } catch {
      // Skip files that can't be analyzed (binary, permission errors, etc.)
    }
  }

  const total = files.length;
  const compliant = files.filter((f) => f.complianceScore === 100).length;
  const nonCompliant = total - compliant;
  const averageScore = total > 0
    ? Math.round(files.reduce((sum, f) => sum + f.complianceScore, 0) / total)
    : 0;

  return {
    files,
    summary: { total, compliant, nonCompliant, averageScore },
  };
}

async function collectFiles(
  dirPath: string,
  extensions: string[],
  recursive: boolean
): Promise<string[]> {
  const results: string[] = [];
  const entries = await readdir(dirPath, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = join(dirPath, entry.name);
    if (entry.isDirectory() && recursive) {
      const nested = await collectFiles(fullPath, extensions, recursive);
      results.push(...nested);
    } else if (entry.isFile() && extensions.includes(extname(entry.name))) {
      results.push(fullPath);
    }
  }

  return results;
}
