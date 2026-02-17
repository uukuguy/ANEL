import { describe, it, expect } from "vitest";
import { analyzeCode } from "../src/core/analyzer.js";
import {
  sampleGoNonCompliant,
  sampleGoCompliant,
  sampleGoPartial,
} from "./fixtures/sample-go.js";
import {
  sampleRustNonCompliant,
  sampleRustCompliant,
} from "./fixtures/sample-rust.js";

describe("AnelAnalyzer", () => {
  describe("Go non-compliant code", () => {
    const result = analyzeCode(sampleGoNonCompliant, "cmd/search.go", "go");

    it("should detect missing emit-spec flag", () => {
      const issue = result.issues.find((i) => i.rule === "emit-spec");
      expect(issue?.status).toBe("missing");
    });

    it("should detect missing dry-run flag", () => {
      const issue = result.issues.find((i) => i.rule === "dry-run");
      expect(issue?.status).toBe("missing");
    });

    it("should detect non-compliant error format", () => {
      const issue = result.issues.find((i) => i.rule === "error-format");
      expect(issue?.status).toBe("non-compliant");
    });

    it("should detect missing NDJSON output", () => {
      const issue = result.issues.find((i) => i.rule === "ndjson-output");
      expect(issue?.status).toBe("missing");
    });

    it("should detect missing trace-id", () => {
      const issue = result.issues.find((i) => i.rule === "trace-id");
      expect(issue?.status).toBe("missing");
    });

    it("should have low compliance score", () => {
      expect(result.complianceScore).toBeLessThanOrEqual(5);
    });

    it("should detect cobra framework", () => {
      expect(result.framework).toBe("cobra");
    });
  });

  describe("Go compliant code", () => {
    const result = analyzeCode(sampleGoCompliant, "cmd/search.go", "go");

    it("should detect present emit-spec flag", () => {
      const issue = result.issues.find((i) => i.rule === "emit-spec");
      expect(issue?.status).toBe("present");
    });

    it("should detect present dry-run flag", () => {
      const issue = result.issues.find((i) => i.rule === "dry-run");
      expect(issue?.status).toBe("present");
    });

    it("should detect present error format", () => {
      const issue = result.issues.find((i) => i.rule === "error-format");
      expect(issue?.status).toBe("present");
    });

    it("should detect present NDJSON output", () => {
      const issue = result.issues.find((i) => i.rule === "ndjson-output");
      expect(issue?.status).toBe("present");
    });

    it("should detect present trace-id", () => {
      const issue = result.issues.find((i) => i.rule === "trace-id");
      expect(issue?.status).toBe("present");
    });

    it("should have perfect compliance score", () => {
      expect(result.complianceScore).toBe(100);
    });
  });

  describe("Go partially compliant code", () => {
    const result = analyzeCode(sampleGoPartial, "cmd/search.go", "go");

    it("should detect present emit-spec", () => {
      const issue = result.issues.find((i) => i.rule === "emit-spec");
      expect(issue?.status).toBe("present");
    });

    it("should detect present dry-run", () => {
      const issue = result.issues.find((i) => i.rule === "dry-run");
      expect(issue?.status).toBe("present");
    });

    it("should detect non-compliant error format", () => {
      const issue = result.issues.find((i) => i.rule === "error-format");
      expect(issue?.status).toBe("non-compliant");
    });

    it("should have intermediate compliance score", () => {
      expect(result.complianceScore).toBeGreaterThan(0);
      expect(result.complianceScore).toBeLessThan(100);
    });
  });

  describe("Rust non-compliant code", () => {
    const result = analyzeCode(sampleRustNonCompliant, "src/main.rs", "rust");

    it("should detect missing emit-spec", () => {
      const issue = result.issues.find((i) => i.rule === "emit-spec");
      expect(issue?.status).toBe("missing");
    });

    it("should detect clap framework", () => {
      expect(result.framework).toBe("clap");
    });
  });

  describe("Rust compliant code", () => {
    const result = analyzeCode(sampleRustCompliant, "src/main.rs", "rust");

    it("should detect present emit-spec", () => {
      const issue = result.issues.find((i) => i.rule === "emit-spec");
      expect(issue?.status).toBe("present");
    });

    it("should have perfect compliance score", () => {
      expect(result.complianceScore).toBe(100);
    });
  });
});
