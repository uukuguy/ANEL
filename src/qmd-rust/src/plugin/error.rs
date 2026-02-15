// Plugin system error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin load failed: {0}")]
    LoadFailed(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Plugin already exists: {0}")]
    AlreadyExists(String),

    #[error("Plugin initialization failed: {0}")]
    InitFailed(String),

    #[error("Plugin execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid plugin: {0}")]
    InvalidPlugin(String),

    #[error("Wasm runtime error: {0}")]
    RuntimeError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, PluginError>;
