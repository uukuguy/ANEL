#!/usr/bin/env node

import { analyze } from "./core/analyzer.js";
import { generateFix } from "./core/generator.js";
import { verify } from "./core/verifier.js";
import { detectFileInfo } from "./core/detector.js";
import { readFile, writeFile } from "fs/promises";

const [, , command, ...args] = process.argv;

async function main() {
  switch (command) {
    case "analyze": {
      if (!args[0]) {
        console.error("Usage: anel-copilot analyze <file>");
        process.exit(1);
      }
      const result = await analyze(args[0]);
      console.log(JSON.stringify(result, null, 2));
      break;
    }

    case "fix": {
      if (!args[0]) {
        console.error("Usage: anel-copilot fix <file>");
        process.exit(1);
      }
      const filePath = args[0];
      const code = await readFile(filePath, "utf-8");
      const fileInfo = detectFileInfo(filePath);
      const modified = generateFix(code, fileInfo.language, fileInfo.framework);

      if (args.includes("--dry-run")) {
        console.log(modified);
      } else {
        await writeFile(filePath, modified);
        console.log(`Fixed: ${filePath}`);
      }
      break;
    }

    case "verify": {
      if (!args[0] || !args[1]) {
        console.error("Usage: anel-copilot verify <binary> <command>");
        process.exit(1);
      }
      const result = await verify(args[0], args[1]);
      console.log(JSON.stringify(result, null, 2));
      break;
    }

    default:
      console.log(`anel-copilot v1.0.0 - ANEL Protocol Copilot

Usage:
  anel-copilot analyze <file>          Analyze code for ANEL compliance
  anel-copilot fix <file> [--dry-run]  Auto-fix code for ANEL compliance
  anel-copilot verify <binary> <cmd>   Verify runtime ANEL compliance`);
      break;
  }
}

main().catch((err) => {
  console.error(err.message);
  process.exit(1);
});
