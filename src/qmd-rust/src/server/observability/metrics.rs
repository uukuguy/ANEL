// Prometheus metrics for QMD HTTP Server

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Metrics collector for QMD server
#[derive(Clone)]
pub struct Metrics {
    // Request metrics
    requests_total: Arc<AtomicU64>,
    requests_in_flight: Arc<AtomicU64>,
    request_duration_sum: Arc<AtomicU64>,
    request_duration_count: Arc<AtomicU64>,

    // Business metrics
    search_total: Arc<AtomicU64>,
    vsearch_total: Arc<AtomicU64>,
    query_total: Arc<AtomicU64>,

    // Error metrics
    errors_total: Arc<AtomicU64>,

    // LLM metrics
    llm_embeddings_total: Arc<AtomicU64>,
    llm_rerank_total: Arc<AtomicU64>,
    llm_errors: Arc<AtomicU64>,
}

impl Metrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self {
            requests_total: Arc::new(AtomicU64::new(0)),
            requests_in_flight: Arc::new(AtomicU64::new(0)),
            request_duration_sum: Arc::new(AtomicU64::new(0)),
            request_duration_count: Arc::new(AtomicU64::new(0)),
            search_total: Arc::new(AtomicU64::new(0)),
            vsearch_total: Arc::new(AtomicU64::new(0)),
            query_total: Arc::new(AtomicU64::new(0)),
            errors_total: Arc::new(AtomicU64::new(0)),
            llm_embeddings_total: Arc::new(AtomicU64::new(0)),
            llm_rerank_total: Arc::new(AtomicU64::new(0)),
            llm_errors: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Increment total requests counter
    pub fn inc_requests_total(&self) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment requests in flight
    pub fn inc_requests_in_flight(&self) {
        self.requests_in_flight.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement requests in flight
    pub fn dec_requests_in_flight(&self) {
        self.requests_in_flight.fetch_sub(1, Ordering::Relaxed);
    }

    /// Record request duration (in milliseconds)
    pub fn record_request_duration(&self, duration_ms: u64) {
        self.request_duration_sum.fetch_add(duration_ms, Ordering::Relaxed);
        self.request_duration_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get request duration average in milliseconds
    pub fn get_request_duration_avg_ms(&self) -> f64 {
        let sum = self.request_duration_sum.load(Ordering::Relaxed);
        let count = self.request_duration_count.load(Ordering::Relaxed);
        if count > 0 {
            sum as f64 / count as f64
        } else {
            0.0
        }
    }

    /// Increment search counter
    pub fn inc_search(&self) {
        self.search_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment vsearch counter
    pub fn inc_vsearch(&self) {
        self.vsearch_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment query counter
    pub fn inc_query(&self) {
        self.query_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment error counter
    pub fn inc_errors(&self) {
        self.errors_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment LLM embeddings counter
    pub fn inc_llm_embeddings(&self) {
        self.llm_embeddings_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment LLM rerank counter
    pub fn inc_llm_rerank(&self) {
        self.llm_rerank_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment LLM errors
    pub fn inc_llm_errors(&self) {
        self.llm_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current values
    pub fn get_requests_total(&self) -> u64 {
        self.requests_total.load(Ordering::Relaxed)
    }

    pub fn get_requests_in_flight(&self) -> u64 {
        self.requests_in_flight.load(Ordering::Relaxed)
    }

    pub fn get_search_total(&self) -> u64 {
        self.search_total.load(Ordering::Relaxed)
    }

    pub fn get_vsearch_total(&self) -> u64 {
        self.vsearch_total.load(Ordering::Relaxed)
    }

    pub fn get_query_total(&self) -> u64 {
        self.query_total.load(Ordering::Relaxed)
    }

    pub fn get_errors_total(&self) -> u64 {
        self.errors_total.load(Ordering::Relaxed)
    }

    pub fn get_llm_embeddings_total(&self) -> u64 {
        self.llm_embeddings_total.load(Ordering::Relaxed)
    }

    pub fn get_llm_rerank_total(&self) -> u64 {
        self.llm_rerank_total.load(Ordering::Relaxed)
    }

    pub fn get_llm_errors(&self) -> u64 {
        self.llm_errors.load(Ordering::Relaxed)
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}
