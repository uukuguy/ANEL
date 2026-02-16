use crate::cli::Commands;
use crate::config::Config;
use anyhow::{Context, Result};
use clap::Parser;
use log::info;

mod anel;
mod cli;
mod config;
mod formatter;
mod llm;
mod mcp;
mod plugin;
mod server;
mod store;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Load configuration
    let mut config = Config::load().context("Failed to load configuration")?;

    info!("Configuration loaded successfully");
    info!("BM25 backend: {:?}", config.bm25.backend);
    info!("Vector backend: {:?}", config.vector.backend);

    // Parse CLI arguments
    let cli = cli::Cli::parse();

    // Dispatch commands
    match &cli.command {
        Commands::Collection(cmd) => {
            crate::cli::collection::handle(cmd, &mut config)?;
        }
        Commands::Ls(cmd) => {
            crate::cli::ls::handle(cmd, &config)?;
        }
        Commands::Context(cmd) => {
            crate::cli::context::handle(cmd, &mut config)?;
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
            let llm = llm::Router::new(&config)?;
            crate::cli::vsearch::handle(cmd, &store, &llm)?;
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
        Commands::Mcp(cmd) => {
            mcp::run_server(cmd, &config)?;
        }
        Commands::Server(cmd) => {
            // Parse API keys from comma-separated string
            let api_keys: Vec<(String, String)> = cmd.api_keys
                .as_ref()
                .map(|s| {
                    s.split(',')
                        .map(|k| (k.trim().to_string(), "default".to_string()))
                        .collect()
                })
                .unwrap_or_default();

            // Parse whitelist IPs from comma-separated string
            let whitelist_ips: Vec<String> = cmd.whitelist_ips
                .as_ref()
                .map(|s| {
                    s.split(',')
                        .map(|ip| ip.trim().to_string())
                        .collect()
                })
                .unwrap_or_default();

            let server_config = server::ServerConfig {
                host: cmd.host.clone(),
                port: cmd.port,
                workers: cmd.workers,
                rate_limit_max: 100,
                rate_limit_window_secs: 60,
                auth_enabled: cmd.auth,
                api_keys,
                whitelist_ips,
            };
            server::run_server(&server_config, &config)?;
        }
        Commands::Agent(cmd) => {
            let store = store::Store::new(&config)?;
            let llm = llm::Router::new(&config)?;
            crate::cli::agent::handle(cmd, &store, &llm)?;
        }
        Commands::Plugin(cmd) => {
            crate::cli::plugin::handle_plugin(cmd, &config)?;
        }
    }

    Ok(())
}
