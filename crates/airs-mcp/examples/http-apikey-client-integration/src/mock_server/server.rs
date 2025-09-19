//! Mock HTTP Server Implementation
//!
//! A lightweight Axum-based HTTP server that implements the MCP protocol
//! for testing HTTP client implementations.

// Layer 1: Standard library imports
use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::post,
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio::time::sleep;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

// Layer 3: Internal module imports
use airs_mcp::protocol::{JsonRpcRequest, JsonRpcResponse};
use super::responses::MockResponses;

/// Mock server configuration
#[derive(Debug, Clone)]
pub struct MockServerConfig {
    /// Server port
    pub port: u16,
    /// Host address
    pub host: String,
    /// Valid API keys for authentication
    pub api_keys: HashSet<String>,
    /// Artificial delay in milliseconds (for testing timeouts)
    pub delay_ms: u64,
    /// Enable fault injection
    pub fault_injection: bool,
    /// Enable debug mode
    pub debug_mode: bool,
}

impl Default for MockServerConfig {
    fn default() -> Self {
        let mut api_keys = HashSet::new();
        api_keys.insert("test-key-123".to_string());
        api_keys.insert("dev-key-456".to_string());
        api_keys.insert("mock-key-789".to_string());

        Self {
            port: 3001,
            host: "127.0.0.1".to_string(),
            api_keys,
            delay_ms: 0,
            fault_injection: false,
            debug_mode: true,
        }
    }
}

impl MockServerConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Port configuration
        if let Ok(port_str) = env::var("MOCK_SERVER_PORT") {
            if let Ok(port) = port_str.parse::<u16>() {
                config.port = port;
            }
        }

        // Host configuration
        if let Ok(host) = env::var("MOCK_SERVER_HOST") {
            config.host = host;
        }

        // API keys configuration
        if let Ok(keys) = env::var("MOCK_API_KEYS") {
            config.api_keys = keys
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // Delay configuration (for testing timeouts)
        if let Ok(delay_str) = env::var("MOCK_DELAY_MS") {
            if let Ok(delay) = delay_str.parse::<u64>() {
                config.delay_ms = delay;
            }
        }

        // Fault injection
        config.fault_injection = env::var("MOCK_FAULT_INJECTION").is_ok();

        // Debug mode
        config.debug_mode = env::var("MOCK_DEBUG").is_ok() || env::var("DEBUG").is_ok();

        config
    }
}

/// Query parameters for authentication
#[derive(Debug, Deserialize)]
struct AuthQuery {
    api_key: Option<String>,
}

/// Mock HTTP server state
#[derive(Debug, Clone)]
pub struct MockServerState {
    config: MockServerConfig,
    request_count: Arc<std::sync::atomic::AtomicU64>,
}

impl MockServerState {
    pub fn new(config: MockServerConfig) -> Self {
        Self {
            config,
            request_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Validate authentication from headers or query parameters
    fn validate_auth(&self, headers: &HeaderMap, query: &Query<AuthQuery>) -> bool {
        // Check X-API-Key header
        if let Some(api_key) = headers.get("x-api-key").and_then(|v| v.to_str().ok()) {
            if self.config.api_keys.contains(api_key) {
                return true;
            }
        }

        // Check Authorization Bearer header
        if let Some(auth_header) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                if self.config.api_keys.contains(token) {
                    return true;
                }
            }
        }

        // Check query parameter
        if let Some(api_key) = &query.api_key {
            if self.config.api_keys.contains(api_key) {
                return true;
            }
        }

        false
    }

    /// Increment request counter
    fn increment_requests(&self) {
        self.request_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get current request count
    fn get_request_count(&self) -> u64 {
        self.request_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Main HTTP MCP endpoint handler
async fn handle_mcp_request(
    State(state): State<MockServerState>,
    headers: HeaderMap,
    query: Query<AuthQuery>,
    Json(request): Json<JsonRpcRequest>,
) -> Result<Json<JsonRpcResponse>, StatusCode> {
    // Increment request counter
    state.increment_requests();

    if state.config.debug_mode {
        info!(
            "Received MCP request: method={}, id={:?}",
            request.method, request.id
        );
    }

    // Validate authentication
    if !state.validate_auth(&headers, &query) {
        warn!("Authentication failed for request");
        return Ok(Json(MockResponses::authentication_error(
            Some(request.id.clone()),
        )));
    }

    // Add artificial delay if configured
    if state.config.delay_ms > 0 {
        if state.config.debug_mode {
            info!("Adding {}ms delay", state.config.delay_ms);
        }
        sleep(Duration::from_millis(state.config.delay_ms)).await;
    }

    // Check for fault injection
    if state.config.fault_injection {
        if let Some(fault_type) = MockResponses::should_inject_fault(&request) {
            warn!("Injecting fault: {}", fault_type);
            match fault_type.as_str() {
                "server_error" => {
                    return Ok(Json(MockResponses::server_error(Some(request.id.clone()))));
                }
                "timeout" => {
                    // Simulate a very long delay
                    sleep(Duration::from_secs(60)).await;
                }
                "malformed" => {
                    // This would require returning invalid JSON, which is tricky with Axum
                    // For now, return a server error
                    return Ok(Json(MockResponses::server_error(Some(request.id.clone()))));
                }
                _ => {}
            }
        }
    }

    // Handle the MCP request
    let response = match request.method.as_str() {
        "initialize" => MockResponses::initialize_response(request.id),
        "tools/list" => MockResponses::tools_list_response(request.id),
        "tools/call" => {
            let params = request.params.unwrap_or_default();
            let tool_name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let arguments = params.get("arguments").cloned().unwrap_or_default();

            MockResponses::tool_call_response(request.id, tool_name, &arguments)
        }
        "resources/list" => MockResponses::resources_list_response(request.id),
        "resources/read" => {
            let params = request.params.unwrap_or_default();
            let uri = params.get("uri").and_then(|v| v.as_str()).unwrap_or("");

            MockResponses::resource_read_response(request.id, uri)
        }
        _ => MockResponses::method_not_found_error(Some(request.id.clone()), &request.method),
    };

    if state.config.debug_mode {
        info!("Sending response for method: {}", request.method);
    }

    Ok(Json(response))
}

/// Health check endpoint
async fn health_check(State(state): State<MockServerState>) -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "server": "http-mock-server",
        "version": "0.1.0",
        "uptime": "unknown",
        "requests_served": state.get_request_count(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Server info endpoint
async fn server_info(State(state): State<MockServerState>) -> Json<Value> {
    Json(json!({
        "name": "HTTP MCP Mock Server",
        "version": "0.1.0",
        "description": "Lightweight HTTP MCP server for testing client implementations",
        "endpoints": {
            "/": "Main MCP JSON-RPC endpoint",
            "/health": "Health check endpoint",
            "/info": "Server information endpoint"
        },
        "authentication": {
            "methods": ["X-API-Key header", "Authorization Bearer", "Query parameter"],
            "valid_keys": state.config.api_keys.len(),
            "debug_mode": state.config.debug_mode
        },
        "configuration": {
            "port": state.config.port,
            "host": &state.config.host,
            "delay_ms": state.config.delay_ms,
            "fault_injection": state.config.fault_injection
        },
        "mcp_capabilities": {
            "tools": true,
            "resources": true,
            "prompts": false,
            "logging": false
        },
        "requests_served": state.get_request_count(),
        "started_at": chrono::Utc::now().to_rfc3339()
    }))
}

/// Mock HTTP server
pub struct MockHttpServer {
    config: MockServerConfig,
}

impl MockHttpServer {
    /// Create a new mock server with default configuration
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            config: MockServerConfig::default(),
        }
    }

    /// Create a new mock server with custom configuration
    pub fn with_config(config: MockServerConfig) -> Self {
        Self { config }
    }

    /// Create a new mock server from environment variables
    #[allow(dead_code)]
    pub fn from_env() -> Self {
        Self {
            config: MockServerConfig::from_env(),
        }
    }

    /// Start the mock server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let state = MockServerState::new(self.config.clone());

        // Build the router
        let app = Router::new()
            .route("/", post(handle_mcp_request))
            .route("/health", axum::routing::get(health_check))
            .route("/info", axum::routing::get(server_info))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive()),
            )
            .with_state(state);

        // Create the listener
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;

        info!("🎭 HTTP Mock Server starting on http://{}", addr);
        info!("📋 Endpoints:");
        info!("   • POST http://{}/           - MCP JSON-RPC endpoint", addr);
        info!("   • GET  http://{}/health     - Health check", addr);
        info!("   • GET  http://{}/info       - Server information", addr);
        info!("🔑 Authentication:");
        info!("   • X-API-Key header: test-key-123, dev-key-456, mock-key-789");
        info!("   • Authorization Bearer: test-key-123, dev-key-456, mock-key-789");
        info!("   • Query parameter: ?api_key=test-key-123");

        if self.config.debug_mode {
            info!("🐛 Debug mode enabled");
        }

        if self.config.fault_injection {
            info!("💥 Fault injection enabled");
        }

        if self.config.delay_ms > 0 {
            info!("⏰ Artificial delay: {}ms", self.config.delay_ms);
        }

        info!("🚀 Server ready for MCP client connections");

        // Start the server
        axum::serve(listener, app).await?;

        Ok(())
    }

    /// Get server configuration
    #[allow(dead_code)]
    pub fn config(&self) -> &MockServerConfig {
        &self.config
    }
}