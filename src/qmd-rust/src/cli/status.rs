use crate::anel::AnelSpec;
use crate::cli::StatusArgs;
use crate::store::Store;
use anyhow::Result;

/// Handle status command - show index status
pub fn handle(
    cmd: &StatusArgs,
    store: &Store,
) -> Result<()> {
    // Handle --emit-spec: output ANEL specification and exit
    if cmd.emit_spec {
        let spec = AnelSpec::status();
        println!("{}", serde_json::to_string_pretty(&spec)?);
        return Ok(());
    }

    // Handle --dry-run: validate parameters without executing
    if cmd.dry_run {
        println!("[DRY-RUN] Would execute status with:");
        println!("  verbose: {}", cmd.verbose);
        println!("  collection: {:?}", cmd.collection);
        return Ok(());
    }

    println!("QMD Index Status");
    println!("{}", "=".repeat(50));

    let stats = store.get_stats()?;

    println!("\nCollections: {}", stats.collection_count);
    println!("Documents: {}", stats.document_count);
    println!("Indexed: {}", stats.indexed_count);
    println!("Pending: {}", stats.pending_count);

    if cmd.verbose {
        println!("\nDetailed Statistics:");
        for (name, count) in &stats.collection_stats {
            println!("  {}: {} documents", name, count);
        }
    }

    Ok(())
}
