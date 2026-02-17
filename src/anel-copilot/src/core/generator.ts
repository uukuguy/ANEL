import type { AnelRule, SupportedLanguage } from "./types.js";
import { createLlmProvider } from "./llm.js";
import { analyzeCode } from "./analyzer.js";

interface FixContext {
  commandName?: string;
  errorCode?: string;
  hintCode?: string;
  hintMessage?: string;
}

const goCobraFlagInit = `
func init() {
	cmd.Flags().Bool("emit-spec", false, "Output ANEL specification")
	cmd.Flags().Bool("dry-run", false, "Validate without executing")
	cmd.Flags().String("output-format", "ndjson", "Output format: json, ndjson, text")
}`;

const goCobraEmitSpec = `
	// ANEL: Handle --emit-spec
	emitSpec, _ := cmd.Flags().GetBool("emit-spec")
	if emitSpec {
		spec := anel.GetSpec("{{commandName}}")
		json.NewEncoder(os.Stdout).Encode(spec)
		return nil
	}`;

const goCobradryRun = `
	// ANEL: Handle --dry-run
	dryRun, _ := cmd.Flags().GetBool("dry-run")
	if dryRun {
		fmt.Fprintf(os.Stderr, \`{"dry_run": true, "command": "{{commandName}}"}\n\`)
		return nil
	}`;

const rustClapFlags = `
    #[arg(long, help = "Output ANEL specification")]
    emit_spec: bool,
    #[arg(long, help = "Validate without executing")]
    dry_run: bool,
    #[arg(long, default_value = "ndjson", help = "Output format")]
    output_format: String,`;

const pythonClickFlags = `
@click.option("--emit-spec", is_flag=True, help="Output ANEL specification")
@click.option("--dry-run", is_flag=True, help="Validate without executing")
@click.option("--output-format", default="ndjson", help="Output format")`;

export function generateFix(
  code: string,
  language: SupportedLanguage,
  framework?: string,
  ctx?: FixContext
): string {
  const commandName = ctx?.commandName ?? "command";
  let modified = code;

  switch (language) {
    case "go":
      modified = applyGoFixes(modified, framework, commandName);
      break;
    case "rust":
      modified = applyRustFixes(modified, commandName);
      break;
    case "python":
      modified = applyPythonFixes(modified, framework, commandName);
      break;
    case "typescript":
      modified = applyTypeScriptFixes(modified, commandName);
      break;
  }

  return modified;
}

function applyGoFixes(code: string, framework: string | undefined, commandName: string): string {
  let modified = code;

  // Add flag definitions if init() exists but flags are missing
  if (!code.includes("emit-spec") && code.includes("func init()")) {
    modified = modified.replace(
      /(func init\(\)\s*\{)/,
      `$1
	cmd.Flags().Bool("emit-spec", false, "Output ANEL specification")
	cmd.Flags().Bool("dry-run", false, "Validate without executing")
	cmd.Flags().String("output-format", "ndjson", "Output format")`
    );
  }

  // Add flag handling in RunE handler
  if (!code.includes("emitSpec") && code.match(/func\s+handle\w+/)) {
    const emitBlock = goCobraEmitSpec.replace("{{commandName}}", commandName);
    const dryRunBlock = goCobradryRun.replace("{{commandName}}", commandName);
    modified = modified.replace(
      /(func\s+handle\w+\([^)]*\)\s*error\s*\{)/,
      `$1${emitBlock}${dryRunBlock}`
    );
  }

  // Add AGENT_IDENTITY_TOKEN env var reading
  if (!code.includes("AGENT_IDENTITY_TOKEN") && code.match(/func\s+handle\w+/)) {
    modified = modified.replace(
      /(func\s+handle\w+\([^)]*\)\s*error\s*\{)/,
      `$1\n\tidentityToken := os.Getenv("AGENT_IDENTITY_TOKEN")`
    );
  }

  return modified;
}

function applyRustFixes(code: string, commandName: string): string {
  let modified = code;

  // Add clap derive flags to struct
  if (!code.includes("emit_spec") && code.includes("#[derive(")) {
    modified = modified.replace(
      /(struct\s+\w+\s*\{)/,
      `$1
${rustClapFlags}`
    );
  }

  // Add AGENT_IDENTITY_TOKEN env var reading
  if (!code.includes("AGENT_IDENTITY_TOKEN") && code.includes("fn main()")) {
    modified = modified.replace(
      /(fn main\(\)\s*\{)/,
      `$1\n    let identity_token = std::env::var("AGENT_IDENTITY_TOKEN").unwrap_or_default();`
    );
  }

  return modified;
}

function applyPythonFixes(code: string, framework: string | undefined, commandName: string): string {
  let modified = code;

  if (!code.includes("emit-spec") && code.includes("@click.command")) {
    modified = modified.replace(
      /(@click\.command[^\n]*\n)/,
      `$1${pythonClickFlags}\n`
    );
  }

  // Add AGENT_IDENTITY_TOKEN env var reading
  if (!code.includes("AGENT_IDENTITY_TOKEN") && code.includes("def ")) {
    modified = modified.replace(
      /(def\s+\w+\([^)]*\):\s*\n)/,
      `$1    identity_token = os.environ.get("AGENT_IDENTITY_TOKEN", "")\n`
    );
  }

  return modified;
}

function applyTypeScriptFixes(code: string, commandName: string): string {
  let modified = code;

  if (!code.includes("emit-spec") && code.includes(".command(")) {
    modified = modified.replace(
      /(\.command\([^)]*\))/,
      `$1
  .option("--emit-spec", "Output ANEL specification")
  .option("--dry-run", "Validate without executing")
  .option("--output-format <format>", "Output format", "ndjson")`
    );
  }

  // Add AGENT_IDENTITY_TOKEN env var reading
  if (!code.includes("AGENT_IDENTITY_TOKEN") && code.includes(".action(")) {
    modified = modified.replace(
      /(\.action\(\s*(?:async\s*)?\([^)]*\)\s*=>\s*\{)/,
      `$1\n    const identityToken = process.env.AGENT_IDENTITY_TOKEN ?? "";`
    );
  }

  return modified;
}

export async function generateFixWithLlm(
  code: string,
  language: SupportedLanguage,
  filePath: string,
  framework?: string,
  mode: "template" | "llm" = "template"
): Promise<string> {
  const provider = createLlmProvider(mode);
  const analysis = analyzeCode(code, filePath, language);
  const missingIssues = analysis.issues.filter((i) => i.status !== "present");

  if (missingIssues.length === 0) {
    return code; // Already compliant
  }

  return provider.generateFix({
    code,
    language,
    framework,
    issues: missingIssues,
  });
}
