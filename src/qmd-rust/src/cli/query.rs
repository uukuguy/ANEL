use crate::cli::{QueryArgs, FormatOptions};
use crate::store::Store;
use crate::llm::Router;
use crate::formatter::Format;
use anyhow::Result;

/// Handle query command - hybrid search with reranking
pub fn handle(
    cmd: &QueryArgs,
    store: &Store,
    llm: &Router,
) -> Result<()> {
    let query = &cmd.query;
    let options = convert_options(&cmd.format);

    // Create a Tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;

    // Perform hybrid search with LLM reranking
    let results = rt.block_on(async {
        store.hybrid_search(query, options.clone(), llm).await
    })?;

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
