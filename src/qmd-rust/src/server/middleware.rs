// HTTP middleware

use axum::{
    body::Body,
    extract::Request,
    http::{header:: HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use serde::Serialize;

/// Rate limiter state
pub struct RateLimitState {
    requests: RwLock<HashMap<String, Vec<Instant>>>,
    max_requests: usize,
    window_secs: u64,
}

impl RateLimitState {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: RwLock::new(HashMap::new()),
            max_requests,
            window_secs,
        }
    }

    /// Check if request is allowed, returns (allowed, remaining, reset_secs)
    pub async fn check(&self, key: &str) -> (bool, usize, u64) {
        let mut requests = self.requests.write().await;
        let now = Instant::now();
        let window_start = now - Duration::from_secs(self.window_secs);

        // Get or create entry
        let times = requests.entry(key.to_string()).or_insert_with(Vec::new);

        // Remove old requests outside window
        times.retain(|&t| t > window_start);

        // Check limit
        let count = times.len();
        let allowed = count < self.max_requests;

        if allowed {
            times.push(now);
        }

        let remaining = self.max_requests.saturating_sub(count + if allowed { 0 } else { 1 });
        let reset_secs = self.window_secs;

        (allowed, remaining, reset_secs)
    }

    /// Clean up old entries (call periodically)
    pub async fn cleanup(&self) {
        let mut requests = self.requests.write().await;
        let window_start = Instant::now() - Duration::from_secs(self.window_secs * 2);
        requests.retain(|_, times| times.iter().any(|&t| t > window_start));
    }
}

pub type SharedRateLimitState = Arc<RateLimitState>;

/// Rate limit error response
#[derive(Serialize)]
struct RateLimitError {
    error: String,
    code: String,
    retry_after: u64,
}

/// Rate limiting middleware
pub async fn rate_limit_mw(
    state: SharedRateLimitState,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Extract client IP
    let client_ip = extract_client_ip(&request);

    // Check rate limit
    let (allowed, remaining, reset_secs) = state.check(&client_ip).await;

    if !allowed {
        tracing::warn!("Rate limit exceeded for {}", client_ip);

        let error = RateLimitError {
            error: "Rate limit exceeded".to_string(),
            code: "RATE_LIMIT_EXCEEDED".to_string(),
            retry_after: reset_secs,
        };

        let response = (
            StatusCode::TOO_MANY_REQUESTS,
            [("X-RateLimit-Remaining", "0"), ("X-RateLimit-Reset", &reset_secs.to_string())],
            Json(error),
        ).into_response();
        return response;
    }

    // Add rate limit headers to response
    let response = next.run(request).await;

    // Inject headers (using axum's response extension or just return as-is)
    let mut response = response;
    response.headers_mut().insert(
        "X-RateLimit-Remaining",
        remaining.to_string().parse().unwrap(),
    );
    response.headers_mut().insert(
        "X-RateLimit-Reset",
        reset_secs.to_string().parse().unwrap(),
    );

    response
}

/// Extract client IP from request
pub fn extract_client_ip(request: &Request<Body>) -> String {
    // Check X-Forwarded-For header first
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take first IP in chain
            return forwarded_str.split(',').next().unwrap_or("unknown").trim().to_string();
        }
    }

    // Check X-Real-IP header
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }

    // Fallback to connection info (would need Ext in actual implementation)
    "unknown".to_string()
}

/// API Key authentication state
pub struct AuthState {
    valid_keys: RwLock<HashMap<String, String>>, // key -> description
    whitelist_ips: RwLock<Vec<String>>,
}

impl AuthState {
    pub fn new(api_keys: Vec<(String, String)>, whitelist_ips: Vec<String>) -> Self {
        let valid_keys: HashMap<String, String> = api_keys.into_iter().collect();
        Self {
            valid_keys: RwLock::new(valid_keys),
            whitelist_ips: RwLock::new(whitelist_ips),
        }
    }

    /// Check if API key is valid
    pub async fn is_allowed(&self, api_key: Option<&str>, client_ip: &str) -> bool {
        // Check whitelist
        let whitelist = self.whitelist_ips.read().await;
        if whitelist.iter().any(|ip| ip == client_ip || ip == "*") {
            return true;
        }
        drop(whitelist);

        // Check API key
        if let Some(key) = api_key {
            let keys = self.valid_keys.read().await;
            return keys.contains_key(key);
        }

        false
    }
}

pub type SharedAuthState = Arc<AuthState>;

/// Auth error response
#[derive(Serialize)]
struct AuthError {
    error: String,
    code: String,
}

/// API Key authentication middleware
pub async fn auth_mw(
    state: SharedAuthState,
    request: Request<Body>,
    next: Next,
) -> Response {
    let client_ip = extract_client_ip(&request);

    // Extract API key from header
    let api_key = request
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok());

    // Check authentication
    if !state.is_allowed(api_key, &client_ip).await {
        tracing::warn!("Unauthorized request from {}", client_ip);

        let error = AuthError {
            error: "Authentication required".to_string(),
            code: "UNAUTHORIZED".to_string(),
        };

        return (
            StatusCode::UNAUTHORIZED,
            [("WWW-Authenticate", "ApiKey")],
            Json(error),
        ).into_response();
    }

    next.run(request).await
}

/// Request tracing middleware
pub async fn trace_request_mw(
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();

    tracing::debug!("→ {} {}", method, uri);

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    tracing::debug!(
        "← {} {} {} ({:?})",
        method,
        uri,
        status.as_u16(),
        duration
    );

    response
}
