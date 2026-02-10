use crate::cli::UpdateArgs;
use crate::store::Store;
use anyhow::Result;

/// Handle update command - refresh index
pub fn handle(
    cmd: &UpdateArgs,
    store: &Store,
) -> Result<()> {
    if cmd.pull {
        println!("Pulling remote changes...");
        // TODO: Implement git pull or other remote sync
    }

    println!("Updating index...");
    store.update_index()?;

    println!("Index updated successfully");

    Ok(())
}
