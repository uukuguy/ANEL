use crate::cli::{build_cli, Commands};
use crate::config::Config;
use anyhow::{Context, Result};
use log::info;

mod cli;
mod config;
mod formatter;
mod llm;
mod mcp;
mod store;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Load configuration
    let config = Config::load().context("Failed to load configuration")?;

    info!("Configuration loaded successfully");
    info!("BM25 backend: {:?}", config.bm25_backend);
    info!("Vector backend: {:?}", config.vector_backend);

    // Build and parse CLI
    let matches = build_cli().get_matches();

    // Dispatch commands
    match matches.subcommand() {
        Some((Commands::Collection(cmd), _)) => {
            crate::cli::collection::handle(cmd, &config)?;
        }
        Some((Commands::Context(cmd), _)) => {
            crate::cli::context::handle(cmd, &config)?;
        }
        Some((Commands::Get(cmd), _)) => {
            crate::cli::get::handle(cmd, &config)?;
        }
        Some((Commands::MultiGet(cmd), _)) => {
            crate::cli::multi_get::handle(cmd, &config)?;
        }
        Some((Commands::Search(cmd), _)) => {
            let store = store::Store::new(&config)?;
            crate::cli::search::handle(cmd, &store)?;
        }
        Some((Commands::Vsearch(cmd), _)) => {
            let store = store::Store::new(&config)?;
            crate::cli::vsearch::handle(cmd, &store)?;
        }
        Some((Commands::Query(cmd), _)) => {
            let store = store::Store::new(&config)?;
            let llm = llm::Router::new(&config)?;
            crate::cli::query::handle(cmd, &store, &llm)?;
        }
        Some((Commands::Embed(cmd), _)) => {
            let store = store::Store::new(&config)?;
            let llm = llm::Router::new(&config)?;
            crate::cli::embed::handle(cmd, &store, &llm)?;
        }
        Some((Commands::Update(cmd), _)) => {
            let store = store::Store::new(&config)?;
            crate::cli::update::handle(cmd, &store)?;
        }
        Some((Commands::Status(cmd), _)) => {
            let store = store::Store::new(&config)?;
            crate::cli::status::handle(cmd, &store)?;
        }
        Some((Commands::Cleanup(cmd), _)) => {
            let store = store::Store::new(&config)?;
            crate::cli::cleanup::handle(cmd, &store)?;
        }
        Some((Commands::Mcp(cmd), _)) => {
            let store = store::Store::new(&config)?;
            let llm = llm::Router::new(&config)?;
            crate::mcp::run_server(cmd, &store, &llm)?;
        }
        Some((Commands::Agent(cmd), _)) => {
            let store = store::Store::new(&config)?;
            let llm = llm::Router::new(&config)?;
            crate::cli::agent::handle(cmd, &store, &llm)?;
        }
        None => {
            // No subcommand, show help
            print!("{}", matches.clone().usage());
        }
    }

    Ok(())
}
