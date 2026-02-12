use crate::cli::{ContextCommands, ContextAddArgs, ContextRemoveArgs};
use crate::config::Config;
use anyhow::Result;
use std::path::PathBuf;

/// Handle context commands
pub fn handle(
    cmd: &crate::cli::ContextArgs,
    config: &mut Config,
) -> Result<()> {
    match &cmd.command {
        ContextCommands::Add(args) => add_context(args, config),
        ContextCommands::List => list_contexts(config),
        ContextCommands::Rm(args) => remove_context(args, config),
    }
}

/// Add a context (path with description for relevance)
fn add_context(args: &ContextAddArgs, config: &mut Config) -> Result<()> {
    let path = match &args.path {
        Some(p) => shellexpand::tilde(p).parse::<PathBuf>()?,
        None => std::env::current_dir()?,
    };

    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    // Check if this path already has a context description
    let existing = config.collections.iter().position(|c| c.path == path);
    match existing {
        Some(i) => {
            // Update existing collection's description
            config.collections[i].description = Some(args.description.clone());
            config.save()?;
            println!("Context updated:");
        }
        None => {
            // Add as a new collection with the path as name
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let collection = crate::config::CollectionConfig {
                name,
                path: path.clone(),
                pattern: Some("**/*".to_string()),
                description: Some(args.description.clone()),
            };
            config.collections.push(collection);
            config.save()?;
            println!("Context added:");
        }
    }

    println!("  Path: {}", path.display());
    println!("  Description: {}", args.description);

    Ok(())
}

/// List all contexts
fn list_contexts(config: &Config) -> Result<()> {
    let contexts: Vec<_> = config
        .collections
        .iter()
        .filter(|c| c.description.is_some())
        .collect();

    if contexts.is_empty() {
        println!("No contexts configured");
        return Ok(());
    }

    println!("Contexts:");
    for collection in contexts {
        println!(
            "  {} — {}",
            collection.path.display(),
            collection.description.as_deref().unwrap_or("")
        );
    }

    Ok(())
}

/// Remove a context
fn remove_context(args: &ContextRemoveArgs, config: &mut Config) -> Result<()> {
    let path = shellexpand::tilde(&args.path).parse::<PathBuf>()?;

    let idx = config.collections.iter().position(|c| c.path == path);
    match idx {
        Some(i) => {
            // Clear description rather than removing the collection entirely
            config.collections[i].description = None;
            config.save()?;
            println!("Context removed for: {}", path.display());
        }
        None => {
            anyhow::bail!("No context found for path: {}", path.display());
        }
    }

    Ok(())
}
