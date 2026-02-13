use crate::store::SearchResult;
use serde::Serialize;

/// Output format types
#[derive(Debug, Clone)]
pub enum Format {
    Cli,
    Json,
    Markdown,
    Csv,
    Files,
    Xml,
}

impl Format {
    /// Create format from string
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => Self::Json,
            "md" | "markdown" => Self::Markdown,
            "csv" => Self::Csv,
            "files" | "paths" => Self::Files,
            "xml" => Self::Xml,
            _ => Self::Cli,
        }
    }

    /// Format search results
    pub fn format_search_results(&self, results: &[SearchResult], limit: usize) -> Result<(), anyhow::Error> {
        let limited_results = &results[..std::cmp::min(results.len(), limit)];

        match self {
            Self::Cli => self.format_cli(limited_results),
            Self::Json => self.format_json(limited_results),
            Self::Markdown => self.format_markdown(limited_results),
            Self::Csv => self.format_csv(limited_results),
            Self::Files => self.format_files(limited_results),
            Self::Xml => self.format_xml(limited_results),
        }
    }

    fn format_cli(&self, results: &[SearchResult]) -> Result<(), anyhow::Error> {
        println!("Found {} results:", results.len());
        println!("{:<6} {:<8} {:<40} Path", "Score", "Lines", "DocID");
        println!("{}", "-".repeat(100));

        for result in results {
            let score = format!("{:.4}", result.score);
            println!("{:<6} {:<8} {:<40} {}", score, result.lines, result.docid, result.path);
        }
        Ok(())
    }

    fn format_json(&self, results: &[SearchResult]) -> Result<(), anyhow::Error> {
        #[derive(Serialize)]
        struct JsonResult {
            query: Option<String>,
            total: usize,
            results: Vec<SearchResult>,
        }

        // Extract query from first result if available
        let query = results.first().and_then(|r| r.query.clone());

        let output = JsonResult {
            query,
            total: results.len(),
            results: results.to_vec(),
        };

        println!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    fn format_markdown(&self, results: &[SearchResult]) -> Result<(), anyhow::Error> {
        println!("# Search Results");
        println!();
        println!("Found {} results:", results.len());
        println!();

        for (i, result) in results.iter().enumerate() {
            println!("## {}. {}", i + 1, result.path);
            println!("- **DocID**: {}", result.docid);
            println!("- **Score**: {:.4}", result.score);
            println!("- **Lines**: {}", result.lines);
            println!();
        }
        Ok(())
    }

    fn format_csv(&self, results: &[SearchResult]) -> Result<(), anyhow::Error> {
        println!("docid,score,lines,path");
        for result in results {
            println!("{},{:.4},{},{}", result.docid, result.score, result.lines, result.path);
        }
        Ok(())
    }

    fn format_files(&self, results: &[SearchResult]) -> Result<(), anyhow::Error> {
        for result in results {
            println!("{}", result.path);
        }
        Ok(())
    }

    fn format_xml(&self, results: &[SearchResult]) -> Result<(), anyhow::Error> {
        println!(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        println!("<results total=\"{}\">", results.len());
        for result in results {
            println!("  <result>");
            println!("    <docid>{}</docid>", escape_xml(&result.docid));
            println!("    <path>{}</path>", escape_xml(&result.path));
            println!("    <collection>{}</collection>", escape_xml(&result.collection));
            println!("    <title>{}</title>", escape_xml(&result.title));
            println!("    <score>{:.4}</score>", result.score);
            println!("    <lines>{}</lines>", result.lines);
            println!("  </result>");
        }
        println!("</results>");
        Ok(())
    }
}

/// Escape special XML characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
