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
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde_json::{json, Value};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};

// Internal module imports
use http_oauth2_client_integration::{OAuth2IntegrationError, TokenClaims};

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
        scope_requirements.insert("initialize".to_string(), vec!["mcp:read".to_string()]);
        scope_requirements.insert("tools/list".to_string(), vec!["mcp:read".to_string()]);
        scope_requirements.insert("resources/list".to_string(), vec!["mcp:read".to_string()]);
        scope_requirements.insert("tools/call".to_string(), vec!["mcp:write".to_string()]);
        scope_requirements.insert("resources/read".to_string(), vec!["mcp:read".to_string()]);

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
            println!(
                "Warning: Using demo public key. In production, would fetch from JWKS URL: {}",
                self.jwks_url
            );
            DecodingKey::from_rsa_pem(DEMO_PUBLIC_KEY.as_bytes()).map_err(|e| {
                OAuth2IntegrationError::TokenValidation {
                    message: format!("Failed to create decoding key: {}", e),
                }
            })?
        };

        // Configure validation with proper audience
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&["mcp-server"]); // Accept "mcp-server" as valid audience

        let token_data = decode::<TokenClaims>(token, &decoding_key, &validation).map_err(|e| {
            OAuth2IntegrationError::TokenValidation {
                message: format!("Token validation failed: {}", e),
            }
        })?;

        Ok(token_data.claims)
    }

    /// Check if token has required scope for operation
    fn check_scope(&self, claims: &TokenClaims, operation: &str) -> bool {
        if let Some(required_scopes) = self.scope_requirements.get(operation) {
            if let Some(token_scope) = &claims.scope {
                let token_scopes: Vec<&str> = token_scope.split_whitespace().collect();
                return required_scopes
                    .iter()
                    .any(|req| token_scopes.contains(&req.as_str()));
            }
            false
        } else {
            true // No scope requirement
        }
    }
}

/// Extract and validate Bearer token from Authorization header
async fn extract_token(headers: &HeaderMap) -> Result<String, StatusCode> {
    // Debug: Print all headers received
    println!("🔍 DEBUG: All headers received:");
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            if name.as_str().to_lowercase() == "authorization" {
                if value_str.starts_with("Bearer ") {
                    println!(
                        "  {}: Bearer [TOKEN_PRESENT:{}...]",
                        name,
                        &value_str[7..std::cmp::min(value_str.len(), 20)]
                    );
                } else {
                    println!("  {}: {}", name, value_str);
                }
            } else {
                println!("  {}: {}", name, value_str);
            }
        } else {
            println!("  {}: [non-UTF8 value]", name);
        }
    }

    let auth_header = headers
        .get("authorization")
        .ok_or_else(|| {
            println!("❌ ERROR: No Authorization header found!");
            StatusCode::UNAUTHORIZED
        })?
        .to_str()
        .map_err(|e| {
            println!("❌ ERROR: Invalid Authorization header format: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    if !auth_header.starts_with("Bearer ") {
        println!(
            "❌ ERROR: Authorization header does not start with 'Bearer ': {}",
            auth_header
        );
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = auth_header[7..].to_string();
    println!(
        "✅ Successfully extracted Bearer token (length: {})",
        token.len()
    );
    Ok(token)
}

/// Middleware for OAuth2 token validation
async fn oauth2_middleware(
    State(state): State<Arc<McpServerState>>,
    headers: HeaderMap,
    operation: &str,
) -> Result<TokenClaims, StatusCode> {
    info!("🔐 Validating OAuth2 token for operation: {}", operation);

    let token = extract_token(&headers).await.map_err(|e| {
        warn!("Failed to extract Bearer token: status {:?}", e);
        e
    })?;

    info!("🎫 Token extracted, validating...");
    let claims = state.validate_token(&token).await.map_err(|e| {
        warn!("Token validation failed: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    if !state.check_scope(&claims, operation) {
        warn!("Insufficient scope for operation: {}", operation);
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        "✅ OAuth2 validation successful for operation: {}",
        operation
    );
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

/// MCP JSON-RPC handler - single endpoint for all MCP operations
async fn mcp_jsonrpc_handler(
    State(state): State<Arc<McpServerState>>,
    headers: HeaderMap,
    Json(request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // Debug: Log all incoming headers
    info!("📥 Incoming MCP request headers:");
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            // Mask the token value for security
            if name.as_str().to_lowercase() == "authorization" {
                if value_str.starts_with("Bearer ") {
                    info!(
                        "  {}: Bearer [TOKEN_PRESENT:{}...]",
                        name,
                        &value_str[7..std::cmp::min(value_str.len(), 15)]
                    );
                } else {
                    info!("  {}: {}", name, value_str);
                }
            } else {
                info!("  {}: {}", name, value_str);
            }
        } else {
            info!("  {}: [non-UTF8 value]", name);
        }
    }

    // Extract JSON-RPC fields
    let method = request
        .get("method")
        .and_then(|m| m.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    info!("🔧 Processing JSON-RPC method: {}", method);

    let id = request.get("id").cloned().unwrap_or(json!(null));
    let _params = request.get("params").cloned().unwrap_or(json!({}));

    // Route based on MCP method
    match method {
        "initialize" => handle_initialize(state, headers, id).await,
        "tools/list" => handle_tools_list(state, headers, id).await,
        "resources/list" => handle_resources_list(state, headers, id).await,
        "tools/call" => handle_tool_call(state, headers, id, _params).await,
        _ => Ok(Json(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": format!("Unknown method: {}", method)
            }
        }))),
    }
}

/// Handle MCP initialize method
async fn handle_initialize(
    state: Arc<McpServerState>,
    headers: HeaderMap,
    id: Value,
) -> Result<Json<Value>, StatusCode> {
    // Check if client is sending proper OAuth2 authentication for initialize
    info!("🤝 Processing MCP initialize request - checking authentication...");
    let _claims = oauth2_middleware(State(state), headers, "initialize").await?;

    info!("✅ Initialize request authenticated successfully");
    Ok(Json(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": "1.0.0",
            "capabilities": {
                "tools": {
                    "listChanged": false
                },
                "resources": {
                    "subscribe": false,
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": "oauth2-mcp-mock-server",
                "version": "1.0.0"
            }
        }
    })))
}

/// Handle tools/list method
async fn handle_tools_list(
    state: Arc<McpServerState>,
    headers: HeaderMap,
    id: Value,
) -> Result<Json<Value>, StatusCode> {
    let _claims = oauth2_middleware(State(state), headers, "tools/list").await?;

    Ok(Json(json!({
        "jsonrpc": "2.0",
        "id": id,
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

/// Handle resources/list method
async fn handle_resources_list(
    state: Arc<McpServerState>,
    headers: HeaderMap,
    id: Value,
) -> Result<Json<Value>, StatusCode> {
    let _claims = oauth2_middleware(State(state), headers, "resources/list").await?;

    Ok(Json(json!({
        "jsonrpc": "2.0",
        "id": id,
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

/// Handle tools/call method
async fn handle_tool_call(
    state: Arc<McpServerState>,
    headers: HeaderMap,
    id: Value,
    params: Value,
) -> Result<Json<Value>, StatusCode> {
    let claims = oauth2_middleware(State(state), headers, "tools/call").await?;

    // Extract tool name and arguments
    let tool_name = params
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("unknown");

    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    Ok(Json(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [
                {
                    "type": "text",
                    "text": format!("Tool '{}' executed successfully with arguments: {}", tool_name, arguments)
                }
            ],
            "isError": false,
            "metadata": {
                "user": claims.sub,
                "client_id": claims.client_id,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        }
    })))
}

/// Demo public key for JWT validation (in production, fetch from JWKS)
const DEMO_PUBLIC_KEY: &str = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAzopdB15OiMljO725B14D
t3mnk3txrTrvg4wvjqjwy2MxxyTfRHoZDDtra8STUv9JDLfMXoY8yqMkusmHMCQX
vjzKXOsYLm5iV7w2aXpqG1b9La8bmlNBMhxjFWjWxFA8UcSgIjiQW1g2zmHn1u6i
bLF8qE7Zb+j22P+Lq6VKQQXsnfSbfJHQN23LIBGu3z/pKy+JMVQJgicJgkc/A3Bz
7r6Vaipwy99ytq22ajApgFxE63PEn/tB1LR+6Fe0lEvmb+bxypgS9HUppedhhJAh
zLd/ZzdPIUpdNTcLQvU8VQlFTe1DxT+tAb9Am7bM4/+z0ZkH1PU4bOrf8fARPyth
vwIDAQAB
-----END PUBLIC KEY-----"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt().with_env_filter("info").init();

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

    // Build the router - MCP uses a single JSON-RPC endpoint
    let app = Router::new()
        .route("/", post(mcp_jsonrpc_handler))
        .route("/health", get(health))
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
        .with_state(state);

    // Parse the socket address
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    info!("🚀 Starting OAuth2-protected MCP server on {}", addr);
    info!("📋 Available endpoints:");
    info!("  POST / - MCP JSON-RPC endpoint (requires OAuth2 authentication)");
    info!("  GET  /health - Health check (no authentication required)");
    info!("📋 Supported MCP methods:");
    info!("  initialize - Initialize MCP session");
    info!("  tools/list - List available tools (requires mcp:read scope)");
    info!("  resources/list - List available resources (requires mcp:read scope)");
    info!("  tools/call - Execute tools (requires mcp:write scope)");

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
