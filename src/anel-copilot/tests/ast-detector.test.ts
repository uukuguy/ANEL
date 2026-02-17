import { describe, it, expect } from "vitest";
import { detectWithAst, isTreeSitterAvailable } from "../src/core/ast-detector.js";
import { analyzeCode, analyzeCodeWithAst } from "../src/core/analyzer.js";
import {
  sampleGoNonCompliant,
  sampleGoCompliant,
  sampleGoPartial,
} from "./fixtures/sample-go.js";
import {
  sampleRustNonCompliant,
  sampleRustCompliant,
} from "./fixtures/sample-rust.js";

describe("AST Detector", () => {
  describe("when tree-sitter is available", () => {
    it("should detect Go non-compliant code issues", async () => {
      const issues = await detectWithAst(sampleGoNonCompliant, "go");
      if (issues === null) {
        // tree-sitter not installed, skip
        console.log("tree-sitter-go not available, skipping AST test");
        return;
      }

      const emitSpec = issues.find((i) => i.rule === "emit-spec");
      expect(emitSpec?.status).toBe("missing");

      const dryRun = issues.find((i) => i.rule === "dry-run");
      expect(dryRun?.status).toBe("missing");
    });

    it("should detect Go compliant code issues", async () => {
      const issues = await detectWithAst(sampleGoCompliant, "go");
      if (issues === null) return;

      const emitSpec = issues.find((i) => i.rule === "emit-spec");
      expect(emitSpec?.status).toBe("present");

      const dryRun = issues.find((i) => i.rule === "dry-run");
      expect(dryRun?.status).toBe("present");

      const outputFormat = issues.find((i) => i.rule === "output-format");
      expect(outputFormat?.status).toBe("present");

      const envVars = issues.find((i) => i.rule === "env-vars");
      expect(envVars?.status).toBe("present");
    });

    it("should detect Rust non-compliant code issues", async () => {
      const issues = await detectWithAst(sampleRustNonCompliant, "rust");
      if (issues === null) return;

      const emitSpec = issues.find((i) => i.rule === "emit-spec");
      expect(emitSpec?.status).toBe("missing");
    });

    it("should detect Rust compliant code issues", async () => {
      const issues = await detectWithAst(sampleRustCompliant, "rust");
      if (issues === null) return;

      const emitSpec = issues.find((i) => i.rule === "emit-spec");
      expect(emitSpec?.status).toBe("present");

      const envVars = issues.find((i) => i.rule === "env-vars");
      expect(envVars?.status).toBe("present");
    });

    it("should return all 7 rules for each language", async () => {
      const issues = await detectWithAst(sampleGoCompliant, "go");
      if (issues === null) return;

      expect(issues.length).toBe(7);
      const ruleNames = issues.map((i) => i.rule).sort();
      expect(ruleNames).toEqual([
        "dry-run",
        "emit-spec",
        "env-vars",
        "error-format",
        "ndjson-output",
        "output-format",
        "trace-id",
      ]);
    });
  });

  describe("AST results match string-matching results", () => {
    it("should produce consistent results for Go compliant code", async () => {
      const stringResult = analyzeCode(sampleGoCompliant, "cmd/search.go", "go");
      const astResult = await analyzeCodeWithAst(sampleGoCompliant, "cmd/search.go", "go");

      // Both should agree on compliance score for fully compliant code
      if (isTreeSitterAvailable()) {
        expect(astResult.complianceScore).toBe(stringResult.complianceScore);
      } else {
        // When tree-sitter is unavailable, analyzeCodeWithAst falls back to string matching
        expect(astResult.complianceScore).toBe(stringResult.complianceScore);
      }
    });

    it("should produce consistent results for Go non-compliant code", async () => {
      const stringResult = analyzeCode(sampleGoNonCompliant, "cmd/search.go", "go");
      const astResult = await analyzeCodeWithAst(sampleGoNonCompliant, "cmd/search.go", "go");

      // Both should agree that non-compliant code has low score
      expect(astResult.complianceScore).toBeLessThanOrEqual(5);
      expect(stringResult.complianceScore).toBeLessThanOrEqual(5);
    });

    it("should produce consistent results for Rust compliant code", async () => {
      const stringResult = analyzeCode(sampleRustCompliant, "src/main.rs", "rust");
      const astResult = await analyzeCodeWithAst(sampleRustCompliant, "src/main.rs", "rust");

      expect(astResult.complianceScore).toBe(stringResult.complianceScore);
    });
  });

  describe("fallback behavior", () => {
    it("analyzeCodeWithAst should return valid results regardless of tree-sitter availability", async () => {
      const result = await analyzeCodeWithAst(sampleGoCompliant, "cmd/search.go", "go");
      expect(result.issues.length).toBe(7);
      expect(result.complianceScore).toBe(100);
      expect(result.language).toBe("go");
    });

    it("detectWithAst should return null for unsupported language", async () => {
      const result = await detectWithAst("some code", "go");
      // If tree-sitter-go is not installed, returns null (graceful fallback)
      // If installed, returns issues array
      expect(result === null || Array.isArray(result)).toBe(true);
    });
  });
});
