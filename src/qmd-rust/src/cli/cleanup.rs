use crate::anel::AnelSpec;
use crate::cli::CleanupArgs;
use crate::store::Store;
use anyhow::Result;

/// Handle cleanup command - remove stale entries
pub fn handle(
    cmd: &CleanupArgs,
    store: &Store,
) -> Result<()> {
    // Handle --emit-spec: output ANEL specification and exit
    if cmd.emit_spec {
        let spec = AnelSpec::cleanup();
        println!("{}", serde_json::to_string_pretty(&spec)?);
        return Ok(());
    }

    let stale_files = store.find_stale_entries(cmd.older_than)?;

    if stale_files.is_empty() {
        println!("No stale entries found");
        return Ok(());
    }

    println!("Found {} stale entries:", stale_files.len());
    for file in &stale_files {
        println!("  {}", file);
    }

    if cmd.dry_run {
        println!("\nDry run - no changes made");
        return Ok(());
    }

    println!("\nRemoving stale entries...");
    store.remove_stale_entries(&stale_files)?;

    println!("Cleanup completed");

    Ok(())
}
