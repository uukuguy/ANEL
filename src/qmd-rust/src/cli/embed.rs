use crate::cli::EmbedArgs;
use crate::store::Store;
use crate::store::chunker::{chunk_document, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP};
use crate::llm::Router;
use anyhow::Result;

/// Handle embed command - generate/update embeddings
pub fn handle(
    cmd: &EmbedArgs,
    store: &Store,
    llm: &Router,
) -> Result<()> {
    let collection = cmd.collection.clone();

    // Create a Tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;

    if let Some(col) = &collection {
        println!("Generating embeddings for collection: {}", col);
        rt.block_on(async {
            embed_collection_async(store, col, llm, cmd.force).await
        })?;
    } else {
        println!("Generating embeddings for all collections...");
        rt.block_on(async {
            embed_all_collections_async(store, llm, cmd.force).await
        })?;
    }

    Ok(())
}

async fn embed_collection_async(
    store: &Store,
    collection: &str,
    llm: &Router,
    force: bool,
) -> Result<()> {
    use log::info;

    info!("Embedding collection: {}", collection);

    if !llm.has_embedder() {
        log::warn!("No embedder available, skipping embedding");
        return Ok(());
    }

    let conn = store.get_connection(collection)?;

    // Get all documents that need embedding
    let mut stmt = if force {
        conn.prepare("SELECT id, hash, doc FROM documents WHERE active = 1")?
    } else {
        conn.prepare(
            "SELECT id, hash, doc FROM documents
             WHERE active = 1
             AND hash NOT IN (SELECT DISTINCT hash FROM content_vectors)"
        )?
    };

    let docs: Vec<(i64, String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    info!("Found {} documents to embed", docs.len());

    // Chunk all documents and build a flat list of (hash, chunk) pairs
    let mut all_chunks: Vec<(String, crate::store::chunker::Chunk)> = Vec::new();

    for (_doc_id, hash, doc) in &docs {
        let chunks = chunk_document(doc, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP);

        // If force re-embed, delete old chunks for this hash
        if force {
            conn.execute("DELETE FROM content_vectors WHERE hash = ?", [hash])?;
            // Delete all vector entries for this hash (any seq)
            let like_pattern = format!("{}_%", hash);
            conn.execute("DELETE FROM vectors_vec WHERE hash_seq LIKE ?", [&like_pattern])?;
        }

        for chunk in chunks {
            all_chunks.push((hash.clone(), chunk));
        }
    }

    info!("Total chunks to embed: {}", all_chunks.len());

    // Process chunks in batches
    let batch_size = 10;
    for (batch_idx, batch) in all_chunks.chunks(batch_size).enumerate() {
        info!("Processing batch {}/{}", batch_idx + 1, (all_chunks.len() + batch_size - 1) / batch_size);

        // Prepare texts for embedding
        let texts: Vec<&str> = batch.iter().map(|(_, chunk)| chunk.text.as_str()).collect();

        // Generate embeddings
        let embedding_result = llm.embed(&texts).await?;

        info!("Generated {} embeddings with model: {}",
              embedding_result.embeddings.len(), embedding_result.model);

        // Store embeddings per chunk
        for (i, (hash, chunk)) in batch.iter().enumerate() {
            let embedding = &embedding_result.embeddings[i];
            let embedding_json = serde_json::to_string(embedding)?;

            // Store in content_vectors metadata table
            conn.execute(
                "INSERT OR REPLACE INTO content_vectors (hash, seq, pos, model, embedded_at)
                 VALUES (?, ?, ?, ?, datetime('now'))",
                rusqlite::params![hash, chunk.seq as i64, chunk.pos as i64, &embedding_result.model],
            )?;

            // Store in vectors_vec table
            let hash_seq = format!("{}_{}", hash, chunk.seq);
            conn.execute(
                "INSERT OR REPLACE INTO vectors_vec (hash_seq, embedding)
                 VALUES (?, ?)",
                [&hash_seq, &embedding_json],
            )?;

            info!("Stored embedding for hash: {} chunk seq: {}", hash, chunk.seq);
        }
    }

    info!("Embedding complete for collection: {}", collection);
    Ok(())
}

async fn embed_all_collections_async(
    store: &Store,
    llm: &Router,
    force: bool,
) -> Result<()> {
    for collection in store.get_collections() {
        embed_collection_async(store, &collection.name, llm, force).await?;
    }
    Ok(())
}
