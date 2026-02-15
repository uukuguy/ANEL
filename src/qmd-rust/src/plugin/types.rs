// Plugin system types

use serde::{Deserialize, Serialize};

/// Plugin trait for custom scorer
pub trait Scorer: Send + Sync {
    fn score(&self, query: &str, title: &str, body: &str) -> f32;
}

/// Plugin trait for custom filter
pub trait Filter: Send + Sync {
    fn filter(&self, title: &str, body: &str) -> bool;
}

/// Plugin trait for custom transform
pub trait Transform: Send + Sync {
    fn transform(&self, title: &str, body: &str) -> Result<(String, String, Vec<(String, String)>), String>;
}

/// Plugin trait for query preprocessing
pub trait Preprocessor: Send + Sync {
    fn preprocess(&self, query: &str) -> String;
}

/// Plugin trait for results postprocessing
pub trait Postprocessor: Send + Sync {
    fn postprocess(&self, results: &mut [SearchResult]);
}

/// Search result structure for plugin processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub title: String,
    pub body: String,
    pub score: f32,
}

/// Transform result from plugin
#[derive(Debug, Clone)]
pub struct TransformResult {
    pub title: String,
    pub body: String,
    pub metadata: Vec<(String, String)>,
}

/// Key-value pair for metadata
#[derive(Debug, Clone)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}
