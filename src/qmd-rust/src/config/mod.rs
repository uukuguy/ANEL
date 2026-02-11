use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use log::info;

const DEFAULT_CONFIG_PATH: &str = "~/.config/qmd/index.yaml";
const DEFAULT_CACHE_PATH: &str = "~/.cache/qmd";

/// BM25 backend type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum BM25Backend {
    #[serde(rename = "sqlite_fts5")]
    #[default]
    SqliteFts5,
    #[serde(rename = "lancedb")]
    LanceDb,
}

/// Vector backend type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum VectorBackend {
    #[serde(rename = "qmd_builtin")]
    #[default]
    QmdBuiltin,
    #[serde(rename = "lancedb")]
    LanceDb,
}

/// Collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    pub name: String,
    pub path: PathBuf,
    pub pattern: Option<String>,
    pub description: Option<String>,
}

/// LLM model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMModelConfig {
    pub local: Option<String>,
    pub remote: Option<String>,
}

/// Configuration for LLM models
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelsConfig {
    pub embed: Option<LLMModelConfig>,
    pub rerank: Option<LLMModelConfig>,
    pub query_expansion: Option<LLMModelConfig>,
}

/// Main configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// BM25 backend configuration
    #[serde(default)]
    pub bm25: BM25BackendConfig,

    /// Vector backend configuration
    #[serde(default)]
    pub vector: VectorBackendConfig,

    /// Collections configuration
    #[serde(default)]
    pub collections: Vec<CollectionConfig>,

    /// LLM models configuration
    #[serde(default)]
    pub models: ModelsConfig,

    /// Cache directory
    #[serde(default = "default_cache_path")]
    pub cache_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BM25BackendConfig {
    #[serde(default)]
    pub backend: BM25Backend,
}

impl Default for BM25BackendConfig {
    fn default() -> Self {
        Self {
            backend: BM25Backend::SqliteFts5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorBackendConfig {
    #[serde(default)]
    pub backend: VectorBackend,
    #[serde(default)]
    pub model: String,
}

impl Default for VectorBackendConfig {
    fn default() -> Self {
        Self {
            backend: VectorBackend::QmdBuiltin,
            model: "embeddinggemma-300M".to_string(),
        }
    }
}

fn default_cache_path() -> PathBuf {
    shellexpand::tilde(DEFAULT_CACHE_PATH).parse().unwrap()
}

impl Config {
    /// Load configuration from default path or create default
    pub fn load() -> Result<Self, anyhow::Error> {
        let config_path = expand_path(DEFAULT_CONFIG_PATH);

        if config_path.exists() {
            info!("Loading configuration from: {:?}", config_path);
            let content = fs::read_to_string(&config_path)?;
            // serde's #[serde(default)] handles all defaults during deserialization
            let mut config: Config = serde_yaml::from_str(&content)?;

            // Expand tilde paths in configuration
            config.cache_path = expand_path(&config.cache_path.to_string_lossy());

            for collection in &mut config.collections {
                collection.path = expand_path(&collection.path.to_string_lossy());
            }

            Ok(config)
        } else {
            info!("Configuration not found, using defaults");
            Ok(Self::default())
        }
    }

    /// Save configuration to default path
    pub fn save(&self) -> Result<(), anyhow::Error> {
        let config_path = expand_path(DEFAULT_CONFIG_PATH);

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_yaml::to_string(self)?;
        fs::write(&config_path, content)?;

        info!("Configuration saved to: {:?}", config_path);
        Ok(())
    }

    /// Get cache directory for a specific collection
    pub fn cache_dir_for(&self, collection: &str) -> PathBuf {
        let mut path = self.cache_path.clone();
        path.push(collection);
        path
    }

    /// Get database path for a collection
    pub fn db_path_for(&self, collection: &str) -> PathBuf {
        let mut path = self.cache_dir_for(collection);
        path.push("index.db");
        path
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bm25: BM25BackendConfig::default(),
            vector: VectorBackendConfig::default(),
            collections: Vec::new(),
            models: ModelsConfig::default(),
            cache_path: default_cache_path(),
        }
    }
}

fn expand_path(path: &str) -> PathBuf {
    shellexpand::tilde(path).parse().unwrap()
}
