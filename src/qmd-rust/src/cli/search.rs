use crate::cli::{SearchArgs, FormatOptions};
use crate::store::Store;
use crate::formatter::Format;
use anyhow::Result;

/// Handle search command - BM25 full-text search
pub fn handle(
    cmd: &SearchArgs,
    store: &Store,
) -> Result<()> {
    let query = &cmd.query;
    let options = convert_options(&cmd.format);

    // Perform search
    let results = store.bm25_search(query, options.clone())?;

    // Format and display results
    let formatter = Format::from_string(&cmd.format.format);
    formatter.format_search_results(&results, options.limit)?;

    Ok(())
}

fn convert_options(cmd: &FormatOptions) -> crate::store::SearchOptions {
    crate::store::SearchOptions {
        limit: cmd.limit,
        min_score: cmd.min_score,
        collection: cmd.collection.clone(),
        search_all: cmd.all,
    }
}
