use crate::cli::{VsearchArgs, FormatOptions};
use crate::store::Store;
use crate::llm::Router;
use crate::formatter::Format;
use anyhow::Result;

/// Handle vsearch command - vector semantic search
pub fn handle(
    cmd: &VsearchArgs,
    store: &Store,
    llm: &Router,
) -> Result<()> {
    let query = &cmd.query;
    let options = convert_options(&cmd.format);

    // Create a Tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;

    // Perform vector search with embedder
    let results = rt.block_on(async {
        vector_search_async(store, query, options.clone(), llm).await
    })?;

    // Format and display results
    let formatter = Format::from_string(&cmd.format.format);
    formatter.format_search_results(&results, options.limit)?;

    Ok(())
}

async fn vector_search_async(
    store: &Store,
    query: &str,
    options: crate::store::SearchOptions,
    llm: &Router,
) -> Result<Vec<crate::store::SearchResult>> {
    use log::info;

    // Generate embedding for the query
    let embedding_result = llm.embed(&[query]).await?;

    info!("Generated embedding with {} dimensions, provider: {}",
          embedding_result.embeddings[0].len(), embedding_result.provider);

    // Perform vector search in the appropriate collection(s)
    let collections = if let Some(ref col) = options.collection {
        vec![col.clone()]
    } else if options.search_all {
        store.get_collections().iter().map(|c| c.name.clone()).collect()
    } else {
        vec![store.get_collections().first()
            .ok_or_else(|| anyhow::anyhow!("No collections configured"))?
            .name.clone()]
    };

    let mut all_results = Vec::new();

    for collection in collections {
        let conn = store.get_connection(&collection)?;
        let results = vector_search_in_db(
            &conn,
            &embedding_result.embeddings[0],
            options.limit,
        )?;
        all_results.extend(results);
    }

    // Sort by score and limit
    all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    all_results.truncate(options.limit);

    Ok(all_results)
}

#[cfg(feature = "sqlite-vec")]
fn vector_search_in_db(
    conn: &rusqlite::Connection,
    query_vector: &[f32],
    limit: usize,
) -> Result<Vec<crate::store::SearchResult>> {
    use crate::store::SearchResult;

    let mut results = Vec::new();

    // Convert query vector to JSON array format for sqlite-vec
    let query_vec_json = serde_json::to_string(query_vector)?;

    // GROUP BY cv.hash aggregates multiple chunks back to one result per document,
    // taking the best (minimum distance) chunk score.
    let mut stmt = conn.prepare(
        "SELECT
            cv.hash,
            d.path,
            d.title,
            d.collection,
            MIN(vec_distance_cosine(v.embedding, ?)) as distance
         FROM content_vectors cv
         JOIN vectors_vec v ON v.hash_seq = cv.hash || '_' || cv.seq
         JOIN documents d ON d.hash = cv.hash
         WHERE d.active = 1
         GROUP BY cv.hash
         ORDER BY distance ASC
         LIMIT ?"
    )?;

    let rows: Vec<(String, String, String, String, f64)> = stmt
        .query_map(rusqlite::params![query_vec_json, limit as i64], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    for (hash, path, title, collection, distance) in rows {
        results.push(SearchResult {
            path,
            collection,
            score: (1.0 - distance as f32).max(0.0), // Convert distance to similarity score
            lines: 0,
            title,
            hash,
        });
    }

    Ok(results)
}

#[cfg(not(feature = "sqlite-vec"))]
fn vector_search_in_db(
    _conn: &rusqlite::Connection,
    _query_vector: &[f32],
    _limit: usize,
) -> Result<Vec<crate::store::SearchResult>> {
    anyhow::bail!("sqlite-vec feature not enabled")
}

fn convert_options(cmd: &FormatOptions) -> crate::store::SearchOptions {
    crate::store::SearchOptions {
        limit: cmd.limit,
        min_score: cmd.min_score,
        collection: cmd.collection.clone(),
        search_all: cmd.all,
    }
}
