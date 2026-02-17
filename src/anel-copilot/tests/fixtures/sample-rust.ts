// Non-compliant Rust code
export const sampleRustNonCompliant = `
use clap::Parser;

#[derive(Parser)]
#[command(name = "search")]
struct SearchArgs {
    query: String,
}

fn main() {
    let args = SearchArgs::parse();
    println!("Searching for: {}", args.query);
}
`;

// Compliant Rust code
export const sampleRustCompliant = `
use clap::Parser;
use serde_json::to_string;

#[derive(Parser)]
#[command(name = "search")]
struct SearchArgs {
    query: String,
    #[arg(long, help = "Output ANEL specification")]
    emit_spec: bool,
    #[arg(long, help = "Validate without executing")]
    dry_run: bool,
    #[arg(long, default_value = "ndjson", help = "Output format")]
    output_format: String,
}

fn main() {
    let trace_id = std::env::var("AGENT_TRACE_ID").unwrap_or_default();
    let args = SearchArgs::parse();

    if args.emit_spec {
        let spec = anel::AnelSpec::new("search");
        println!("{}", serde_json::to_string(&spec).unwrap());
        return;
    }

    if args.dry_run {
        eprintln!(r#"{{"dry_run": true, "command": "search", "trace_id": "{}"}}"#, trace_id);
        return;
    }

    match do_search(&args.query) {
        Ok(results) => {
            for r in results {
                println!("{}", serde_json::to_string(&r).unwrap());
            }
        }
        Err(e) => {
            let err = anel::AnelError::new("E_SEARCH_FAILED", &e.to_string())
                .with_recovery_hint("REINDEX", "Run update to refresh index");
            err.emit_stderr();
        }
    }
}
`;
