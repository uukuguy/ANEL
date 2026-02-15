use crate::anel::AnelSpec;
use crate::cli::GetArgs;
use crate::config::Config;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Handle get command - retrieve document content
pub fn handle(
    cmd: &GetArgs,
    _config: &Config,
) -> Result<()> {
    let file_spec = &cmd.file;

    // Handle --emit-spec: output ANEL specification and exit
    if cmd.emit_spec {
        let spec = AnelSpec::get();
        println!("{}", serde_json::to_string_pretty(&spec)?);
        return Ok(());
    }

    // Handle --dry-run: validate parameters without executing
    if cmd.dry_run {
        println!("[DRY-RUN] Would execute get with:");
        println!("  file: {}", file_spec);
        println!("  limit: {}", cmd.limit);
        println!("  from: {}", cmd.from);
        println!("  full: {}", cmd.full);
        return Ok(());
    }

    // Parse file path with optional :line suffix
    let (file_path, line_spec) = parse_file_spec(file_spec)?;

    // Resolve full path
    let full_path = resolve_path(&file_path, _config)?;

    if !full_path.exists() {
        anyhow::bail!("File not found: {}", full_path.display());
    }

    if full_path.is_dir() {
        anyhow::bail!("Path is a directory, not a file: {}", full_path.display());
    }

    let content = fs::read_to_string(&full_path)
        .with_context(|| format!("Failed to read file: {}", full_path.display()))?;

    let lines: Vec<&str> = content.lines().collect();

    // Handle line specifications
    let (start, end) = if let Some(line) = line_spec {
        // :line or :line-end format
        let (start_line, end_line) = parse_line_range(&line, lines.len())?;
        (start_line, end_line)
    } else if cmd.full {
        (0, lines.len())
    } else {
        (cmd.from, std::cmp::min(cmd.from + cmd.limit, lines.len()))
    };

    for (i, line) in lines[start..end].iter().enumerate() {
        let line_num = start + i + 1;
        println!("{:>6}: {}", line_num, line);
    }

    if end < lines.len() {
        println!("... ({} more lines)", lines.len() - end);
    }

    Ok(())
}

fn parse_file_spec(spec: &str) -> Result<(String, Option<String>)> {
    if let Some((path, line)) = spec.rsplit_once(':') {
        // Check if line part is numeric or range
        if line.parse::<usize>().is_ok() || line.contains('-') {
            Ok((path.to_string(), Some(line.to_string())))
        } else {
            // The colon is part of the filename
            Ok((spec.to_string(), None))
        }
    } else {
        Ok((spec.to_string(), None))
    }
}

fn parse_line_range(line_spec: &str, total_lines: usize) -> Result<(usize, usize)> {
    if let Some((start, end)) = line_spec.split_once('-') {
        let start: usize = start.parse().context("Invalid start line")?;
        let end: usize = end.parse().context("Invalid end line")?;
        Ok((start.saturating_sub(1), std::cmp::min(end, total_lines)))
    } else {
        let line: usize = line_spec.parse().context("Invalid line number")?;
        let start = line.saturating_sub(1);
        let end = std::cmp::min(start + 1, total_lines);
        Ok((start, end))
    }
}

fn resolve_path(relative: &str, _config: &Config) -> Result<PathBuf> {
    let path = PathBuf::from(relative);

    if path.is_absolute() {
        Ok(path)
    } else {
        // Try relative to current directory
        match std::env::current_dir() {
            Ok(cwd) => Ok(cwd.join(path)),
            Err(_) => anyhow::bail!("Failed to get current directory"),
        }
    }
}
