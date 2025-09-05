//! Simple HTTP MCP Server for Inspector Testing
//!
//! This is a minimal HTTP server that implements the MCP protocol
//! for testing with MCP Inspector tools.
//!
//! Usage:
//!   cargo run --example mcp-inspector-test-server
//!
//! Test with MCP Inspector:
//!   npx @modelcontextprotocol/inspector-cli --transport http --server-url http://localhost:3001/mcp

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::signal;
use tracing::{info, warn};
use tower_http::cors::CorsLayer;
use chrono;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

impl JsonRpcResponse {
    fn success(result: Value, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(error: Value, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Clone)]
struct AppState {
    server_name: String,
    server_version: String,
    api_keys: Arc<HashMap<String, ApiKeyInfo>>,
    auth_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiKeyInfo {
    name: String,
    description: String,
    created_at: String,
    last_used: Option<String>,
}

#[derive(Debug, Clone)]
struct AuthenticatedUser {
    api_key: String,
    key_info: ApiKeyInfo,
}

/// API Key Authentication Middleware
async fn api_key_auth(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<Value>)> {
    // Skip authentication if disabled
    if !state.auth_enabled {
        return Ok(next.run(request).await);
    }

    // Skip authentication for health and info endpoints
    let path = request.uri().path();
    if path == "/health" || path == "/info" {
        return Ok(next.run(request).await);
    }

    // Extract API key from Authorization header
    let headers = request.headers();
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .or_else(|| {
            // Also check for X-API-Key header
            headers
                .get("x-api-key")
                .and_then(|h| h.to_str().ok())
        });

    let api_key = match auth_header {
        Some(key) => key,
        None => {
            warn!("Missing API key in request to {}", path);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32001,
                        "message": "Authentication required",
                        "data": {
                            "auth_type": "api_key",
                            "header_options": ["Authorization: Bearer <key>", "X-API-Key: <key>"]
                        }
                    }
                })),
            ));
        }
    };

    // Validate API key
    let key_info = match state.api_keys.get(api_key) {
        Some(info) => info.clone(),
        None => {
            warn!("Invalid API key attempted: {}", api_key);
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32002,
                        "message": "Invalid API key",
                        "data": {
                            "hint": "Check your API key and try again"
                        }
                    }
                })),
            ));
        }
    };

    // Add authenticated user info to request extensions
    let auth_user = AuthenticatedUser {
        api_key: api_key.to_string(),
        key_info: key_info.clone(),
    };
    
    info!("Authenticated request from API key: {}", key_info.name);
    request.extensions_mut().insert(auth_user);
    Ok(next.run(request).await)
}

/// Handle MCP JSON-RPC requests
async fn handle_mcp_request(
    State(state): State<Arc<AppState>>,
    Json(request): Json<JsonRpcRequest>,
) -> Result<Json<JsonRpcResponse>, StatusCode> {
    info!("Received MCP request: {}", request.method);

    let response = match request.method.as_str() {
        "initialize" => handle_initialize(&state, request.params),
        "resources/list" => handle_resources_list(),
        "resources/templates/list" => handle_resources_templates_list(),
        "resources/read" => handle_resources_read(request.params),
        "tools/list" => handle_tools_list(),
        "tools/call" => handle_tools_call(request.params),
        "prompts/list" => handle_prompts_list(),
        "prompts/get" => handle_prompts_get(request.params),
        "ping" => Ok(json!({"message": "pong"})),
        _ => Err(json!({
            "code": -32601,
            "message": format!("Method not found: {}", request.method)
        })),
    };

    let json_response = match response {
        Ok(result) => JsonRpcResponse::success(result, request.id),
        Err(error) => JsonRpcResponse::error(error, request.id),
    };

    Ok(Json(json_response))
}

fn handle_initialize(state: &AppState, _params: Option<Value>) -> Result<Value, Value> {
    let capabilities = json!({
        "resources": {
            "subscribe": false,
            "listChanged": false
        },
        "tools": {},
        "prompts": {
            "listChanged": false
        }
    });

    let server_info = json!({
        "name": state.server_name,
        "version": state.server_version
    });

    Ok(json!({
        "protocolVersion": "2024-11-05",
        "capabilities": capabilities,
        "serverInfo": server_info
    }))
}

fn handle_resources_list() -> Result<Value, Value> {
    let resources = json!([
        {
            "uri": "file://test.txt",
            "name": "Test File",
            "description": "A simple test file",
            "mimeType": "text/plain"
        },
        {
            "uri": "file://config.json", 
            "name": "Config File",
            "description": "Configuration file",
            "mimeType": "application/json"
        }
    ]);

    Ok(json!({
        "resources": resources
    }))
}

fn handle_resources_templates_list() -> Result<Value, Value> {
    // Return empty resource templates list (this is optional in MCP)
    Ok(json!({
        "resourceTemplates": []
    }))
}

fn handle_resources_read(params: Option<Value>) -> Result<Value, Value> {
    let params = params.ok_or_else(|| json!({
        "code": -32602,
        "message": "Missing parameters"
    }))?;

    let uri = params.get("uri")
        .and_then(|u| u.as_str())
        .ok_or_else(|| json!({
            "code": -32602,
            "message": "Missing required parameter: uri"
        }))?;

    let content = match uri {
        "file://test.txt" => json!([{
            "type": "text",
            "text": "Hello from the test file!",
            "uri": uri
        }]),
        "file://config.json" => json!([{
            "type": "text", 
            "text": "{\"server\": \"mcp-inspector-test\", \"version\": \"1.0.0\"}",
            "uri": uri
        }]),
        _ => return Err(json!({
            "code": -32602,
            "message": format!("Resource not found: {}", uri)
        }))
    };

    Ok(json!({
        "contents": content
    }))
}

fn handle_tools_list() -> Result<Value, Value> {
    let tools = json!([
        {
            "name": "add",
            "description": "Add two numbers together",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "a": {"type": "number", "description": "First number"},
                    "b": {"type": "number", "description": "Second number"}
                },
                "required": ["a", "b"]
            }
        },
        {
            "name": "greet",
            "description": "Generate a personalized greeting",
            "inputSchema": {
                "type": "object", 
                "properties": {
                    "name": {"type": "string", "description": "Name to greet"}
                },
                "required": ["name"]
            }
        }
    ]);

    Ok(json!({
        "tools": tools
    }))
}

fn handle_tools_call(params: Option<Value>) -> Result<Value, Value> {
    let params = params.ok_or_else(|| json!({
        "code": -32602,
        "message": "Missing parameters"
    }))?;

    let name = params.get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| json!({
            "code": -32602, 
            "message": "Missing required parameter: name"
        }))?;

    let empty_args = json!({});
    let arguments = params.get("arguments")
        .unwrap_or(&empty_args);

    let result = match name {
        "add" => {
            let a = arguments.get("a")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| json!({
                    "code": -32602,
                    "message": "Missing or invalid parameter: a"
                }))?;
            let b = arguments.get("b")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| json!({
                    "code": -32602,
                    "message": "Missing or invalid parameter: b"
                }))?;
            
            let sum = a + b;
            json!({
                "content": [{
                    "type": "text",
                    "text": format!("Result: {} + {} = {}", a, b, sum)
                }]
            })
        }
        "greet" => {
            let name = arguments.get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| json!({
                    "code": -32602,
                    "message": "Missing or invalid parameter: name"
                }))?;
            
            json!({
                "content": [{
                    "type": "text", 
                    "text": format!("Hello, {}! Welcome to the MCP Inspector test server.", name)
                }]
            })
        }
        _ => return Err(json!({
            "code": -32602,
            "message": format!("Tool not found: {}", name)
        }))
    };

    Ok(result)
}

fn handle_prompts_list() -> Result<Value, Value> {
    let prompts = json!([
        {
            "name": "test_review",
            "description": "Generate a test review prompt",
            "arguments": [{
                "name": "test_type",
                "description": "Type of test to review", 
                "required": false
            }]
        },
        {
            "name": "help", 
            "description": "Get help with MCP commands"
        }
    ]);

    Ok(json!({
        "prompts": prompts
    }))
}

fn handle_prompts_get(params: Option<Value>) -> Result<Value, Value> {
    let params = params.ok_or_else(|| json!({
        "code": -32602,
        "message": "Missing parameters"
    }))?;

    let name = params.get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| json!({
            "code": -32602,
            "message": "Missing required parameter: name"
        }))?;

    let empty_args = json!({});
    let arguments = params.get("arguments")
        .unwrap_or(&empty_args);

    let (description, messages) = match name {
        "test_review" => {
            let test_type = arguments.get("test_type")
                .and_then(|v| v.as_str())
                .unwrap_or("general");
            
            (
                "Test Review Prompt".to_string(),
                json!([{
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!("Please review this {} test and provide feedback on:\n1. Test coverage\n2. Test quality\n3. Suggestions for improvement\n4. Best practices compliance", test_type)
                    }
                }])
            )
        }
        "help" => {
            (
                "MCP Help".to_string(),
                json!([{
                    "role": "user", 
                    "content": {
                        "type": "text",
                        "text": "Available MCP commands:\n• Resources: list and read test files\n• Tools: add numbers and generate greetings\n• Prompts: test review and help prompts\n\nThis server provides a simple MCP implementation for testing."
                    }
                }])
            )
        }
        _ => return Err(json!({
            "code": -32602,
            "message": format!("Prompt not found: {}", name)
        }))
    };

    Ok(json!({
        "description": description,
        "messages": messages
    }))
}

/// Health check endpoint
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "mcp-inspector-test-server",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "capabilities": ["resources", "tools", "prompts"]
    }))
}

/// Server info endpoint
async fn server_info(State(state): State<Arc<AppState>>) -> Json<Value> {
    let auth_info = if state.auth_enabled {
        json!({
            "enabled": true,
            "type": "API Key",
            "methods": [
                "Authorization: Bearer <api-key>",
                "X-API-Key: <api-key>"
            ],
            "test_keys": state.api_keys.iter().map(|(key, info)| {
                json!({
                    "key": key,
                    "name": info.name,
                    "description": info.description
                })
            }).collect::<Vec<_>>()
        })
    } else {
        json!({
            "enabled": false,
            "note": "Authentication is disabled - all requests allowed"
        })
    };

    let inspector_command = if state.auth_enabled {
        "npx @modelcontextprotocol/inspector-cli --transport http --server-url http://localhost:3001/mcp --header \"Authorization: Bearer demo-key-123\""
    } else {
        "npx @modelcontextprotocol/inspector-cli --transport http --server-url http://localhost:3001/mcp"
    };

    Json(json!({
        "service": "MCP Inspector Test Server",
        "version": "1.0.0",
        "endpoints": {
            "/mcp": "Main MCP JSON-RPC endpoint (requires auth if enabled)",
            "/health": "Health check endpoint (no auth required)",
            "/info": "Server information (no auth required)",
            "/auth/test": "API key test endpoint (requires auth if enabled)"
        },
        "authentication": auth_info,
        "test_with_inspector": {
            "command": inspector_command,
            "available_methods": [
                "initialize",
                "resources/list",
                "resources/read", 
                "tools/list",
                "tools/call",
                "prompts/list",
                "prompts/get"
            ]
        }
    }))
}

/// Authentication test endpoint
async fn auth_test(request: Request) -> Json<Value> {
    // Check if user is authenticated
    let auth_user = request.extensions().get::<AuthenticatedUser>();
    
    match auth_user {
        Some(user) => {
            Json(json!({
                "authenticated": true,
                "api_key": user.api_key,
                "user_info": {
                    "name": user.key_info.name,
                    "description": user.key_info.description,
                    "created_at": user.key_info.created_at
                },
                "message": "Authentication successful! You can now access protected MCP endpoints.",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }
        None => {
            Json(json!({
                "authenticated": false,
                "message": "No authentication provided or authentication disabled",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("mcp_inspector_test_server=info")
        .init();

    info!("🚀 Starting MCP Inspector Test Server");

    // Check if authentication should be enabled
    let auth_enabled = std::env::var("MCP_AUTH_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase() == "true";

    // Create API keys for testing
    let mut api_keys = HashMap::new();
    api_keys.insert(
        "demo-key-123".to_string(),
        ApiKeyInfo {
            name: "Demo User".to_string(),
            description: "Demo API key for testing MCP Inspector".to_string(),
            created_at: "2025-09-05T00:00:00Z".to_string(),
            last_used: None,
        },
    );
    api_keys.insert(
        "production-key-456".to_string(),
        ApiKeyInfo {
            name: "Production User".to_string(),
            description: "Production API key for testing".to_string(),
            created_at: "2025-09-05T00:00:00Z".to_string(),
            last_used: None,
        },
    );
    api_keys.insert(
        "inspector-test-789".to_string(),
        ApiKeyInfo {
            name: "MCP Inspector".to_string(),
            description: "Special key for MCP Inspector testing".to_string(),
            created_at: "2025-09-05T00:00:00Z".to_string(),
            last_used: None,
        },
    );

    // Create application state
    let state = Arc::new(AppState {
        server_name: "MCP Inspector Test Server".to_string(),
        server_version: "1.0.0".to_string(),
        api_keys: Arc::new(api_keys),
        auth_enabled,
    });

    // Create router with authentication middleware
    let app = Router::new()
        .route("/mcp", post(handle_mcp_request))
        .route("/health", get(health_check))
        .route("/info", get(server_info))
        .route("/auth/test", post(auth_test))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            api_key_auth,
        ))
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    info!("✅ Server started successfully");
    info!("🌐 Server Address: http://{}", addr);
    info!("📡 MCP Endpoint: http://{}/mcp", addr);
    info!("🏥 Health Check: http://{}/health", addr);
    info!("ℹ️  Server Info: http://{}/info", addr);
    info!("🔐 Auth Test: http://{}/auth/test", addr);
    info!("");
    
    if state.auth_enabled {
        info!("🔑 Authentication: ENABLED (API Key required)");
        info!("📋 Test API Keys:");
        for (key, info) in state.api_keys.iter() {
            info!("   • {} ({}): {}", key, info.name, info.description);
        }
        info!("");
        info!("🔍 Test with MCP Inspector (with authentication):");
        info!("   npx @modelcontextprotocol/inspector-cli \\");
        info!("     --transport http \\");
        info!("     --server-url http://localhost:3001/mcp \\");
        info!("     --header \"Authorization: Bearer demo-key-123\"");
        info!("");
        info!("💡 Or test manually with curl:");
        info!("   curl -X POST http://localhost:3001/auth/test \\");
        info!("     -H \"Content-Type: application/json\" \\");
        info!("     -H \"Authorization: Bearer demo-key-123\" \\");
        info!("     -d '{{}}'");
    } else {
        info!("🔓 Authentication: DISABLED (all requests allowed)");
        info!("");
        info!("🔍 Test with MCP Inspector (no authentication):");
        info!("   npx @modelcontextprotocol/inspector-cli \\");
        info!("     --transport http \\");
        info!("     --server-url http://localhost:3001/mcp");
    }
    info!("");
    info!("📋 Available MCP Methods:");
    info!("   • initialize - Start MCP session");
    info!("   • resources/list - List test resources");
    info!("   • resources/read - Read test resources");
    info!("   • tools/list - List available tools");
    info!("   • tools/call - Execute tools (add, greet)");
    info!("   • prompts/list - List available prompts");
    info!("   • prompts/get - Get prompt content");
    info!("");
    info!("Press Ctrl+C to shutdown...");

    // Run server
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("👋 Server shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
