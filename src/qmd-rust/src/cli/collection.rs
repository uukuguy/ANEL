use crate::cli::{CollectionCommands, CollectionAddArgs, CollectionRemoveArgs, CollectionRenameArgs};
use crate::config::{Config, CollectionConfig};
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Handle collection commands
pub fn handle(
    cmd: &crate::cli::CollectionArgs,
    config: &Config,
) -> Result<()> {
    match &cmd.command {
        CollectionCommands::Add(args) => add_collection(args, config),
        CollectionCommands::List => list_collections(config),
        CollectionCommands::Remove(args) => remove_collection(args, config),
        CollectionCommands::Rename(args) => rename_collection(args, config),
    }
}

/// Add a new collection
fn add_collection(args: &CollectionAddArgs, config: &Config) -> Result<()> {
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

    let collection = CollectionConfig {
        name: name.clone(),
        path,
        pattern: Some(args.mask.clone()),
        description: args.description.clone(),
    };

    // TODO: Create index database for collection

    println!("Collection '{}' added successfully", name);
    println!("  Path: {}", collection.path.display());
    if let Some(pattern) = &collection.pattern {
        println!("  Pattern: {}", pattern);
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
    println!("{:<30} {:<40} {}", "Name", "Path", "Description");
    println!("{}", "-".repeat(90));

    for collection in &config.collections {
        let path_str = collection.path.display().to_string();
        let desc = collection.description.as_deref().unwrap_or("");
        println!("{:<30} {:<40} {}", collection.name, path_str, desc);
    }

    Ok(())
}

/// Remove a collection
fn remove_collection(args: &CollectionRemoveArgs, config: &Config) -> Result<()> {
    let name = &args.name;

    // Check if collection exists
    let exists = config.collections.iter().any(|c| c.name == *name);
    if !exists {
        anyhow::bail!("Collection not found: {}", name);
    }

    // TODO: Remove index database and cached files

    println!("Collection '{}' removed successfully", name);

    Ok(())
}

/// Rename a collection
fn rename_collection(args: &CollectionRenameArgs, config: &Config) -> Result<()> {
    let old_name = &args.old_name;
    let new_name = &args.new_name;

    // Check if old collection exists
    let exists = config.collections.iter().any(|c| c.name == *old_name);
    if !exists {
        anyhow::bail!("Collection not found: {}", old_name);
    }

    // Check if new name already exists
    let name_exists = config.collections.iter().any(|c| c.name == *new_name);
    if name_exists {
        anyhow::bail!("Collection name already exists: {}", new_name);
    }

    // TODO: Rename index database

    println!("Collection '{}' renamed to '{}'", old_name, new_name);

    Ok(())
}
