// Standard library imports
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

// Third-party crate imports
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use clap::{Arg, Command};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde_json::{json, Value};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};

// Internal module imports
use http_oauth2_client_integration::{
    OAuth2IntegrationError, TokenClaims,
};

/// OAuth2-Protected MCP Mock Server State
#[derive(Clone)]
struct McpServerState {
    /// JWKS URL for token validation
    jwks_url: String,
    /// Cached public key for JWT verification
    public_key: Option<DecodingKey>,
    /// Required scopes for different operations
    scope_requirements: HashMap<String, Vec<String>>,
}

impl McpServerState {
    fn new(jwks_url: String) -> Self {
        let mut scope_requirements = HashMap::new();
        scope_requirements.insert("tools/list".to_string(), vec!["mcp:read".to_string()]);
        scope_requirements.insert("resources/list".to_string(), vec!["mcp:read".to_string()]);
        scope_requirements.insert("tools/call".to_string(), vec!["mcp:write".to_string()]);
        scope_requirements.insert("resources/read".to_string(), vec!["mcp:read".to_string()]);
        scope_requirements.insert("test/protected".to_string(), vec!["mcp:read".to_string()]);

        Self {
            jwks_url,
            public_key: None,
            scope_requirements,
        }
    }

    /// Validate JWT token against JWKS
    async fn validate_token(&self, token: &str) -> Result<TokenClaims, OAuth2IntegrationError> {
        // Skip validation for demo token
        if token == "demo-token-for-testing" {
            return Ok(TokenClaims {
                sub: "demo-user".to_string(),
                aud: vec!["mcp-server".to_string()],
                iss: "http://localhost:3001".to_string(),
                exp: chrono::Utc::now().timestamp() as usize + 3600,
                iat: chrono::Utc::now().timestamp() as usize,
                scope: Some("mcp:read mcp:write".to_string()),
                client_id: Some("test-client".to_string()),
            });
        }

        // Use cached public key if available, otherwise fetch from JWKS URL
        let decoding_key = if let Some(ref key) = self.public_key {
            key.clone()
        } else {
            // In a real implementation, you would fetch from self.jwks_url
            // For demo purposes, use the hardcoded key
            println!("Warning: Using demo public key. In production, would fetch from JWKS URL: {}", self.jwks_url);
            DecodingKey::from_rsa_pem(DEMO_PUBLIC_KEY.as_bytes())
                .map_err(|e| OAuth2IntegrationError::TokenValidation {
                    message: format!("Failed to create decoding key: {}", e),
                })?
        };

        // Basic validation without signature verification for demo
        let validation = Validation::new(Algorithm::RS256);
        let token_data = decode::<TokenClaims>(token, &decoding_key, &validation)
            .map_err(|e| OAuth2IntegrationError::TokenValidation {
                message: format!("Token validation failed: {}", e),
            })?;

        Ok(token_data.claims)
    }

    /// Check if token has required scope for operation
    fn check_scope(&self, claims: &TokenClaims, operation: &str) -> bool {
        if let Some(required_scopes) = self.scope_requirements.get(operation) {
            if let Some(token_scope) = &claims.scope {
                let token_scopes: Vec<&str> = token_scope.split_whitespace().collect();
                return required_scopes.iter().any(|req| token_scopes.contains(&req.as_str()));
            }
            false
        } else {
            true // No scope requirement
        }
    }
}

/// Extract and validate Bearer token from Authorization header
async fn extract_token(headers: &HeaderMap) -> Result<String, StatusCode> {
    let auth_header = headers
        .get("authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(auth_header[7..].to_string())
}

/// Middleware for OAuth2 token validation
async fn oauth2_middleware(
    State(state): State<Arc<McpServerState>>,
    headers: HeaderMap,
    operation: &str,
) -> Result<TokenClaims, StatusCode> {
    let token = extract_token(&headers).await?;
    
    let claims = state
        .validate_token(&token)
        .await
        .map_err(|e| {
            warn!("Token validation failed: {}", e);
            StatusCode::UNAUTHORIZED
        })?;

    if !state.check_scope(&claims, operation) {
        warn!("Insufficient scope for operation: {}", operation);
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(claims)
}

/// Health check endpoint
async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "server": "mcp-mock-server",
        "authentication": "oauth2",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// MCP Tools List endpoint
async fn tools_list(
    State(state): State<Arc<McpServerState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    let _claims = oauth2_middleware(State(state), headers, "tools/list").await?;

    Ok(Json(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "tools": [
                {
                    "name": "echo",
                    "description": "Echo back the input text",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "Text to echo back"
                            }
                        },
                        "required": ["text"]
                    }
                },
                {
                    "name": "calculate",
                    "description": "Perform basic arithmetic calculations",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "expression": {
                                "type": "string",
                                "description": "Mathematical expression to evaluate"
                            }
                        },
                        "required": ["expression"]
                    }
                }
            ]
        }
    })))
}

/// MCP Resources List endpoint
async fn resources_list(
    State(state): State<Arc<McpServerState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    let _claims = oauth2_middleware(State(state), headers, "resources/list").await?;

    Ok(Json(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": {
            "resources": [
                {
                    "uri": "file://demo/readme.txt",
                    "name": "Demo README",
                    "description": "A sample README file",
                    "mimeType": "text/plain"
                },
                {
                    "uri": "data://demo/config.json",
                    "name": "Demo Configuration",
                    "description": "Sample configuration data",
                    "mimeType": "application/json"
                }
            ]
        }
    })))
}

/// Protected test endpoint
async fn test_protected(
    State(state): State<Arc<McpServerState>>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let claims = oauth2_middleware(State(state), headers, "test/protected").await?;

    Ok(Json(json!({
        "message": "Access granted to protected resource",
        "user": claims.sub,
        "client_id": claims.client_id,
        "scope": claims.scope,
        "request_payload": payload,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// OAuth2 token info endpoint (for debugging)
async fn token_info(
    State(state): State<Arc<McpServerState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    let claims = oauth2_middleware(State(state), headers, "token/info").await?;

    Ok(Json(json!({
        "sub": claims.sub,
        "iss": claims.iss,
        "aud": claims.aud,
        "exp": claims.exp,
        "iat": claims.iat,
        "scope": claims.scope,
        "client_id": claims.client_id
    })))
}

/// Demo public key for JWT validation (in production, fetch from JWKS)
const DEMO_PUBLIC_KEY: &str = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA1234567890abcdef...
-----END PUBLIC KEY-----"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🛡️  Starting OAuth2-Protected MCP Mock Server");

    let matches = Command::new("http-mcp-mock-server")
        .version("0.1.0")
        .author("AIRS Stack Contributors")
        .about("OAuth2-protected MCP server mock for testing OAuth2 integration")
        .arg(
            Arg::new("host")
                .long("host")
                .value_name("HOST")
                .help("Host to bind to")
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .value_name("PORT")
                .help("Port to listen on")
                .default_value("8081"),
        )
        .arg(
            Arg::new("jwks-url")
                .long("jwks-url")
                .value_name("URL")
                .help("JWKS URL for token validation")
                .default_value("http://localhost:8080/.well-known/jwks.json"),
        )
        .get_matches();

    let host = matches.get_one::<String>("host").unwrap();
    let port = matches.get_one::<String>("port").unwrap();
    let jwks_url = matches.get_one::<String>("jwks-url").unwrap();

    info!("📋 Configuration:");
    info!("  Host: {}", host);
    info!("  Port: {}", port);
    info!("  JWKS URL: {}", jwks_url);

    // Initialize server state
    let state = Arc::new(McpServerState::new(jwks_url.clone()));

    // Build the router
    let app = Router::new()
        .route("/health", get(health))
        .route("/tools/list", post(tools_list))
        .route("/resources/list", post(resources_list))
        .route("/test/protected", post(test_protected))
        .route("/token/info", get(token_info))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
        )
        .with_state(state);

    // Parse the socket address
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    
    info!("🚀 Starting OAuth2-protected MCP server on {}", addr);
    info!("📋 Available endpoints:");
    info!("  GET  /health - Health check");
    info!("  POST /tools/list - List available tools (requires mcp:read scope)");
    info!("  POST /resources/list - List available resources (requires mcp:read scope)");
    info!("  POST /test/protected - Test protected operation (requires mcp:read scope)");
    info!("  GET  /token/info - Token information (for debugging)");

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    // Graceful shutdown handler
    tokio::select! {
        result = axum::serve(listener, app) => {
            if let Err(e) = result {
                error!("Server error: {}", e);
            }
        }
        _ = signal::ctrl_c() => {
            info!("🛑 Received shutdown signal, stopping server...");
        }
    }

    info!("👋 OAuth2-protected MCP server stopped");
    Ok(())
}