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
}

impl Format {
    /// Create format from string
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => Self::Json,
            "md" | "markdown" => Self::Markdown,
            "csv" => Self::Csv,
            "files" | "paths" => Self::Files,
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
        }
    }

    fn format_cli(&self, results: &[SearchResult]) -> Result<(), anyhow::Error> {
        println!("Found {} results:", results.len());
        println!("{:<6} {:<8} {}", "Score", "Lines", "Path");
        println!("{}", "-".repeat(80));

        for result in results {
            let score = format!("{:.4}", result.score);
            println!("{:<6} {:<8} {}", score, result.lines, result.path);
        }
        Ok(())
    }

    fn format_json(&self, results: &[SearchResult]) -> Result<(), anyhow::Error> {
        #[derive(Serialize)]
        struct JsonResult {
            query: String,
            total: usize,
            results: Vec<SearchResult>,
        }

        let output = JsonResult {
            query: String::new(), // TODO: Add query to SearchResult
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
            println!("- **Score**: {:.4}", result.score);
            println!("- **Lines**: {}", result.lines);
            println!();
        }
        Ok(())
    }

    fn format_csv(&self, results: &[SearchResult]) -> Result<(), anyhow::Error> {
        println!("score,lines,path");
        for result in results {
            println!("{:.4},{},{}", result.score, result.lines, result.path);
        }
        Ok(())
    }

    fn format_files(&self, results: &[SearchResult]) -> Result<(), anyhow::Error> {
        for result in results {
            println!("{}", result.path);
        }
        Ok(())
    }
}
