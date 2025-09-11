//! Production-Ready OAuth2 MCP Server Example
//!
//! Demonstrates a production-grade MCP server with OAuth2 authentication and authorization,
//! comprehensive error handling, structured logging, and real-world security practices.
//!
//! This example uses the refactored `AxumHttpServer` with its fluent API for OAuth2 integration,
//! showcasing the zero-cost authorization architecture and proper JSON-RPC method extraction.

// Layer 1: Standard library imports
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

// Layer 2: Third-party crate imports
use axum::{
    body::Body,
    extract::{ConnectInfo, Query, Request, State},
    http::{StatusCode, Method, header::LOCATION},
    middleware::{self, Next},
    response::{Json, Redirect, Response},
    routing::{get, post},
    Form, Router,
};
use serde_json::{json, Value};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{debug, error, info, warn};
use url::Url;
use uuid::Uuid;

// JWT and RSA imports
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use rsa::{RsaPrivateKey, RsaPublicKey, pkcs1::EncodeRsaPublicKey, pkcs8::EncodePrivateKey, traits::PublicKeyParts};
use base64::{Engine as _, engine::general_purpose};
use std::sync::OnceLock;
use sha2::{Sha256, Digest};

// Layer 3: Internal module imports
use airs_mcp::{
    authentication::strategies::oauth2::OAuth2Strategy,
    base::jsonrpc::concurrent::{ConcurrentProcessor, ProcessorConfig},
    oauth2::{
        config::OAuth2Config,
        validator::create_default_validator,
    },
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
                oauth2::OAuth2StrategyAdapter,
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

/// RSA keypair for JWT token signing (development only)
static DEV_KEYPAIR: OnceLock<DevKeyPair> = OnceLock::new();

/// Development RSA keypair for JWT signing
#[derive(Debug, Clone)]
struct DevKeyPair {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
    public_key_pem: String,
    kid: String, // Key ID
}

/// JWT Claims for development tokens
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct JwtClaims {
    /// Subject (user ID)
    sub: String,
    /// Audience (who the token is intended for)
    aud: String,
    /// Issuer (who created the token)
    iss: String,
    /// Expiration time (unix timestamp)
    exp: usize,
    /// Issued at (unix timestamp)
    iat: usize,
    /// JWT ID (unique identifier for this token)
    jti: String,
    /// OAuth scopes
    scope: String,
}

/// OAuth2 authorization request parameters
#[derive(Debug, serde::Deserialize)]
struct AuthorizeRequest {
    /// OAuth2 response type (must be "code")
    response_type: String,
    /// OAuth2 client ID
    client_id: String,
    /// Redirect URI for callback
    redirect_uri: String,
    /// Requested scopes (space-separated)
    scope: Option<String>,
    /// OAuth2 state parameter (opaque value)
    state: Option<String>,
    /// PKCE code challenge
    code_challenge: String,
    /// PKCE code challenge method ("S256" or "plain")
    code_challenge_method: Option<String>,
}

/// OAuth2 token exchange request parameters
#[derive(Debug, serde::Deserialize)]
struct TokenRequest {
    /// Grant type (must be "authorization_code")
    grant_type: String,
    /// Authorization code from /authorize endpoint
    code: String,
    /// Redirect URI (must match the one from /authorize)
    redirect_uri: String,
    /// OAuth2 client ID
    client_id: String,
    /// PKCE code verifier
    code_verifier: String,
}

/// Stored authorization code data
#[derive(Debug, Clone)]
struct AuthorizationCode {
    /// The authorization code value
    code: String,
    /// Client ID that requested this code
    client_id: String,
    /// Redirect URI that was used
    redirect_uri: String,
    /// Requested scopes
    scope: String,
    /// PKCE code challenge
    code_challenge: String,
    /// PKCE code challenge method
    code_challenge_method: String,
    /// When this code expires
    expires_at: SystemTime,
    /// Whether this code has been used
    used: bool,
}

/// In-memory storage for authorization codes
type AuthCodeStorage = Arc<Mutex<HashMap<String, AuthorizationCode>>>;

/// Global storage for authorization codes
static AUTH_CODE_STORAGE: OnceLock<AuthCodeStorage> = OnceLock::new();

/// Application state shared across request handlers
#[derive(Clone)]
struct AppState {
    /// Server instance ID for debugging
    server_id: String,
    /// Server start time for health checks
    start_time: std::time::Instant,
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

/// Generate RSA keypair for development JWT signing
fn generate_dev_keypair() -> Result<DevKeyPair, Box<dyn std::error::Error>> {
    info!("Generating RSA keypair for development JWT tokens...");
    
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits)
        .map_err(|e| format!("Failed to generate RSA private key: {}", e))?;
    let public_key = RsaPublicKey::from(&private_key);
    
    // Convert to PEM format for JWKS
    let public_key_pem = public_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .map_err(|e| format!("Failed to encode public key to PEM: {}", e))?;
    
    let kid = format!("dev-key-{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
    
    info!(kid = %kid, "Generated RSA keypair for development");
    
    Ok(DevKeyPair {
        private_key,
        public_key,
        public_key_pem,
        kid,
    })
}

/// Verify PKCE code challenge against code verifier
fn verify_pkce_challenge(
    code_verifier: &str,
    code_challenge: &str,
    code_challenge_method: &str,
) -> bool {
    match code_challenge_method {
        "S256" => {
            let mut hasher = Sha256::new();
            hasher.update(code_verifier.as_bytes());
            let hash = hasher.finalize();
            let encoded = general_purpose::URL_SAFE_NO_PAD.encode(&hash);
            encoded == code_challenge
        }
        "plain" => code_verifier == code_challenge,
        _ => false,
    }
}

/// Generate a random authorization code
fn generate_authorization_code() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Create a development JWT token with proper claims and signature
fn create_dev_jwt_token(keypair: &DevKeyPair) -> Result<String, Box<dyn std::error::Error>> {
    create_dev_jwt_token_with_scope(
        keypair,
        "mcp:tools:execute mcp:resources:read mcp:prompts:read mcp:resources:list mcp:tools:read mcp:prompts:list"
    )
}

/// Create a development JWT token with custom scope
fn create_dev_jwt_token_with_scope(keypair: &DevKeyPair, scope: &str) -> Result<String, Box<dyn std::error::Error>> {
    let now = chrono::Utc::now();
    let exp = now + chrono::Duration::hours(1); // 1 hour expiration
    
    let claims = JwtClaims {
        sub: "dev_user_123".to_string(),
        aud: "mcp-server".to_string(),
        iss: "https://auth.example.com".to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
        jti: uuid::Uuid::new_v4().to_string(),
        scope: scope.to_string(),
    };
    
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(keypair.kid.clone());
    
    // Convert RSA private key to PEM for jsonwebtoken
    let private_key_pem = keypair.private_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
        .map_err(|e| format!("Failed to encode private key: {}", e))?;
    
    let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
        .map_err(|e| format!("Failed to create encoding key: {}", e))?;
    
    let token = encode(&header, &claims, &encoding_key)
        .map_err(|e| format!("Failed to encode JWT: {}", e))?;
    
    Ok(token)
}

/// Proxy server state for routing and logging
#[derive(Clone)]
struct ProxyState {
    /// HTTP client for forwarding requests
    client: reqwest::Client,
    /// MCP server URL (port 3004)
    mcp_server_url: String,
    /// Custom routes server URL (port 3003)
    custom_routes_url: String,
    /// Server startup time for metrics
    start_time: std::time::Instant,
}

/// Smart proxy server that routes requests between MCP and custom routes servers
async fn proxy_handler(
    State(state): State<ProxyState>,
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    req: Request,
) -> Result<Response, StatusCode> {
    let start_time = std::time::Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path();
    let query = uri.query().unwrap_or("");
    let headers = req.headers().clone();
    
    // Log incoming request with full details
    info!(
        target: "proxy_log",
        method = %method,
        uri = %uri,
        path = %path,
        query = %query,
        client_ip = %client_addr.ip(),
        client_port = %client_addr.port(),
        "=== PROXY: INCOMING REQUEST ==="
    );
    
    // Log all headers
    for (name, value) in headers.iter() {
        debug!(
            target: "proxy_log",
            header_name = %name,
            header_value = %value.to_str().unwrap_or("[INVALID_UTF8]"),
            "Request header"
        );
    }
    
    // Determine target server based on path
    let (target_url, target_name) = if path.starts_with("/mcp") {
        (&state.mcp_server_url, "MCP_SERVER")
    } else {
        (&state.custom_routes_url, "CUSTOM_ROUTES")
    };
    
    info!(
        target: "proxy_log",
        target_server = target_name,
        target_url = target_url,
        "Routing request"
    );
    
    // Extract request body
    let body = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Log request body if present
    if !body.is_empty() {
        debug!(
            target: "proxy_log",
            body_size = body.len(),
            body = %String::from_utf8_lossy(&body),
            "Request body"
        );
    }
    
    // Build target URL with query parameters
    let full_target_url = if query.is_empty() {
        format!("{}{}", target_url, path)
    } else {
        format!("{}{}?{}", target_url, path, query)
    };
    
    // Forward request to target server
    let mut request_builder = state.client
        .request(method.clone(), &full_target_url);
    
    // Copy headers (excluding host)
    for (name, value) in headers.iter() {
        if name.as_str().to_lowercase() != "host" {
            request_builder = request_builder.header(name, value);
        }
    }
    
    // Add body if present
    if !body.is_empty() {
        request_builder = request_builder.body(body);
    }
    
    // Execute request
    let response = match request_builder.send().await {
        Ok(resp) => resp,
        Err(e) => {
            error!(
                target: "proxy_log",
                error = %e,
                target_url = full_target_url,
                "Failed to forward request"
            );
            return Err(StatusCode::BAD_GATEWAY);
        }
    };
    
    let response_status = response.status();
    let response_headers = response.headers().clone();
    let response_body = match response.bytes().await {
        Ok(body) => body,
        Err(e) => {
            error!(
                target: "proxy_log",
                error = %e,
                "Failed to read response body"
            );
            return Err(StatusCode::BAD_GATEWAY);
        }
    };
    
    let duration = start_time.elapsed();
    
    // Log response
    info!(
        target: "proxy_log",
        method = %method,
        uri = %uri,
        target_server = target_name,
        status_code = %response_status.as_u16(),
        duration_ms = %duration.as_millis(),
        body_size = response_body.len(),
        "=== PROXY: RESPONSE SENT ==="
    );
    
    // Log response headers
    for (name, value) in response_headers.iter() {
        debug!(
            target: "proxy_log",
            header_name = %name,
            header_value = %value.to_str().unwrap_or("[INVALID_UTF8]"),
            "Response header"
        );
    }
    
    // Log response body for debugging
    if !response_body.is_empty() {
        debug!(
            target: "proxy_log",
            body = %String::from_utf8_lossy(&response_body),
            "Response body"
        );
    }
    
    // Build Axum response
    let mut axum_response = Response::builder()
        .status(response_status);
    
    // Copy response headers
    for (name, value) in response_headers.iter() {
        axum_response = axum_response.header(name, value);
    }
    
    axum_response
        .body(axum::body::Body::from(response_body))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Create proxy router with comprehensive logging and CORS
fn create_proxy_router(state: ProxyState) -> Router {
    Router::new()
        .fallback(proxy_handler)
        .with_state(state)
        .layer(
            ServiceBuilder::new()
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
        )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging with enhanced proxy and access logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("airs_mcp=debug".parse().expect("Valid log directive"))
            .add_directive("mcp_oauth2_server=debug".parse().expect("Valid log directive"))
            .add_directive("tower_http::trace=info".parse().expect("Valid log directive"))
            .add_directive("axum=debug".parse().expect("Valid log directive"))
            .add_directive("access_log=info".parse().expect("Valid log directive"))
            .add_directive("proxy_log=info".parse().expect("Valid log directive"))
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
    
    let server_id = Uuid::new_v4().to_string();
    info!(server_id = %server_id, "Starting OAuth2-based MCP server");
    
    // Generate development RSA keypair for JWT tokens
    let keypair = generate_dev_keypair()?;
    DEV_KEYPAIR.set(keypair.clone()).map_err(|_| "Failed to set global keypair")?;
    
    // Initialize authorization code storage
    let auth_storage: AuthCodeStorage = Arc::new(Mutex::new(HashMap::new()));
    AUTH_CODE_STORAGE.set(auth_storage).map_err(|_| "Failed to set global auth storage")?;
    
    info!("Development keypair and authorization code storage initialized");

    // OAuth2 Configuration
    // In production, load these from environment variables or configuration files
    let mut oauth2_config = OAuth2Config::default();
    oauth2_config.issuer = std::env::var("OAUTH2_ISSUER")
        .unwrap_or_else(|_| "https://auth.example.com".to_string());
    oauth2_config.jwks_url = std::env::var("OAUTH2_JWKS_URL")
        .map(|url| Url::parse(&url).expect("Valid JWKS URL"))
        .unwrap_or_else(|_| Url::parse("http://127.0.0.1:3003/.well-known/jwks.json").expect("Valid local JWKS URL"));
    oauth2_config.audience = std::env::var("OAUTH2_AUDIENCE")
        .unwrap_or_else(|_| "mcp-server".to_string());
    
    info!(
        issuer = %oauth2_config.issuer,
        audience = %oauth2_config.audience,
        "OAuth2 configuration loaded"
    );

    // Create OAuth2 validator using the builder pattern
    let validator = create_default_validator(oauth2_config)
        .map_err(|e| {
            error!(error = %e, "Failed to create OAuth2 validator");
            format!("OAuth2 validator creation failed: {}", e)
        })?;
    
    info!("OAuth2 validator created with default MCP scope mappings");

    // Create OAuth2 authentication strategy and adapter
    let oauth2_strategy = OAuth2Strategy::new(validator);
    let oauth2_adapter = OAuth2StrategyAdapter::new(oauth2_strategy);
    
    info!("OAuth2 authentication strategy initialized");

    // Build MCP server with production providers
    let temp_dir = tempfile::tempdir()
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    // Create sample files for demonstration (mirrors API key example)
    let temp_path = temp_dir.path();
    tokio::fs::write(temp_path.join("welcome.txt"),
        "Welcome to the OAuth2 MCP Server!\n\nThis server provides:\n- Filesystem resources\n- Mathematical tools\n- Code review prompts\n\nAuthenticate via OAuth2: Use Authorization: Bearer <token>.\nObtain tokens using the authorization code + PKCE flow.")
        .await
        .map_err(|e| format!("Failed to create welcome.txt: {}", e))?;

    tokio::fs::write(temp_path.join("config.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "server": {
                "name": "OAuth2 MCP Server",
                "version": "1.0.0",
                "authentication": "oauth2"
            },
            "capabilities": {
                "resources": true,
                "tools": true,
                "prompts": true
            },
            "endpoints": {
                "mcp": "http://127.0.0.1:3002/mcp",
                "health": "http://127.0.0.1:3002/health",
                "oauth2_discovery": "http://127.0.0.1:3002/.well-known/oauth-authorization-server"
            }
        }))?)
        .await
        .map_err(|e| format!("Failed to create config.json: {}", e))?;

    tokio::fs::write(temp_path.join("sample.md"),
        "# MCP Server Resources (OAuth2)\n\n## Available Resources\n\n- **welcome.txt**: Server introduction\n- **config.json**: Server configuration\n- **sample.md**: This markdown file\n- **oauth2-config.yaml**: OAuth2 flow information\n\n## Authentication\n\nUse OAuth2 Authorization Code Flow with PKCE.\nThe server expects: Authorization: Bearer <access_token>\n")
        .await
        .map_err(|e| format!("Failed to create sample.md: {}", e))?;

    tokio::fs::write(temp_path.join("oauth2-config.yaml"),
        "# OAuth2 Configuration\noauth2:\n  flow: authorization_code\n  pkce: S256\n  discovery: http://127.0.0.1:3002/.well-known/oauth-authorization-server\n  endpoints:\n    authorize: http://127.0.0.1:3003/authorize\n    token: http://127.0.0.1:3003/token\n  scopes:\n    - mcp:tools:execute\n    - mcp:resources:read\n    - mcp:prompts:read\n    - mcp:resources:list\n    - mcp:tools:read\n    - mcp:prompts:list\n")
        .await
        .map_err(|e| format!("Failed to create oauth2-config.yaml: {}", e))?;

    let fs_provider = FileSystemResourceProvider::new(temp_dir.path())
        .map_err(|e| format!("Failed to create filesystem provider: {}", e))?;
    info!(path = %temp_dir.path().display(), file_count = 4, "Added filesystem resource provider with sample files");

    let math_provider = MathToolProvider::new();
    info!("Added math tool provider");

    let prompt_provider = CodeReviewPromptProvider::new();
    info!("Added code review prompt provider");
    
    // Create MCP handlers with custom server configuration
    let server_info = ServerInfo {
        name: "OAuth2 MCP Server".to_string(),
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
    
    // Create custom MCP server configuration with OAuth2-specific instructions
    let mcp_config = McpServerConfig {
        server_info,
        capabilities: server_capabilities,
        protocol_version: ProtocolVersion::current(),
        strict_validation: true,
        log_operations: true,
        instructions: Some("OAuth2 authenticated MCP server with filesystem resources, mathematical tools, and code review prompts. Use Authorization: Bearer <token> header for authentication. Obtain tokens via OAuth2 authorization flow.".to_string()),
    };
    
    // Create MCP handlers builder with configuration
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
        .bind_address("127.0.0.1:3004".parse().unwrap()) // MCP server now on port 3004
        .max_connections(1000)
        .session_timeout(std::time::Duration::from_secs(3600))
        .request_timeout(std::time::Duration::from_secs(30))
        .max_message_size(10 * 1024 * 1024); // 10MB
    
    // Create authentication configuration
    let auth_config = HttpAuthConfig {
        skip_paths: vec![
            "/health".to_string(),
            "/auth/token".to_string(),
            "/auth/info".to_string(),
            "/server/info".to_string(),
            // OAuth2 discovery endpoints (bypassed for MCP Inspector compatibility)
            "/.well-known/oauth-authorization-server".to_string(),
            "/.well-known/oauth-authorization-server/mcp".to_string(),
            "/.well-known/openid-configuration".to_string(),
            "/.well-known/openid-configuration/mcp".to_string(),
            "/.well-known/oauth-protected-resource".to_string(),
            "/.well-known/oauth-protected-resource/mcp".to_string(),
            "/mcp/.well-known/openid-configuration".to_string(),
        ],
        include_error_details: false, // Set to true for development
        auth_realm: "MCP Server".to_string(),
        request_timeout_secs: 30,
    };
    
    info!("HTTP transport configuration created");

    // Build the AxumHttpServer with OAuth2 authentication and authorization for MCP
    let mut oauth2_server = AxumHttpServer::new(
        connection_manager,
        jsonrpc_processor,
        Arc::new(mcp_handlers_builder.build()),
        http_config.clone(),
    )
    .await
    .map_err(|e| format!("Failed to create HTTP server: {}", e))?
    .with_oauth2_authorization(oauth2_adapter, auth_config);
    
    info!("OAuth2 HTTP server created with scope-based authorization");

    // Create application state for custom routes
    let app_state = AppState {
        server_id: server_id.clone(),
        start_time: std::time::Instant::now(),
    };
    
    // === NEW PROXY ARCHITECTURE ===
    // Port 3002: Smart Proxy Server (public-facing)
    // Port 3003: Custom Routes Server (OAuth2, dev tools)
    // Port 3004: MCP Server (pure MCP functionality)
    
    // Start MCP server on port 3004 (in background)
    let mcp_bind_addr: SocketAddr = http_config.bind_address; // 3004
    oauth2_server.bind(mcp_bind_addr).await
        .map_err(|e| format!("Failed to bind MCP server to {}: {}", mcp_bind_addr, e))?;
    
    info!(mcp_addr = %mcp_bind_addr, "MCP server bound to address");
    
    tokio::spawn(async move {
        if let Err(e) = oauth2_server.serve().await {
            error!(error = %e, "MCP server error");
        }
    });
    
    info!("MCP server started in background on port 3004");
    
    // Start custom routes server on port 3003 (in background)
    let custom_routes_addr: SocketAddr = "127.0.0.1:3003".parse().unwrap();
    let custom_routes_app = create_custom_routes(app_state)
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
    
    let custom_listener = tokio::net::TcpListener::bind(custom_routes_addr).await
        .map_err(|e| format!("Failed to bind custom routes server to {}: {}", custom_routes_addr, e))?;
    
    tokio::spawn(async move {
        if let Err(e) = axum::serve(custom_listener, custom_routes_app).await {
            error!(error = %e, "Custom routes server error");
        }
    });
    
    info!("Custom routes server started in background on port 3003");
    
    // Create and start proxy server on port 3002 (public-facing)
    let proxy_addr: SocketAddr = "127.0.0.1:3002".parse().unwrap();
    let proxy_state = ProxyState {
        client: reqwest::Client::new(),
        mcp_server_url: "http://127.0.0.1:3004".to_string(),
        custom_routes_url: "http://127.0.0.1:3003".to_string(),
        start_time: std::time::Instant::now(),
    };
    
    let proxy_app = create_proxy_router(proxy_state);
    let proxy_listener = tokio::net::TcpListener::bind(proxy_addr).await
        .map_err(|e| format!("Failed to bind proxy server to {}: {}", proxy_addr, e))?;
    
    info!("=== 🚀 PROXY ARCHITECTURE READY ===");
    info!("📡 Proxy server (public): http://{}", proxy_addr);
    info!("🔧 Custom routes server: http://{}", custom_routes_addr);
    info!("⚡ MCP server: http://{}", mcp_bind_addr);
    info!("");
    info!("🎯 MCP Inspector should connect to: http://{}/mcp", proxy_addr);
    info!("🔑 Get dev token from: http://{}/auth/token", proxy_addr);
    info!("🔍 OAuth2 discovery at: http://{}/.well-known/oauth-authorization-server", proxy_addr);
    info!("💊 Health check: http://{}/health", proxy_addr);
    info!("");
    
    // Start proxy server (this will block and handle all traffic)
    axum::serve(proxy_listener, proxy_app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .map_err(|e| format!("Proxy server error: {}", e))?;
    
    Ok(())
}

/// Create custom routes for health checks, token management, and debugging
fn create_custom_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/auth/token", post(token_handler))
        .route("/auth/info", get(auth_info_handler))
        .route("/server/info", get(server_info_handler))
        .route("/.well-known/jwks.json", get(jwks_handler))
        // OAuth2 Discovery endpoints (RFC 8414)
        .route("/.well-known/oauth-authorization-server", get(oauth2_metadata_handler))
        .route("/.well-known/openid-configuration", get(oauth2_metadata_handler))
        // OAuth2 Authorization Code Flow endpoints
        .route("/authorize", get(authorize_handler))
        .route("/token", post(oauth_token_handler))
        .with_state(app_state)
}

/// Health check endpoint
async fn health_handler(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let uptime = state.start_time.elapsed();
    
    Ok(Json(json!({
        "status": "healthy",
        "server_id": state.server_id,
        "uptime_seconds": uptime.as_secs(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0"
    })))
}

/// Development token endpoint (for testing purposes only)
async fn token_handler() -> Result<Json<Value>, StatusCode> {
    warn!("Development token endpoint accessed - not for production use");
    
    // Get the development keypair
    let keypair = DEV_KEYPAIR.get()
        .ok_or_else(|| {
            error!("Development keypair not initialized");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Generate a real RSA-signed JWT token
    let token = create_dev_jwt_token(keypair)
        .map_err(|e| {
            error!(error = %e, "Failed to generate JWT token");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    info!("Generated development JWT token with RSA signature");
    
    Ok(Json(json!({
        "access_token": token,
        "token_type": "Bearer", 
        "expires_in": 3600,
        "scope": "mcp:tools:execute mcp:resources:read mcp:prompts:read mcp:resources:list mcp:tools:read mcp:prompts:list",
        "note": "Development token with RSA signature - not for production use",
        "algorithm": "RS256",
        "key_id": keypair.kid.clone()
    })))
}

/// OAuth2 authentication info endpoint
async fn auth_info_handler() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "auth_method": "oauth2",
        "authorization_type": "scope_based",
        "supported_flows": ["bearer_token", "authorization_code"],
        "pkce_support": true,
        "scopes": {
            "mcp:tools:execute": "Execute MCP tools",
            "mcp:resources:read": "Read MCP resources",
            "mcp:resources:write": "Write MCP resources",
            "mcp:resources:list": "List MCP resources",
            "mcp:tools:read": "Read MCP tools",
            "mcp:prompts:read": "Read MCP prompts",
            "mcp:prompts:list": "List MCP prompts",
            "mcp:server:admin": "Server administration"
        },
        "endpoints": {
            "mcp": "/mcp",
            "health": "/health",
            "token": "/auth/token",
            "authorize": "/authorize",
            "oauth_token": "/token",
            "jwks": "/.well-known/jwks.json"
        }
    })))
}

/// Server information endpoint
async fn server_info_handler(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "server_id": state.server_id,
        "name": "OAuth2 MCP Server",
        "version": "1.0.0",
        "description": "Production OAuth2 MCP Server Example",
        "capabilities": {
            "tools": ["math/calculate"],
            "resources": ["filesystem"],
            "prompts": ["code_review"]
        },
        "transport": "http",
        "authentication": "oauth2",
        "authorization": "scope_based",
        "uptime_seconds": state.start_time.elapsed().as_secs()
    })))
}

/// OAuth2 authorization endpoint - handles authorization code flow with PKCE
async fn authorize_handler(
    Query(params): Query<AuthorizeRequest>,
) -> Result<Response, StatusCode> {
    info!(
        client_id = %params.client_id,
        redirect_uri = %params.redirect_uri,
        scope = ?params.scope,
        "OAuth2 authorization request received"
    );
    
    // Validate request parameters
    if params.response_type != "code" {
        error!(response_type = %params.response_type, "Invalid response_type, must be 'code'");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Validate PKCE parameters
    if params.code_challenge.is_empty() {
        error!("Missing required PKCE code_challenge parameter");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let code_challenge_method = params.code_challenge_method
        .as_deref()
        .unwrap_or("plain"); // Default to plain if not specified
    
    if code_challenge_method != "S256" && code_challenge_method != "plain" {
        error!(method = %code_challenge_method, "Invalid code_challenge_method, must be 'S256' or 'plain'");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Generate authorization code
    let auth_code = generate_authorization_code();
    let expires_at = SystemTime::now() + Duration::from_secs(600); // 10 minute expiration
    
    // Default scope if none provided
    let scope = params.scope.unwrap_or_else(|| {
        "mcp:tools:execute mcp:resources:read mcp:prompts:read mcp:resources:list mcp:tools:read mcp:prompts:list".to_string()
    });
    
    // Store authorization code
    let auth_data = AuthorizationCode {
        code: auth_code.clone(),
        client_id: params.client_id.clone(),
        redirect_uri: params.redirect_uri.clone(),
        scope: scope.clone(),
        code_challenge: params.code_challenge.clone(),
        code_challenge_method: code_challenge_method.to_string(),
        expires_at,
        used: false,
    };
    
    let storage = AUTH_CODE_STORAGE.get()
        .ok_or_else(|| {
            error!("Authorization code storage not initialized");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    {
        let mut codes = storage.lock().unwrap();
        codes.insert(auth_code.clone(), auth_data);
    }
    
    info!(
        code = %auth_code,
        client_id = %params.client_id,
        scope = %scope,
        "Authorization code generated and stored"
    );
    
    // Build redirect URL with authorization code
    let mut redirect_url = Url::parse(&params.redirect_uri)
        .map_err(|e| {
            error!(error = %e, redirect_uri = %params.redirect_uri, "Invalid redirect URI");
            StatusCode::BAD_REQUEST
        })?;
    
    // Add query parameters
    redirect_url.query_pairs_mut()
        .append_pair("code", &auth_code);
    
    if let Some(state) = &params.state {
        redirect_url.query_pairs_mut()
            .append_pair("state", state);
    }
    
    info!(redirect_url = %redirect_url, "Redirecting to callback with authorization code");
    
    // Return redirect response
    Ok(Response::builder()
        .status(StatusCode::FOUND)
        .header(LOCATION, redirect_url.to_string())
        .body(axum::body::Body::empty())
        .unwrap())
}

/// OAuth2 token endpoint - exchanges authorization codes for JWT tokens
async fn oauth_token_handler(
    Form(params): Form<TokenRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!(
        client_id = %params.client_id,
        code = %params.code,
        "OAuth2 token exchange request received"
    );
    
    // Validate grant type
    if params.grant_type != "authorization_code" {
        error!(grant_type = %params.grant_type, "Invalid grant_type, must be 'authorization_code'");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Get authorization code storage
    let storage = AUTH_CODE_STORAGE.get()
        .ok_or_else(|| {
            error!("Authorization code storage not initialized");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Look up and validate authorization code
    let auth_data = {
        let mut codes = storage.lock().unwrap();
        let auth_data = codes.get_mut(&params.code)
            .ok_or_else(|| {
                error!(code = %params.code, "Invalid authorization code");
                StatusCode::BAD_REQUEST
            })?
            .clone();
        
        // Mark code as used
        if let Some(code_entry) = codes.get_mut(&params.code) {
            if code_entry.used {
                error!(code = %params.code, "Authorization code already used");
                return Err(StatusCode::BAD_REQUEST);
            }
            code_entry.used = true;
        }
        
        auth_data
    };
    
    // Check if code has expired
    if SystemTime::now() > auth_data.expires_at {
        error!(code = %params.code, "Authorization code expired");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Validate client_id matches
    if params.client_id != auth_data.client_id {
        error!(
            provided_client_id = %params.client_id,
            stored_client_id = %auth_data.client_id,
            "Client ID mismatch"
        );
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Validate redirect_uri matches
    if params.redirect_uri != auth_data.redirect_uri {
        error!(
            provided_redirect_uri = %params.redirect_uri,
            stored_redirect_uri = %auth_data.redirect_uri,
            "Redirect URI mismatch"
        );
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Verify PKCE code verifier
    if !verify_pkce_challenge(
        &params.code_verifier,
        &auth_data.code_challenge,
        &auth_data.code_challenge_method,
    ) {
        error!(
            code_challenge = %auth_data.code_challenge,
            code_challenge_method = %auth_data.code_challenge_method,
            "PKCE verification failed"
        );
        return Err(StatusCode::BAD_REQUEST);
    }
    
    info!(
        client_id = %params.client_id,
        scope = %auth_data.scope,
        "PKCE verification successful, generating JWT token"
    );
    
    // Get the development keypair
    let keypair = DEV_KEYPAIR.get()
        .ok_or_else(|| {
            error!("Development keypair not initialized");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Create JWT token with the authorized scope
    let token = create_dev_jwt_token_with_scope(keypair, &auth_data.scope)
        .map_err(|e| {
            error!(error = %e, "Failed to generate JWT token");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    info!(
        client_id = %params.client_id,
        scope = %auth_data.scope,
        "JWT token generated successfully via OAuth2 code exchange"
    );
    
    Ok(Json(json!({
        "access_token": token,
        "token_type": "Bearer",
        "expires_in": 3600,
        "scope": auth_data.scope,
        "note": "JWT token via OAuth2 authorization code flow with PKCE",
        "algorithm": "RS256",
        "key_id": keypair.kid
    })))
}

/// OAuth2 Authorization Server Metadata endpoint (RFC 8414)
/// Returns OAuth2 server configuration for client discovery
async fn oauth2_metadata_handler() -> Result<Json<Value>, StatusCode> {
    info!("OAuth2 metadata discovery endpoint accessed");
    
    Ok(Json(json!({
        "issuer": "https://auth.example.com",
        "authorization_endpoint": "http://127.0.0.1:3003/authorize",
        "token_endpoint": "http://127.0.0.1:3003/token", 
        "jwks_uri": "http://127.0.0.1:3003/.well-known/jwks.json",
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code"],
        "code_challenge_methods_supported": ["S256", "plain"],
        "scopes_supported": [
            "mcp:tools:execute",
            "mcp:resources:read",
            "mcp:resources:write", 
            "mcp:resources:list",
            "mcp:tools:read",
            "mcp:prompts:read",
            "mcp:prompts:list",
            "mcp:server:admin"
        ],
        "token_endpoint_auth_methods_supported": ["none"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "claims_supported": ["sub", "aud", "iss", "exp", "iat", "scope"]
    })))
}

/// JWKS endpoint for JWT validation (development only)
async fn jwks_handler() -> Result<Json<Value>, StatusCode> {
    let keypair = DEV_KEYPAIR.get()
        .ok_or_else(|| {
            error!("Development keypair not initialized");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Extract n (modulus) and e (exponent) from RSA public key for JWKS format
    let public_key = &keypair.public_key;
    let n = general_purpose::URL_SAFE_NO_PAD.encode(public_key.n().to_bytes_be());
    let e = general_purpose::URL_SAFE_NO_PAD.encode(public_key.e().to_bytes_be());
    
    Ok(Json(json!({
        "keys": [
            {
                "kty": "RSA",
                "use": "sig",
                "kid": keypair.kid,
                "alg": "RS256",
                "n": n,
                "e": e
            }
        ]
    })))
}
