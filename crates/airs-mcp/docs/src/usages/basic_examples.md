# Basic Examples

Examples showing common usage patterns with the actual AIRS MCP implementation.

## MCP Client Examples

### Basic Client Operations

```rust
use airs_mcp::integration::{McpClientBuilder, McpResult};
use airs_mcp::transport::adapters::stdio::StdioTransportClientBuilder;
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> McpResult<()> {
    // Create STDIO transport
    let transport = StdioTransportClientBuilder::new()
        .command("your-mcp-server")
        .timeout(Duration::from_secs(30))
        .build()
        .await?;
    
    // Create and initialize client
    let mut client = McpClientBuilder::new()
        .client_info("example-client", "1.0.0")
        .build(transport);
    
    let capabilities = client.initialize().await?;
    println!("Server capabilities: {:?}", capabilities);
    
    // List available tools
    let tools_response = client.list_tools().await?;
    println!("Available tools: {:?}", tools_response.tools);
    
    // Call a tool if available
    if let Some(tool) = tools_response.tools.first() {
        let result = client.call_tool(&tool.name, Some(json!({"input": "test data"}))).await?;
        println!("Tool result: {:?}", result);
    }
    
    // List and read resources
    let resources_response = client.list_resources().await?;
    if let Some(resource) = resources_response.resources.first() {
        let content = client.read_resource(&resource.uri).await?;
        println!("Resource content: {:?}", content);
    }
    
    // List available prompts
    let prompts_response = client.list_prompts().await?;
    println!("Available prompts: {:?}", prompts_response.prompts);
    
    client.close().await?;
    Ok(())
}
```

### HTTP Client Example

```rust
use airs_mcp::integration::{McpClientBuilder, McpResult};
use airs_mcp::transport::adapters::http::{HttpTransportClientBuilder, AuthMethod};
use std::time::Duration;

#[tokio::main]
async fn main() -> McpResult<()> {
    // Create HTTP transport with authentication
    let transport = HttpTransportClientBuilder::new()
        .endpoint("http://localhost:3000/mcp")?
        .auth(AuthMethod::Bearer {
            token: "your-access-token".to_string(),
        })
        .timeout(Duration::from_secs(30))
        .build()
        .await?;
    
    // Create client
    let mut client = McpClientBuilder::new()
        .client_info("http-client", "1.0.0")
        .build(transport);
    
    // Initialize and perform operations
    let _capabilities = client.initialize().await?;
    let tools = client.list_tools().await?;
    let resources = client.list_resources().await?;
    let prompts = client.list_prompts().await?;
    
    println!("Connected via HTTP: {} tools, {} resources, {} prompts", 
             tools.tools.len(), 
             resources.resources.len(), 
             prompts.prompts.len());
    
    Ok(())
}
```

### Error Handling Patterns

```rust
use airs_mcp::integration::{McpError, McpClientBuilder, McpResult};

async fn robust_client_handling() -> McpResult<()> {
    // ... create transport and client ...
    
    match client.initialize().await {
        Ok(capabilities) => {
            println!("Successfully initialized with capabilities: {:?}", capabilities);
        }
        Err(McpError::Integration(integration_error)) => {
            eprintln!("Integration error: {}", integration_error);
            return Err(McpError::Integration(integration_error));
        }
        Err(McpError::Protocol(protocol_error)) => {
            eprintln!("Protocol error: {}", protocol_error);
            return Err(McpError::Protocol(protocol_error));
        }
        Err(McpError::NotConnected) => {
            eprintln!("Not connected to server");
            return Err(McpError::NotConnected);
        }
        Err(e) => {
            eprintln!("Unexpected error: {:?}", e);
            return Err(e);
        }
    }
    
    // Continue with operations...
    match client.list_tools().await {
        Ok(response) => {
            for tool in &response.tools {
                println!("Tool: {} - {}", tool.name, 
                         tool.description.as_deref().unwrap_or("No description"));
            }
        }
        Err(e) => {
            eprintln!("Failed to list tools: {:?}", e);
        }
    }
    
    Ok(())
}
```

## Configuration Examples

### Client Configuration

```rust
use airs_mcp::integration::McpClientBuilder;
use airs_mcp::protocol::types::{ClientCapabilities, ProtocolVersion};
use std::time::Duration;

let client = McpClientBuilder::new()
    .client_info("my-application", "2.1.0")
    .capabilities(ClientCapabilities {
        experimental: None,
        sampling: None,
    })
    .protocol_version(ProtocolVersion::V2024_11_05)
    .timeout(Duration::from_secs(60))
    .build(transport);
```

### Transport Configuration Examples

```rust
// STDIO with multiple arguments
let stdio_transport = StdioTransportClientBuilder::new()
    .command("mcp-server")
    .arg("--config")
    .arg("/path/to/config.json")
    .arg("--verbose")
    .timeout(Duration::from_secs(45))
    .build()
    .await?;

// HTTP with OAuth2
let http_transport = HttpTransportClientBuilder::new()
    .endpoint("https://api.example.com/mcp")?
    .auth(AuthMethod::OAuth2 {
        access_token: "access_token_here".to_string(),
        token_type: Some("Bearer".to_string()),
    })
    .timeout(Duration::from_secs(30))
    .build()
    .await?;

// HTTP with API key
let api_key_transport = HttpTransportClientBuilder::new()
    .endpoint("https://api.example.com/mcp")?
    .auth(AuthMethod::ApiKey {
        key: "api-key-here".to_string(),
        header: "X-API-Key".to_string(),
    })
    .build()
    .await?;
```

## Prompts and Resources

### Working with Prompts

```rust
use airs_mcp::protocol::types::{GetPromptRequest, RequestId, GetPromptParams};

// List available prompts
let prompts_response = client.list_prompts().await?;

for prompt in &prompts_response.prompts {
    println!("Prompt: {} - {}", prompt.name, 
             prompt.description.as_deref().unwrap_or("No description"));
}

// Get a specific prompt
if let Some(prompt) = prompts_response.prompts.first() {
    let prompt_request = GetPromptRequest {
        method: "prompts/get".to_string(),
        params: GetPromptParams {
            name: prompt.name.clone(),
            arguments: None,
        },
        id: RequestId::new_string("prompt-get-1".to_string()),
        jsonrpc: "2.0".to_string(),
    };
    
    let prompt_response = client.get_prompt(prompt_request).await?;
    println!("Prompt response: {:?}", prompt_response);
}
```

### Working with Resources

```rust
use airs_mcp::protocol::types::{ReadResourceRequest, ReadResourceParams};

// List resources
let resources_response = client.list_resources().await?;

for resource in &resources_response.resources {
    println!("Resource: {} ({})", resource.name, resource.uri);
    if let Some(description) = &resource.description {
        println!("  Description: {}", description);
    }
}

// Read a specific resource
if let Some(resource) = resources_response.resources.first() {
    let read_request = ReadResourceRequest {
        method: "resources/read".to_string(),
        params: ReadResourceParams {
            uri: resource.uri.clone(),
        },
        id: RequestId::new_string("resource-read-1".to_string()),
        jsonrpc: "2.0".to_string(),
    };
    
    let content_response = client.read_resource(read_request).await?;
    println!("Resource content: {:?}", content_response.contents);
}
```

## Testing and Debugging

### Simple Test Client

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_client_operations() -> McpResult<()> {
        let transport = StdioTransportClientBuilder::new()
            .command("echo-server")  // Your test server
            .build()
            .await?;
        
        let mut client = McpClientBuilder::new()
            .client_info("test-client", "1.0.0")
            .build(transport);
        
        let capabilities = client.initialize().await?;
        assert!(capabilities.tools.is_some() || capabilities.resources.is_some());
        
        let tools = client.list_tools().await?;
        // Add your assertions based on expected server behavior
        
        Ok(())
    }
}
```

### Debug Logging

```rust
use log::{info, debug, error};

// Enable logging in your client code
#[tokio::main]
async fn main() -> McpResult<()> {
    env_logger::init();
    
    info!("Starting MCP client");
    
    let transport = StdioTransportClientBuilder::new()
        .command("your-server")
        .build()
        .await?;
    
    debug!("Transport created successfully");
    
    let mut client = McpClientBuilder::new().build(transport);
    
    match client.initialize().await {
        Ok(caps) => info!("Client initialized: {:?}", caps),
        Err(e) => {
            error!("Initialization failed: {:?}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

```

## Error Handling Patterns

### Comprehensive Error Handling

```rust
use airs_mcp::integration::{McpClientBuilder, McpError};
use airs_mcp::transport::adapters::stdio::StdioTransportClientBuilder;
use std::time::Duration;
use serde_json::json;

async fn robust_client_example() -> Result<(), Box<dyn std::error::Error>> {
    let transport = StdioTransportClientBuilder::new()
        .timeout(Duration::from_secs(30))
        .build()
        .await?;
    let mut client = McpClientBuilder::new()
        .client_info("robust-client", "1.0.0")
        .build(transport);
    
    // Initialize first
    client.initialize().await?;
    
    // Call a tool with error handling
    match client.call_tool("risky_operation", Some(json!({"data": "test"}))).await {
        Ok(result) => {
            println!("Success: {:?}", result);
        }
        Err(McpError::Transport(transport_error)) => {
            eprintln!("Transport error: {}", transport_error);
        }
        Err(McpError::Protocol(protocol_error)) => {
            eprintln!("Protocol error: {}", protocol_error);
        }
        Err(McpError::Json(json_error)) => {
            eprintln!("JSON parsing error: {}", json_error);
        }
        Err(McpError::Timeout { timeout_ms }) => {
            eprintln!("Request timed out after {}ms", timeout_ms);
        }
        Err(McpError::UnexpectedResponse { details }) => {
            eprintln!("Unexpected response format: {}", details);
        }
        Err(McpError::Shutdown) => {
            eprintln!("Client has been shutdown");
        }
        Err(e) => {
            eprintln!("Unexpected error: {}", e);
        }
    }
    
    Ok(())
}
```


## Transport Configuration

### Custom Transport Setup

Creating your own transport adapter allows you to support different communication protocols while maintaining compatibility with the MCP client interface.

#### Implementing TransportClient Trait

```rust
use airs_mcp::protocol::{TransportClient, JsonRpcRequest, JsonRpcResponse, TransportError};
use async_trait::async_trait;
use std::time::Duration;
use tokio::time::timeout;

/// Example custom transport that communicates over TCP
pub struct TcpTransportClient {
    address: String,
    timeout: Duration,
    // Add your connection fields here
}

impl TcpTransportClient {
    pub fn new(address: String, timeout: Duration) -> Self {
        Self {
            address,
            timeout,
        }
    }
}

#[async_trait]
impl TransportClient for TcpTransportClient {
    type Error = TransportError;

    async fn call(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse, Self::Error> {
        // 1. Serialize the request
        let request_json = request.to_json()
            .map_err(|e| TransportError::Serialization { source: e })?;

        // 2. Send request over your transport (TCP, WebSocket, etc.)
        // Example: send over TCP connection
        let response_json = timeout(
            self.timeout,
            self.send_and_receive(&request_json)
        ).await
        .map_err(|_| TransportError::RequestTimeout { duration: self.timeout })??;

        // 3. Deserialize the response
        let response = JsonRpcResponse::from_json(&response_json)
            .map_err(|e| TransportError::Serialization { source: e })?;

        Ok(response)
    }

    fn is_ready(&self) -> bool {
        // Return true if your transport is ready to send requests
        true // Implement your readiness check
    }

    fn transport_type(&self) -> &'static str {
        "tcp"
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        // Clean up your transport resources
        Ok(())
    }
}

impl TcpTransportClient {
    async fn send_and_receive(&mut self, request: &str) -> Result<String, TransportError> {
        // Implement your actual transport logic here
        // This is where you'd:
        // 1. Connect to your server
        // 2. Send the JSON-RPC request
        // 3. Receive the JSON-RPC response
        // 4. Return the response string
        
        // Example placeholder:
        todo!("Implement your transport protocol here")
    }
}
```

#### Transport Builder Pattern

```rust
use std::time::Duration;

/// Builder for creating TCP transport clients
pub struct TcpTransportClientBuilder {
    address: Option<String>,
    timeout: Duration,
    // Add other configuration fields
}

impl TcpTransportClientBuilder {
    pub fn new() -> Self {
        Self {
            address: None,
            timeout: Duration::from_secs(30),
        }
    }

    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub async fn build(self) -> Result<TcpTransportClient, TransportError> {
        let address = self.address.ok_or_else(|| {
            TransportError::Connection {
                message: "Address is required".to_string(),
            }
        })?;

        Ok(TcpTransportClient::new(address, self.timeout))
    }
}
```

#### Using Your Custom Transport

```rust
use airs_mcp::integration::{McpClientBuilder, McpResult};

async fn use_custom_transport() -> McpResult<()> {
    // Create your custom transport
    let transport = TcpTransportClientBuilder::new()
        .address("tcp://localhost:8080")
        .timeout(Duration::from_secs(60))
        .build()
        .await?;

    // Use it with the MCP client
    let mut client = McpClientBuilder::new()
        .client_info("tcp-client", "1.0.0")
        .build(transport);

    // Now you can use the client normally
    client.initialize().await?;
    let tools = client.list_tools().await?;
    
    println!("Tools available: {:?}", tools);
    Ok(())
}
```

#### WebSocket Transport Example

```rust
// Example of a WebSocket transport implementation
pub struct WebSocketTransportClient {
    url: String,
    // Add WebSocket connection fields
}

#[async_trait]
impl TransportClient for WebSocketTransportClient {
    type Error = TransportError;

    async fn call(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse, Self::Error> {
        // Implementation would use a WebSocket library like tokio-tungstenite
        // to send JSON-RPC requests and receive responses
        todo!("Implement WebSocket transport")
    }

    fn is_ready(&self) -> bool {
        // Check if WebSocket connection is open
        true
    }

    fn transport_type(&self) -> &'static str {
        "websocket"
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        // Close WebSocket connection
        Ok(())
    }
}
```

### Connection Management

```rust
use tokio::time::{timeout, Duration};
use airs_mcp::integration::{McpClientBuilder, McpResult};
use airs_mcp::transport::adapters::stdio::StdioTransportClientBuilder;

// Graceful connection handling
async fn managed_connection() -> McpResult<()> {
    let transport = StdioTransportClientBuilder::new()
        .command("your-server")
        .build()
        .await?;
    
    let mut client = McpClientBuilder::new()
        .client_info("managed-client", "1.0.0")
        .build(transport);
    
    // Initialize the client
    client.initialize().await?;
    
    // Set reasonable timeout for operations
    let result = timeout(
        Duration::from_secs(10),
        client.list_tools()
    ).await??;
    
    println!("Available tools: {:?}", result.tools);
    
    // Always clean up
    client.close().await?;
    Ok(())
}
```
