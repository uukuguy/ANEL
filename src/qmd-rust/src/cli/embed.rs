use crate::cli::{EmbedArgs};
use crate::store::Store;
use crate::llm::Router;
use anyhow::Result;
use std::path::PathBuf;

/// Handle embed command - generate/update embeddings
pub fn handle(
    cmd: &EmbedArgs,
    store: &Store,
    llm: &Router,
) -> Result<()> {
    let collection = cmd.collection.clone();

    if let Some(col) = &collection {
        println!("Generating embeddings for collection: {}", col);
        store.embed_collection(col, llm, cmd.force)?;
    } else {
        println!("Generating embeddings for all collections...");
        store.embed_all_collections(llm, cmd.force)?;
    }

    Ok(())
}
