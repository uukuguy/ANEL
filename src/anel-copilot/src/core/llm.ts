import type { AnelIssue, SupportedLanguage } from "./types.js";
import { generateFix } from "./generator.js";

export interface LlmProvider {
  generateFix(params: {
    code: string;
    language: SupportedLanguage;
    framework?: string;
    issues: AnelIssue[];
    context?: string;
  }): Promise<string>;
}

/**
 * Wraps existing template-based generator as an LlmProvider.
 * No API key needed — uses regex substitution.
 */
export class TemplateLlmProvider implements LlmProvider {
  async generateFix(params: {
    code: string;
    language: SupportedLanguage;
    framework?: string;
    issues: AnelIssue[];
  }): Promise<string> {
    return generateFix(params.code, params.language, params.framework);
  }
}

const ANEL_SYSTEM_PROMPT = `You are an expert at making CLI tools compliant with the ANEL (Agent-Native Execution Layer) protocol.

ANEL protocol requirements:
1. --emit-spec: Output JSON schema describing command parameters
2. --dry-run: Validate parameters without executing
3. --output-format: Support json/ndjson/text output formats
4. Error format: RFC 7807 with error_code, message, severity, recovery_hints[]
5. NDJSON output: All stdout must be newline-delimited JSON
6. AGENT_TRACE_ID: Support environment variable for distributed tracing
7. AGENT_IDENTITY_TOKEN: Support environment variable for bearer auth

When fixing code:
- Preserve existing functionality
- Add only the missing ANEL features
- Follow the language's idiomatic patterns
- Return ONLY the complete modified source code, no explanations`;

/**
 * Uses Anthropic Claude API for intelligent code fixes.
 * Requires ANTHROPIC_API_KEY environment variable.
 */
export class AnthropicLlmProvider implements LlmProvider {
  private apiKey: string;
  private model: string;

  constructor(apiKey: string, model = "claude-sonnet-4-20250514") {
    this.apiKey = apiKey;
    this.model = model;
  }

  buildPrompt(params: {
    code: string;
    language: SupportedLanguage;
    framework?: string;
    issues: AnelIssue[];
    context?: string;
  }): { system: string; user: string } {
    const missingRules = params.issues
      .filter((i) => i.status !== "present")
      .map((i) => `- ${i.rule} (${i.severity}): ${i.suggestion}`)
      .join("\n");

    const user = `Fix this ${params.language}${params.framework ? ` (${params.framework})` : ""} code for ANEL compliance.

Missing/non-compliant rules:
${missingRules}

${params.context ? `Additional context: ${params.context}\n` : ""}Source code:
\`\`\`${params.language}
${params.code}
\`\`\`

Return ONLY the fixed source code inside a single code block.`;

    return { system: ANEL_SYSTEM_PROMPT, user };
  }

  async generateFix(params: {
    code: string;
    language: SupportedLanguage;
    framework?: string;
    issues: AnelIssue[];
    context?: string;
  }): Promise<string> {
    const { system, user } = this.buildPrompt(params);

    const response = await fetch("https://api.anthropic.com/v1/messages", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "x-api-key": this.apiKey,
        "anthropic-version": "2023-06-01",
      },
      body: JSON.stringify({
        model: this.model,
        max_tokens: 4096,
        system,
        messages: [{ role: "user", content: user }],
      }),
    });

    if (!response.ok) {
      throw new Error(`Anthropic API error: ${response.status} ${response.statusText}`);
    }

    const data = (await response.json()) as {
      content: Array<{ type: string; text?: string }>;
    };
    const text = data.content.find((c) => c.type === "text")?.text ?? "";

    // Extract code from markdown code block
    const codeMatch = text.match(/```[\w]*\n([\s\S]*?)```/);
    return codeMatch ? codeMatch[1].trim() : text.trim();
  }
}

/**
 * Create the appropriate LLM provider based on available configuration.
 * Falls back to template mode if no API key is available.
 */
export function createLlmProvider(mode: "template" | "llm" = "template"): LlmProvider {
  if (mode === "llm") {
    const apiKey = process.env.ANTHROPIC_API_KEY;
    if (apiKey) {
      return new AnthropicLlmProvider(apiKey);
    }
    // Fallback to template if no API key
    console.error("Warning: ANTHROPIC_API_KEY not set, falling back to template mode");
  }
  return new TemplateLlmProvider();
}
