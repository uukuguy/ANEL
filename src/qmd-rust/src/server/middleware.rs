// HTTP middleware

use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;

/// Request tracing middleware
pub async fn trace_request_mw(
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

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
