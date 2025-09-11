//! OAuth2 MCP Inspector Test Server
//!
//! This example demonstrates OAuth2 authentication integration with the AirsStack
//! MCP HTTP transport server. It provides a complete OAuth2-protected MCP server
//! that can be tested with MCP Inspector tools.
//!
//! Features:
//! - Real OAuth2 JWT token validation using JWKS
//! - Mock JWKS endpoint for testing
//! - Scope-based authorization for MCP methods
//! - MCP-compliant JSON-RPC over HTTP
//! - Test token generation for different scenarios
//!
//! Usage:
//!   cargo run --example mcp-inspector-oauth2-server
//!
//! Test with MCP Inspector:
//!   # 1. Get test tokens
//!   curl http://localhost:3001/auth/tokens | jq
//!   
//!   # 2. Use with MCP Inspector
//!   npx @modelcontextprotocol/inspector-cli \
//!     --transport http \
//!     --server-url http://localhost:3001/mcp \
//!     --header "Authorization: Bearer <test-token>"

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use axum::{extract::State, response::Json, routing::get, Router};
use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tracing::{info, warn};
use url::Url;
use uuid::Uuid;

// Layer 3: Internal module imports - using existing AirsStack infrastructure
use airs_mcp::authentication::strategies::oauth2::OAuth2Strategy;
use airs_mcp::oauth2::{
    config::{CacheConfig, OAuth2Config, ValidationConfig},
    types::JwtClaims,
    validator::{Jwt, Scope, Validator},
};
use airs_mcp::providers::{
    CodeReviewPromptProvider, FileSystemResourceProvider, MathToolProvider,
    StructuredLoggingHandler,
};
use airs_mcp::transport::adapters::http::{
    auth::{middleware::HttpAuthConfig, oauth2::OAuth2StrategyAdapter},
    axum::{AxumHttpServer, McpHandlersBuilder},
    config::HttpTransportConfig,
    connection_manager::{HealthCheckConfig, HttpConnectionManager},
};

/// Test JWT signing keys and JWKS data
#[derive(Clone)]
struct TestKeys {
    /// Private key for signing JWTs
    encoding_key: EncodingKey,
    /// Public key in JWK format for JWKS endpoint
    jwks_response: Value,
}

impl std::fmt::Debug for TestKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestKeys")
            .field("encoding_key", &"<EncodingKey>")
            .field("jwks_response", &self.jwks_response)
            .finish()
    }
}

/// Generate test RSA keys for JWT signing and validation
fn generate_test_keys() -> Result<TestKeys, Box<dyn std::error::Error>> {
    // Use a fixed RSA key for consistent testing
    // In production, keys would be rotated and managed securely
    let private_key_pem = include_str!("../test_data/test_rsa_key.pem");
    let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?;

    // Create JWKS response with the public key
    let jwks_response = json!({
        "keys": [
            {
                "kty": "RSA",
                "use": "sig",
                "kid": "test-key-oauth2-mcp",
                "alg": "RS256",
                "n": "4f5wg5l2hKsTeNem_V41fGnJm6gOdrj8ydBtQZ4fCI0FgNX-JmFBD-jRwqhwn6b7cDi2QGnfOFcg3s2nWcMaH_yU4pjvNe0rOKE1-Cc5I7Ia_BF2GF4MEDfnTOpN2v5nAK9Q2-QDQ2c5I2z2C5I3Y2w2c5I-D1I-V9I-g-zFcjPz",
                "e": "AQAB"
            }
        ]
    });

    Ok(TestKeys {
        encoding_key,
        jwks_response,
    })
}

/// Generate a test JWT token with specified claims
fn generate_test_token(
    subject: &str,
    scopes: &[&str],
    audience: &str,
    issuer: &str,
    expires_in_minutes: i64,
    encoding_key: &EncodingKey,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + ChronoDuration::minutes(expires_in_minutes);

    let claims = JwtClaims {
        sub: subject.to_string(),
        aud: Some(audience.to_string()),
        iss: Some(issuer.to_string()),
        exp: Some(exp.timestamp()),
        nbf: Some(now.timestamp()),
        iat: Some(now.timestamp()),
        jti: Some(Uuid::new_v4().to_string()),
        scope: Some(scopes.join(" ")),
        scopes: Some(scopes.iter().map(|s| s.to_string()).collect()),
    };

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test-key-oauth2-mcp".to_string());

    encode(&header, &claims, encoding_key)
}

/// Test token configurations for different scenarios
#[derive(Debug, Clone)]
struct TokenConfig {
    name: String,
    description: String,
    subject: String,
    scopes: Vec<String>,
    expires_minutes: i64,
}

impl TokenConfig {
    fn full_access() -> Self {
        Self {
            name: "Full Access".to_string(),
            description: "Complete access to all MCP operations".to_string(),
            subject: "admin@test.local".to_string(),
            scopes: vec!["mcp:*".to_string()],
            expires_minutes: 60,
        }
    }

    fn tools_only() -> Self {
        Self {
            name: "Tools Only".to_string(),
            description: "Access to tools operations only".to_string(),
            subject: "tools-user@test.local".to_string(),
            scopes: vec![
                "mcp:tools:list".to_string(),
                "mcp:tools:execute".to_string(),
            ],
            expires_minutes: 30,
        }
    }

    fn resources_only() -> Self {
        Self {
            name: "Resources Only".to_string(),
            description: "Access to resources operations only".to_string(),
            subject: "resources-user@test.local".to_string(),
            scopes: vec![
                "mcp:resources:list".to_string(),
                "mcp:resources:read".to_string(),
            ],
            expires_minutes: 30,
        }
    }

    fn read_only() -> Self {
        Self {
            name: "Read Only".to_string(),
            description: "Read-only access to resources and tools listing".to_string(),
            subject: "readonly@test.local".to_string(),
            scopes: vec![
                "mcp:resources:list".to_string(),
                "mcp:tools:list".to_string(),
                "mcp:prompts:list".to_string(),
            ],
            expires_minutes: 15,
        }
    }
}

/// Application state for the OAuth2 MCP server
#[derive(Clone)]
struct OAuth2McpServerState {
    /// Test RSA keys for JWT operations
    test_keys: TestKeys,
    /// OAuth2 configuration
    oauth2_config: OAuth2Config,
    /// Available test token configurations
    token_configs: HashMap<String, TokenConfig>,
}

impl OAuth2McpServerState {
    fn new(test_keys: TestKeys, oauth2_config: OAuth2Config) -> Self {
        let mut token_configs = HashMap::new();

        token_configs.insert("full".to_string(), TokenConfig::full_access());
        token_configs.insert("tools".to_string(), TokenConfig::tools_only());
        token_configs.insert("resources".to_string(), TokenConfig::resources_only());
        token_configs.insert("readonly".to_string(), TokenConfig::read_only());

        Self {
            test_keys,
            oauth2_config,
            token_configs,
        }
    }
}

/// JWKS endpoint handler for JWT validation
async fn jwks_endpoint(State(state): State<OAuth2McpServerState>) -> Json<Value> {
    Json(state.test_keys.jwks_response.clone())
}

/// Generate test tokens endpoint
async fn generate_tokens(State(state): State<OAuth2McpServerState>) -> Json<Value> {
    let mut tokens = HashMap::new();

    for (key, config) in &state.token_configs {
        let scopes: Vec<&str> = config.scopes.iter().map(|s| s.as_str()).collect();

        match generate_test_token(
            &config.subject,
            &scopes,
            &state.oauth2_config.audience,
            &state.oauth2_config.issuer,
            config.expires_minutes,
            &state.test_keys.encoding_key,
        ) {
            Ok(token) => {
                let inspector_cmd = format!(
                    "npx @modelcontextprotocol/inspector-cli --transport http --server-url http://localhost:3001/mcp --header \"Authorization: Bearer {token}\""
                );

                tokens.insert(key, json!({
                    "name": config.name,
                    "description": config.description,
                    "subject": config.subject,
                    "scopes": config.scopes,
                    "expires_minutes": config.expires_minutes,
                    "token": token,
                    "inspector_command": inspector_cmd,
                    "curl_test": format!("curl -H 'Authorization: Bearer {}' http://localhost:3001/mcp -X POST -H 'Content-Type: application/json' -d '{{\"jsonrpc\":\"2.0\",\"id\":\"test\",\"method\":\"initialize\",\"params\":{{}}}}'", token)
                }));
            }
            Err(e) => {
                warn!("Failed to generate token for {}: {}", key, e);
            }
        }
    }

    Json(json!({
        "service": "OAuth2 MCP Test Server",
        "description": "Test tokens for OAuth2 MCP Inspector testing",
        "jwks_url": "http://localhost:3001/.well-known/jwks.json",
        "mcp_endpoint": "http://localhost:3001/mcp",
        "tokens": tokens,
        "usage_instructions": {
            "step1": "Copy a token from above",
            "step2": "Use the 'inspector_command' to test with MCP Inspector",
            "step3": "Or use the 'curl_test' command to test manually"
        },
        "scope_explanations": {
            "mcp:*": "Full access to all MCP operations",
            "mcp:tools:*": "Access to all tool operations",
            "mcp:resources:*": "Access to all resource operations",
            "mcp:prompts:*": "Access to all prompt operations",
            "mcp:tools:list": "List available tools",
            "mcp:tools:execute": "Execute tools",
            "mcp:resources:list": "List available resources",
            "mcp:resources:read": "Read resource contents"
        }
    }))
}

/// Server information endpoint
async fn server_info() -> Json<Value> {
    Json(json!({
        "service": "OAuth2 MCP Inspector Test Server",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "MCP server with OAuth2 JWT authentication for testing",
        "endpoints": {
            "/mcp": "Main MCP JSON-RPC endpoint (OAuth2 protected)",
            "/health": "Health check endpoint",
            "/status": "Server status endpoint",
            "/metrics": "Server metrics endpoint",
            "/auth/tokens": "Generate test OAuth2 tokens",
            "/.well-known/jwks.json": "JWKS endpoint for JWT validation"
        },
        "authentication": {
            "type": "OAuth2 JWT",
            "method": "Authorization: Bearer <jwt-token>",
            "jwks_endpoint": "/.well-known/jwks.json",
            "supported_algorithms": ["RS256"],
            "scope_based_authorization": true
        },
        "testing": {
            "get_tokens": "GET /auth/tokens",
            "test_authentication": "Use tokens with MCP Inspector or curl",
            "inspector_compatible": true
        },
        "mcp_capabilities": {
            "resources": true,
            "tools": true,
            "prompts": true,
            "logging": true
        }
    }))
}

/// Create mock JWKS server for JWT validation
async fn start_mock_jwks_server(test_keys: TestKeys) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/.well-known/jwks.json", get(jwks_endpoint))
        .route("/auth/tokens", get(generate_tokens))
        .route("/info", get(server_info))
        .with_state(OAuth2McpServerState::new(
            test_keys,
            OAuth2Config::default(), // Will be updated with proper config
        ));

    let listener = TcpListener::bind("127.0.0.1:3002").await?;
    info!("🔑 Mock JWKS server started on http://127.0.0.1:3002");
    info!("📋 JWKS endpoint: http://127.0.0.1:3002/.well-known/jwks.json");
    info!("🎫 Test tokens: http://127.0.0.1:3002/auth/tokens");

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            warn!("JWKS server error: {}", e);
        }
    });

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("oauth2_mcp_server=info,airs_mcp=info")
        .init();

    info!("🚀 Starting OAuth2 MCP Inspector Test Server");

    // Generate test keys for JWT operations
    let test_keys = generate_test_keys()?;

    // Start mock JWKS server for JWT validation
    start_mock_jwks_server(test_keys.clone()).await?;

    // Create OAuth2 configuration
    let jwks_url = Url::parse("http://localhost:3002/.well-known/jwks.json")?;
    let oauth2_config = OAuth2Config::builder()
        .jwks_url(jwks_url)
        .audience("mcp-oauth2-test-server".to_string())
        .issuer("oauth2-mcp-test-issuer".to_string())
        .validation_config(ValidationConfig {
            require_exp: true,
            require_aud: true,
            require_iss: true,
            validate_nbf: true,
            leeway: Duration::from_secs(60),
            algorithms: vec!["RS256".to_string()],
        })
        .cache_config(CacheConfig {
            jwks_cache_ttl: Duration::from_secs(300),
            jwks_cache_max_size: 10,
            token_cache_ttl: Duration::from_secs(60),
            token_cache_max_size: 100,
        })
        .build()?;

    // Create OAuth2 validators and strategy
    let jwt_validator = Jwt::new(oauth2_config.clone())?;
    let scope_validator = Scope::with_default_mappings();
    let validator = Validator::new(jwt_validator, scope_validator);
    let oauth2_strategy = OAuth2Strategy::new(validator);
    let oauth2_adapter = OAuth2StrategyAdapter::new(oauth2_strategy);

    // Create HTTP authentication configuration
    let auth_config = HttpAuthConfig {
        include_error_details: true,
        auth_realm: "OAuth2 MCP Test Server".to_string(),
        request_timeout_secs: 30,
        skip_paths: vec![
            "/health".to_string(),
            "/status".to_string(),
            "/metrics".to_string(),
            "/info".to_string(),
        ],
    };

    // Create temporary directory for filesystem provider
    let temp_dir = tempfile::TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create test files
    std::fs::write(
        temp_path.join("oauth2-test.txt"),
        "Hello from OAuth2 protected MCP server!",
    )?;
    std::fs::write(
        temp_path.join("config.json"),
        r#"{"server": "oauth2-mcp-test", "version": "1.0.0", "auth": "oauth2"}"#,
    )?;
    std::fs::write(
        temp_path.join("README.md"),
        "# OAuth2 MCP Test Server\n\nThis file is accessible through OAuth2 protected resources.",
    )?;

    // Create MCP handlers with providers
    let handlers = McpHandlersBuilder::new()
        .with_resource_provider(Arc::new(
            FileSystemResourceProvider::new(&temp_path.canonicalize()?)
                .expect("Failed to create filesystem provider"),
        ))
        .with_tool_provider(Arc::new(MathToolProvider::new()))
        .with_prompt_provider(Arc::new(CodeReviewPromptProvider::new()))
        .with_logging_handler(Arc::new(StructuredLoggingHandler::new()));

    // Create server infrastructure
    let connection_manager = Arc::new(HttpConnectionManager::new(
        1000,
        HealthCheckConfig::default(),
    ));

    // Create HTTP transport configuration
    let transport_config = HttpTransportConfig::new()
        .bind_address("127.0.0.1:3001".parse()?)
        .max_connections(1000)
        .request_timeout(Duration::from_secs(30))
        .enable_buffer_pool()
        .buffer_pool_size(100);

    // Create the OAuth2-enabled MCP server
    let server = AxumHttpServer::with_handlers(
        connection_manager,
        handlers,
        transport_config,
    )
    .await?
    .with_authentication(oauth2_adapter, auth_config);

    info!("✅ OAuth2 MCP Server configured successfully");
    info!("🌐 Server will bind to: http://127.0.0.1:3001");
    info!("📡 MCP Endpoint: http://127.0.0.1:3001/mcp (OAuth2 protected)");
    info!("🏥 Health Check: http://127.0.0.1:3001/health");
    info!("📊 Server Status: http://127.0.0.1:3001/status");
    info!("📈 Server Metrics: http://127.0.0.1:3001/metrics");
    info!("🎫 Test Tokens: http://127.0.0.1:3002/auth/tokens");
    info!("🔑 JWKS Endpoint: http://127.0.0.1:3002/.well-known/jwks.json");
    info!("");
    info!("🔐 OAuth2 Configuration:");
    info!("   • Audience: mcp-oauth2-test-server");
    info!("   • Issuer: oauth2-mcp-test-issuer");
    info!("   • Algorithms: RS256");
    info!("   • JWKS URL: http://localhost:3002/.well-known/jwks.json");
    info!("");
    info!("🎯 Testing Instructions:");
    info!("   1. Get test tokens: curl http://localhost:3002/auth/tokens");
    info!("   2. Copy a token for your test scenario");
    info!("   3. Use MCP Inspector with OAuth2:");
    info!("      npx @modelcontextprotocol/inspector-cli \\");
    info!("        --transport http \\");
    info!("        --server-url http://localhost:3001/mcp \\");
    info!("        --header \"Authorization: Bearer <your-token>\"");
    info!("");
    info!("📋 Available Test Scenarios:");
    info!("   • full: Complete access to all MCP operations");
    info!("   • tools: Access to tools operations only");
    info!("   • resources: Access to resources operations only");
    info!("   • readonly: Read-only access to listings");
    info!("");
    info!("🔍 MCP Methods Available:");
    info!("   • initialize - Start MCP session");
    info!("   • resources/list - List OAuth2 protected resources");
    info!("   • resources/read - Read OAuth2 protected resources");
    info!("   • tools/list - List OAuth2 protected tools");
    info!("   • tools/call - Execute OAuth2 protected tools");
    info!("   • prompts/list - List OAuth2 protected prompts");
    info!("   • prompts/get - Get OAuth2 protected prompt content");
    info!("");
    info!("Press Ctrl+C to shutdown...");

    // Start the server
    let addr = "127.0.0.1:3001".parse()?;
    let mut server = server;
    server.bind(addr).await?;
    server.serve().await?;

    info!("👋 OAuth2 MCP Server shutdown complete");
    Ok(())
}
