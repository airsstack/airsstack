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
use airs_mcp::integration::handler::RequestHandler;
use async_trait::async_trait;

#[derive(Debug)]
struct NotificationHandler;

#[async_trait]
impl RequestHandler for NotificationHandler {
    async fn handle_notification(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<(), airs_mcp::integration::error::IntegrationError> {
        match method {
            "status/update" => {
                if let Some(params) = params {
                    println!("Status update: {}", params);
                }
            }
            "ping" => {
                println!("Received ping notification");
            }
            _ => {
                println!("Unknown notification: {}", method);
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
use airs_mcp::base::jsonrpc::message::ErrorCode;

async fn robust_client_example() -> Result<(), Box<dyn std::error::Error>> {
    let transport = StdioTransport::new().await?;
    let mut client = JsonRpcClient::new(transport).await?;
    
    match client.call("risky/operation", Some(json!({"data": "test"}))).await {
        Ok(result) => {
            println!("Success: {}", result);
        }
        Err(IntegrationError::JsonRpc { code, message, .. }) => {
            match code {
                ErrorCode::InvalidRequest => {
                    eprintln!("Invalid request format: {}", message);
                }
                ErrorCode::MethodNotFound => {
                    eprintln!("Method not supported: {}", message);
                }
                ErrorCode::InvalidParams => {
                    eprintln!("Invalid parameters: {}", message);
                }
                ErrorCode::InternalError => {
                    eprintln!("Server error: {}", message);
                }
                _ => {
                    eprintln!("JSON-RPC error {}: {}", code as i32, message);
                }
            }
        }
        Err(IntegrationError::Transport { source, .. }) => {
            eprintln!("Transport error: {}", source);
        }
        Err(IntegrationError::Timeout { duration }) => {
            eprintln!("Request timed out after {:?}", duration);
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
use airs_mcp::integration::router::RequestRouter;
use airs_mcp::integration::handler::RequestHandler;

let mut router = RequestRouter::new();

// Register method handlers
router.register_method("math/add", Box::new(AddHandler));
router.register_method("math/subtract", Box::new(SubtractHandler));
router.register_method("string/reverse", Box::new(ReverseHandler));

// Register notification handlers
router.register_notification("status/*", Box::new(StatusHandler));
router.register_notification("log/*", Box::new(LogHandler));
```

## Handler Registration

### Method Handler Implementation

```rust
use async_trait::async_trait;
use serde_json::{json, Value};

#[derive(Debug)]
struct AddHandler;

#[async_trait]
impl RequestHandler for AddHandler {
    async fn handle_request(
        &self,
        _method: &str,
        params: Option<Value>,
    ) -> Result<Value, IntegrationError> {
        let params = params.ok_or_else(|| {
            IntegrationError::InvalidParams {
                message: "Parameters required".to_string(),
            }
        })?;
        
        let a = params["a"].as_f64().ok_or_else(|| {
            IntegrationError::InvalidParams {
                message: "Parameter 'a' must be a number".to_string(),
            }
        })?;
        
        let b = params["b"].as_f64().ok_or_else(|| {
            IntegrationError::InvalidParams {
                message: "Parameter 'b' must be a number".to_string(),
            }
        })?;
        
        Ok(json!({"result": a + b}))
    }
}
```

## Transport Configuration

### Custom Transport Setup

```rust
use airs_mcp::transport::stdio::StdioTransport;
use airs_mcp::base::jsonrpc::streaming::StreamingConfig;

let config = StreamingConfig::builder()
    .buffer_size(16384)
    .enable_compression(true)
    .max_message_size(1024 * 1024) // 1MB
    .build();

let transport = StdioTransport::with_config(config).await?;
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
