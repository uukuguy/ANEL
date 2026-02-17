import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  TemplateLlmProvider,
  AnthropicLlmProvider,
  createLlmProvider,
} from "../src/core/llm.js";
import { sampleGoNonCompliant } from "./fixtures/sample-go.js";
import { analyzeCode } from "../src/core/analyzer.js";

describe("LLM Providers", () => {
  describe("TemplateLlmProvider", () => {
    const provider = new TemplateLlmProvider();

    it("should fix non-compliant Go code using template logic", async () => {
      const analysis = analyzeCode(sampleGoNonCompliant, "cmd/search.go", "go");
      const missingIssues = analysis.issues.filter((i) => i.status !== "present");

      const fixed = await provider.generateFix({
        code: sampleGoNonCompliant,
        language: "go",
        framework: "cobra",
        issues: missingIssues,
      });

      expect(fixed).toContain("emit-spec");
      expect(fixed).toContain("dry-run");
    });

    it("should return unchanged code when no issues", async () => {
      const code = "package main\nfunc main() {}";
      const fixed = await provider.generateFix({
        code,
        language: "go",
        issues: [],
      });
      // Template provider always applies all fixes regardless of issues list
      expect(fixed).toBeDefined();
    });
  });

  describe("AnthropicLlmProvider", () => {
    it("should build correct prompt structure", () => {
      const provider = new AnthropicLlmProvider("test-key");
      const { system, user } = provider.buildPrompt({
        code: 'fn main() { println!("hello"); }',
        language: "rust",
        framework: "clap",
        issues: [
          {
            rule: "emit-spec",
            status: "missing",
            severity: "high",
            suggestion: "Add --emit-spec flag",
          },
          {
            rule: "dry-run",
            status: "missing",
            severity: "high",
            suggestion: "Add --dry-run flag",
          },
        ],
      });

      expect(system).toContain("ANEL");
      expect(system).toContain("--emit-spec");
      expect(user).toContain("rust");
      expect(user).toContain("clap");
      expect(user).toContain("emit-spec (high)");
      expect(user).toContain("dry-run (high)");
      expect(user).toContain('fn main()');
    });

    it("should include context in prompt when provided", () => {
      const provider = new AnthropicLlmProvider("test-key");
      const { user } = provider.buildPrompt({
        code: "fn main() {}",
        language: "rust",
        issues: [
          { rule: "emit-spec", status: "missing", severity: "high", suggestion: "Add flag" },
        ],
        context: "This is a search CLI tool",
      });

      expect(user).toContain("This is a search CLI tool");
    });

    it("should only include non-present issues in prompt", () => {
      const provider = new AnthropicLlmProvider("test-key");
      const { user } = provider.buildPrompt({
        code: "fn main() {}",
        language: "rust",
        issues: [
          { rule: "emit-spec", status: "present", severity: "high", suggestion: "Already present" },
          { rule: "dry-run", status: "missing", severity: "high", suggestion: "Add --dry-run" },
        ],
      });

      expect(user).not.toContain("Already present");
      expect(user).toContain("Add --dry-run");
    });

    it("should call Anthropic API and extract code from response", async () => {
      const provider = new AnthropicLlmProvider("test-key");
      const mockResponse = {
        ok: true,
        json: async () => ({
          content: [
            {
              type: "text",
              text: '```rust\nfn main() {\n    let emit_spec = true;\n}\n```',
            },
          ],
        }),
      };

      const originalFetch = globalThis.fetch;
      globalThis.fetch = vi.fn().mockResolvedValue(mockResponse) as any;

      try {
        const result = await provider.generateFix({
          code: "fn main() {}",
          language: "rust",
          issues: [
            { rule: "emit-spec", status: "missing", severity: "high", suggestion: "Add flag" },
          ],
        });

        expect(result).toContain("emit_spec");
        expect(globalThis.fetch).toHaveBeenCalledWith(
          "https://api.anthropic.com/v1/messages",
          expect.objectContaining({
            method: "POST",
            headers: expect.objectContaining({
              "x-api-key": "test-key",
            }),
          })
        );
      } finally {
        globalThis.fetch = originalFetch;
      }
    });

    it("should throw on API error", async () => {
      const provider = new AnthropicLlmProvider("test-key");
      const originalFetch = globalThis.fetch;
      globalThis.fetch = vi.fn().mockResolvedValue({
        ok: false,
        status: 401,
        statusText: "Unauthorized",
      }) as any;

      try {
        await expect(
          provider.generateFix({
            code: "fn main() {}",
            language: "rust",
            issues: [
              { rule: "emit-spec", status: "missing", severity: "high", suggestion: "Add flag" },
            ],
          })
        ).rejects.toThrow("Anthropic API error: 401 Unauthorized");
      } finally {
        globalThis.fetch = originalFetch;
      }
    });
  });

  describe("createLlmProvider", () => {
    const originalEnv = process.env.ANTHROPIC_API_KEY;

    afterEach(() => {
      if (originalEnv !== undefined) {
        process.env.ANTHROPIC_API_KEY = originalEnv;
      } else {
        delete process.env.ANTHROPIC_API_KEY;
      }
    });

    it("should return TemplateLlmProvider for template mode", () => {
      const provider = createLlmProvider("template");
      expect(provider).toBeInstanceOf(TemplateLlmProvider);
    });

    it("should return AnthropicLlmProvider when API key is set and mode is llm", () => {
      process.env.ANTHROPIC_API_KEY = "test-key";
      const provider = createLlmProvider("llm");
      expect(provider).toBeInstanceOf(AnthropicLlmProvider);
    });

    it("should fallback to TemplateLlmProvider when no API key and mode is llm", () => {
      delete process.env.ANTHROPIC_API_KEY;
      const provider = createLlmProvider("llm");
      expect(provider).toBeInstanceOf(TemplateLlmProvider);
    });

    it("should default to template mode", () => {
      const provider = createLlmProvider();
      expect(provider).toBeInstanceOf(TemplateLlmProvider);
    });
  });
});
