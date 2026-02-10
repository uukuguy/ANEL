use crate::cli::StatusArgs;
use crate::store::Store;
use anyhow::Result;

/// Handle status command - show index status
pub fn handle(
    cmd: &StatusArgs,
    store: &Store,
) -> Result<()> {
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
