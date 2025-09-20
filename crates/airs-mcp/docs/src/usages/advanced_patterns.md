# Patterns

*Implementation patterns and usage techniques*


## Buffer Pooling and Performance Optimization

### HTTP Buffer Pool Implementation

AIRS MCP provides a production-ready HTTP buffer pool that reduces allocation overhead:

```rust
use airs_mcp::transport::http::{BufferPool, BufferPoolConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure HTTP buffer pool
    let config = BufferPoolConfig::new()
        .max_buffers(100)              // Pool up to 100 buffers
        .buffer_size(8 * 1024)         // 8KB buffers
        .adaptive_sizing(true);        // Enable adaptive sizing
    
    let pool = BufferPool::new(config);
    
    // Get buffers with automatic pooling
    let mut buffer = pool.get_buffer();
    buffer.extend_from_slice(b"Hello, World!");
    
    // Buffer automatically returns to pool when dropped
    
    Ok(())
}
```

### Buffer Pool Metrics and Monitoring

```rust
use airs_mcp::transport::http::BufferPool;

async fn monitor_buffer_performance(pool: &BufferPool) {
    let stats = pool.stats();
    
    println!("HTTP Buffer Pool Performance:");
    println!("  Available Buffers: {}", stats.available_buffers);
    println!("  Total Buffers: {}", stats.total_buffers);
    println!("  Max Buffers: {}", stats.max_buffers);
}
```


## JSON-RPC Message Patterns

### Working with Different Message Types

```rust
use airs_mcp::{JsonRpcRequest, JsonRpcResponse, JsonRpcNotification, RequestId, JsonRpcMessageTrait};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create different message types
    let request = JsonRpcRequest::new(
        "ping",
        Some(json!({"message": "hello"})),
        RequestId::new_string("req-001")
    );
    
    let response = JsonRpcResponse::success(
        json!({"message": "pong"}),
        RequestId::new_string("req-001")
    );
    
    let notification = JsonRpcNotification::new(
        "heartbeat",
        Some(json!({"timestamp": "2025-09-20T19:00:00Z"}))
    );
    
    // All message types implement the same serialization trait
    println!("Request: {}", request.to_json()?);
    println!("Response: {}", response.to_json()?);
    println!("Notification: {}", notification.to_json()?);
    
    Ok(())
}
```

### Request ID Patterns

```rust
use airs_mcp::{JsonRpcRequest, RequestId};
use serde_json::json;

fn request_id_examples() -> Result<(), Box<dyn std::error::Error>> {
    // Numeric IDs for simple counting
    let numeric_request = JsonRpcRequest::new(
        "calculate",
        Some(json!({"operation": "add", "values": [1, 2, 3]})),
        RequestId::new_number(42)
    );
    
    // String IDs for UUIDs or correlation tracking
    let string_request = JsonRpcRequest::new(
        "fetch_data",
        Some(json!({"table": "users", "limit": 10})),
        RequestId::new_string("fetch-users-001")
    );
    
    // Access ID values
    match numeric_request.id {
        Some(RequestId::Number(n)) => println!("Numeric ID: {}", n),
        Some(RequestId::String(s)) => println!("String ID: {}", s),
        None => println!("No ID (notification)"),
    }
    
    Ok(())
}
```

## MCP Client Patterns

### Basic Client Usage

```rust
use airs_mcp::integration::{McpClientBuilder, McpResult};
use airs_mcp::transport::adapters::stdio::StdioTransportClientBuilder;
use std::time::Duration;

async fn basic_client_example() -> McpResult<()> {
    // Create transport
    let transport = StdioTransportClientBuilder::new()
        .command("python")
        .args(vec!["-m".to_string(), "my_mcp_server".to_string()])
        .timeout(Duration::from_secs(30))
        .build()
        .await?;
    
    // Create client
    let mut client = McpClientBuilder::new()
        .client_info("my-client", "1.0.0")
        .timeout(Duration::from_secs(60))
        .build(transport);
    
    // Initialize connection
    let capabilities = client.initialize().await?;
    println!("Server capabilities: {:?}", capabilities);
    
    // List available tools
    let tools = client.list_tools().await?;
    println!("Available tools: {}", tools.tools.len());
    
    // List available resources
    let resources = client.list_resources().await?;
    println!("Available resources: {}", resources.resources.len());
    
    client.close().await?;
    Ok(())
}
```

### HTTP Client with Authentication

```rust
use airs_mcp::integration::{McpClientBuilder, McpResult};
use airs_mcp::transport::adapters::http::{HttpTransportClientBuilder, AuthMethod};
use std::time::Duration;

async fn http_client_example() -> McpResult<()> {
    // Create HTTP transport with Bearer token
    let transport = HttpTransportClientBuilder::new()
        .endpoint("https://api.example.com/mcp")?
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
    
    // Use the client
    client.initialize().await?;
    let tools = client.list_tools().await?;
    
    // Call a specific tool
    if let Some(tool) = tools.tools.first() {
        let result = client.call_tool(&tool.name, None).await?;
        println!("Tool result: {:?}", result);
    }
    
    client.close().await?;
    Ok(())
}
```

## Error Handling Patterns

### Comprehensive Error Handling

```rust
use airs_mcp::integration::{McpClient, McpError, McpResult};
use airs_mcp::transport::adapters::stdio::StdioTransportClient;

async fn error_handling_example(
    client: &mut McpClient<StdioTransportClient>
) -> McpResult<()> {
    match client.call_tool("calculator", Some(serde_json::json!({"operation": "divide", "a": 10, "b": 0}))).await {
        Ok(result) => {
            println!("Success: {:?}", result);
        }
        Err(McpError::Protocol(protocol_error)) => {
            eprintln!("Protocol error: {}", protocol_error);
        }
        Err(McpError::Transport(transport_error)) => {
            eprintln!("Transport error: {}", transport_error);
        }
        Err(McpError::Timeout) => {
            eprintln!("Request timed out");
        }
        Err(McpError::InvalidState(msg)) => {
            eprintln!("Invalid state: {}", msg);
        }
        Err(other) => {
            eprintln!("Other error: {}", other);
        }
    }
    
    Ok(())
}
```

## Real-World Integration Patterns

### Multi-Tool Workflow

```rust
use airs_mcp::integration::{McpClient, McpResult};
use airs_mcp::transport::adapters::stdio::StdioTransportClient;
use serde_json::json;

async fn multi_tool_workflow(
    client: &mut McpClient<StdioTransportClient>
) -> McpResult<()> {
    // Step 1: Get available tools
    let tools = client.list_tools().await?;
    println!("Found {} tools", tools.tools.len());
    
    // Step 2: Find specific tools
    let calculator = tools.tools.iter().find(|t| t.name == "calculator");
    let text_processor = tools.tools.iter().find(|t| t.name == "text_processor");
    
    if let Some(calc) = calculator {
        // Step 3: Perform calculation
        let calc_result = client.call_tool(
            &calc.name,
            Some(json!({"operation": "multiply", "a": 25, "b": 4}))
        ).await?;
        
        println!("Calculation result: {:?}", calc_result);
        
        // Step 4: Process the result with text tool
        if let Some(processor) = text_processor {
            let text_result = client.call_tool(
                &processor.name,
                Some(json!({
                    "action": "format",
                    "text": format!("The result is: {:?}", calc_result.content)
                }))
            ).await?;
            
            println!("Formatted result: {:?}", text_result);
        }
    }
    
    Ok(())
}
```

### Resource Management

```rust
use airs_mcp::integration::{McpClient, McpResult};
use airs_mcp::transport::adapters::stdio::StdioTransportClient;

async fn resource_management_example(
    client: &mut McpClient<StdioTransportClient>
) -> McpResult<()> {
    // List all available resources
    let resources = client.list_resources().await?;
    
    for resource in &resources.resources {
        println!("Resource: {} ({})", resource.name, resource.uri);
        
        // Read each resource
        match client.read_resource(&resource.uri).await {
            Ok(content) => {
                println!("  Content type: {:?}", content.mimeType);
                if let Some(text) = content.text {
                    println!("  Preview: {}...", 
                        text.chars().take(100).collect::<String>());
                }
            }
            Err(e) => {
                eprintln!("  Failed to read: {}", e);
            }
        }
    }
    
    Ok(())
}
```

## Transport Layer Patterns

### Custom Configuration

```rust
use airs_mcp::transport::adapters::stdio::StdioTransportClientBuilder;
use std::time::Duration;

async fn custom_transport_config() -> Result<(), Box<dyn std::error::Error>> {
    // Custom STDIO configuration
    let transport = StdioTransportClientBuilder::new()
        .command("python")
        .args(vec!["-m".to_string(), "my_server".to_string()])
        .timeout(Duration::from_secs(45))
        .build()
        .await?;
    
    // Use the transport...
    Ok(())
}
```

## Testing Patterns

### Mock Responses for Testing

```rust
use airs_mcp::{JsonRpcResponse, RequestId, JsonRpcMessageTrait};
use serde_json::json;

fn create_test_responses() -> Result<(), Box<dyn std::error::Error>> {
    // Create success response
    let success = JsonRpcResponse::success(
        json!({"result": "operation completed", "data": [1, 2, 3]}),
        RequestId::new_number(1)
    );
    
    // Create error response
    let error = JsonRpcResponse::error(
        json!({"code": -32602, "message": "Invalid params", "data": "Expected number"}),
        Some(RequestId::new_number(2))
    );
    
    // Serialize for testing
    let success_json = success.to_json()?;
    let error_json = error.to_json()?;
    
    println!("Success response: {}", success_json);
    println!("Error response: {}", error_json);
    
    Ok(())
}
```

## Best Practices

### Session Management

```rust
use airs_mcp::integration::{McpClientBuilder, McpSessionState, McpResult, McpError};
use airs_mcp::transport::adapters::stdio::StdioTransportClientBuilder;
use std::time::Duration;

async fn session_management_example() -> McpResult<()> {
    let transport = StdioTransportClientBuilder::new()
        .command("python")
        .args(vec!["-m".to_string(), "my_server".to_string()])
        .timeout(Duration::from_secs(30))
        .build()
        .await?;
        
    let mut client = McpClientBuilder::new()
        .client_info("session-client", "1.0.0")
        .build(transport);
    
    // Check session state before operations
    match client.session_state() {
        McpSessionState::NotInitialized => {
            println!("Initializing session...");
            client.initialize().await?;
        }
        McpSessionState::Ready => {
            println!("Session already initialized");
        }
        McpSessionState::Initializing => {
            println!("Session is initializing...");
            // Wait or handle appropriately
        }
        McpSessionState::Failed => {
            return Err(McpError::InvalidState("Session failed".to_string()));
        }
    }
    
    // Perform operations...
    let tools = client.list_tools().await?;
    
    // Always clean up
    client.close().await?;
    
    Ok(())
}
```
