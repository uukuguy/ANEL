use crate::anel::AnelSpec;
use crate::cli::{CollectionArgs, CollectionCommands, CollectionAddArgs, CollectionRemoveArgs, CollectionRenameArgs};
use crate::config::{Config, CollectionConfig};
use anyhow::Result;
use std::path::PathBuf;

/// Handle collection commands
pub fn handle(
    cmd: &CollectionArgs,
    config: &mut Config,
) -> Result<()> {
    // Handle --emit-spec: output ANEL specification and exit
    if cmd.emit_spec {
        let spec = AnelSpec::collection();
        println!("{}", serde_json::to_string_pretty(&spec)?);
        return Ok(());
    }

    // Handle --dry-run: validate parameters without executing
    if cmd.dry_run {
        println!("[DRY-RUN] Would execute collection with:");
        println!("  format: {}", cmd.format);
        match &cmd.command {
            CollectionCommands::Add(args) => {
                println!("  action: add");
                println!("  path: {}", args.path);
                println!("  name: {:?}", args.name);
                println!("  mask: {}", args.mask);
                println!("  description: {:?}", args.description);
            }
            CollectionCommands::List => {
                println!("  action: list");
            }
            CollectionCommands::Remove(args) => {
                println!("  action: remove");
                println!("  name: {}", args.name);
            }
            CollectionCommands::Rename(args) => {
                println!("  action: rename");
                println!("  old_name: {}", args.old_name);
                println!("  new_name: {}", args.new_name);
            }
        }
        return Ok(());
    }

    match &cmd.command {
        CollectionCommands::Add(args) => add_collection(args, config),
        CollectionCommands::List => list_collections(config),
        CollectionCommands::Remove(args) => remove_collection(args, config),
        CollectionCommands::Rename(args) => rename_collection(args, config),
    }
}

/// Add a new collection
fn add_collection(args: &CollectionAddArgs, config: &mut Config) -> Result<()> {
    let name = args.name.clone().unwrap_or_else(|| {
        PathBuf::from(&args.path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });

    let path = shellexpand::tilde(&args.path).parse::<PathBuf>()?;

    if path.exists() && !path.is_dir() {
        anyhow::bail!("Path exists but is not a directory: {}", args.path);
    }

    // Check for duplicate name
    if config.collections.iter().any(|c| c.name == name) {
        anyhow::bail!("Collection '{}' already exists", name);
    }

    let collection = CollectionConfig {
        name: name.clone(),
        path: path.clone(),
        pattern: Some(args.mask.clone()),
        description: args.description.clone(),
    };

    config.collections.push(collection);
    config.save()?;

    // Create cache directory for the collection
    let cache_dir = config.cache_dir_for(&name);
    std::fs::create_dir_all(&cache_dir)?;

    println!("Collection '{}' added successfully", name);
    println!("  Path: {}", path.display());
    println!("  Pattern: {}", args.mask);
    if let Some(desc) = &args.description {
        println!("  Description: {}", desc);
    }

    Ok(())
}

/// List all collections
fn list_collections(config: &Config) -> Result<()> {
    if config.collections.is_empty() {
        println!("No collections configured");
        return Ok(());
    }

    println!("Collections:");
    println!("{:<20} {:<40} {:<15} Description", "Name", "Path", "Pattern");
    println!("{}", "-".repeat(95));

    for collection in &config.collections {
        let path_str = collection.path.display().to_string();
        let pattern = collection.pattern.as_deref().unwrap_or("**/*");
        let desc = collection.description.as_deref().unwrap_or("");
        println!("{:<20} {:<40} {:<15} {}", collection.name, path_str, pattern, desc);
    }

    Ok(())
}

/// Remove a collection
fn remove_collection(args: &CollectionRemoveArgs, config: &mut Config) -> Result<()> {
    let name = &args.name;

    let idx = config.collections.iter().position(|c| c.name == *name);
    match idx {
        Some(i) => {
            config.collections.remove(i);
            config.save()?;

            // Remove cached index database
            let cache_dir = config.cache_dir_for(name);
            if cache_dir.exists() {
                std::fs::remove_dir_all(&cache_dir)?;
                println!("Removed cache directory: {}", cache_dir.display());
            }

            println!("Collection '{}' removed successfully", name);
        }
        None => {
            anyhow::bail!("Collection not found: {}", name);
        }
    }

    Ok(())
}

/// Rename a collection
fn rename_collection(args: &CollectionRenameArgs, config: &mut Config) -> Result<()> {
    let old_name = &args.old_name;
    let new_name = &args.new_name;

    // Check if new name already exists
    if config.collections.iter().any(|c| c.name == *new_name) {
        anyhow::bail!("Collection name already exists: {}", new_name);
    }

    let idx = config.collections.iter().position(|c| c.name == *old_name);
    match idx {
        Some(i) => {
            config.collections[i].name = new_name.clone();
            config.save()?;

            // Rename cache directory
            let old_cache = config.cache_dir_for(old_name);
            let new_cache = config.cache_dir_for(new_name);
            if old_cache.exists() {
                std::fs::rename(&old_cache, &new_cache)?;
                println!("Renamed cache directory: {} -> {}", old_cache.display(), new_cache.display());
            }

            println!("Collection '{}' renamed to '{}'", old_name, new_name);
        }
        None => {
            anyhow::bail!("Collection not found: {}", old_name);
        }
    }

    Ok(())
}
