use crate::anel::AnelSpec;
use crate::cli::LsArgs;
use crate::config::Config;
use crate::store::path;
use crate::store::Store;
use anyhow::Result;
use serde::Serialize;

/// List collections or files in a collection
pub fn handle(cmd: &LsArgs, config: &Config) -> Result<()> {
    // Handle --emit-spec: output ANEL specification and exit
    if cmd.emit_spec {
        let spec = AnelSpec::ls();
        println!("{}", serde_json::to_string_pretty(&spec)?);
        return Ok(());
    }

    // Handle --dry-run: validate parameters without executing
    if cmd.dry_run {
        println!("[DRY-RUN] Would execute ls with:");
        println!("  path: {:?}", cmd.path);
        return Ok(());
    }

    match &cmd.path {
        None => list_collections(config),
        Some(path) => list_files(config, path),
    }
}

/// List all collections with file counts
fn list_collections(config: &Config) -> Result<()> {
    let collections = &config.collections;

    if collections.is_empty() {
        println!("No collections found. Run 'qmd collection add .' to index files.");
        return Ok(());
    }

    // Get file counts from each collection's database
    let store = Store::new(config)?;

    println!("\nCollections:\n");

    for coll in collections {
        // Query file count from database
        let count = get_collection_file_count(&store, &coll.name).unwrap_or(0);

        // Format output similar to TypeScript version:
        //   qmd://collection/  (N files)
        println!(
            "  qmd://{}/  ({} files)",
            coll.name, count
        );
    }

    println!();
    Ok(())
}

/// Get file count for a collection
fn get_collection_file_count(store: &Store, collection: &str) -> Result<usize> {
    let conn = store.get_connection(collection)?;

    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM documents WHERE active = 1",
        [],
        |row| row.get(0),
    )?;

    Ok(count as usize)
}

/// List files in a collection
fn list_files(config: &Config, path_arg: &str) -> Result<()> {
    // Parse the path argument
    let (collection_name, path_prefix) = parse_ls_path(path_arg)?;

    // Get the collection config
    let collection = config.collections.iter()
        .find(|c| c.name == collection_name)
        .ok_or_else(|| anyhow::anyhow!("Collection not found: {}", collection_name))?;

    let store = Store::new(config)?;
    let conn = store.get_connection(&collection.name)?;

    // Query files
    let files = if let Some(prefix) = &path_prefix {
        // List files under a specific path
        let pattern = format!("{}%", prefix);
        let mut stmt = conn.prepare(
            "SELECT d.path, d.title, d.modified_at, LENGTH(ct.doc) as size
             FROM documents d
             JOIN content ct ON d.hash = ct.hash
             WHERE d.collection = ? AND d.path LIKE ? AND d.active = 1
             ORDER BY d.path"
        )?;

        let files = stmt.query_map([&collection.name, &pattern], |row| {
            Ok(FileEntry {
                path: row.get(0)?,
                title: row.get(1)?,
                modified_at: row.get(2)?,
                size: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();
        files
    } else {
        // List all files in the collection
        let mut stmt = conn.prepare(
            "SELECT d.path, d.title, d.modified_at, LENGTH(ct.doc) as size
             FROM documents d
             JOIN content ct ON d.hash = ct.hash
             WHERE d.collection = ? AND d.active = 1
             ORDER BY d.path"
        )?;

        let files = stmt.query_map([&collection.name], |row| {
            Ok(FileEntry {
                path: row.get(0)?,
                title: row.get(1)?,
                modified_at: row.get(2)?,
                size: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();
        files
    };

    if files.is_empty() {
        if let Some(prefix) = &path_prefix {
            println!("No files found under qmd://{}/{}", collection_name, prefix);
        } else {
            println!("No files found in collection: {}", collection_name);
        }
        return Ok(());
    }

    // Output in ls -l style
    for file in &files {
        let size_str = format_bytes(file.size);
        let time_str = format_time(&file.modified_at);
        println!(
            "{:>8}  {}  qmd://{}/{}",
            size_str,
            time_str,
            collection_name,
            file.path
        );
    }

    Ok(())
}

/// Parse ls path argument into (collection_name, path_prefix)
fn parse_ls_path(path_arg: &str) -> Result<(String, Option<String>)> {
    if path::is_virtual_path(path_arg) {
        // Virtual path format: qmd://collection/path
        let parsed = path::parse_virtual_path(path_arg)
            .ok_or_else(|| anyhow::anyhow!("Invalid virtual path: {}", path_arg))?;
        Ok((parsed.collection, Some(parsed.path)))
    } else {
        // Just collection name or collection/path
        let parts: Vec<&str> = path_arg.split('/').collect();
        let collection_name = parts.first().unwrap_or(&"").to_string();
        if collection_name.is_empty() {
            anyhow::bail!("Invalid path: {}", path_arg);
        }

        let path_prefix = if parts.len() > 1 {
            Some(parts[1..].join("/"))
        } else {
            None
        };

        Ok((collection_name, path_prefix))
    }
}

#[derive(Serialize)]
struct FileEntry {
    path: String,
    title: String,
    modified_at: String,
    size: i64,
}

/// Format bytes as human-readable string
fn format_bytes(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}K", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

/// Format timestamp for ls-style output
fn format_time(timestamp: &str) -> String {
    // Parse the timestamp (ISO 8601 format)
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        let now = chrono::Local::now();
        let six_months = chrono::Duration::days(180);

        if now.signed_duration_since(dt) > six_months {
            // Older than 6 months: show year
            dt.format("%b %d  %Y").to_string()
        } else {
            // Recent: show month and time
            dt.format("%b %d %H:%M").to_string()
        }
    } else {
        // Fallback: just show the raw timestamp
        timestamp.to_string()
    }
}
