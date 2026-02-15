/**
 * ANEL spec generators for QMD commands.
 */

import { ANEL_VERSION, type ErrorCode } from "./index.js";

export type AnelSpec = {
  version: string;
  command: string;
  input_schema: Record<string, unknown>;
  output_schema: Record<string, unknown>;
  error_codes: ErrorCode[];
};

export function searchSpec(): AnelSpec {
  return {
    version: ANEL_VERSION,
    command: "search",
    input_schema: {
      type: "object",
      properties: {
        query: { type: "string" },
        limit: { type: "integer", default: 20 },
        min_score: { type: "number", default: 0.0 },
        collection: { type: "string" },
        all: { type: "boolean", default: false },
      },
      required: ["query"],
    },
    output_schema: {
      type: "object",
      properties: {
        results: {
          type: "array",
          items: {
            type: "object",
            properties: {
              docid: { type: "string" },
              path: { type: "string" },
              score: { type: "number" },
              lines: { type: "integer" },
            },
          },
        },
        total: { type: "integer" },
      },
    },
    error_codes: ["SEARCH_FAILED", "INDEX_NOT_READY", "QUERY_PARSE_ERROR"],
  };
}

export function vsearchSpec(): AnelSpec {
  return {
    version: ANEL_VERSION,
    command: "vsearch",
    input_schema: {
      type: "object",
      properties: {
        query: { type: "string" },
        limit: { type: "integer", default: 20 },
        collection: { type: "string" },
        all: { type: "boolean", default: false },
      },
      required: ["query"],
    },
    output_schema: {
      type: "object",
      properties: {
        results: {
          type: "array",
          items: {
            type: "object",
            properties: {
              docid: { type: "string" },
              path: { type: "string" },
              score: { type: "number" },
              lines: { type: "integer" },
            },
          },
        },
        total: { type: "integer" },
      },
    },
    error_codes: ["SEARCH_FAILED", "INDEX_NOT_READY", "EMBEDDING_FAILED", "MODEL_NOT_FOUND"],
  };
}

export function querySpec(): AnelSpec {
  return {
    version: ANEL_VERSION,
    command: "query",
    input_schema: {
      type: "object",
      properties: {
        query: { type: "string" },
        limit: { type: "integer", default: 20 },
        collection: { type: "string" },
        all: { type: "boolean", default: false },
      },
      required: ["query"],
    },
    output_schema: {
      type: "object",
      properties: {
        results: {
          type: "array",
          items: {
            type: "object",
            properties: {
              docid: { type: "string" },
              path: { type: "string" },
              score: { type: "number" },
              lines: { type: "integer" },
              reranked: { type: "boolean" },
            },
          },
        },
        total: { type: "integer" },
      },
    },
    error_codes: [
      "SEARCH_FAILED", "INDEX_NOT_READY", "EMBEDDING_FAILED",
      "MODEL_NOT_FOUND", "QUERY_PARSE_ERROR",
    ],
  };
}

export function getSpec(): AnelSpec {
  return {
    version: ANEL_VERSION,
    command: "get",
    input_schema: {
      type: "object",
      properties: {
        file: { type: "string", description: "File path with optional :line suffix" },
        limit: { type: "integer", default: 50 },
        from: { type: "integer", default: 0 },
        full: { type: "boolean", default: false },
      },
      required: ["file"],
    },
    output_schema: {
      type: "object",
      properties: {
        path: { type: "string" },
        lines: { type: "array", items: { type: "string" } },
        total_lines: { type: "integer" },
      },
    },
    error_codes: ["NOT_FOUND", "INVALID_INPUT"],
  };
}

export function multiGetSpec(): AnelSpec {
  return {
    version: ANEL_VERSION,
    command: "multi_get",
    input_schema: {
      type: "object",
      properties: {
        pattern: { type: "string", description: "Glob pattern for files" },
        limit: { type: "integer", default: 50 },
        max_bytes: { type: "integer" },
      },
      required: ["pattern"],
    },
    output_schema: {
      type: "object",
      properties: {
        files: {
          type: "array",
          items: {
            type: "object",
            properties: {
              path: { type: "string" },
              lines: { type: "array", items: { type: "string" } },
              truncated: { type: "boolean" },
            },
          },
        },
        total_files: { type: "integer" },
        errors: { type: "integer" },
      },
    },
    error_codes: ["INVALID_INPUT", "NOT_FOUND"],
  };
}

export function collectionSpec(): AnelSpec {
  return {
    version: ANEL_VERSION,
    command: "collection",
    input_schema: {
      type: "object",
      properties: {
        action: { type: "string", enum: ["add", "list", "remove", "rename"] },
        name: { type: "string" },
        path: { type: "string" },
        mask: { type: "string", default: "**/*" },
        description: { type: "string" },
        new_name: { type: "string" },
      },
    },
    output_schema: {
      type: "object",
      properties: {
        collections: {
          type: "array",
          items: {
            type: "object",
            properties: {
              name: { type: "string" },
              path: { type: "string" },
              pattern: { type: "string" },
              description: { type: "string" },
            },
          },
        },
        action: { type: "string" },
      },
    },
    error_codes: ["COLLECTION_NOT_FOUND", "COLLECTION_EXISTS", "INVALID_INPUT"],
  };
}

export function contextSpec(): AnelSpec {
  return {
    version: ANEL_VERSION,
    command: "context",
    input_schema: {
      type: "object",
      properties: {
        action: { type: "string", enum: ["add", "list", "rm"] },
        path: { type: "string" },
        description: { type: "string" },
      },
      required: ["action"],
    },
    output_schema: {
      type: "object",
      properties: {
        contexts: {
          type: "array",
          items: {
            type: "object",
            properties: {
              path: { type: "string" },
              description: { type: "string" },
            },
          },
        },
        action: { type: "string" },
      },
    },
    error_codes: ["NOT_FOUND", "INVALID_INPUT"],
  };
}

export function embedSpec(): AnelSpec {
  return {
    version: ANEL_VERSION,
    command: "embed",
    input_schema: {
      type: "object",
      properties: {
        force: { type: "boolean", default: false },
        collection: { type: "string" },
      },
    },
    output_schema: {
      type: "object",
      properties: {
        collections_processed: { type: "integer" },
        documents_embedded: { type: "integer" },
        chunks_embedded: { type: "integer" },
        model: { type: "string" },
      },
    },
    error_codes: ["EMBEDDING_FAILED", "MODEL_NOT_FOUND", "MODEL_LOAD_FAILED", "COLLECTION_NOT_FOUND"],
  };
}

export function updateSpec(): AnelSpec {
  return {
    version: ANEL_VERSION,
    command: "update",
    input_schema: {
      type: "object",
      properties: {
        pull: { type: "boolean", default: false },
        collection: { type: "string" },
      },
    },
    output_schema: {
      type: "object",
      properties: {
        collections_updated: { type: "integer" },
        documents_indexed: { type: "integer" },
        documents_removed: { type: "integer" },
      },
    },
    error_codes: ["INDEX_NOT_READY", "COLLECTION_NOT_FOUND", "STORAGE_ERROR"],
  };
}

export function statusSpec(): AnelSpec {
  return {
    version: ANEL_VERSION,
    command: "status",
    input_schema: {
      type: "object",
      properties: {
        verbose: { type: "boolean", default: false },
        collection: { type: "string" },
      },
    },
    output_schema: {
      type: "object",
      properties: {
        collections: {
          type: "array",
          items: {
            type: "object",
            properties: {
              name: { type: "string" },
              documents: { type: "integer" },
              chunks: { type: "integer" },
              embeddings: { type: "integer" },
              last_updated: { type: "string" },
            },
          },
        },
      },
    },
    error_codes: ["COLLECTION_NOT_FOUND", "INDEX_NOT_READY"],
  };
}

export function cleanupSpec(): AnelSpec {
  return {
    version: ANEL_VERSION,
    command: "cleanup",
    input_schema: {
      type: "object",
      properties: {
        dry_run: { type: "boolean", default: false },
        older_than: { type: "integer", default: 30 },
        collection: { type: "string" },
      },
    },
    output_schema: {
      type: "object",
      properties: {
        entries_removed: { type: "integer" },
        dry_run: { type: "boolean" },
      },
    },
    error_codes: ["COLLECTION_NOT_FOUND", "STORAGE_ERROR"],
  };
}

export function mcpSpec(): AnelSpec {
  return {
    version: ANEL_VERSION,
    command: "mcp",
    input_schema: {
      type: "object",
      properties: {
        transport: { type: "string", default: "stdio" },
        port: { type: "integer", default: 8080 },
      },
    },
    output_schema: {
      type: "object",
      properties: {
        status: { type: "string" },
        transport: { type: "string" },
        port: { type: "integer" },
      },
    },
    error_codes: ["CONFIG_ERROR", "BACKEND_UNAVAILABLE"],
  };
}

// Lookup table
const SPEC_GETTERS: Record<string, () => AnelSpec> = {
  search: searchSpec,
  vsearch: vsearchSpec,
  query: querySpec,
  get: getSpec,
  "multi-get": multiGetSpec,
  multi_get: multiGetSpec,
  collection: collectionSpec,
  context: contextSpec,
  embed: embedSpec,
  update: updateSpec,
  status: statusSpec,
  cleanup: cleanupSpec,
  mcp: mcpSpec,
};

export function getSpecForCommand(command: string): AnelSpec | null {
  const getter = SPEC_GETTERS[command];
  return getter ? getter() : null;
}
