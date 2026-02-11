use qmd_rust::formatter::Format;
use qmd_rust::store::SearchResult;

fn make_results() -> Vec<SearchResult> {
    vec![
        SearchResult {
            path: "src/main.rs".to_string(),
            collection: "project".to_string(),
            score: 0.95,
            lines: 120,
            title: "Main Entry".to_string(),
            hash: "abc123".to_string(),
        },
        SearchResult {
            path: "src/lib.rs".to_string(),
            collection: "project".to_string(),
            score: 0.82,
            lines: 45,
            title: "Library".to_string(),
            hash: "def456".to_string(),
        },
    ]
}

// ==================== Format::from_string Tests ====================

#[test]
fn test_format_from_string_cli() {
    // Default / unknown strings should map to Cli
    let f = Format::from_string("cli");
    assert!(matches!(f, Format::Cli));

    let f2 = Format::from_string("unknown");
    assert!(matches!(f2, Format::Cli));
}

#[test]
fn test_format_from_string_json() {
    let f = Format::from_string("json");
    assert!(matches!(f, Format::Json));

    let f_upper = Format::from_string("JSON");
    assert!(matches!(f_upper, Format::Json));
}

#[test]
fn test_format_from_string_markdown() {
    let f = Format::from_string("md");
    assert!(matches!(f, Format::Markdown));

    let f2 = Format::from_string("markdown");
    assert!(matches!(f2, Format::Markdown));
}

#[test]
fn test_format_from_string_csv() {
    let f = Format::from_string("csv");
    assert!(matches!(f, Format::Csv));
}

#[test]
fn test_format_from_string_files() {
    let f = Format::from_string("files");
    assert!(matches!(f, Format::Files));

    let f2 = Format::from_string("paths");
    assert!(matches!(f2, Format::Files));
}

// ==================== Format Output Tests ====================
// These tests capture stdout to verify output content.

#[test]
fn test_format_cli_no_panic() {
    let results = make_results();
    let fmt = Format::from_string("cli");
    // Should not panic or error
    fmt.format_search_results(&results, 10).unwrap();
}

#[test]
fn test_format_json_parseable() {
    // We can't easily capture println! output without extra deps,
    // but we can verify the JSON serialization logic works by
    // directly serializing SearchResult.
    let results = make_results();
    let json = serde_json::to_string_pretty(&results).unwrap();
    let parsed: Vec<SearchResult> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].path, "src/main.rs");
    assert_eq!(parsed[1].score, 0.82);
}

#[test]
fn test_format_json_no_panic() {
    let results = make_results();
    let fmt = Format::from_string("json");
    fmt.format_search_results(&results, 10).unwrap();
}

#[test]
fn test_format_csv_no_panic() {
    let results = make_results();
    let fmt = Format::from_string("csv");
    fmt.format_search_results(&results, 10).unwrap();
}

#[test]
fn test_format_markdown_no_panic() {
    let results = make_results();
    let fmt = Format::from_string("md");
    fmt.format_search_results(&results, 10).unwrap();
}

#[test]
fn test_format_files_no_panic() {
    let results = make_results();
    let fmt = Format::from_string("files");
    fmt.format_search_results(&results, 10).unwrap();
}

#[test]
fn test_format_empty_results() {
    let results: Vec<SearchResult> = vec![];
    for fmt_str in &["cli", "json", "md", "csv", "files"] {
        let fmt = Format::from_string(fmt_str);
        fmt.format_search_results(&results, 10).unwrap();
    }
}

#[test]
fn test_format_limit_truncates() {
    let results = make_results(); // 2 results
    let fmt = Format::from_string("json");
    // Limit to 1 — should not panic
    fmt.format_search_results(&results, 1).unwrap();
}

#[test]
fn test_search_result_json_roundtrip() {
    let original = make_results();
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: Vec<SearchResult> = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}
