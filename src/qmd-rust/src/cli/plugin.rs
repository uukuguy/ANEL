// Plugin CLI commands
// Provides plugin management commands: install, list, remove, info

use crate::cli::{Cli, PluginArgs, PluginCommands};
use crate::config::Config;
use crate::plugin::{PluginInfo, PluginManager};
use anyhow::Result;
use std::path::PathBuf;

/// Handle plugin subcommands
pub fn handle_plugin(args: &PluginArgs, config: &Config) -> Result<()> {
    let plugins_dir = get_plugins_dir(config);
    let manager = PluginManager::new(&plugins_dir)?;

    match &args.command {
        PluginCommands::List => list_plugins(&manager),
        PluginCommands::Install { path, name } => install_plugin(&manager, path, name.as_deref()),
        PluginCommands::Remove { name } => remove_plugin(&manager, name),
        PluginCommands::Info { name } => plugin_info(&manager, name),
        PluginCommands::Dir => show_plugins_dir(&manager),
    }
}

/// List all available plugins
fn list_plugins(manager: &PluginManager) -> Result<()> {
    let available = manager.list_available_plugins()?;
    let loaded = manager.list_plugins();

    println!("Available plugins (in {}):", manager.plugins_dir().display());
    if available.is_empty() {
        println!("  (no plugins found)");
    } else {
        for plugin in &available {
            let status = if manager.is_loaded(&plugin.name) {
                "[loaded]"
            } else {
                ""
            };
            println!("  {} {} - v{}", plugin.name, status, plugin.version);
            if !plugin.description.is_empty() {
                println!("    {}", plugin.description);
            }
        }
    }

    println!("\nLoaded plugins:");
    if loaded.is_empty() {
        println!("  (none)");
    } else {
        for plugin in &loaded {
            println!("  {} - v{}", plugin.name, plugin.version);
        }
    }

    Ok(())
}

/// Install a plugin
fn install_plugin(manager: &PluginManager, path: &str, name: Option<&str>) -> Result<()> {
    let path = PathBuf::from(path);
    let plugin_name = name.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
    });

    println!("Installing plugin: {} from {}", plugin_name, path.display());
    manager.load_plugin(plugin_name, &path)?;
    println!("Plugin installed successfully!");

    Ok(())
}

/// Remove a plugin
fn remove_plugin(manager: &PluginManager, name: &str) -> Result<()> {
    if !manager.is_loaded(name) {
        anyhow::bail!("Plugin '{}' is not loaded", name);
    }

    manager.unload_plugin(name)?;
    println!("Plugin '{}' unloaded successfully!", name);

    Ok(())
}

/// Show plugin info
fn plugin_info(manager: &PluginManager, name: &str) -> Result<()> {
    let info = manager.get_plugin_info(name)?;
    println!("Plugin: {}", info.name);
    println!("  Version: {}", info.version);
    println!("  Path: {}", info.path.display());
    println!("  Author: {}", info.author);
    println!("  Description: {}", info.description);

    Ok(())
}

/// Show plugins directory
fn show_plugins_dir(manager: &PluginManager) -> Result<()> {
    println!("{}", manager.plugins_dir().display());
    Ok(())
}

/// Get plugins directory from config
fn get_plugins_dir(config: &Config) -> PathBuf {
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("~/.cache"))
        .join("qmd")
        .join("plugins");
    cache_dir
}
