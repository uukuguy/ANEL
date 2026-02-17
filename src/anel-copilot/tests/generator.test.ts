import { describe, it, expect } from "vitest";
import { generateFix } from "../src/core/generator.js";
import { sampleGoNonCompliant } from "./fixtures/sample-go.js";
import { analyzeCode } from "../src/core/analyzer.js";

describe("Generator", () => {
  describe("Go cobra fixes", () => {
    it("should add emit-spec and dry-run flags to init()", () => {
      const code = `
package cmd

func init() {
    searchCmd.Flags().String("query", "", "Search query")
}

func handleSearch(cmd *cobra.Command, args []string) error {
    return nil
}
`;
      const fixed = generateFix(code, "go", "cobra");
      expect(fixed).toContain("emit-spec");
      expect(fixed).toContain("dry-run");
      expect(fixed).toContain("output-format");
    });

    it("should add emit-spec handler to function body", () => {
      const code = `
package cmd

func init() {
    searchCmd.Flags().String("query", "", "Search query")
}

func handleSearch(cmd *cobra.Command, args []string) error {
    return nil
}
`;
      const fixed = generateFix(code, "go", "cobra");
      expect(fixed).toContain("emitSpec");
      expect(fixed).toContain("dryRun");
    });

    it("should not double-add flags if already present", () => {
      const code = `
func init() {
    cmd.Flags().Bool("emit-spec", false, "spec")
}
func handleSearch(cmd *cobra.Command, args []string) error {
    emitSpec, _ := cmd.Flags().GetBool("emit-spec")
    return nil
}
`;
      const fixed = generateFix(code, "go", "cobra");
      // Should not add duplicate
      const emitSpecCount = (fixed.match(/emit-spec/g) || []).length;
      expect(emitSpecCount).toBe(2); // one in init, one in handler
    });
  });

  describe("Rust clap fixes", () => {
    it("should add ANEL fields to struct", () => {
      const code = `
#[derive(Parser)]
struct Args {
    query: String,
}
`;
      const fixed = generateFix(code, "rust", "clap");
      expect(fixed).toContain("emit_spec");
      expect(fixed).toContain("dry_run");
    });
  });

  describe("Python click fixes", () => {
    it("should add ANEL options to click command", () => {
      const code = `
@click.command()
def search(query):
    print(query)
`;
      const fixed = generateFix(code, "python", "click");
      expect(fixed).toContain("--emit-spec");
      expect(fixed).toContain("--dry-run");
    });
  });

  describe("TypeScript commander fixes", () => {
    it("should add ANEL options to commander command", () => {
      const code = `
program
  .command("search")
  .action(() => {});
`;
      const fixed = generateFix(code, "typescript", "commander");
      expect(fixed).toContain("--emit-spec");
      expect(fixed).toContain("--dry-run");
    });
  });

  describe("Integration: fix improves compliance", () => {
    it("should improve compliance score after fix", () => {
      const before = analyzeCode(sampleGoNonCompliant, "cmd/search.go", "go");
      const fixed = generateFix(sampleGoNonCompliant, "go", "cobra");
      const after = analyzeCode(fixed, "cmd/search.go", "go");

      expect(after.complianceScore).toBeGreaterThan(before.complianceScore);
    });
  });
});
