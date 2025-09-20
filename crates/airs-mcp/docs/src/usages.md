# Usage Guide

This section provides practical guidance for using the AIRS MCP library in your applications.

## Getting Started

Start with the fundamentals:

- [Quick Start Guide](./usages/quick_start.md) - Basic setup and first steps
- [Basic Examples](./usages/basic_examples.md) - Common usage patterns
- [Patterns](./usages/advanced_patterns.md) - Complex integration scenarios

## Integration Guides

Specific integration scenarios:

- [Claude Desktop Integration](./usages/claude_integration.md) - Setting up with Claude Desktop
- [Custom Transports](./usages/custom_transports.md) - Implementing custom transport layers

## Usage Overview

AIRS MCP provides:

### Client Capabilities
- Connect to MCP servers via STDIO or HTTP
- Perform tool calls, resource access, and prompt operations
- Handle authentication and session management
- Automatic retry and error handling

### Server Features  
- Implement MCP servers with provider interfaces
- Support for resources, tools, and prompts
- Built-in authentication and validation
- Transport abstraction for different connection types

### Transport Options
- **STDIO Transport**: For local process communication
- **HTTP Transport**: For remote service communication with authentication
- **Custom Transports**: Extensible interface for new transport types

## When to Use AIRS MCP

AIRS MCP is suitable when you need:

- Full MCP protocol compliance
- Type-safe Rust implementation
- Multiple transport options
- Authentication support
- Production-ready reliability

## Code Examples

Basic client usage:

```rust
use airs_mcp::integration::McpClientBuilder;
use airs_mcp::transport::adapters::stdio::StdioTransportClientBuilder;

// Create transport and client
let transport = StdioTransportClientBuilder::new()
    .command("your-mcp-server")
    .build()
    .await?;

let mut client = McpClientBuilder::new().build(transport);

// Initialize and use
let capabilities = client.initialize().await?;
let tools = client.list_tools().await?;
```

Basic server setup:

```rust
use airs_mcp::integration::server::McpServer;
use airs_mcp::transport::adapters::stdio::StdioTransport;

// Create server with providers
let server = McpServer::new()
    .with_resource_provider(resource_provider)
    .with_tool_provider(tool_provider);

// Start with transport
let transport = StdioTransportClientBuilder::new();
server.serve(transport).await?;
```
