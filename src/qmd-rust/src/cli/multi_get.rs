use crate::cli::{MultiGetArgs};
use crate::config::Config;
use anyhow::{Context, Result};
use glob::glob;
use std::fs;
use std::path::PathBuf;

/// Handle multi-get command - retrieve multiple documents by pattern
pub fn handle(
    cmd: &MultiGetArgs,
    _config: &Config,
) -> Result<()> {
    let pattern = &cmd.pattern;

    // Expand the glob pattern
    let entries = glob(pattern)
        .with_context(|| format!("Invalid glob pattern: {}", pattern))?;

    let mut count = 0;
    let mut errors = 0;

    for entry in entries {
        match entry {
            Ok(path) => {
                if path.is_file() {
                    match read_file_preview(&path, cmd.limit, cmd.max_bytes) {
                        Ok(_) => count += 1,
                        Err(e) => {
                            eprintln!("Error reading {}: {}", path.display(), e);
                            errors += 1;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Glob error: {}", e);
                errors += 1;
            }
        }
    }

    println!("\nProcessed {} files ({} errors)", count, errors);

    Ok(())
}

fn read_file_preview(path: &PathBuf, limit: usize, max_bytes: Option<usize>) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read: {}", path.display()))?;

    println!("\n=== {} ===", path.display());
    println!("---");

    let lines: Vec<&str> = content.lines().collect();
    let preview_lines = std::cmp::min(limit, lines.len());

    for (i, line) in lines[..preview_lines].iter().enumerate() {
        println!("{:>4}: {}", i + 1, line);
    }

    if let Some(max) = max_bytes {
        if content.len() > max {
            println!("... ({} bytes truncated)", content.len() - max);
        }
    }

    if lines.len() > preview_lines {
        println!("... ({} more lines)", lines.len() - preview_lines);
    }

    Ok(())
}
