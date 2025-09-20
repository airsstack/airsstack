# Core Component Design

This document describes the core components of the current AIRS MCP implementation.

## Architecture Overview

The AIRS MCP implementation uses a layered architecture focused on type safety, performance, and clean abstractions:

```
Integration Layer (McpClient/McpServer)
    ↓
Protocol Layer (JSON-RPC 2.0 + MCP messages)
    ↓
Transport Layer (HTTP/STDIO TransportClient)
    ↓
Network/Process Communication
```

## JSON-RPC 2.0 Foundation

The protocol layer provides complete JSON-RPC 2.0 compliance with MCP extensions:

```rust
// Core JSON-RPC 2.0 types
// Located in: src/protocol/jsonrpc/

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<RequestId>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<RequestId>,
    #[serde(flatten)]
    pub payload: ResponsePayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    Number(i64),
    String(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponsePayload {
    Success { result: Value },
    Error { error: JsonRpcError },
}
```

## Transport Client Architecture

The current implementation uses a clean request-response pattern through the `TransportClient` trait:

```rust
// Transport abstraction
// Located in: src/transport/

#[async_trait]
pub trait TransportClient: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn call(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse, Self::Error>;
    async fn close(&self) -> Result<(), Self::Error>;
}
```

### STDIO Transport Client

```rust
// STDIO transport implementation
// Located in: src/transport/adapters/stdio/

pub struct StdioTransportClient {
    // Internal implementation details...
}

impl StdioTransportClient {
    pub async fn new(config: StdioTransportConfig) -> Result<Self, StdioTransportError> {
        // Create STDIO transport with configured process
        // Handles process spawning, stdin/stdout management
    }
}

#[async_trait]
impl TransportClient for StdioTransportClient {
    type Error = StdioTransportError;

    async fn call(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse, Self::Error> {
        // Send request via stdin, receive response via stdout
        // Handles JSON serialization/deserialization
        // Provides timeout and error handling
    }
}
```

### HTTP Transport Client

```rust
// HTTP transport implementation
// Located in: src/transport/adapters/http/

pub struct HttpTransportClient {
    // Internal implementation with HTTP client
}

impl HttpTransportClient {
    pub async fn new(config: HttpTransportConfig) -> Result<Self, HttpTransportError> {
        // Create HTTP client with authentication
        // Supports Bearer tokens, API keys, OAuth2
    }
}

#[async_trait]
impl TransportClient for HttpTransportClient {
    type Error = HttpTransportError;

    async fn call(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse, Self::Error> {
        // Send HTTP POST with JSON-RPC request
        // Handle authentication headers
        // Parse JSON-RPC response from HTTP response
    }
}
```

## MCP Client Architecture

The `McpClient` provides high-level MCP operations built on the transport layer:

```rust
// High-level MCP client
// Located in: src/integration/client.rs

pub struct McpClient<T: TransportClient> {
    transport: T,
    session_state: McpSessionState,
    config: McpClientConfig,
}

impl<T: TransportClient> McpClient<T> {
    pub async fn initialize(&mut self) -> McpResult<InitializeResult> {
        // Send MCP initialize request
        // Negotiate protocol version and capabilities
        // Update session state
    }

    pub async fn list_tools(&self) -> McpResult<ListToolsResult> {
        // Send tools/list request
        // Parse and return available tools
    }

    pub async fn call_tool(&self, request: Value) -> McpResult<CallToolResult> {
        // Send tools/call request with tool arguments
        // Handle tool execution response
    }

    pub async fn list_resources(&self) -> McpResult<ListResourcesResult> {
        // Send resources/list request
        // Return available resources
    }

    pub async fn read_resource(&self, uri: String) -> McpResult<ReadResourceResult> {
        // Send resources/read request
        // Return resource content
    }
}
```

## MCP Server Architecture

The `McpServer` handles incoming MCP requests and delegates to providers:

```rust
// High-level MCP server
// Located in: src/integration/server.rs

pub struct McpServer<T> {
    providers: ProviderRegistry,
    server_info: ServerInfo,
    capabilities: ServerCapabilities,
    _phantom: PhantomData<T>,
}

impl<T> McpServer<T> {
    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "tools/list" => self.handle_list_tools().await,
            "tools/call" => self.handle_call_tool(request.params).await,
            "resources/list" => self.handle_list_resources().await,
            "resources/read" => self.handle_read_resource(request.params).await,
            "prompts/list" => self.handle_list_prompts().await,
            "prompts/get" => self.handle_get_prompt(request.params).await,
            _ => Err(McpError::MethodNotFound(request.method.clone())),
        };

        // Convert result to JsonRpcResponse
        self.create_response(request.id, result)
    }
}
```

## Provider System

The provider system allows extending server capabilities:

```rust
// Provider traits
// Located in: src/providers/

#[async_trait]
pub trait ResourceProvider: Send + Sync {
    async fn list_resources(&self) -> Result<Vec<Resource>, ProviderError>;
    async fn read_resource(&self, uri: &str) -> Result<ResourceContent, ProviderError>;
}

#[async_trait]
pub trait ToolProvider: Send + Sync {
    async fn list_tools(&self) -> Result<Vec<Tool>, ProviderError>;
    async fn call_tool(&self, name: &str, arguments: Value) -> Result<ToolResult, ProviderError>;
}

#[async_trait]
pub trait PromptProvider: Send + Sync {
    async fn list_prompts(&self) -> Result<Vec<Prompt>, ProviderError>;
    async fn get_prompt(&self, name: &str, arguments: Option<Value>) -> Result<GetPromptResult, ProviderError>;
}
```

## Builder Pattern

The implementation uses builder patterns for clean configuration:

```rust
// Client builder
let transport = StdioTransportClientBuilder::new()
    .command("mcp-server")
    .timeout(Duration::from_secs(30))
    .build()
    .await?;

let mut client = McpClientBuilder::new()
    .client_info("my-client", "1.0.0")
    .timeout(Duration::from_secs(60))
    .build(transport);

// Server builder  
let transport = HttpTransportClientBuilder::new()
    .endpoint("http://localhost:3000/mcp")?
    .auth(AuthMethod::Bearer { token: "token".to_string() })
    .build()
    .await?;
```

## Architecture Benefits

### Type Safety
- **Compile-time validation**: MCP message types validated at compile time
- **Error handling**: Comprehensive error types for different failure modes
- **Protocol compliance**: Type system enforces MCP specification requirements

### Performance
- **Buffer Management**: Uses `bytes` crate for efficient buffer management
- **Async-native**: Built on tokio for efficient concurrent operations
- **Minimal allocations**: Careful memory management in hot paths

### Modularity
- **Transport abstraction**: Clean separation between protocol and transport
- **Provider system**: Extensible server capabilities through traits
- **Builder pattern**: Ergonomic configuration with sensible defaults

The current architecture eliminates the complexity of correlation management by using a simple request-response pattern through the `TransportClient` trait, providing better performance and maintainability.
