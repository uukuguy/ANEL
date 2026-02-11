use crate::config::Config;
use anyhow::Result;
use std::fmt;
use std::path::PathBuf;

/// Common query expansion terms for knowledge base searches
const EXPANSION_TERMS: &[(&str, &[&str])] = &[
    ("how", &["how to", "guide", "tutorial"]),
    ("what", &["what is", "definition", "explanation"]),
    ("why", &["reason", "explanation", "purpose"]),
    ("config", &["configuration", "settings", "setup"]),
    ("install", &["installation", "setup", "deployment"]),
    ("error", &["error", "issue", "problem", "bug"]),
    ("api", &["api", "interface", "endpoint"]),
    ("doc", &["documentation", "docs", "guide"]),
];

/// LLM provider types
#[derive(Debug, Clone)]
pub enum LLMProvider {
    Local,
    Remote,
}

impl fmt::Display for LLMProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LLMProvider::Local => write!(f, "local"),
            LLMProvider::Remote => write!(f, "remote"),
        }
    }
}

/// Embedding result
#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    pub embeddings: Vec<Vec<f32>>,
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
    local_query_expander: Option<LocalQueryExpander>,
    remote_query_expander: Option<RemoteQueryExpander>,
}

impl Router {
    /// Create a new LLM router
    pub fn new(config: &Config) -> Result<Self> {
        let mut router = Self {
            config: config.clone(),
            local_embedder: None,
            remote_embedder: None,
            local_reranker: None,
            remote_reranker: None,
            local_query_expander: None,
            remote_query_expander: None,
        };

        // Initialize embedder models
        if let Some(ref models) = config.models.embed {
            if let Some(ref local) = models.local {
                router.local_embedder = Some(LocalEmbedder::new(local)?);
            }
            if let Some(ref remote) = models.remote {
                router.remote_embedder = Some(RemoteEmbedder::new(remote)?);
            }
        }

        // Initialize reranker models
        if let Some(ref models) = config.models.rerank {
            if let Some(ref local) = models.local {
                router.local_reranker = Some(LocalReranker::new(local)?);
            }
            if let Some(ref remote) = models.remote {
                router.remote_reranker = Some(RemoteReranker::new(remote)?);
            }
        }

        // Initialize query expansion models
        if let Some(ref models) = config.models.query_expansion {
            if let Some(ref local) = models.local {
                router.local_query_expander = Some(LocalQueryExpander::new(local)?);
            }
            if let Some(ref remote) = models.remote {
                router.remote_query_expander = Some(RemoteQueryExpander::new(remote)?);
            }
        }

        Ok(router)
    }

    /// Check if any embedder is available
    pub fn has_embedder(&self) -> bool {
        self.local_embedder.is_some() || self.remote_embedder.is_some()
    }

    /// Check if any reranker is available
    pub fn has_reranker(&self) -> bool {
        self.local_reranker.is_some() || self.remote_reranker.is_some()
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

        anyhow::bail!("No embedder available")
    }

    /// Synchronous embedding wrapper
    ///
    /// Uses tokio runtime to run async embed() from sync context.
    /// Useful for embedding queries in synchronous code paths.
    pub fn embed_sync(&self, text: &str) -> Result<EmbeddingResult> {
        let text_ref = &[text];
        tokio::runtime::Handle::current().block_on(async {
            self.embed(text_ref).await
        })
    }

    /// Synchronous reranking wrapper
    ///
    /// Uses tokio runtime to run async rerank() from sync context.
    pub fn rerank_sync(&self, query: &str, docs: &[crate::store::SearchResult]) -> Result<Vec<f32>> {
        tokio::runtime::Handle::current().block_on(async {
            self.rerank(query, docs).await
        })
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
    ///
    /// Generates multiple query variations to improve search recall:
    /// 1. Original query
    /// 2. Rule-based expansions (keywords, synonyms)
    /// 3. LLM-generated variations (if available)
    pub fn expand_query(&self, query: &str) -> Result<Vec<String>> {
        // Always include the original query
        let mut expansions = vec![query.to_string()];

        // Try local query expander first
        if let Some(ref local) = self.local_query_expander {
            match local.expand(query) {
                Ok(mut local_expansions) => {
                    expansions.append(&mut local_expansions);
                    log::info!("Local query expansion generated {} variants", local_expansions.len());
                }
                Err(e) => {
                    log::warn!("Local query expander failed: {}, trying remote", e);
                }
            }
        }

        // Fall back to remote if local is not available
        if expansions.len() == 1 {
            if let Some(ref remote) = self.remote_query_expander {
                match remote.expand(query) {
                    Ok(mut remote_expansions) => {
                        expansions.append(&mut remote_expansions);
                        log::info!("Remote query expansion generated {} variants", remote_expansions.len());
                    }
                    Err(e) => {
                        log::warn!("Remote query expander failed: {}", e);
                    }
                }
            }
        }

        // Limit the number of expansions to avoid excessive queries
        let max_expansions = 5;
        if expansions.len() > max_expansions {
            expansions.truncate(max_expansions);
        }

        // Remove duplicates while preserving order
        expansions.sort();
        expansions.dedup();

        Ok(expansions)
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

        // Check if model file exists
        if !model_path.exists() {
            log::warn!("Model file not found: {}. Embeddings will use fallback.", model_path.display());
        }

        Ok(Self {
            model_path,
            model_name: model_name.to_string(),
        })
    }

    pub fn model_name(&self) -> String {
        self.model_name.clone()
    }

    pub async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        log::info!("Local embedding with model: {} ({})", self.model_name, self.model_path.display());

        // Check if model exists, fallback to random if not
        if !self.model_path.exists() {
            log::warn!("Model not found, using random embeddings as fallback");
            let dim = 384;
            return Ok(texts.iter()
                .map(|_| (0..dim).map(|_| rand::random::<f32>()).collect())
                .collect());
        }

        #[cfg(feature = "llama-cpp")]
        {
            self.embed_with_llama_cpp(texts).await
        }

        #[cfg(not(feature = "llama-cpp"))]
        {
            log::warn!("llama-cpp feature not enabled, using random embeddings as fallback");
            let dim = 384;
            Ok(texts.iter()
                .map(|_| (0..dim).map(|_| rand::random::<f32>()).collect())
                .collect())
        }
    }

    #[cfg(feature = "llama-cpp")]
    async fn embed_with_llama_cpp(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        use llama_cpp_2::llama_backend::LlamaBackend;
        use llama_cpp_2::model::{LlamaModel, AddBos};
        use llama_cpp_2::model::params::LlamaModelParams;
        use llama_cpp_2::context::params::LlamaContextParams;
        use llama_cpp_2::llama_batch::LlamaBatch;

        // Initialize llama.cpp backend
        let backend = LlamaBackend::init()
            .map_err(|e| anyhow::anyhow!("Failed to initialize llama backend: {:?}", e))?;

        // Load model with GPU acceleration if available
        let model_params = LlamaModelParams::default()
            .with_n_gpu_layers(1000); // Offload all layers to GPU if available

        let model = LlamaModel::load_from_file(&backend, &self.model_path, &model_params)
            .map_err(|e| anyhow::anyhow!("Failed to load model: {:?}", e))?;

        log::info!("Model loaded: vocab={}, embd_dim={}", model.n_vocab(), model.n_embd());

        // Create context with embeddings enabled
        let ctx_params = LlamaContextParams::default()
            .with_embeddings(true)
            .with_n_threads_batch(8);

        let mut ctx = model.new_context(&backend, ctx_params)
            .map_err(|e| anyhow::anyhow!("Failed to create context: {:?}", e))?;

        let mut all_embeddings = Vec::new();

        // Process each text
        for text in texts {
            // Tokenize text
            let tokens = model.str_to_token(text, AddBos::Always)
                .map_err(|e| anyhow::anyhow!("Failed to tokenize: {:?}", e))?;

            // Create batch and process
            let mut batch = LlamaBatch::new(ctx.n_ctx() as usize, 1);
            batch.add_sequence(&tokens, 0, false)
                .map_err(|e| anyhow::anyhow!("Failed to add sequence: {:?}", e))?;

            ctx.decode(&mut batch)
                .map_err(|e| anyhow::anyhow!("Failed to decode: {:?}", e))?;

            // Get embeddings (mean pooling by default)
            let embeddings = ctx.embeddings_seq_ith(0)
                .map_err(|e| anyhow::anyhow!("Failed to get embeddings: {:?}", e))?;

            // Normalize embeddings for cosine similarity
            let normalized = Self::normalize_embedding(embeddings);
            all_embeddings.push(normalized);
        }

        log::info!("Generated {} embeddings with dimension {}", all_embeddings.len(), all_embeddings[0].len());

        Ok(all_embeddings)
    }

    /// Normalize embedding vector for cosine similarity search
    fn normalize_embedding(embedding: &[f32]) -> Vec<f32> {
        let magnitude: f32 = embedding.iter()
            .map(|&x| x * x)
            .sum::<f32>()
            .sqrt();

        if magnitude > 0.0 {
            embedding.iter().map(|&x| x / magnitude).collect()
        } else {
            embedding.to_vec()
        }
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

    pub async fn rerank(&self, _query: &str, docs: &[&str]) -> Result<Vec<f32>> {
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

    pub async fn rerank(&self, _query: &str, docs: &[&str]) -> Result<Vec<f32>> {
        log::info!("Remote reranking with model: {}", self.model);

        // Placeholder: return random scores
        Ok(docs.iter().map(|_| rand::random::<f32>()).collect())
    }
}

/// Query expander trait
pub trait QueryExpander {
    /// Expand a query into multiple variations
    fn expand(&self, query: &str) -> Result<Vec<String>>;
}

/// Local query expander (rule-based)
pub struct LocalQueryExpander {
    model_name: String,
}

impl LocalQueryExpander {
    pub fn new(model_name: &str) -> Result<Self> {
        Ok(Self {
            model_name: model_name.to_string(),
        })
    }
}

impl QueryExpander for LocalQueryExpander {
    fn expand(&self, query: &str) -> Result<Vec<String>> {
        log::info!("Local query expansion with model: {}", self.model_name);

        let mut expansions = Vec::new();
        let query_lower = query.to_lowercase();

        // Rule-based query expansion
        for (keyword, synonyms) in EXPANSION_TERMS {
            if query_lower.contains(keyword) {
                for synonym in *synonyms {
                    let expansion = query_lower.replace(keyword, synonym);
                    if expansion != query_lower && !expansions.contains(&expansion) {
                        expansions.push(expansion);
                    }
                }
            }
        }

        // If no rule-based expansions, try keyword-based expansion
        if expansions.is_empty() {
            let words: Vec<&str> = query.split_whitespace().collect();
            if words.len() > 1 {
                // Create phrase-based expansions
                for i in 0..words.len() {
                    let phrase: String = words[i..].join(" ");
                    if phrase != query && !expansions.contains(&phrase) {
                        expansions.push(phrase);
                    }
                }
            }
        }

        // Limit expansions
        let max_expansions = 3;
        if expansions.len() > max_expansions {
            expansions.truncate(max_expansions);
        }

        Ok(expansions)
    }
}

/// Remote query expander (LLM-based)
pub struct RemoteQueryExpander {
    api_key: String,
    base_url: String,
    model: String,
}

impl RemoteQueryExpander {
    pub fn new(model: &str) -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))?;

        let (base_url, model) = if model.starts_with("gpt-") {
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
}

impl QueryExpander for RemoteQueryExpander {
    fn expand(&self, query: &str) -> Result<Vec<String>> {
        log::info!("Remote query expansion with model: {}", self.model);

        // Placeholder: generate simple variations
        // In a real implementation, this would call an LLM API
        let mut expansions = Vec::new();

        // Generate a "what is" variation if query doesn't start with it
        if !query.to_lowercase().starts_with("what") {
            expansions.push(format!("what is {}", query));
        }

        // Generate a "how to" variation
        if !query.to_lowercase().starts_with("how") {
            expansions.push(format!("how to {}", query));
        }

        // Generate an explanation variation
        expansions.push(format!("explain {}", query));

        Ok(expansions)
    }
}
