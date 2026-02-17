import { describe, it, expect } from "vitest";
import { calculateScore } from "../src/core/rules.js";
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
      ];
      expect(calculateScore(issues)).toBe(75);
    });

    it("should deduct 5 per missing low severity issue", () => {
      const issues: AnelIssue[] = [
        { rule: "emit-spec", status: "present", severity: "high" },
        { rule: "dry-run", status: "present", severity: "high" },
        { rule: "error-format", status: "present", severity: "high" },
        { rule: "ndjson-output", status: "present", severity: "high" },
        { rule: "trace-id", status: "missing", severity: "low" },
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
      ];
      expect(calculateScore(issues)).toBe(0);
    });
  });
});
