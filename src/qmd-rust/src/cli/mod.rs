use clap::{Args, Parser, Subcommand};

/// QMD - AI-powered search with hybrid BM25 and vector search
#[derive(Parser, Debug)]
#[command(name = "qmd")]
#[command(author = "QMD Team")]
#[command(version = "0.1.0")]
#[command(about = "AI-powered search with hybrid BM25 and vector search", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

// CLI submodule declarations
pub mod collection;
pub mod context;
pub mod get;
pub mod ls;
pub mod multi_get;
pub mod search;
pub mod vsearch;
pub mod query;
pub mod embed;
pub mod update;
pub mod status;
pub mod cleanup;
pub mod agent;
pub mod plugin;

/// Output format options
#[derive(Debug, Clone, Args)]
pub struct LsArgs {
    /// Optional path: collection or collection/path
    /// Supports qmd:// prefix: qmd://collection/path
    pub path: Option<String>,
    /// Output format: cli, json, ndjson
    #[arg(long, default_value = "cli")]
    pub format: String,
    /// Emit ANEL specification (JSON Schema) instead of executing
    #[arg(long)]
    pub emit_spec: bool,
    /// Dry-run mode: validate parameters without executing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Args)]
pub struct FormatOptions {
    /// Output format: cli, json, ndjson, md, csv, files, xml
    #[arg(long, default_value = "cli")]
    pub format: String,
    /// Number of results to return
    #[arg(short, long, default_value = "20")]
    pub limit: usize,
    /// Minimum score threshold
    #[arg(long, default_value = "0.0")]
    pub min_score: f32,
    /// Collection to search
    #[arg(short, long)]
    pub collection: Option<String>,
    /// Search all collections
    #[arg(long)]
    pub all: bool,
    /// BM25 backend: sqlite_fts5, lancedb
    #[arg(long, default_value = "sqlite_fts5")]
    pub fts_backend: String,
    /// Vector backend: qmd_builtin, lancedb
    #[arg(long, default_value = "qmd_builtin")]
    pub vector_backend: String,
    /// Emit ANEL specification (JSON Schema) instead of executing
    #[arg(long)]
    pub emit_spec: bool,
    /// Dry-run mode: validate parameters without executing
    #[arg(long)]
    pub dry_run: bool,
}

/// Collection management commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Collection management
    Collection(CollectionArgs),

    /// List collections or files in a collection
    Ls(LsArgs),

    /// Context management
    Context(ContextArgs),

    /// Get document content
    Get(GetArgs),

    /// Multi-get documents by pattern
    MultiGet(MultiGetArgs),

    /// BM25 full-text search
    Search(SearchArgs),

    /// Vector semantic search
    Vsearch(VsearchArgs),

    /// Hybrid search with reranking
    Query(QueryArgs),

    /// Generate/update embeddings
    Embed(EmbedArgs),

    /// Update index
    Update(UpdateArgs),

    /// Show index status
    Status(StatusArgs),

    /// Cleanup stale entries
    Cleanup(CleanupArgs),

    /// Run as MCP server
    Mcp(McpArgs),

    /// Run as standalone HTTP server
    Server(ServerArgs),

    /// Run in agent mode
    Agent(AgentArgs),

    /// Plugin management
    Plugin(PluginArgs),
}

#[derive(Args, Debug)]
pub struct CollectionArgs {
    #[command(subcommand)]
    pub command: CollectionCommands,
    /// Output format: cli, json, ndjson
    #[arg(long, default_value = "cli")]
    pub format: String,
    /// Emit ANEL specification (JSON Schema) instead of executing
    #[arg(long)]
    pub emit_spec: bool,
    /// Dry-run mode: validate parameters without executing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Subcommand, Debug)]
pub enum CollectionCommands {
    /// Add a collection
    Add(CollectionAddArgs),
    /// List collections
    List,
    /// Remove a collection
    Remove(CollectionRemoveArgs),
    /// Rename a collection
    Rename(CollectionRenameArgs),
}

#[derive(Args, Debug)]
pub struct CollectionAddArgs {
    /// Collection path
    pub path: String,
    /// Collection name
    #[arg(short, long)]
    pub name: Option<String>,
    /// File pattern (glob)
    #[arg(long, default_value = "**/*")]
    pub mask: String,
    /// Collection description
    #[arg(short, long)]
    pub description: Option<String>,
}

#[derive(Args, Debug)]
pub struct CollectionRemoveArgs {
    pub name: String,
}

#[derive(Args, Debug)]
pub struct CollectionRenameArgs {
    pub old_name: String,
    pub new_name: String,
}

#[derive(Args, Debug)]
pub struct ContextArgs {
    #[command(subcommand)]
    pub command: ContextCommands,
    /// Output format: cli, json, ndjson
    #[arg(long, default_value = "cli")]
    pub format: String,
    /// Emit ANEL specification (JSON Schema) instead of executing
    #[arg(long)]
    pub emit_spec: bool,
    /// Dry-run mode: validate parameters without executing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Subcommand, Debug)]
pub enum ContextCommands {
    /// Add context
    Add(ContextAddArgs),
    /// List contexts
    List,
    /// Check for missing contexts
    Check,
    /// Remove context
    Rm(ContextRemoveArgs),
}

#[derive(Args, Debug)]
pub struct ContextAddArgs {
    /// Path to add
    pub path: Option<String>,
    /// Description
    pub description: String,
}

#[derive(Args, Debug)]
pub struct ContextRemoveArgs {
    pub path: String,
}

#[derive(Args, Debug)]
pub struct GetArgs {
    /// File path (with optional :line suffix)
    pub file: String,
    /// Number of lines
    #[arg(short, long, default_value = "50")]
    pub limit: usize,
    /// Start line
    #[arg(long, default_value = "0")]
    pub from: usize,
    /// Full content (no limit)
    #[arg(long)]
    pub full: bool,
    /// Output format: cli, json, ndjson
    #[arg(long, default_value = "cli")]
    pub format: String,
    /// Emit ANEL specification (JSON Schema) instead of executing
    #[arg(long)]
    pub emit_spec: bool,
    /// Dry-run mode: validate parameters without executing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct MultiGetArgs {
    /// File pattern
    pub pattern: String,
    /// Number of lines per file
    #[arg(short, long, default_value = "50")]
    pub limit: usize,
    /// Maximum bytes per file
    #[arg(long)]
    pub max_bytes: Option<usize>,
    /// Output format: cli, json, ndjson
    #[arg(long, default_value = "cli")]
    pub format: String,
    /// Emit ANEL specification (JSON Schema) instead of executing
    #[arg(long)]
    pub emit_spec: bool,
    /// Dry-run mode: validate parameters without executing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search query
    pub query: String,
    #[command(flatten)]
    pub format: FormatOptions,
}

#[derive(Args, Debug)]
pub struct VsearchArgs {
    /// Search query
    pub query: String,
    #[command(flatten)]
    pub format: FormatOptions,
}

#[derive(Args, Debug)]
pub struct QueryArgs {
    /// Search query
    pub query: String,
    #[command(flatten)]
    pub format: FormatOptions,
}

#[derive(Args, Debug)]
pub struct EmbedArgs {
    /// Force regeneration
    #[arg(short, long)]
    pub force: bool,
    /// Collection to embed
    #[arg(short, long)]
    pub collection: Option<String>,
    /// Output format: cli, json, ndjson
    #[arg(long, default_value = "cli")]
    pub format: String,
    /// Emit ANEL specification (JSON Schema) instead of executing
    #[arg(long)]
    pub emit_spec: bool,
    /// Dry-run mode: validate parameters without executing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// Pull remote changes first
    #[arg(long)]
    pub pull: bool,
    /// Collection to update (default: all)
    #[arg(short, long)]
    pub collection: Option<String>,
    /// Output format: cli, json, ndjson
    #[arg(long, default_value = "cli")]
    pub format: String,
    /// Emit ANEL specification (JSON Schema) instead of executing
    #[arg(long)]
    pub emit_spec: bool,
    /// Dry-run mode: validate parameters without executing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Show detailed status
    #[arg(long)]
    pub verbose: bool,
    /// Collection to show status for
    #[arg(short, long)]
    pub collection: Option<String>,
    /// Output format: cli, json, ndjson
    #[arg(long, default_value = "cli")]
    pub format: String,
    /// Emit ANEL specification (JSON Schema) instead of executing
    #[arg(long)]
    pub emit_spec: bool,
    /// Dry-run mode: validate parameters without executing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct CleanupArgs {
    /// Dry run only
    #[arg(long)]
    pub dry_run: bool,
    /// Remove entries older than N days
    #[arg(long, default_value = "30")]
    pub older_than: u32,
    /// Collection to clean up
    #[arg(short, long)]
    pub collection: Option<String>,
    /// Output format: cli, json, ndjson
    #[arg(long, default_value = "cli")]
    pub format: String,
    /// Emit ANEL specification (JSON Schema) instead of executing
    #[arg(long)]
    pub emit_spec: bool,
}

#[derive(Args, Debug)]
pub struct McpArgs {
    /// Transport: stdio, sse
    #[arg(long, default_value = "stdio")]
    pub transport: String,
    /// Port for SSE
    #[arg(long, default_value = "8080")]
    pub port: u16,
    /// Output format: cli, json, ndjson
    #[arg(long, default_value = "cli")]
    pub format: String,
    /// Emit ANEL specification (JSON Schema) instead of executing
    #[arg(long)]
    pub emit_spec: bool,
    /// Dry-run mode: validate parameters without executing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct ServerArgs {
    /// Host to bind to
    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,
    /// Port to listen on
    #[arg(long, default_value = "8080")]
    pub port: u16,
    /// Number of worker threads
    #[arg(long, default_value = "4")]
    pub workers: usize,
    /// Enable API key authentication
    #[arg(long)]
    pub auth: bool,
    /// Comma-separated list of API keys
    #[arg(long)]
    pub api_keys: Option<String>,
    /// Comma-separated list of whitelisted IPs (skip auth)
    #[arg(long)]
    pub whitelist_ips: Option<String>,
}

#[derive(Args, Debug)]
pub struct AgentArgs {
    /// Interactive mode
    #[arg(long)]
    pub interactive: bool,
    /// Also run MCP server
    #[arg(long)]
    pub mcp: bool,
    /// MCP transport
    #[arg(long, default_value = "stdio")]
    pub transport: String,
    /// Query to process (non-interactive mode)
    pub query: Option<String>,
    /// Output format: cli, json, ndjson
    #[arg(long, default_value = "cli")]
    pub format: String,
    /// Emit ANEL specification (JSON Schema) instead of executing
    #[arg(long)]
    pub emit_spec: bool,
    /// Dry-run mode: validate parameters without executing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct PluginArgs {
    #[command(subcommand)]
    pub command: PluginCommands,
}

#[derive(Subcommand, Debug)]
pub enum PluginCommands {
    /// List available plugins
    List,
    /// Install a plugin
    Install {
        /// Path to plugin .wasm file
        path: String,
        /// Optional plugin name (defaults to filename)
        name: Option<String>,
    },
    /// Remove an installed plugin
    Remove {
        /// Plugin name to remove
        name: String,
    },
    /// Show plugin information
    Info {
        /// Plugin name
        name: String,
    },
    /// Show plugins directory
    Dir,
}
