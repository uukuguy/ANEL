import { exec } from "child_process";
import { promisify } from "util";
import type { AnelVerifyResult } from "./types.js";

const execAsync = promisify(exec);

export async function verify(
  binaryPath: string,
  command: string
): Promise<AnelVerifyResult> {
  const details: string[] = [];

  // Test --emit-spec
  try {
    const { stdout } = await execAsync(`${binaryPath} ${command} --emit-spec`);
    const spec = JSON.parse(stdout.trim());
    if (spec.version || spec.command || spec.parameters) {
      details.push("[PASS] --emit-spec outputs valid JSON schema");
    } else {
      details.push("[WARN] --emit-spec outputs JSON but missing expected fields (version, command, parameters)");
    }
  } catch {
    details.push("[FAIL] --emit-spec not working or not returning valid JSON");
  }

  // Test --dry-run
  try {
    const { stderr, stdout } = await execAsync(
      `${binaryPath} ${command} "test-query" --dry-run`
    );
    const output = stderr || stdout;
    if (output.includes("dry_run") || output.includes("dry-run")) {
      details.push("[PASS] --dry-run outputs expected format");
    } else {
      details.push("[WARN] --dry-run runs but output format unclear");
    }
  } catch {
    details.push("[FAIL] --dry-run not working");
  }

  // Test error format (trigger with empty/invalid input)
  try {
    const { stderr } = await execAsync(`${binaryPath} ${command} "" 2>&1`);
    try {
      const error = JSON.parse(stderr.trim());
      if (error.error_code && error.recovery_hints) {
        details.push("[PASS] Error format includes error_code and recovery_hints");
      } else if (error.error_code) {
        details.push("[WARN] Error format has error_code but missing recovery_hints");
      } else {
        details.push("[WARN] Error output is JSON but missing ANEL fields");
      }
    } catch {
      details.push("[FAIL] Error output is not valid JSON");
    }
  } catch (e: unknown) {
    const err = e as { stderr?: string };
    if (err.stderr) {
      try {
        const error = JSON.parse(err.stderr.trim());
        if (error.error_code) {
          details.push("[PASS] Error format includes error_code");
        }
      } catch {
        details.push("[FAIL] Error output is not valid JSON");
      }
    } else {
      details.push("[FAIL] Could not trigger error output");
    }
  }

  // Test NDJSON output
  try {
    const { stdout } = await execAsync(`${binaryPath} ${command} "test-query"`);
    const lines = stdout.trim().split("\n").filter(Boolean);
    const allJson = lines.every((line) => {
      try {
        JSON.parse(line);
        return true;
      } catch {
        return false;
      }
    });
    if (allJson && lines.length > 0) {
      details.push("[PASS] Output is valid NDJSON");
    } else {
      details.push("[WARN] Output is not NDJSON format");
    }
  } catch {
    details.push("[FAIL] Could not verify NDJSON output");
  }

  const passCount = details.filter((d) => d.startsWith("[PASS]")).length;
  return {
    binary: binaryPath,
    command,
    passed: passCount >= 2,
    details,
  };
}
