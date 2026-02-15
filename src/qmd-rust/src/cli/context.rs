use crate::anel::AnelSpec;
use crate::cli::{ContextCommands, ContextAddArgs, ContextRemoveArgs};
use crate::config::Config;
use crate::store::Store;
use anyhow::Result;
use std::path::PathBuf;

/// Handle context commands
pub fn handle(
    cmd: &crate::cli::ContextArgs,
    config: &mut Config,
) -> Result<()> {
    // Handle --emit-spec: output ANEL specification and exit
    if cmd.emit_spec {
        let spec = AnelSpec::context();
        println!("{}", serde_json::to_string_pretty(&spec)?);
        return Ok(());
    }

    // Handle --dry-run: validate parameters without executing
    if cmd.dry_run {
        let action = match &cmd.command {
            ContextCommands::Add(args) => {
                println!("[DRY-RUN] Would execute context add with:");
                println!("  path: {:?}", args.path);
                println!("  description: {}", args.description);
                "add"
            }
            ContextCommands::List => {
                println!("[DRY-RUN] Would execute context list");
                "list"
            }
            ContextCommands::Rm(args) => {
                println!("[DRY-RUN] Would execute context rm with:");
                println!("  path: {}", args.path);
                "rm"
            }
        };
        println!("  action: {}", action);
        return Ok(());
    }

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

    let path_str = path.display().to_string();

    // Check if this path already has a context description
    let existing = config.collections.iter().position(|c| c.path == path);
    let collection_name = match existing {
        Some(i) => {
            config.collections[i].description = Some(args.description.clone());
            config.save()?;
            println!("Context updated:");
            config.collections[i].name.clone()
        }
        None => {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let collection = crate::config::CollectionConfig {
                name: name.clone(),
                path: path.clone(),
                pattern: Some("**/*".to_string()),
                description: Some(args.description.clone()),
            };
            config.collections.push(collection);
            config.save()?;
            println!("Context added:");
            name
        }
    };

    // Also persist to database
    if let Ok(store) = Store::new(config) {
        if let Err(e) = store.set_path_context(&collection_name, &path_str, &args.description) {
            log::warn!("Failed to persist context to database: {e}");
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
    let path_str = path.display().to_string();

    let idx = config.collections.iter().position(|c| c.path == path);
    match idx {
        Some(i) => {
            let collection_name = config.collections[i].name.clone();
            config.collections[i].description = None;
            config.save()?;

            // Also remove from database
            if let Ok(store) = Store::new(config) {
                if let Err(e) = store.remove_path_context(&collection_name, &path_str) {
                    log::warn!("Failed to remove context from database: {e}");
                }
            }

            println!("Context removed for: {}", path.display());
        }
        None => {
            anyhow::bail!("No context found for path: {}", path.display());
        }
    }

    Ok(())
}
