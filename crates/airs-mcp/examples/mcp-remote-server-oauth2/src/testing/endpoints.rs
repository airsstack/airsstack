//! Test Token Generation Endpoints
//!
//! This module provides HTTP endpoints for generating test JWT tokens
//! with different OAuth2 scopes for testing the AirsStack MCP server.

use std::collections::HashMap;
use axum::{extract::State, response::Json, routing::get, Router};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tracing::{info, warn};

use crate::auth::{keys::TestKeys, tokens::{TokenConfig, generate_test_token, generate_inspector_command, generate_curl_test}};

/// Token generation server state
#[derive(Clone)]
pub struct TokenServerState {
    pub test_keys: TestKeys,
    pub audience: String,
    pub issuer: String,
    pub token_configs: HashMap<String, TokenConfig>,
}

/// Generate test tokens endpoint
async fn generate_tokens(State(state): State<TokenServerState>) -> Json<Value> {
    let mut tokens = HashMap::new();

    for (key, config) in &state.token_configs {
        let scopes: Vec<&str> = config.scopes.iter().map(|s| s.as_str()).collect();
        
        match generate_test_token(
            &config.subject,
            &scopes,
            &state.audience,
            &state.issuer,
            config.expires_minutes,
            &state.test_keys.encoding_key,
        ) {
            Ok(token) => {
                let inspector_cmd = generate_inspector_command(&token);
                let curl_test = generate_curl_test(&token);

                tokens.insert(key, json!({
                    "name": config.name,
                    "description": config.description,
                    "subject": config.subject,
                    "scopes": config.scopes,
                    "expires_minutes": config.expires_minutes,
                    "token": token,
                    "inspector_command": inspector_cmd,
                    "curl_test": curl_test
                }));
            }
            Err(e) => {
                warn!("Failed to generate token for {}: {}", key, e);
            }
        }
    }

    Json(json!({
        "service": "OAuth2 MCP Remote Server",
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
        "service": "OAuth2 MCP Remote Server",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "MCP server with OAuth2 JWT authentication for testing AirsStack components",
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
        "airs_mcp_integration": {
            "transport": "AxumHttpServer from airs_mcp::transport::adapters::http",
            "authentication": "OAuth2Strategy from airs_mcp::authentication::strategies::oauth2",
            "providers": ["FileSystemResourceProvider", "MathToolProvider", "CodeReviewPromptProvider"]
        }
    }))
}

/// Token endpoints server for test token generation
pub struct TokenEndpoints;

impl TokenEndpoints {
    /// Start token generation endpoints server
    ///
    /// This provides endpoints for generating test tokens and server information
    pub async fn start(
        test_keys: TestKeys,
        audience: String,
        issuer: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut token_configs = HashMap::new();
        token_configs.insert("full".to_string(), TokenConfig::full_access());
        token_configs.insert("tools".to_string(), TokenConfig::tools_only());
        token_configs.insert("resources".to_string(), TokenConfig::resources_only());
        token_configs.insert("readonly".to_string(), TokenConfig::read_only());

        let state = TokenServerState {
            test_keys,
            audience,
            issuer,
            token_configs,
        };

        let app = Router::new()
            .route("/auth/tokens", get(generate_tokens))
            .route("/info", get(server_info))
            .with_state(state);

        let listener = TcpListener::bind("127.0.0.1:3003").await?;
        info!("🎫 Token endpoints server started on http://127.0.0.1:3003");
        info!("🎫 Test tokens: http://127.0.0.1:3003/auth/tokens");
        info!("📋 Server info: http://127.0.0.1:3003/info");

        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                warn!("Token endpoints server error: {}", e);
            }
        });

        Ok(())
    }
}
