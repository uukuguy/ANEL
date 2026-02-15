// Distributed tracing for QMD HTTP Server

use uuid::Uuid;

/// Tracing configuration
#[derive(Clone)]
pub struct Tracing {
    service_name: String,
}

impl Tracing {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }

    /// Get service name
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// Generate a new request ID
    pub fn generate_request_id() -> String {
        Uuid::new_v4().to_string()
    }
}

impl Default for Tracing {
    fn default() -> Self {
        Self::new("qmd-server")
    }
}

/// Request ID extractor for handlers
pub struct RequestId(pub String);
