import type { FileInfo, SupportedLanguage } from "./types.js";

export function detectFileInfo(filePath: string): FileInfo {
  const ext = filePath.split(".").pop()?.toLowerCase();
  const language = detectLanguage(ext);
  const framework = detectFramework(filePath, language);
  return { path: filePath, language, framework };
}

function detectLanguage(ext: string | undefined): SupportedLanguage {
  switch (ext) {
    case "go":
      return "go";
    case "rs":
      return "rust";
    case "py":
      return "python";
    case "ts":
    case "js":
      return "typescript";
    default:
      throw new Error(`Unsupported file extension: .${ext}`);
  }
}

function detectFramework(filePath: string, language: SupportedLanguage): string | undefined {
  switch (language) {
    case "go":
      return "cobra"; // most common Go CLI framework
    case "rust":
      return "clap";
    case "python":
      return "click";
    case "typescript":
      return "commander";
    default:
      return undefined;
  }
}

export function detectFrameworkFromCode(code: string, language: SupportedLanguage): string | undefined {
  switch (language) {
    case "go":
      if (code.includes("cobra")) return "cobra";
      if (code.includes("urfave/cli")) return "urfave-cli";
      return undefined;
    case "rust":
      if (code.includes("clap")) return "clap";
      return undefined;
    case "python":
      if (code.includes("click")) return "click";
      if (code.includes("argparse")) return "argparse";
      if (code.includes("typer")) return "typer";
      return undefined;
    case "typescript":
      if (code.includes("commander")) return "commander";
      if (code.includes("oclif")) return "oclif";
      if (code.includes("yargs")) return "yargs";
      return undefined;
    default:
      return undefined;
  }
}
