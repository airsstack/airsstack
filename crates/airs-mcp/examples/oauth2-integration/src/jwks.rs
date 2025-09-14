//! JWKS Server Module
//!
//! This module provides a mock JWKS (JSON Web Key Set) server for testing
//! JWT token validation in the OAuth2 MCP integration.

use std::collections::HashMap;

use axum::{extract::State, response::Json, routing::get, Router};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tracing::{debug, info, warn};

use airs_mcp::oauth2::config::OAuth2Config;

use crate::tokens::{generate_test_token, TestKeys, TokenConfig};

/// Application state for the OAuth2 MCP server
#[derive(Clone)]
pub struct OAuth2McpServerState {
    /// Test RSA keys for JWT operations
    pub test_keys: TestKeys,
    /// OAuth2 configuration
    pub oauth2_config: OAuth2Config,
    /// Available test token configurations
    pub token_configs: HashMap<String, TokenConfig>,
}

impl OAuth2McpServerState {
    pub fn new(test_keys: TestKeys, oauth2_config: OAuth2Config) -> Self {
        Self {
            test_keys,
            oauth2_config,
            token_configs: TokenConfig::all_configs(),
        }
    }
}

/// JWKS endpoint handler for JWT validation
async fn jwks_endpoint(State(state): State<OAuth2McpServerState>) -> Json<Value> {
    debug!("🔍 JWKS endpoint requested");
    debug!("📋 JWKS response: {}", serde_json::to_string_pretty(&state.test_keys.jwks_response).unwrap_or_else(|_| "Failed to serialize".to_string()));
    Json(state.test_keys.jwks_response.clone())
}

/// Generate test tokens endpoint
async fn generate_tokens(State(state): State<OAuth2McpServerState>) -> Json<Value> {
    debug!("🎫 Test tokens endpoint requested");
    let mut tokens = HashMap::new();

    for (key, config) in &state.token_configs {
        debug!("🔧 Generating token for scenario: {}", key);
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
                debug!("✅ Token generated for {}: {} chars", key, token.len());
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
                warn!("❌ Failed to generate token for {}: {}", key, e);
            }
        }
    }

    info!("📋 Generated {} test tokens", tokens.len());
    Json(json!({
        "service": "OAuth2 MCP Test Server",
        "description": "Test tokens for OAuth2 MCP Inspector testing",
        "jwks_url": "http://localhost:3002/.well-known/jwks.json",
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
        "service": "OAuth2 MCP Integration Test Server",
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
pub async fn start_mock_jwks_server(
    test_keys: TestKeys, 
    oauth2_config: OAuth2Config
) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/.well-known/jwks.json", get(jwks_endpoint))
        .route("/auth/tokens", get(generate_tokens))
        .route("/info", get(server_info))
        .with_state(OAuth2McpServerState::new(test_keys, oauth2_config));

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