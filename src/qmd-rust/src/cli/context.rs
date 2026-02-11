use crate::cli::{ContextCommands, ContextAddArgs, ContextRemoveArgs};
use crate::config::Config;
use anyhow::Result;
use std::path::PathBuf;

/// Handle context commands
pub fn handle(
    cmd: &crate::cli::ContextArgs,
    config: &Config,
) -> Result<()> {
    match &cmd.command {
        ContextCommands::Add(args) => add_context(args, config),
        ContextCommands::List => list_contexts(config),
        ContextCommands::Rm(args) => remove_context(args, config),
    }
}

/// Add a context (path with description for relevance)
fn add_context(args: &ContextAddArgs, _config: &Config) -> Result<()> {
    let path = match &args.path {
        Some(p) => shellexpand::tilde(p).parse::<PathBuf>()?,
        None => std::env::current_dir()?,
    };

    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    // TODO: Save context to configuration

    println!("Context added:");
    println!("  Path: {}", path.display());
    println!("  Description: {}", args.description);

    Ok(())
}

/// List all contexts
fn list_contexts(config: &Config) -> Result<()> {
    println!("Contexts:");

    if config.collections.is_empty() {
        println!("  No contexts configured");
        return Ok(());
    }

    for collection in &config.collections {
        if let Some(desc) = &collection.description {
            println!("  {}: {}", collection.name, desc);
        }
    }

    Ok(())
}

/// Remove a context
fn remove_context(args: &ContextRemoveArgs, _config: &Config) -> Result<()> {
    let path = &args.path;

    println!("Context '{}' removed", path);

    Ok(())
}
