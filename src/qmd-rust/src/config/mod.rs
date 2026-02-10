use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

const DEFAULT_CONFIG_PATH: &str = "~/.config/qmd/index.yaml";
const DEFAULT_CACHE_PATH: &str = "~/.cache/qmd";

/// BM25 backend type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BM25Backend {
    #[serde(rename = "sqlite_fts5")]
    SqliteFts5,
    #[serde(rename = "lancedb")]
    LanceDb,
}

impl Default for BM25Backend {
    fn default() -> Self {
        Self::SqliteFts5
    }
}

/// Vector backend type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorBackend {
    #[serde(rename = "qmd_builtin")]
    QmdBuiltin,
    #[serde(rename = "lancedb")]
    LanceDb,
}

impl Default for VectorBackend {
    fn default() -> Self {
        Self::QmdBuiltin
    }
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
            let config: Config = serde_yaml::from_str(&content)?;

            // Set defaults for missing values
            let config = Config {
                bm25: config.bm25.unwrap_or_default(),
                vector: config.vector.unwrap_or_default(),
                collections: config.collections.unwrap_or_default(),
                models: config.models.unwrap_or_default(),
                cache_path: config.cache_path.unwrap_or_else(default_cache_path),
            };

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
