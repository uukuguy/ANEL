import { describe, it, expect } from "vitest";
import { anelRules, calculateScore } from "../src/core/rules.js";
import type { AnelIssue } from "../src/core/types.js";

describe("Rules", () => {
  describe("calculateScore", () => {
    it("should return 100 for all present issues", () => {
      const issues: AnelIssue[] = [
        { rule: "emit-spec", status: "present", severity: "high" },
        { rule: "dry-run", status: "present", severity: "high" },
        { rule: "error-format", status: "present", severity: "high" },
        { rule: "ndjson-output", status: "present", severity: "high" },
        { rule: "trace-id", status: "present", severity: "low" },
        { rule: "output-format", status: "present", severity: "medium" },
        { rule: "env-vars", status: "present", severity: "medium" },
      ];
      expect(calculateScore(issues)).toBe(100);
    });

    it("should deduct 25 per missing high severity issue", () => {
      const issues: AnelIssue[] = [
        { rule: "emit-spec", status: "missing", severity: "high" },
        { rule: "dry-run", status: "present", severity: "high" },
        { rule: "error-format", status: "present", severity: "high" },
        { rule: "ndjson-output", status: "present", severity: "high" },
        { rule: "trace-id", status: "present", severity: "low" },
        { rule: "output-format", status: "present", severity: "medium" },
        { rule: "env-vars", status: "present", severity: "medium" },
      ];
      expect(calculateScore(issues)).toBe(75);
    });

    it("should deduct 10 per missing medium severity issue", () => {
      const issues: AnelIssue[] = [
        { rule: "emit-spec", status: "present", severity: "high" },
        { rule: "dry-run", status: "present", severity: "high" },
        { rule: "error-format", status: "present", severity: "high" },
        { rule: "ndjson-output", status: "present", severity: "high" },
        { rule: "trace-id", status: "present", severity: "low" },
        { rule: "output-format", status: "missing", severity: "medium" },
        { rule: "env-vars", status: "present", severity: "medium" },
      ];
      expect(calculateScore(issues)).toBe(90);
    });

    it("should deduct 5 per missing low severity issue", () => {
      const issues: AnelIssue[] = [
        { rule: "emit-spec", status: "present", severity: "high" },
        { rule: "dry-run", status: "present", severity: "high" },
        { rule: "error-format", status: "present", severity: "high" },
        { rule: "ndjson-output", status: "present", severity: "high" },
        { rule: "trace-id", status: "missing", severity: "low" },
        { rule: "output-format", status: "present", severity: "medium" },
        { rule: "env-vars", status: "present", severity: "medium" },
      ];
      expect(calculateScore(issues)).toBe(95);
    });

    it("should not go below 0", () => {
      const issues: AnelIssue[] = [
        { rule: "emit-spec", status: "missing", severity: "high" },
        { rule: "dry-run", status: "missing", severity: "high" },
        { rule: "error-format", status: "non-compliant", severity: "high" },
        { rule: "ndjson-output", status: "missing", severity: "high" },
        { rule: "trace-id", status: "missing", severity: "low" },
        { rule: "output-format", status: "missing", severity: "medium" },
        { rule: "env-vars", status: "missing", severity: "medium" },
      ];
      expect(calculateScore(issues)).toBe(0);
    });
  });

  describe("output-format rule", () => {
    const rule = anelRules.find((r) => r.name === "output-format")!;

    it("should detect --output-format flag", () => {
      const result = rule.check('cmd.Flags().String("output-format", "ndjson")', "go");
      expect(result.status).toBe("present");
    });

    it("should detect output_format field", () => {
      const result = rule.check("output_format: String,", "rust");
      expect(result.status).toBe("present");
    });

    it("should detect OutputFormat type", () => {
      const result = rule.check("type OutputFormat = 'json' | 'ndjson'", "typescript");
      expect(result.status).toBe("present");
    });

    it("should report missing when absent", () => {
      const result = rule.check("fn main() { println!(\"hello\"); }", "rust");
      expect(result.status).toBe("missing");
    });
  });

  describe("env-vars rule", () => {
    const rule = anelRules.find((r) => r.name === "env-vars")!;

    it("should detect AGENT_IDENTITY_TOKEN", () => {
      const result = rule.check('os.Getenv("AGENT_IDENTITY_TOKEN")', "go");
      expect(result.status).toBe("present");
    });

    it("should detect identity_token variable", () => {
      const result = rule.check("let identity_token = std::env::var(\"AGENT_IDENTITY_TOKEN\")", "rust");
      expect(result.status).toBe("present");
    });

    it("should detect IdentityToken type", () => {
      const result = rule.check("const IdentityToken = process.env.TOKEN", "typescript");
      expect(result.status).toBe("present");
    });

    it("should report missing when absent", () => {
      const result = rule.check("fn main() { println!(\"hello\"); }", "rust");
      expect(result.status).toBe("missing");
    });
  });
});
