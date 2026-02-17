import { describe, it, expect } from "vitest";
import { detectFileInfo, detectFrameworkFromCode } from "../src/core/detector.js";

describe("Detector", () => {
  describe("detectFileInfo", () => {
    it("should detect Go files", () => {
      const info = detectFileInfo("cmd/search.go");
      expect(info.language).toBe("go");
    });

    it("should detect Rust files", () => {
      const info = detectFileInfo("src/main.rs");
      expect(info.language).toBe("rust");
    });

    it("should detect Python files", () => {
      const info = detectFileInfo("cli/search.py");
      expect(info.language).toBe("python");
    });

    it("should detect TypeScript files", () => {
      const info = detectFileInfo("src/cli.ts");
      expect(info.language).toBe("typescript");
    });

    it("should throw for unsupported extensions", () => {
      expect(() => detectFileInfo("file.xyz")).toThrow("Unsupported");
    });
  });

  describe("detectFrameworkFromCode", () => {
    it("should detect cobra in Go code", () => {
      expect(detectFrameworkFromCode('import "github.com/spf13/cobra"', "go")).toBe("cobra");
    });

    it("should detect clap in Rust code", () => {
      expect(detectFrameworkFromCode("use clap::Parser;", "rust")).toBe("clap");
    });

    it("should detect click in Python code", () => {
      expect(detectFrameworkFromCode("import click", "python")).toBe("click");
    });

    it("should detect typer in Python code", () => {
      expect(detectFrameworkFromCode("import typer", "python")).toBe("typer");
    });

    it("should detect commander in TS code", () => {
      expect(detectFrameworkFromCode('import { Command } from "commander"', "typescript")).toBe("commander");
    });

    it("should return undefined for unknown framework", () => {
      expect(detectFrameworkFromCode("print('hello')", "python")).toBeUndefined();
    });
  });
});
