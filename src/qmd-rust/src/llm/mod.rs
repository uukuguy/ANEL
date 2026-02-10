use crate::config::Config;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// LLM provider types
#[derive(Debug, Clone)]
pub enum LLMProvider {
    Local,
    Remote,
}

/// Embedding result
#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    pub embeddings: Vec<Vec<f32>>,
    pub provider: LLMProvider,
    pub model: String,
}

/// Reranking result
#[derive(Debug, Clone)]
pub struct RerankResult {
    pub scores: Vec<f32>,
    pub provider: LLMProvider,
    pub model: String,
}

/// LLM Router - routes requests to local or remote providers
pub struct Router {
    config: Config,
    local_embedder: Option<LocalEmbedder>,
    remote_embedder: Option<RemoteEmbedder>,
    local_reranker: Option<LocalReranker>,
    remote_reranker: Option<RemoteReranker>,
}

impl Router {
    /// Create a new LLM router
    pub fn new(config: &Config) -> Result<Self> {
        let router = Self {
            config: config.clone(),
            local_embedder: None,
            remote_embedder: None,
            local_reranker: None,
            remote_reranker: None,
        };

        // Initialize local models if configured
        if let Some(ref models) = config.models.embed {
            if let Some(ref local) = models.local {
                router.local_embedder = Some(LocalEmbedder::new(local)?);
            }
        }

        // Initialize remote models if configured
        if let Some(ref models) = config.models.embed {
            if let Some(ref remote) = models.remote {
                router.remote_embedder = Some(RemoteEmbedder::new(remote)?);
            }
        }

        Ok(router)
    }

    /// Generate embeddings
    pub async fn embed(&self, texts: &[&str]) -> Result<EmbeddingResult> {
        // Try local first, then remote
        if let Some(ref local) = self.local_embedder {
            match local.embed(texts).await {
                Ok(embeddings) => {
                    return Ok(EmbeddingResult {
                        embeddings,
                        provider: LLMProvider::Local,
                        model: local.model_name(),
                    });
                }
                Err(e) => {
                    log::warn!("Local embedder failed: {}, trying remote", e);
                }
            }
        }

        if let Some(ref remote) = self.remote_embedder {
            match remote.embed(texts).await {
                Ok(embeddings) => {
                    return Ok(EmbeddingResult {
                        embeddings,
                        provider: LLMProvider::Remote,
                        model: remote.model_name(),
                    });
                }
                Err(e) => {
                    log::error!("Remote embedder failed: {}", e);
                }
            }
        }

        anyhow::bail!("No embedder available");
    }

    /// Rerank documents
    pub async fn rerank(&self, query: &str, docs: &[crate::store::SearchResult]) -> Result<Vec<f32>> {
        let doc_texts: Vec<&str> = docs.iter().map(|d| d.title.as_str()).collect();

        // Try local first
        if let Some(ref local) = self.local_reranker {
            match local.rerank(query, &doc_texts).await {
                Ok(scores) => return Ok(scores),
                Err(e) => {
                    log::warn!("Local reranker failed: {}, trying remote", e);
                }
            }
        }

        if let Some(ref remote) = self.remote_reranker {
            match remote.rerank(query, &doc_texts).await {
                Ok(scores) => return Ok(scores),
                Err(e) => {
                    log::error!("Remote reranker failed: {}", e);
                }
            }
        }

        anyhow::bail!("No reranker available");
    }

    /// Expand query using LLM
    pub fn expand_query(&self, query: &str) -> Result<Vec<String>> {
        // TODO: Implement query expansion using local or remote LLM
        Ok(vec![query.to_string()])
    }
}

/// Local embedding provider (llama.cpp)
pub struct LocalEmbedder {
    model_path: PathBuf,
    model_name: String,
}

impl LocalEmbedder {
    pub fn new(model_name: &str) -> Result<Self> {
        let cache_path = shellexpand::tilde("~/.cache/qmd/models").parse::<PathBuf>()?;
        let model_path = cache_path.join(format!("{}.gguf", model_name));

        Ok(Self {
            model_path,
            model_name: model_name.to_string(),
        })
    }

    pub fn model_name(&self) -> String {
        self.model_name.clone()
    }

    pub async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        // TODO: Implement local embedding using llama.cpp
        // This would use the llama-cpp-python bindings or a Rust-native solution
        log::info!("Local embedding with model: {}", self.model_name);

        // Placeholder: return random embeddings
        let dim = 384;
        Ok(texts.iter()
            .map(|_| (0..dim).map(|_| rand::random::<f32>()).collect())
            .collect())
    }
}

/// Remote embedding provider (OpenAI, Anthropic, etc.)
pub struct RemoteEmbedder {
    api_key: String,
    base_url: String,
    model: String,
}

impl RemoteEmbedder {
    pub fn new(model: &str) -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))?;

        let (base_url, model) = if model.starts_with("text-embedding-") {
            ("https://api.openai.com/v1".to_string(), model.to_string())
        } else {
            // Default to OpenAI-compatible API
            ("https://api.openai.com/v1".to_string(), model.to_string())
        };

        Ok(Self {
            api_key,
            base_url,
            model,
        })
    }

    pub fn model_name(&self) -> String {
        self.model.clone()
    }

    pub async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        log::info!("Remote embedding with model: {}", self.model);

        // Placeholder: return random embeddings
        let dim = 1536; // OpenAI ada-002 dimension
        Ok(texts.iter()
            .map(|_| (0..dim).map(|_| rand::random::<f32>()).collect())
            .collect())
    }
}

/// Local reranking provider
pub struct LocalReranker {
    model_path: PathBuf,
    model_name: String,
}

impl LocalReranker {
    pub fn new(model_name: &str) -> Result<Self> {
        let cache_path = shellexpand::tilde("~/.cache/qmd/models").parse::<PathBuf>()?;
        let model_path = cache_path.join(format!("{}.gguf", model_name));

        Ok(Self {
            model_path,
            model_name: model_name.to_string(),
        })
    }

    pub async fn rerank(&self, query: &str, docs: &[&str]) -> Result<Vec<f32>> {
        log::info!("Local reranking with model: {}", self.model_name);

        // Placeholder: return random scores
        Ok(docs.iter().map(|_| rand::random::<f32>()).collect())
    }
}

/// Remote reranking provider
pub struct RemoteReranker {
    api_key: String,
    base_url: String,
    model: String,
}

impl RemoteReranker {
    pub fn new(model: &str) -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))?;

        let base_url = "https://api.openai.com/v1".to_string();

        Ok(Self {
            api_key,
            base_url,
            model: model.to_string(),
        })
    }

    pub async fn rerank(&self, query: &str, docs: &[&str]) -> Result<Vec<f32>> {
        log::info!("Remote reranking with model: {}", self.model);

        // Placeholder: return random scores
        Ok(docs.iter().map(|_| rand::random::<f32>()).collect())
    }
}
