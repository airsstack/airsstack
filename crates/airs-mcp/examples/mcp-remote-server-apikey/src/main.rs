//! Simple ApiKey-based MCP Server Example
//!
//! Demonstrates a production-ready MCP server with ApiKey authentication - much simpler than OAuth2
//! while still providing solid security for many use cases. Perfect for:
//! - Internal services and APIs
//! - Machine-to-machine communication
//! - Microservices authentication
//! - Development and testing environments
//!
//! This example showcases the `AxumHttpServer` with ApiKey authentication strategy,
//! demonstrating the zero-cost authorization architecture with straightforward key-based auth.

// Layer 1: Standard library imports
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
};

// Layer 2: Third-party crate imports
use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode, Uri},
    middleware::{self, Next},
    response::{Json, Response},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// Layer 3: Internal module imports
use airs_mcp::{
    authentication::{
        AuthContext, AuthMethod,
        strategies::apikey::{ApiKeyStrategy, ApiKeyAuthData, ApiKeySource, InMemoryApiKeyValidator},
    },
    base::jsonrpc::concurrent::{ConcurrentProcessor, ProcessorConfig},
    providers::{
        FileSystemResourceProvider,
        MathToolProvider,
        CodeReviewPromptProvider,
    },
    shared::protocol::{
        ServerInfo, ServerCapabilities, ProtocolVersion,
        capabilities::{ResourceCapabilities, ToolCapabilities, PromptCapabilities},
    },
    integration::mcp::server::McpServerConfig,
    transport::{
        adapters::http::{
            auth::{
                apikey::{ApiKeyConfig, ApiKeyStrategyAdapter},
                middleware::HttpAuthConfig,
            },
            axum::{
                AxumHttpServer,
                McpHandlersBuilder,
            },
            config::HttpTransportConfig,
            connection_manager::{HttpConnectionManager, HealthCheckConfig},
        },
    },
};

/// Application state shared across request handlers
#[derive(Clone)]
struct AppState {
    /// Server instance ID for debugging
    server_id: String,
    /// Server start time for health checks
    start_time: std::time::Instant,
    /// Valid API keys for demonstration (for display purposes only)
    valid_api_keys: Vec<String>,
}

/// Comprehensive access logging middleware with body logging
async fn access_logging_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = std::time::Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();
    
    // Extract and log request body for POST requests
    let (parts, body) = req.into_parts();
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!(target: "access_log", error = %e, "Failed to read request body");
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    if method == Method::POST && !body_bytes.is_empty() {
        let body_str = String::from_utf8_lossy(&body_bytes);
        info!(
            target: "access_log",
            method = %method,
            uri = %uri,
            body_size = body_bytes.len(),
            request_body = %body_str,
            "=== REQUEST BODY ==="
        );
    }
    
    // Reconstruct the request
    let req = Request::from_parts(parts, Body::from(body_bytes));
    
    // Log incoming request with all details
    info!(
        target: "access_log",
        method = %method,
        uri = %uri,
        remote_addr = ?req.extensions().get::<std::net::SocketAddr>(),
        user_agent = ?headers.get("user-agent").map(|h| h.to_str().unwrap_or("invalid")),
        content_type = ?headers.get("content-type").map(|h| h.to_str().unwrap_or("invalid")),
        content_length = ?headers.get("content-length").map(|h| h.to_str().unwrap_or("invalid")),
        authorization = ?headers.get("authorization").map(|_| "[REDACTED]"),
        x_api_key = ?headers.get("x-api-key").map(|_| "[REDACTED]"),
        "=== INCOMING REQUEST ==="
    );
    
    // Log all headers (with sensitive ones redacted)
    debug!(
        target: "access_log", 
        "Request headers:"
    );
    for (name, value) in headers.iter() {
        let header_name = name.as_str().to_lowercase();
        let header_value = if header_name.contains("auth") || header_name.contains("key") {
            "[REDACTED]"
        } else {
            value.to_str().unwrap_or("[INVALID_UTF8]")
        };
        debug!(
            target: "access_log",
            header_name = %name,
            header_value = %header_value,
            "Request header"
        );
    }
    
    // Process the request
    let response = next.run(req).await;
    
    let duration = start_time.elapsed();
    let status = response.status();
    
    // Log response with timing
    info!(
        target: "access_log",
        method = %method,
        uri = %uri,
        status_code = %status.as_u16(),
        status_text = %status.canonical_reason().unwrap_or("unknown"),
        duration_ms = %duration.as_millis(),
        duration_us = %duration.as_micros(),
        "=== RESPONSE SENT ==="
    );
    
    // Log response headers
    debug!(
        target: "access_log",
        "Response headers:"
    );
    for (name, value) in response.headers().iter() {
        debug!(
            target: "access_log",
            header_name = %name,
            header_value = %value.to_str().unwrap_or("[INVALID_UTF8]"),
            "Response header"
        );
    }
    
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging with enhanced access logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("airs_mcp=debug".parse().expect("Valid log directive"))
            .add_directive("mcp_apikey_server=debug".parse().expect("Valid log directive"))
            .add_directive("tower_http::trace=debug".parse().expect("Valid log directive"))
            .add_directive("axum=debug".parse().expect("Valid log directive"))
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
    
    let server_id = Uuid::new_v4().to_string();
    info!(server_id = %server_id, "Starting ApiKey-based MCP server");
    
    // Create API key validator with valid keys for demonstration
    // In production, these would come from a database or configuration
    let mut validator = InMemoryApiKeyValidator::new(HashMap::new());
    
    // Add valid API keys with their authentication context
    let api_keys = vec![
        "mcp_dev_key_12345",
        "mcp_prod_key_67890", 
        "mcp_test_key_abcdef",
    ];
    
    for (idx, key) in api_keys.iter().enumerate() {
        let auth_context = AuthContext::new(
            AuthMethod::new("api_key"),
            ApiKeyAuthData {
                key_id: format!("user_{}", idx + 1),
                source: ApiKeySource::Header("X-API-Key".to_string()),
            },
        );
        validator.add_key(key.to_string(), auth_context);
    }
    
    info!(
        key_count = api_keys.len(),
        "Initialized API key validation with {} valid keys",
        api_keys.len()
    );

    // Create ApiKey authentication strategy and adapter
    let api_key_strategy = ApiKeyStrategy::new(validator);
    let api_key_config = ApiKeyConfig::default(); // Uses standard headers and bearer token
    let api_key_adapter = ApiKeyStrategyAdapter::new(api_key_strategy, api_key_config);
    
    info!("ApiKey authentication strategy initialized");

    // Build MCP server with production providers
    let temp_dir = tempfile::tempdir()
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;
        
    // Create sample files for demonstration
    let temp_path = temp_dir.path();
    tokio::fs::write(temp_path.join("welcome.txt"), 
        "Welcome to the MCP API Key Server!\n\nThis server provides:\n- Filesystem resources\n- Mathematical tools\n- Code review prompts\n\nAuthenticate with X-API-Key or Authorization Bearer token.").await
        .map_err(|e| format!("Failed to create welcome.txt: {}", e))?;
        
    tokio::fs::write(temp_path.join("config.json"), 
        serde_json::to_string_pretty(&serde_json::json!({
            "server": {
                "name": "ApiKey MCP Server",
                "version": "1.0.0",
                "authentication": "api_key"
            },
            "capabilities": {
                "resources": true,
                "tools": true,
                "prompts": true
            },
            "endpoints": {
                "mcp": "http://127.0.0.1:3001/mcp",
                "health": "http://127.0.0.1:3002/health",
                "keys": "http://127.0.0.1:3002/keys"
            }
        }))?).await
        .map_err(|e| format!("Failed to create config.json: {}", e))?;
        
    tokio::fs::write(temp_path.join("sample.md"), 
        "# MCP Server Resources\n\n## Available Resources\n\n- **welcome.txt**: Server introduction\n- **config.json**: Server configuration\n- **sample.md**: This markdown file\n- **api-keys.yaml**: API key information\n\n## Authentication\n\nUse one of these API keys:\n- mcp_dev_key_12345\n- mcp_prod_key_67890\n- mcp_test_key_abcdef\n").await
        .map_err(|e| format!("Failed to create sample.md: {}", e))?;
        
    tokio::fs::write(temp_path.join("api-keys.yaml"), 
        "# API Keys Configuration\napi_keys:\n  - key: mcp_dev_key_12345\n    name: Development Key\n    scope: full\n    environment: development\n  - key: mcp_prod_key_67890\n    name: Production Key\n    scope: full\n    environment: production\n  - key: mcp_test_key_abcdef\n    name: Test Key\n    scope: full\n    environment: testing\n\nusage:\n  header_methods:\n    - \"X-API-Key: <api_key>\"\n    - \"Authorization: Bearer <api_key>\"\n").await
        .map_err(|e| format!("Failed to create api-keys.yaml: {}", e))?;
        
    let fs_provider = FileSystemResourceProvider::new(temp_dir.path())
        .map_err(|e| format!("Failed to create filesystem provider: {}", e))?;
    info!(path = %temp_dir.path().display(), file_count = 4, "Added filesystem resource provider with sample files");

    let math_provider = MathToolProvider::new();
    info!("Added math tool provider");

    let prompt_provider = CodeReviewPromptProvider::new();
    info!("Added code review prompt provider");
    
    // Create MCP handlers with custom server configuration
    let server_info = ServerInfo {
        name: "ApiKey MCP Server".to_string(),
        version: "1.0.0".to_string(),
    };
    
    // Create server capabilities based on our providers
    let server_capabilities = ServerCapabilities {
        resources: Some(ResourceCapabilities {
            subscribe: Some(false),
            list_changed: Some(false),
        }),
        tools: Some(ToolCapabilities::default()),
        prompts: Some(PromptCapabilities {
            list_changed: Some(false),
        }),
        logging: None,
        experimental: None,
    };
    
    // Create custom MCP server configuration with ApiKey-specific instructions
    let mcp_config = McpServerConfig {
        server_info,
        capabilities: server_capabilities,
        protocol_version: ProtocolVersion::current(),
        strict_validation: true,
        log_operations: true,
        instructions: Some("API key authenticated MCP server with filesystem resources, mathematical tools, and code review prompts. Use X-API-Key header or Authorization: Bearer <api_key> for authentication.".to_string()),
    };
    
    // Create MCP handlers with custom configuration
    let mcp_handlers_builder = McpHandlersBuilder::new()
        .with_resource_provider(Arc::new(fs_provider))
        .with_tool_provider(Arc::new(math_provider))
        .with_prompt_provider(Arc::new(prompt_provider))
        .with_config(mcp_config);
    
    info!("MCP handlers built with custom configuration, filesystem, math, and code review providers");

    // Create HTTP transport infrastructure
    let health_config = HealthCheckConfig {
        check_interval: std::time::Duration::from_secs(30),
        max_idle_time: std::time::Duration::from_secs(300), // 5 minutes
        max_requests_per_connection: 1000,
        auto_cleanup: true,
    };
    let connection_manager = Arc::new(HttpConnectionManager::new(
        1000, // max_connections
        health_config, 
    ));
    
    let processor_config = ProcessorConfig {
        worker_count: 4,
        queue_capacity: 1000,
        max_batch_size: 10,
        processing_timeout: chrono::Duration::seconds(30),
        enable_ordering: false,
        enable_backpressure: true,
    };
    let jsonrpc_processor = Arc::new(ConcurrentProcessor::new(processor_config));
    
    info!("HTTP transport components initialized");

    // Create HTTP transport configuration
    let http_config = HttpTransportConfig::new()
        .bind_address("127.0.0.1:3001".parse().unwrap())
        .max_connections(1000)
        .session_timeout(std::time::Duration::from_secs(3600))
        .request_timeout(std::time::Duration::from_secs(30))
        .max_message_size(10 * 1024 * 1024); // 10MB
    
    // Create authentication configuration
    let auth_config = HttpAuthConfig {
        skip_paths: vec![
            "/health".to_string(),
            "/keys".to_string(),
            "/auth/info".to_string(),
            "/server/info".to_string(),
        ],
        include_error_details: true, // Helpful for ApiKey debugging
        auth_realm: "MCP Server".to_string(),
        request_timeout_secs: 30,
    };
    
    info!("HTTP transport configuration created");

    // Build the AxumHttpServer with ApiKey authentication
    let mut api_key_server = AxumHttpServer::new(
        connection_manager,
        jsonrpc_processor,
        Arc::new(mcp_handlers_builder.build()),
        http_config.clone(),
    )
    .await
    .map_err(|e| format!("Failed to create HTTP server: {}", e))?
    .with_authentication(api_key_adapter, auth_config);
    
    info!("ApiKey HTTP server created");

    // Create application state for utility routes
    let app_state = AppState {
        server_id: server_id.clone(),
        start_time: std::time::Instant::now(),
        valid_api_keys: api_keys.iter().map(|s| s.to_string()).collect(),
    };
    
    // Start AxumHttpServer for MCP functionality (in background)
    let bind_addr: SocketAddr = http_config.bind_address;
    api_key_server.bind(bind_addr).await
        .map_err(|e| format!("Failed to bind MCP server to {}: {}", bind_addr, e))?;
    
    info!(bind_addr = %bind_addr, "MCP server bound to address");
    
    // Start MCP server in background
    tokio::spawn(async move {
        if let Err(e) = api_key_server.serve().await {
            error!(error = %e, "MCP server error");
        }
    });
    
    info!("MCP server started in background");
    
    // Start utility routes server on a different port with comprehensive logging
    let utility_routes_addr: SocketAddr = "127.0.0.1:3002".parse().unwrap();
    let utility_routes_app = create_utility_routes(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(access_logging_middleware))
                .layer(TraceLayer::new_for_http()
                    .make_span_with(tower_http::trace::DefaultMakeSpan::new()
                        .level(tracing::Level::INFO)
                        .include_headers(true))
                    .on_request(tower_http::trace::DefaultOnRequest::new()
                        .level(tracing::Level::INFO))
                    .on_response(tower_http::trace::DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(tower_http::LatencyUnit::Micros))
                    .on_failure(tower_http::trace::DefaultOnFailure::new()
                        .level(tracing::Level::ERROR))
                )
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any)
                        .allow_credentials(false)
                        .expose_headers(Any)
                        .max_age(std::time::Duration::from_secs(86400)) // 24 hours
                ),
        );
    
    let utility_listener = tokio::net::TcpListener::bind(utility_routes_addr).await
        .map_err(|e| format!("Failed to bind utility routes server to {}: {}", utility_routes_addr, e))?;
    
    info!(utility_addr = %utility_routes_addr, mcp_addr = %bind_addr, "ApiKey MCP Server ready with comprehensive access logging");
    info!("📊 COMPREHENSIVE ACCESS LOGGING ENABLED:");
    info!("   • All incoming requests logged with full details");
    info!("   • Request/response headers logged (auth headers redacted)");
    info!("   • Response timing and status codes logged");
    info!("   • Debug level logging for detailed troubleshooting");
    info!("🔗 ENDPOINTS:");
    info!("   • MCP JSON-RPC endpoint: http://{}/mcp", bind_addr);
    info!("   • Health check endpoint: http://{}/health", utility_routes_addr);
    info!("   • API keys info endpoint: http://{}/keys", utility_routes_addr);
    info!("   • Authentication info: http://{}/auth/info", utility_routes_addr);
    info!("🔑 AUTHENTICATION:");
    info!("   • X-API-Key: mcp_dev_key_12345");
    info!("   • Authorization: Bearer mcp_dev_key_12345");
    
    // Run utility routes server (this will block)
    axum::serve(utility_listener, utility_routes_app)
        .await
        .map_err(|e| format!("Utility routes server error: {}", e))?;
    
    Ok(())
}

/// CORS preflight handler for all routes
async fn cors_preflight_handler() -> Result<Response, StatusCode> {
    info!(target: "handler", "CORS preflight OPTIONS request handled");
    
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization, X-API-Key, X-Requested-With")
        .header("Access-Control-Max-Age", "86400")
        .header("Content-Length", "0")
        .body(Body::empty())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
    Ok(response)
}

/// Create utility routes for health checks, key management, and debugging
fn create_utility_routes(app_state: AppState) -> Router {
    use axum::routing::options;
    
    Router::new()
        .route("/health", get(health_handler).options(cors_preflight_handler))
        .route("/keys", get(keys_handler).post(create_key_handler).options(cors_preflight_handler))
        .route("/auth/info", get(auth_info_handler).options(cors_preflight_handler))
        .route("/server/info", get(server_info_handler).options(cors_preflight_handler))
        // Catch-all OPTIONS handler
        .fallback(options(cors_preflight_handler))
        .with_state(app_state)
}

/// Health check endpoint
async fn health_handler(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    info!(target: "handler", "Health check endpoint called");
    let uptime = state.start_time.elapsed();
    
    let response = json!({
        "status": "healthy",
        "server_id": state.server_id,
        "uptime_seconds": uptime.as_secs(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0"
    });
    
    debug!(target: "handler", response = %response, "Health check response");
    Ok(Json(response))
}

/// API keys management endpoint (for demo purposes)
async fn keys_handler(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    warn!(target: "handler", "API keys endpoint accessed - development use only");
    info!(target: "handler", "Returning API keys information");
    
    Ok(Json(json!({
        "valid_keys": {
            "mcp_dev_key_12345": {
                "name": "Development Key",
                "scope": "full",
                "environment": "development"
            },
            "mcp_prod_key_67890": {
                "name": "Production Key",
                "scope": "full",
                "environment": "production"
            },
            "mcp_test_key_abcdef": {
                "name": "Test Key",
                "scope": "full",
                "environment": "testing"
            }
        },
        "usage": {
            "header_name": "X-API-Key",
            "alternative_header": "Authorization: Bearer <api_key>",
            "example": "curl -H \"X-API-Key: mcp_dev_key_12345\" http://127.0.0.1:3001/mcp"
        },
        "configured_keys_count": state.valid_api_keys.len(),
        "note": "This endpoint is for development only - remove in production"
    })))
}

/// Create new API key endpoint (for demo purposes)
async fn create_key_handler() -> Result<Json<Value>, StatusCode> {
    let new_key = format!("mcp_gen_key_{}", uuid::Uuid::new_v4().to_string().replace('-', "")[..12].to_string());
    
    warn!(generated_key = %new_key, "Generated new API key - development use only");
    
    Ok(Json(json!({
        "api_key": new_key,
        "type": "generated",
        "expires": "never",
        "scope": "full",
        "note": "Add this key to your server configuration to use it",
        "warning": "This endpoint is for development only - implement proper key management in production"
    })))
}

/// Authentication information endpoint
async fn auth_info_handler() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "auth_method": "api_key",
        "authorization_type": "key_based",
        "supported_headers": ["X-API-Key", "Authorization"],
        "key_formats": {
            "x_api_key": "X-API-Key: <your_api_key>",
            "bearer_format": "Authorization: Bearer <your_api_key>"
        },
        "examples": {
            "curl_x_api_key": "curl -H \"X-API-Key: mcp_dev_key_12345\" http://127.0.0.1:3001/mcp",
            "curl_bearer": "curl -H \"Authorization: Bearer mcp_dev_key_12345\" http://127.0.0.1:3001/mcp"
        },
        "endpoints": {
            "mcp": "/mcp",
            "health": "/health",
            "keys": "/keys"
        }
    })))
}

/// Server information endpoint
async fn server_info_handler(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "server_id": state.server_id,
        "name": "ApiKey MCP Server",
        "version": "1.0.0",
        "description": "Simple ApiKey-based MCP Server Example",
        "capabilities": {
            "tools": ["math/calculate"],
            "resources": ["filesystem"],
            "prompts": ["code_review"]
        },
        "transport": "http",
        "authentication": "api_key",
        "authorization": "key_based",
        "uptime_seconds": state.start_time.elapsed().as_secs(),
        "api_keys_count": state.valid_api_keys.len()
    })))
}
