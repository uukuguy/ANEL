import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { analyzeDirectory } from "../src/core/batch.js";
import { mkdtemp, writeFile, mkdir, rm } from "fs/promises";
import { join } from "path";
import { tmpdir } from "os";
import { sampleGoCompliant, sampleGoNonCompliant } from "./fixtures/sample-go.js";
import { sampleRustCompliant } from "./fixtures/sample-rust.js";

describe("Batch Analysis", () => {
  let tempDir: string;

  beforeAll(async () => {
    tempDir = await mkdtemp(join(tmpdir(), "anel-batch-"));
    await mkdir(join(tempDir, "sub"), { recursive: true });
    await writeFile(join(tempDir, "compliant.go"), sampleGoCompliant);
    await writeFile(join(tempDir, "noncompliant.go"), sampleGoNonCompliant);
    await writeFile(join(tempDir, "sub", "compliant.rs"), sampleRustCompliant);
    await writeFile(join(tempDir, "readme.md"), "# Not a code file");
  });

  afterAll(async () => {
    await rm(tempDir, { recursive: true, force: true });
  });

  it("should analyze all matching files recursively", async () => {
    const result = await analyzeDirectory(tempDir);
    expect(result.files.length).toBe(3);
    expect(result.summary.total).toBe(3);
  });

  it("should calculate correct summary stats", async () => {
    const result = await analyzeDirectory(tempDir);
    expect(result.summary.compliant).toBeGreaterThanOrEqual(1);
    expect(result.summary.nonCompliant).toBeGreaterThanOrEqual(1);
    expect(result.summary.averageScore).toBeGreaterThan(0);
    expect(result.summary.averageScore).toBeLessThan(100);
  });

  it("should skip non-code files", async () => {
    const result = await analyzeDirectory(tempDir);
    const files = result.files.map((f) => f.file);
    expect(files.every((f) => !f.endsWith(".md"))).toBe(true);
  });

  it("should respect non-recursive option", async () => {
    const result = await analyzeDirectory(tempDir, { recursive: false });
    expect(result.files.length).toBe(2); // only top-level .go files
  });

  it("should filter by custom extensions", async () => {
    const result = await analyzeDirectory(tempDir, { extensions: [".rs"] });
    expect(result.files.length).toBe(1);
    expect(result.files[0].language).toBe("rust");
  });
});
