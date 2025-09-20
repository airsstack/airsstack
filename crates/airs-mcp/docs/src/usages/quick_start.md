# Quick Start Guide

This guide shows how to get started with AIRS MCP using the actual implemented APIs.

## Installation

Add AIRS MCP to your `Cargo.toml`:

```toml
[dependencies]
airs-mcp = "0.1.0"
tokio = { version = "1.35", features = ["full"] }
serde_json = "1.0"
```

## Your First MCP Client

```rust
use airs_mcp::integration::{McpClientBuilder, McpResult};
use airs_mcp::transport::adapters::stdio::StdioTransportClientBuilder;
use std::time::Duration;

#[tokio::main]
async fn main() -> McpResult<()> {
    // Create STDIO transport client
    let transport = StdioTransportClientBuilder::new()
        .command("your-mcp-server-command")
        .timeout(Duration::from_secs(30))
        .build()
        .await?;
    
    // Create MCP client
    let mut client = McpClientBuilder::new()
        .client_info("my-client", "1.0.0")
        .build(transport);
    
    // Initialize connection with server
    let capabilities = client.initialize().await?;
    println!("Server capabilities: {:?}", capabilities);
    
    // List available tools
    let tools = client.list_tools().await?;
    println!("Available tools: {:?}", tools.tools);
    
    // List available resources
    let resources = client.list_resources().await?;
    println!("Available resources: {:?}", resources.resources);
    
    // List available prompts
    let prompts = client.list_prompts().await?;
    println!("Available prompts: {:?}", prompts.prompts);
    
    Ok(())
}
```

## Your First HTTP MCP Client

```rust
use airs_mcp::integration::{McpClientBuilder, McpResult};
use airs_mcp::transport::adapters::http::{HttpTransportClientBuilder, AuthMethod};
use std::time::Duration;

#[tokio::main]
async fn main() -> McpResult<()> {
    // Create HTTP transport client with authentication
    let transport = HttpTransportClientBuilder::new()
        .endpoint("http://localhost:3000/mcp")?
        .auth(AuthMethod::Bearer {
            token: "your-access-token".to_string(),
        })
        .timeout(Duration::from_secs(30))
        .build()
        .await?;
    
    // Create MCP client
    let mut client = McpClientBuilder::new()
        .client_info("my-http-client", "1.0.0")
        .build(transport);
    
    // Initialize and use
    let capabilities = client.initialize().await?;
    let tools = client.list_tools().await?;
    
    println!("Connected via HTTP, found {} tools", tools.tools.len());
    
    Ok(())
}
```

## Working with Tools

```rust
use airs_mcp::protocol::types::{CallToolRequest, RequestId};
use serde_json::json;

// Call a tool (after client initialization)
let tool_result = client.call_tool(CallToolRequest {
    method: "tools/call".to_string(),
    params: airs_mcp::protocol::types::CallToolParams {
        name: "echo".to_string(),
        arguments: Some(json!({"message": "Hello, World!"})),
    },
    id: RequestId::new_string("tool-call-1".to_string()),
    jsonrpc: "2.0".to_string(),
}).await?;

println!("Tool result: {:?}", tool_result);
```

## Working with Resources

```rust
use airs_mcp::protocol::types::{ReadResourceRequest, RequestId};

// Read a resource (after client initialization)  
let resource_content = client.read_resource(ReadResourceRequest {
    method: "resources/read".to_string(),
    params: airs_mcp::protocol::types::ReadResourceParams {
        uri: "file:///path/to/file.txt".to_string(),
    },
    id: RequestId::new_string("resource-read-1".to_string()),
    jsonrpc: "2.0".to_string(),
}).await?;

println!("Resource content: {:?}", resource_content);
```

## Error Handling

```rust
use airs_mcp::integration::{McpError, McpResult};

async fn handle_mcp_operations() -> McpResult<()> {
    // ... create client ...
    
    match client.initialize().await {
        Ok(capabilities) => {
            println!("Successfully connected: {:?}", capabilities);
        }
        Err(McpError::ConnectionFailed(msg)) => {
            eprintln!("Connection failed: {}", msg);
            return Err(McpError::ConnectionFailed(msg));
        }
        Err(McpError::ProtocolError(msg)) => {
            eprintln!("Protocol error: {}", msg);
            return Err(McpError::ProtocolError(msg));
        }
        Err(e) => {
            eprintln!("Other error: {:?}", e);
            return Err(e);
        }
    }
    
    Ok(())
}
```

## Configuration Options

### Client Configuration

```rust
use airs_mcp::protocol::types::{ClientCapabilities, ClientInfo, ProtocolVersion};
use std::time::Duration;

let client = McpClientBuilder::new()
    .client_info("my-app", "2.1.0")
    .capabilities(ClientCapabilities {
        experimental: None,
        sampling: None,
    })
    .protocol_version(ProtocolVersion::V2024_11_05)
    .timeout(Duration::from_secs(60))
    .build(transport);
```

### Transport Configuration

```rust
// STDIO Transport
let stdio_transport = StdioTransportClientBuilder::new()
    .command("mcp-server")
    .arg("--config")
    .arg("config.json")
    .timeout(Duration::from_secs(30))
    .build()
    .await?;

// HTTP Transport
let http_transport = HttpTransportClientBuilder::new()
    .endpoint("https://api.example.com/mcp")?
    .auth(AuthMethod::OAuth2 {
        access_token: "token".to_string(),
        token_type: Some("Bearer".to_string()),
    })
    .timeout(Duration::from_secs(45))
    .build()
    .await?;
```

## Next Steps

- See [Basic Examples](./basic_examples.md) for more detailed usage patterns
- Check [Advanced Patterns](./advanced_patterns.md) for complex scenarios  
- Read [Claude Desktop Integration](./claude_integration.md) for desktop integration
- Explore [Custom Transports](./custom_transports.md) for implementing new transport layers
    server.run_stdio().await?;
    Ok(())
}
```

## HTTP Server with Authentication

The AIRS MCP library supports generic authentication middleware:

```rust
use airs_mcp::transport::adapters::http::axum::AxumHttpServer;
use airs_mcp::transport::adapters::http::auth::middleware::HttpAuthConfig;
use airs_mcp::transport::adapters::http::auth::apikey::ApiKeyStrategyAdapter;
use airs_mcp::authentication::strategies::apikey::{ApiKeyStrategy, InMemoryApiKeyValidator, ApiKeyAuthData};
use airs_mcp::authentication::{AuthMethod, AuthContext};
use airs_mcp::authentication::strategies::apikey::types::ApiKeySource;
use airs_mcp::protocol::jsonrpc::concurrent::{ConcurrentProcessor, ProcessorConfig};
use airs_mcp::transport::adapters::http::config::HttpTransportConfig;
use airs_mcp::transport::adapters::http::connection_manager::HttpConnectionManager;
use airs_mcp::transport::adapters::http::session::SessionManager;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create infrastructure components (simplified - see examples for full setup)
    let connection_manager = Arc::new(HttpConnectionManager::new(10, Default::default()));
    let session_manager = Arc::new(SessionManager::new(
        // ... correlation manager setup
    ));
    let jsonrpc_processor = Arc::new(ConcurrentProcessor::new(
        ProcessorConfig::default()
    ));
    let config = HttpTransportConfig::new();
    
    // Option 1: Server without authentication (default)
    let server = AxumHttpServer::new_with_empty_handlers(
        connection_manager.clone(),
        session_manager.clone(),
        jsonrpc_processor.clone(),
        config.clone(),
    ).await?;
    
    // Option 2: Server with API key authentication
    let mut api_keys = HashMap::new();
    api_keys.insert(
        "your-api-key".to_string(),
        AuthContext::new(
            AuthMethod::new("apikey"),
            ApiKeyAuthData {
                key_id: "user_123".to_string(),
                source: ApiKeySource::AuthorizationBearer,
            },
        ),
    );
    
    let validator = InMemoryApiKeyValidator::new(api_keys);
    let strategy = ApiKeyStrategy::new(validator);
    let adapter = ApiKeyStrategyAdapter::new(strategy, Default::default());
    let auth_config = HttpAuthConfig {
        include_error_details: false,
        auth_realm: "MCP API".to_string(),
        request_timeout_secs: 30,
        skip_paths: vec!["/health".to_string(), "/metrics".to_string()],
    };
    
    let auth_server = server.with_authentication(adapter, auth_config);
    
    // Note: This example shows setup - actual binding/serving requires more infrastructure
    // See examples/axum_server_with_handlers.rs for complete working example
    
    Ok(())
}
```

**Key Benefits:**
- **Zero Runtime Overhead**: No `Box<dyn>` trait objects or vtable lookups
- Compile-Time Optimization: Authentication methods inlined by compiler
- Type Safety: Different authentication strategies create different server types
- Builder Pattern: Ergonomic `.with_authentication()` for type conversion
- Backward Compatibility: Existing code continues to work unchanged

## OAuth2 MCP Server

OAuth2 authentication with MCP Inspector validation.

```bash
# Quick start with OAuth2 example
cd examples/mcp-remote-server-oauth2
cargo run

# Server starts on three ports:
# - Proxy Server: http://127.0.0.1:3002 (public endpoint)
# - OAuth2 Endpoints: http://127.0.0.1:3003 (authentication)
# - MCP Server: http://127.0.0.1:3004 (protocol implementation)
```

**Test with MCP Inspector:**
```bash
npm install -g @modelcontextprotocol/inspector
npx @modelcontextprotocol/inspector

# Configure OAuth2 server:
# Endpoint: http://127.0.0.1:3002/mcp
# Authentication: OAuth2
# Discovery: http://127.0.0.1:3002/.well-known/oauth-authorization-server
# Client ID: mcp-inspector-client
```

**Features Demonstrated:**
- Complete OAuth2 flow (authorization code + PKCE + JWT)
- MCP Inspector compatibility validation
- Three-server architecture with smart proxy routing
- Scope-based authorization for MCP operations
- Error handling and audit logging

## Next Steps

- [Basic Examples](./basic_examples.md) - Learn common patterns
- [OAuth2 Integration](./oauth2_integration.md) - Complete OAuth2 guide with MCP Inspector
- [Authentication Guide](./authentication_guide.md) - Complete guide to authentication patterns
- [Claude Integration](./claude_integration.md) - Connect to Claude Desktop
- [Advanced Patterns](./advanced_patterns.md) - Complex usage patterns
