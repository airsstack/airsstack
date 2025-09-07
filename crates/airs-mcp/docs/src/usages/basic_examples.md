# Basic Examples

*Production-ready integration patterns using actual AIRS MCP APIs*

## MCP Client Patterns

### Simple MCP Operations

```rust
use airs_mcp::integration::mcp::{McpClientBuilder, McpError};
use airs_mcp::transport::stdio::StdioTransport;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create transport and MCP client
    let transport = StdioTransport::spawn_server("./my-mcp-server").await?;
    let mut client = McpClientBuilder::new()
        .client_info("example-client", "1.0.0")
        .build(transport)
        .await?;
    
    // Initialize MCP connection
    client.initialize().await?;
    
    // List and call a tool
    let tools = client.list_tools().await?;
    if let Some(tool) = tools.first() {
        let result = client.call_tool(&tool.name, json!({
            "input": "test data"
        })).await?;
        println!("Tool result: {:?}", result);
    }
    
    // List and read a resource
    let resources = client.list_resources().await?;
    if let Some(resource) = resources.first() {
        let content = client.read_resource(&resource.uri).await?;
        println!("Resource content: {:?}", content);
    }
    
    // Clean shutdown
    client.shutdown().await?;
    Ok(())
}
```

### Error Handling with MCP Client

```rust
use airs_mcp::integration::mcp::{McpError, McpClientBuilder};

async fn robust_mcp_client() -> Result<(), Box<dyn std::error::Error>> {
    let transport = StdioTransport::spawn_server("./mcp-server").await?;
    let mut client = McpClientBuilder::new()
        .client_info("robust-client", "1.0.0")
        .default_timeout(std::time::Duration::from_secs(30))
        .auto_retry(true)
        .build(transport)
        .await?;
    
    match client.initialize().await {
        Ok(_) => println!("MCP client initialized successfully"),
        Err(McpError::Transport { source }) => {
            eprintln!("Transport error: {}", source);
            return Err(source.into());
        }
        Err(McpError::ProtocolError { message, .. }) => {
            eprintln!("Protocol error: {}", message);
        }
        Err(e) => {
            eprintln!("Unexpected error: {}", e);
        }
    }
    
    Ok(())
}
```

## Notification Handling

### Sending Notifications

```rust
// Send notification (no response expected)
client.notify("status/update", Some(json!({
    "status": "processing",
    "progress": 75
}))).await?;

// Send notification without parameters
client.notify("ping", None).await?;
```

### Receiving Notifications

```rust
use airs_mcp::integration::handler::NotificationHandler;
use airs_mcp::base::jsonrpc::JsonRpcNotification;
use async_trait::async_trait;

#[derive(Debug)]
struct StatusNotificationHandler;

#[async_trait]
impl NotificationHandler for StatusNotificationHandler {
    async fn handle_notification(
        &self,
        notification: &JsonRpcNotification,
    ) -> Result<(), airs_mcp::integration::error::IntegrationError> {
        match notification.method.as_str() {
            "status/update" => {
                if let Some(params) = &notification.params {
                    println!("Status update: {}", params);
                }
            }
            "ping" => {
                println!("Received ping notification");
            }
            _ => {
                println!("Unknown notification: {}", notification.method);
            }
        }
        Ok(())
    }
}
```

## Error Handling Patterns

### Comprehensive Error Handling

```rust
use airs_mcp::integration::error::IntegrationError;
use airs_mcp::integration::JsonRpcClient;
use airs_mcp::transport::StdioTransport;
use serde_json::json;

async fn robust_client_example() -> Result<(), Box<dyn std::error::Error>> {
    let transport = StdioTransport::new().await?;
    let mut client = JsonRpcClient::new(transport).await?;
    
    match client.call("risky/operation", Some(json!({"data": "test"}))).await {
        Ok(result) => {
            println!("Success: {}", result);
        }
        Err(IntegrationError::Transport(transport_error)) => {
            eprintln!("Transport error: {}", transport_error);
        }
        Err(IntegrationError::Correlation(correlation_error)) => {
            eprintln!("Correlation error: {}", correlation_error);
        }
        Err(IntegrationError::Json(json_error)) => {
            eprintln!("JSON parsing error: {}", json_error);
        }
        Err(IntegrationError::Timeout { timeout_ms }) => {
            eprintln!("Request timed out after {}ms", timeout_ms);
        }
        Err(IntegrationError::UnexpectedResponse { details }) => {
            eprintln!("Unexpected response format: {}", details);
        }
        Err(IntegrationError::Shutdown) => {
            eprintln!("Client has been shutdown");
        }
        Err(e) => {
            eprintln!("Unexpected error: {}", e);
        }
    }
    
    Ok(())
}
```

## Message Routing

### Router Configuration

```rust
use airs_mcp::integration::router::{MessageRouter, RouteConfig};
use airs_mcp::integration::handler::{RequestHandler, NotificationHandler};
use std::sync::Arc;

let config = RouteConfig::default();
let mut router = MessageRouter::new(config);

// Register request handlers
router.register_request_handler("math/add", Arc::new(AddHandler))?;
router.register_request_handler("math/subtract", Arc::new(SubtractHandler))?;
router.register_request_handler("string/reverse", Arc::new(ReverseHandler))?;

// Register notification handlers
router.register_notification_handler("status/update", Arc::new(StatusHandler))?;
router.register_notification_handler("log/info", Arc::new(LogHandler))?;
```

## Handler Registration

### Method Handler Implementation

```rust
use async_trait::async_trait;
use serde_json::{json, Value};
use airs_mcp::integration::handler::RequestHandler;
use airs_mcp::integration::error::IntegrationError;
use airs_mcp::base::jsonrpc::JsonRpcRequest;

#[derive(Debug)]
struct AddHandler;

#[async_trait]
impl RequestHandler for AddHandler {
    async fn handle_request(
        &self,
        request: &JsonRpcRequest,
    ) -> Result<Value, IntegrationError> {
        let params = request.params.as_ref().ok_or_else(|| {
            IntegrationError::other("Parameters required")
        })?;
        
        let a = params["a"].as_f64().ok_or_else(|| {
            IntegrationError::other("Parameter 'a' must be a number")
        })?;
        
        let b = params["b"].as_f64().ok_or_else(|| {
            IntegrationError::other("Parameter 'b' must be a number")
        })?;
        
        Ok(json!({"result": a + b}))
    }
}
```

## Transport Configuration

### Custom Transport Setup

```rust
use airs_mcp::transport::StdioTransport;
use airs_mcp::base::jsonrpc::streaming::StreamingConfig;

// Configure streaming parser settings
let config = StreamingConfig {
    read_buffer_size: 16384,        // 16KB read buffer
    max_message_size: 1024 * 1024,  // 1MB max message size
    strict_validation: true,         // Enable strict JSON validation
};

// Use default transport (most common case)
let transport = StdioTransport::new().await?;
```

### Connection Management

```rust
use tokio::time::{timeout, Duration};

// Graceful connection handling
async fn managed_connection() -> Result<(), Box<dyn std::error::Error>> {
    let transport = StdioTransport::new().await?;
    let mut client = JsonRpcClient::new(transport).await?;
    
    // Set reasonable timeout for operations
    let result = timeout(
        Duration::from_secs(10),
        client.call("long/operation", None)
    ).await??;
    
    // Always clean up
    client.shutdown().await?;
    Ok(())
}
```

---

*Next: [Advanced Patterns](./advanced_patterns.md) | Return to [Usages Overview](../usages.md)*
