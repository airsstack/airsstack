# Simple MCP Client Example using AIRS MCP Library

A comprehensive example demonstrating **actual client ↔ server communication** using the AIRS MCP client library with a custom subprocess transport. This example shows how to use the high-level AIRS MCP client API to interact with MCP servers without dealing with JSON-RPC complexity.

## Project Structure

This example is part of the AIRS MCP examples collection:

```
airs/crates/airs-mcp/examples/
├── simple-mcp-server/          # MCP server example (test target)
│   ├── src/main.rs            # Server implementation
│   ├── Cargo.toml             # Server dependencies
│   └── README.md              # Server documentation
└── simple-mcp-client/         # This client example
    ├── src/main.rs            # Client implementation with SubprocessTransport
    ├── Cargo.toml             # Client dependencies (airs-mcp library)
    └── README.md              # This documentation
```

**Key relationship**: The client automatically spawns and manages the server as a subprocess, demonstrating real MCP protocol interactions through the AIRS library.

## Features

- ✅ **AIRS MCP Library Integration**: Uses the high-level `McpClient` API from the AIRS library
- ✅ **Custom Transport Implementation**: Demonstrates how to create a `SubprocessTransport` 
- ✅ **Real Server Interaction**: Spawns and communicates with actual MCP server processes
- ✅ **Complete MCP Operations**: Resources, tools, prompts, and state management
- ✅ **Production Ready**: Uses the same patterns you'd use in real applications
- ✅ **Type Safety**: Full Rust type safety throughout the protocol interactions
- ✅ **Error Handling**: Comprehensive error handling with graceful degradation

## Quick Start

### Testing with the Simple MCP Server

This example demonstrates how to use the AIRS MCP client library by connecting to a real server:

```bash
# Build the server first (client will automatically spawn it)
cd ../simple-mcp-server && cargo build

# Run the client (automatically spawns and manages server)
cd ../simple-mcp-client
cargo run
```

You'll see output showing the high-level API in action:
```
🚀 Starting MCP Client Example using AIRS MCP Library
📍 Server path: ../simple-mcp-server/target/debug/simple-mcp-server
🚀 Spawning MCP server: ../simple-mcp-server/target/debug/simple-mcp-server
✅ Server process spawned successfully (PID: 81115)
🔗 Creating MCP client with subprocess transport...
✅ MCP client created successfully using AIRS library
```

The client **automatically spawns and manages the server process** - you don't need to run the server manually!

### Command Line Usage

Specify a different server:

```bash
cargo run -- --server-path /path/to/your/mcp-server
```

## What You'll See

The client demonstrates the complete MCP protocol flow with **automatic server management**:

1. **🚀 Automatic Server Spawning**: Client spawns and manages the server subprocess automatically
2. **🤝 Initialization**: Complete MCP handshake with capability negotiation  
3. **📂 Resources**: Discovery and reading of server resources
4. **🔧 Tools**: Tool discovery and execution with real parameters
5. **💡 Prompts**: Prompt templates with argument substitution
6. **🛑 Automatic Cleanup**: Graceful server shutdown when client exits

The AIRS MCP library handles all the subprocess management - you just run the client!

## Example Output

When you run the client, you'll see the AIRS library API in action:

**Resource Reading with High-Level API:**
```
📖 Reading resource using AIRS client: file:///tmp/example.txt
   ✅ Resource content received:
      📄 Content: Hello from the MCP server!
      This is example content.
```

**Tool Execution through AIRS Client:**
```
⚙️  Calling tool using AIRS client: add
   ✅ Tool execution successful:
      🎯 Result: {
        "operation": "addition",
        "result": 42.0
      }

⚙️  Calling second tool using AIRS client: greet  
   ✅ Second tool execution successful:
      🎯 Result: {
        "greeting": "Hello, Rust Developer! Welcome to the MCP server!"
      }
```

**Client State Management:**
```
🔍 Step 4: Checking Client State
   📊 Connection state: Initialized
   🔗 Is initialized: true
   ✅ Server capabilities available:
      📂 Resources: true
      🔧 Tools: true
      💡 Prompts: true
      📝 Logging: false
```

## Key Learning Points

- **High-Level API**: See how AIRS MCP library simplifies MCP client development
- **Transport Abstraction**: Learn how to implement custom transports for different communication methods
- **Type Safety**: Understand how Rust's type system ensures protocol correctness
- **Error Handling**: Observe comprehensive error handling with meaningful error types
- **State Management**: See automatic connection state tracking and capability management
- **Resource Operations**: Learn the simple API for resource discovery and content reading
- **Tool Integration**: Understand easy tool discovery and execution patterns
- **Prompt Handling**: See how prompts work with dynamic argument substitution
- **Process Management**: Learn proper subprocess lifecycle management

## Architecture Highlights

The example demonstrates key AIRS MCP library concepts:

```rust
// Custom transport implementing the Transport trait
impl Transport for SubprocessTransport {
    fn send(&mut self, data: &[u8]) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn receive(&mut self) -> impl Future<Output = Result<Vec<u8>, Self::Error>> + Send;
    fn close(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

// High-level client builder pattern
let client = McpClientBuilder::new()
    .client_info("simple-mcp-client", "0.1.0")
    .timeout(Duration::from_secs(30))
    .auto_retry(true, 3)
    .build(transport)
    .await?;

// Clean, async API calls
let resources = client.list_resources().await?;
let content = client.read_resource(uri).await?;
let result = client.call_tool("add", Some(args)).await?;
```

## Integration with Your Application

This example shows practical patterns for MCP client integration:

### Custom Transport Implementation
```rust
// Implement Transport trait for your communication method
struct YourTransport { /* your fields */ }

impl Transport for YourTransport {
    type Error = YourError;
    // Implement required async methods...
}
```

### Client Configuration
```rust
// Configure client with your requirements
let client = McpClientBuilder::new()
    .client_info("your-app", "1.0.0")
    .timeout(Duration::from_secs(60))
    .auto_retry(true, 5)
    .build(your_transport)
    .await?;
```

### Error Handling Patterns
```rust
// Comprehensive error handling
match client.call_tool("tool_name", args).await {
    Ok(result) => handle_success(result),
    Err(McpError::InvalidRequest(msg)) => handle_validation_error(msg),
    Err(McpError::ServerError(code, msg)) => handle_server_error(code, msg),
    Err(McpError::Transport(err)) => handle_transport_error(err),
}
```

### Resource Discovery Pattern
```rust
// Efficient resource discovery and access
let resources = client.list_resources().await?;
for resource in resources.resources {
    if resource.uri.ends_with(".txt") {
        let content = client.read_resource(&resource.uri).await?;
        process_text_content(content);
    }
}
```

## File Overview

### `src/main.rs`
The main client implementation showcasing:
- **SubprocessTransport**: Custom transport that spawns and manages MCP server subprocesses
- **McpClient Integration**: High-level API usage for all MCP operations
- **Error Handling**: Comprehensive error handling with graceful degradation
- **Real Operations**: Actual resource reading, tool calling, and prompt testing

### Key Components
- **Transport Layer**: `SubprocessTransport` implements the `Transport` trait
- **Client Layer**: Uses `McpClientBuilder` for configuration and `McpClient` for operations
- **Process Management**: Automatic server spawning, communication, and cleanup
- **Protocol Handling**: All JSON-RPC complexity hidden behind type-safe APIs

## Next Steps

After running this example, you can:

- **Extend the Transport**: Add network transport for remote MCP servers
- **Custom Client Logic**: Build domain-specific MCP client applications
- **Advanced Features**: Explore streaming, notifications, and progress tracking
- **Production Usage**: Apply error handling and monitoring patterns
- **Multi-Server**: Connect to multiple MCP servers simultaneously

This example provides the foundation for building production MCP integrations using the AIRS library!
