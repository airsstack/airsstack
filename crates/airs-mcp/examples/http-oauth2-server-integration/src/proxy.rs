//! Smart Proxy Server Implementation
//!
//! This module implements the three-server proxy architecture required for MCP Inspector compatibility.
//! The proxy server routes requests intelligently:
//! - OAuth2 discovery and authorization requests → Custom Routes Server (port 3003)
//! - MCP protocol requests → Main MCP Server (port 3001)
//!
//! This solves the critical MCP Inspector requirement that OAuth2 discovery endpoints
//! must be accessible on the same port as the MCP endpoint.

// Layer 1: Standard library imports
use std::time::Instant;

// Layer 2: Third-party crate imports
use axum::{
    body::Body,
    extract::{Request, State},
    http::{Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use reqwest::Client;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{debug, error, info};

/// Proxy server state for routing and request forwarding
#[derive(Clone)]
pub struct ProxyState {
    /// HTTP client for forwarding requests
    pub client: Client,
    /// MCP server URL (port 3001)
    pub mcp_server_url: String,
    /// Custom routes server URL (port 3003)
    pub custom_routes_url: String,
    /// Server start time for uptime tracking
    #[allow(dead_code)]
    pub start_time: Instant,
}

impl ProxyState {
    /// Create new proxy state with server URLs
    pub fn new(mcp_server_url: String, custom_routes_url: String) -> Self {
        Self {
            client: Client::new(),
            mcp_server_url,
            custom_routes_url,
            start_time: Instant::now(),
        }
    }
}

/// Smart proxy handler that routes requests between MCP and custom routes servers
pub async fn proxy_handler(
    State(state): State<ProxyState>,
    request: Request,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path();

    info!(
        method = %method,
        path = %path,
        query = ?uri.query(),
        "=== PROXY: INCOMING REQUEST ==="
    );

    // Determine target server based on request path
    let target_url = if path.starts_with("/mcp") {
        // Route MCP protocol requests to main MCP server
        debug!(path = %path, target = %state.mcp_server_url, "Routing to MCP server");
        &state.mcp_server_url
    } else {
        // Route OAuth2 discovery, authorization, and dev tools to custom routes server
        debug!(path = %path, target = %state.custom_routes_url, "Routing to custom routes server");
        &state.custom_routes_url
    };

    // Build the target URL
    let target_uri = format!(
        "{}{}",
        target_url,
        uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("")
    );

    debug!(
        original_uri = %uri,
        target_uri = %target_uri,
        "=== PROXY: REQUEST ROUTING ==="
    );

    // Forward the request to the appropriate server
    match forward_request(&state.client, method, &target_uri, request).await {
        Ok(response) => {
            info!(
                target_uri = %target_uri,
                status = %response.status(),
                "=== PROXY: REQUEST FORWARDED SUCCESSFULLY ==="
            );
            Ok(response)
        }
        Err(e) => {
            error!(
                target_uri = %target_uri,
                error = %e,
                "=== PROXY: REQUEST FORWARDING FAILED ==="
            );
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}

/// Forward HTTP request to target server
async fn forward_request(
    client: &Client,
    method: Method,
    target_uri: &str,
    request: Request,
) -> Result<Response, reqwest::Error> {
    debug!(
        method = %method,
        target_uri = %target_uri,
        "Forwarding request to target server"
    );

    // Store headers before consuming the request
    let headers = request.headers().clone();

    // Extract the request body
    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!(error = %e, "Failed to read request body");
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Failed to read request body"))
                .unwrap()
                .into_response());
        }
    };

    // Convert axum Method to reqwest Method
    let reqwest_method = match method {
        Method::GET => reqwest::Method::GET,
        Method::POST => reqwest::Method::POST,
        Method::PUT => reqwest::Method::PUT,
        Method::DELETE => reqwest::Method::DELETE,
        Method::HEAD => reqwest::Method::HEAD,
        Method::OPTIONS => reqwest::Method::OPTIONS,
        Method::PATCH => reqwest::Method::PATCH,
        Method::TRACE => reqwest::Method::TRACE,
        _ => reqwest::Method::GET, // Default fallback
    };

    // Build the forwarded request
    let mut forwarded_request = client
        .request(reqwest_method, target_uri)
        .body(body_bytes.to_vec());

    // Copy relevant headers (excluding hop-by-hop headers)
    for (name, value) in headers.iter() {
        let header_name = name.as_str().to_lowercase();
        if !is_hop_by_hop_header(&header_name) {
            if let Ok(header_value) = value.to_str() {
                // Convert header name to string and use reqwest's header methods
                forwarded_request = forwarded_request.header(name.as_str(), header_value);
            }
        }
    }

    // Execute the request
    let response = forwarded_request.send().await?;
    let status_code = response.status().as_u16();
    let headers = response.headers().clone();
    let body_bytes = response.bytes().await?;

    debug!(
        status = status_code,
        body_size = body_bytes.len(),
        "Received response from target server"
    );

    // Convert reqwest StatusCode to axum StatusCode
    let axum_status =
        StatusCode::from_u16(status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    // Build the response
    let mut response_builder = Response::builder().status(axum_status);

    // Copy response headers (excluding hop-by-hop headers)
    for (name, value) in headers.iter() {
        let header_name = name.as_str().to_lowercase();
        if !is_hop_by_hop_header(&header_name) {
            if let Ok(header_value) = value.to_str() {
                // Convert header name to string and use axum's header methods
                response_builder = response_builder.header(name.as_str(), header_value);
            }
        }
    }

    Ok(response_builder
        .body(Body::from(body_bytes))
        .unwrap()
        .into_response())
}

/// Check if header is hop-by-hop and should not be forwarded
fn is_hop_by_hop_header(header_name: &str) -> bool {
    matches!(
        header_name,
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailers"
            | "transfer-encoding"
            | "upgrade"
    )
}

/// Access logging middleware for proxy requests
pub async fn access_logging_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path();

    debug!(
        method = %method,
        path = %path,
        "=== PROXY ACCESS LOG: REQUEST START ==="
    );

    let response = next.run(request).await;
    let elapsed = start.elapsed();

    info!(
        method = %method,
        path = %path,
        status = %response.status(),
        duration_ms = elapsed.as_millis(),
        "=== PROXY ACCESS LOG: REQUEST COMPLETE ==="
    );

    Ok(response)
}

/// Create proxy router with comprehensive logging and CORS
pub fn create_proxy_router(state: ProxyState) -> Router {
    Router::new()
        .route("/*path", any(proxy_handler))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(access_logging_middleware))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(
                            tower_http::trace::DefaultMakeSpan::new()
                                .level(tracing::Level::INFO)
                                .include_headers(true),
                        )
                        .on_request(
                            tower_http::trace::DefaultOnRequest::new().level(tracing::Level::INFO),
                        )
                        .on_response(
                            tower_http::trace::DefaultOnResponse::new()
                                .level(tracing::Level::INFO)
                                .latency_unit(tower_http::LatencyUnit::Micros),
                        )
                        .on_failure(
                            tower_http::trace::DefaultOnFailure::new().level(tracing::Level::ERROR),
                        ),
                )
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any)
                        .allow_credentials(false)
                        .expose_headers(Any)
                        .max_age(std::time::Duration::from_secs(86400)), // 24 hours
                ),
        )
}

/// Start the proxy server on the specified address
pub async fn start_proxy_server(
    bind_addr: &str,
    mcp_server_url: String,
    custom_routes_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🚀 Starting smart proxy server on {}", bind_addr);

    let proxy_state = ProxyState::new(mcp_server_url.clone(), custom_routes_url.clone());
    let proxy_app = create_proxy_router(proxy_state);

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;

    info!("=== 🌐 PROXY SERVER READY ===");
    info!("📡 Proxy server (public): http://{}", bind_addr);
    info!("🔧 Routes to MCP server: {}", mcp_server_url);
    info!("🔧 Routes to custom routes: {}", custom_routes_url);
    info!("📋 Routing logic:");
    info!("   • /mcp/* → MCP Server (OAuth2-protected MCP operations)");
    info!("   • /* → Custom Routes Server (OAuth2 discovery, authorization, dev tools)");

    axum::serve(listener, proxy_app).await?;

    Ok(())
}
