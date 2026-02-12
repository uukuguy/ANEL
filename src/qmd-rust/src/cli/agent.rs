use crate::cli::AgentArgs;
use crate::formatter::Format;
use crate::llm::Router;
use crate::store::{SearchOptions, Store};
use anyhow::Result;
use dialoguer::Input;
use log::info;

/// Query intent classification
#[derive(Debug, Clone, PartialEq)]
pub enum QueryIntent {
    /// Short keyword queries → BM25 full-text search
    Keyword,
    /// Natural language / semantic queries → vector search
    Semantic,
    /// Complex queries with mixed signals → hybrid search
    Complex,
}

impl std::fmt::Display for QueryIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryIntent::Keyword => write!(f, "keyword (BM25)"),
            QueryIntent::Semantic => write!(f, "semantic (vector)"),
            QueryIntent::Complex => write!(f, "complex (hybrid)"),
        }
    }
}

/// Question words that indicate semantic/natural language queries
const QUESTION_WORDS: &[&str] = &[
    "how", "what", "why", "when", "where", "who", "which", "explain", "describe", "compare",
];

/// Boolean operators that indicate keyword search intent
const BOOLEAN_OPS: &[&str] = &["AND", "OR", "NOT"];

/// Classify query intent based on heuristics:
/// - 1-2 words without question words → Keyword
/// - Contains boolean operators or quoted phrases → Keyword
/// - Question words or long natural language → Semantic
/// - Mixed signals or medium length → Complex
pub fn classify_intent(query: &str) -> QueryIntent {
    let trimmed = query.trim();
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    let word_count = words.len();
    let lower = trimmed.to_lowercase();

    // Empty or single word → keyword
    if word_count <= 1 {
        return QueryIntent::Keyword;
    }

    // Contains boolean operators → keyword
    if words.iter().any(|w| BOOLEAN_OPS.contains(w)) {
        return QueryIntent::Keyword;
    }

    // Contains quoted phrase → keyword
    if trimmed.contains('"') {
        return QueryIntent::Keyword;
    }

    // Starts with question word → semantic
    let first_word = lower.split_whitespace().next().unwrap_or("");
    let is_question = QUESTION_WORDS.iter().any(|q| first_word == *q);

    // Ends with question mark → semantic
    let ends_with_question = trimmed.ends_with('?');

    if is_question || ends_with_question {
        if word_count >= 6 {
            return QueryIntent::Complex;
        }
        return QueryIntent::Semantic;
    }

    // Short phrases (2-3 words) without question signals → keyword
    if word_count <= 3 {
        return QueryIntent::Keyword;
    }

    // Medium length (4-5 words) → complex (benefits from hybrid)
    if word_count <= 5 {
        return QueryIntent::Complex;
    }

    // Long queries (6+ words) → semantic
    QueryIntent::Semantic
}

/// Default search options for agent mode
fn default_options() -> SearchOptions {
    SearchOptions {
        limit: 10,
        min_score: 0.0,
        collection: None,
        search_all: false,
    }
}

/// Execute a search based on classified intent
fn execute_search(
    query: &str,
    intent: &QueryIntent,
    store: &Store,
    llm: &Router,
) -> Result<Vec<crate::store::SearchResult>> {
    let options = default_options();
    let rt = tokio::runtime::Runtime::new()?;

    match intent {
        QueryIntent::Keyword => {
            store.bm25_search(query, options)
        }
        QueryIntent::Semantic => {
            rt.block_on(async {
                let embedding_result = llm.embed(&[query]).await?;
                let collections = resolve_collections(store, &options)?;
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
                all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
                all_results.truncate(options.limit);
                Ok(all_results)
            })
        }
        QueryIntent::Complex => {
            rt.block_on(async {
                store.hybrid_search(query, options, llm).await
            })
        }
    }
}

/// Resolve which collections to search
fn resolve_collections(store: &Store, options: &SearchOptions) -> Result<Vec<String>> {
    if let Some(ref col) = options.collection {
        Ok(vec![col.clone()])
    } else if options.search_all {
        Ok(store.get_collections().iter().map(|c| c.name.clone()).collect())
    } else {
        Ok(vec![store
            .get_collections()
            .first()
            .ok_or_else(|| anyhow::anyhow!("No collections configured"))?
            .name
            .clone()])
    }
}

#[cfg(feature = "sqlite-vec")]
fn vector_search_in_db(
    conn: &rusqlite::Connection,
    query_vector: &[f32],
    limit: usize,
) -> Result<Vec<crate::store::SearchResult>> {
    use crate::store::SearchResult;

    let query_vec_json = serde_json::to_string(query_vector)?;

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

    let mut results = Vec::new();
    for (hash, path, title, collection, distance) in rows {
        results.push(SearchResult {
            path,
            collection,
            score: (1.0 - distance as f32).max(0.0),
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

/// Handle agent command - autonomous search mode
pub fn handle(cmd: &AgentArgs, store: &Store, llm: &Router) -> Result<()> {
    if cmd.interactive {
        run_interactive_agent(store, llm)?;
    } else {
        let query: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Enter search query")
            .interact_text()?;

        run_single_query(&query, store, llm)?;
    }

    Ok(())
}

/// Run a single query through the classify → route → display pipeline
fn run_single_query(query: &str, store: &Store, llm: &Router) -> Result<()> {
    let intent = classify_intent(query);
    info!("Query: {:?} → Intent: {}", query, intent);
    println!("[{}] {}", intent, query);

    let results = execute_search(query, &intent, store, llm)?;

    let formatter = Format::Cli;
    formatter.format_search_results(&results, default_options().limit)?;

    Ok(())
}

fn run_interactive_agent(store: &Store, llm: &Router) -> Result<()> {
    println!("QMD Agent Mode - Interactive");
    println!("Commands: 'exit' quit | 'help' commands | 'mode' show routing info");
    println!();

    loop {
        let input: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("qmd>")
            .interact_text()?;

        let trimmed = input.trim();

        if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
            break;
        }

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.eq_ignore_ascii_case("help") {
            print_help();
            continue;
        }

        if trimmed.eq_ignore_ascii_case("mode") {
            print_mode_info();
            continue;
        }

        // Handle forced routing: /bm25, /vector, /hybrid prefixes
        if let Some(forced) = parse_forced_intent(trimmed) {
            let (intent, query) = forced;
            info!("Forced intent: {} for query: {:?}", intent, query);
            println!("[forced: {}] {}", intent, query);

            let results = execute_search(query, &intent, store, llm)?;
            let formatter = Format::Cli;
            formatter.format_search_results(&results, default_options().limit)?;
            println!();
            continue;
        }

        if let Err(e) = run_single_query(trimmed, store, llm) {
            eprintln!("Error: {}", e);
        }
        println!();
    }

    Ok(())
}

/// Parse forced intent prefix: /bm25 <query>, /vector <query>, /hybrid <query>
fn parse_forced_intent(input: &str) -> Option<(QueryIntent, &str)> {
    if let Some(query) = input.strip_prefix("/bm25 ") {
        Some((QueryIntent::Keyword, query.trim()))
    } else if let Some(query) = input.strip_prefix("/vector ") {
        Some((QueryIntent::Semantic, query.trim()))
    } else if let Some(query) = input.strip_prefix("/hybrid ") {
        Some((QueryIntent::Complex, query.trim()))
    } else {
        None
    }
}

fn print_help() {
    println!("QMD Agent - Intelligent Search Router");
    println!();
    println!("  <query>          Auto-classify and route to best search method");
    println!("  /bm25 <query>    Force BM25 full-text search");
    println!("  /vector <query>  Force vector semantic search");
    println!("  /hybrid <query>  Force hybrid search (BM25 + vector + rerank)");
    println!("  mode             Show routing classification rules");
    println!("  help             Show this help");
    println!("  exit             Quit agent mode");
}

fn print_mode_info() {
    println!("Query Intent Classification Rules:");
    println!();
    println!("  Keyword (BM25):");
    println!("    - 1 word queries");
    println!("    - Contains boolean operators (AND, OR, NOT)");
    println!("    - Contains quoted phrases");
    println!("    - Short phrases (2-3 words) without question signals");
    println!();
    println!("  Semantic (Vector):");
    println!("    - Starts with question word (how, what, why, ...)");
    println!("    - Ends with question mark");
    println!("    - Long natural language (6+ words)");
    println!();
    println!("  Complex (Hybrid):");
    println!("    - Question queries with 6+ words");
    println!("    - Medium-length phrases (4-5 words)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_single_word_as_keyword() {
        assert_eq!(classify_intent("rust"), QueryIntent::Keyword);
        assert_eq!(classify_intent("config"), QueryIntent::Keyword);
    }

    #[test]
    fn classify_empty_as_keyword() {
        assert_eq!(classify_intent(""), QueryIntent::Keyword);
        assert_eq!(classify_intent("  "), QueryIntent::Keyword);
    }

    #[test]
    fn classify_boolean_ops_as_keyword() {
        assert_eq!(classify_intent("rust AND python"), QueryIntent::Keyword);
        assert_eq!(classify_intent("error OR warning"), QueryIntent::Keyword);
        assert_eq!(classify_intent("config NOT debug"), QueryIntent::Keyword);
    }

    #[test]
    fn classify_quoted_phrase_as_keyword() {
        assert_eq!(classify_intent("\"exact match\""), QueryIntent::Keyword);
        assert_eq!(classify_intent("search \"hello world\""), QueryIntent::Keyword);
    }

    #[test]
    fn classify_short_phrase_as_keyword() {
        assert_eq!(classify_intent("rust config"), QueryIntent::Keyword);
        assert_eq!(classify_intent("error handling code"), QueryIntent::Keyword);
    }

    #[test]
    fn classify_question_as_semantic() {
        assert_eq!(classify_intent("how to configure"), QueryIntent::Semantic);
        assert_eq!(classify_intent("what is embedding"), QueryIntent::Semantic);
        assert_eq!(classify_intent("why use vectors"), QueryIntent::Semantic);
    }

    #[test]
    fn classify_question_mark_as_semantic() {
        assert_eq!(classify_intent("is this working?"), QueryIntent::Semantic);
    }

    #[test]
    fn classify_long_question_as_complex() {
        assert_eq!(
            classify_intent("how do I configure vector search with custom embeddings"),
            QueryIntent::Complex
        );
    }

    #[test]
    fn classify_medium_phrase_as_complex() {
        assert_eq!(classify_intent("vector search configuration options"), QueryIntent::Complex);
        assert_eq!(classify_intent("rust async runtime setup"), QueryIntent::Complex);
    }

    #[test]
    fn classify_long_natural_language_as_semantic() {
        assert_eq!(
            classify_intent("documents about machine learning and neural network architectures"),
            QueryIntent::Semantic
        );
    }

    #[test]
    fn parse_forced_bm25() {
        let result = parse_forced_intent("/bm25 rust config");
        assert_eq!(result, Some((QueryIntent::Keyword, "rust config")));
    }

    #[test]
    fn parse_forced_vector() {
        let result = parse_forced_intent("/vector how to search");
        assert_eq!(result, Some((QueryIntent::Semantic, "how to search")));
    }

    #[test]
    fn parse_forced_hybrid() {
        let result = parse_forced_intent("/hybrid complex query here");
        assert_eq!(result, Some((QueryIntent::Complex, "complex query here")));
    }

    #[test]
    fn parse_no_forced_intent() {
        assert_eq!(parse_forced_intent("regular query"), None);
        assert_eq!(parse_forced_intent("/unknown query"), None);
    }
}
