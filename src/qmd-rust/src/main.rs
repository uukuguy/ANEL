use crate::cli::Commands;
use crate::config::Config;
use anyhow::{Context, Result};
use clap::Parser;
use log::info;

mod cli;
mod config;
mod formatter;
mod llm;
// mod mcp;  // Temporarily disabled due to mcp-sdk dependency issues
mod store;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Load configuration
    let config = Config::load().context("Failed to load configuration")?;

    info!("Configuration loaded successfully");
    info!("BM25 backend: {:?}", config.bm25.backend);
    info!("Vector backend: {:?}", config.vector.backend);

    // Parse CLI arguments
    let cli = cli::Cli::parse();

    // Dispatch commands
    match &cli.command {
        Commands::Collection(cmd) => {
            crate::cli::collection::handle(cmd, &config)?;
        }
        Commands::Context(cmd) => {
            crate::cli::context::handle(cmd, &config)?;
        }
        Commands::Get(cmd) => {
            crate::cli::get::handle(cmd, &config)?;
        }
        Commands::MultiGet(cmd) => {
            crate::cli::multi_get::handle(cmd, &config)?;
        }
        Commands::Search(cmd) => {
            let store = store::Store::new(&config)?;
            crate::cli::search::handle(cmd, &store)?;
        }
        Commands::Vsearch(cmd) => {
            let store = store::Store::new(&config)?;
            crate::cli::vsearch::handle(cmd, &store)?;
        }
        Commands::Query(cmd) => {
            let store = store::Store::new(&config)?;
            let llm = llm::Router::new(&config)?;
            crate::cli::query::handle(cmd, &store, &llm)?;
        }
        Commands::Embed(cmd) => {
            let store = store::Store::new(&config)?;
            let llm = llm::Router::new(&config)?;
            crate::cli::embed::handle(cmd, &store, &llm)?;
        }
        Commands::Update(cmd) => {
            let store = store::Store::new(&config)?;
            crate::cli::update::handle(cmd, &store)?;
        }
        Commands::Status(cmd) => {
            let store = store::Store::new(&config)?;
            crate::cli::status::handle(cmd, &store)?;
        }
        Commands::Cleanup(cmd) => {
            let store = store::Store::new(&config)?;
            crate::cli::cleanup::handle(cmd, &store)?;
        }
        Commands::Mcp(_cmd) => {
            log::warn!("MCP server is temporarily disabled due to dependency issues");
            log::info!("To re-enable, uncomment mcp-sdk in Cargo.toml and fix API compatibility");
        }
        Commands::Agent(cmd) => {
            let store = store::Store::new(&config)?;
            let llm = llm::Router::new(&config)?;
            crate::cli::agent::handle(cmd, &store, &llm)?;
        }
    }

    Ok(())
}
