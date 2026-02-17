import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";
import { analyze } from "./core/analyzer.js";
import { generateFix, generateFixWithLlm } from "./core/generator.js";
import { verify } from "./core/verifier.js";
import { analyzeDirectory } from "./core/batch.js";
import { readFile, writeFile } from "fs/promises";
import { detectFileInfo } from "./core/detector.js";

const server = new McpServer({
  name: "anel-copilot",
  version: "1.0.0",
});

server.registerTool(
  "anel_analyze",
  {
    title: "ANEL Analyze",
    description: "Analyze code for ANEL protocol compliance",
    inputSchema: {
      filePath: z.string().describe("Path to code file to analyze"),
    },
  },
  async ({ filePath }) => {
    const result = await analyze(filePath);
    return {
      content: [{ type: "text" as const, text: JSON.stringify(result, null, 2) }],
    };
  }
);

server.registerTool(
  "anel_fix",
  {
    title: "ANEL Fix",
    description: "Automatically fix code to comply with ANEL protocol",
    inputSchema: {
      filePath: z.string().describe("Path to code file to fix"),
      rules: z
        .array(z.string())
        .optional()
        .describe("Specific rules to apply (default: all)"),
      dryRun: z.boolean().optional().describe("Preview changes without writing to file"),
      mode: z.enum(["template", "llm"]).optional().describe("Fix mode: template (default) or llm (requires ANTHROPIC_API_KEY)"),
    },
  },
  async ({ filePath, rules, dryRun, mode }) => {
    const code = await readFile(filePath, "utf-8");
    const fileInfo = detectFileInfo(filePath);

    const modified = mode === "llm"
      ? await generateFixWithLlm(code, fileInfo.language, filePath, fileInfo.framework, "llm")
      : generateFix(code, fileInfo.language, fileInfo.framework);

    if (dryRun) {
      return {
        content: [
          {
            type: "text" as const,
            text: JSON.stringify({ success: true, file: filePath, dryRun: true, content: modified }, null, 2),
          },
        ],
      };
    }

    await writeFile(filePath, modified);

    return {
      content: [
        {
          type: "text" as const,
          text: JSON.stringify({ success: true, file: filePath }, null, 2),
        },
      ],
    };
  }
);

server.registerTool(
  "anel_analyze_dir",
  {
    title: "ANEL Analyze Directory",
    description: "Batch analyze all code files in a directory for ANEL compliance",
    inputSchema: {
      dirPath: z.string().describe("Path to directory to analyze"),
      recursive: z.boolean().optional().describe("Recursively scan subdirectories (default: true)"),
      extensions: z.array(z.string()).optional().describe("File extensions to include (default: .go,.rs,.py,.ts,.js)"),
    },
  },
  async ({ dirPath, recursive, extensions }) => {
    const result = await analyzeDirectory(dirPath, { recursive, extensions });
    return {
      content: [{ type: "text" as const, text: JSON.stringify(result, null, 2) }],
    };
  }
);

server.registerTool(
  "anel_verify",
  {
    title: "ANEL Verify",
    description: "Verify ANEL protocol implementation at runtime",
    inputSchema: {
      binaryPath: z.string().describe("Path to compiled binary"),
      command: z.string().describe("Command to test"),
    },
  },
  async ({ binaryPath, command }) => {
    const result = await verify(binaryPath, command);
    return {
      content: [{ type: "text" as const, text: JSON.stringify(result, null, 2) }],
    };
  }
);

server.registerTool(
  "anel_explain",
  {
    title: "ANEL Explain",
    description: "Explain ANEL protocol requirements",
    inputSchema: {
      question: z.string().describe("Question about ANEL protocol"),
    },
  },
  async ({ question }) => {
    const explanations: Record<string, string> = {
      "emit-spec":
        "The --emit-spec flag outputs a JSON schema describing the command's parameters, types, and constraints. This allows AI agents to discover and understand CLI capabilities programmatically.",
      "dry-run":
        "The --dry-run flag validates all parameters and preconditions without executing the actual operation. It outputs a validation result to stderr in JSON format.",
      "error-format":
        "ANEL errors follow RFC 7807 with extensions: error_code, message, severity, recovery_hints[]. Recovery hints provide machine-readable remediation steps.",
      "ndjson-output":
        "All stdout output must be NDJSON (newline-delimited JSON). Each line is a self-contained JSON object, enabling streaming and piping.",
      "trace-id":
        "Support the AGENT_TRACE_ID environment variable for distributed tracing. Include it in all error outputs and logs for request correlation.",
    };

    const key = question.toLowerCase().replace(/[^a-z-]/g, "");
    const answer =
      explanations[key] ||
      `ANEL (Agent-Native Execution Layer) is a protocol that standardizes how AI agents interact with CLI tools. Key requirements: --emit-spec, --dry-run, NDJSON output, structured errors with recovery hints, and AGENT_TRACE_ID support.`;

    return {
      content: [{ type: "text" as const, text: answer }],
    };
  }
);

const transport = new StdioServerTransport();
await server.connect(transport);
