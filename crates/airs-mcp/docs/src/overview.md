# AIRS MCP: Model Context Protocol Implementation

## Project Description

AIRS MCP is a Rust implementation of the Model Context Protocol (MCP) that provides both server and client libraries for integrating AI applications with external systems. The implementation includes type-safe APIs, multiple transport options, and comprehensive protocol compliance.

## Current Status

The implementation is complete and operational with:

- **Protocol Compliance**: Full JSON-RPC 2.0 foundation with MCP extensions
- **Transport Support**: STDIO and HTTP transport implementations
- **Authentication**: OAuth2 integration with PKCE support  
- **Testing**: Comprehensive test suite with integration examples
- **Documentation**: Complete API documentation and usage guides

## Core Features

### Protocol Implementation

- JSON-RPC 2.0 message types with MCP extensions
- Bidirectional communication support (client ↔ server requests)
- Three-phase connection lifecycle management
- Capability negotiation between clients and servers
- Transport abstraction layer

### Server Features

- **Resources**: URI-based resource access with subscription support
- **Tools**: Function calling with JSON Schema validation
- **Prompts**: Template-based prompt management
- **Logging**: Comprehensive logging and audit capabilities

### Client Features  

- **Connection Management**: Automatic server connection handling
- **Request Handling**: Type-safe API for all MCP operations
- **Session State**: Proper lifecycle and capability tracking
- **Error Handling**: Comprehensive error types and recovery

### Transport Layer

- **STDIO Transport**: Process-based communication for local servers
- **HTTP Transport**: RESTful communication with authentication support
- **Custom Transports**: Extensible transport interface

### Authentication & Security

- OAuth2 authentication with PKCE support
- Bearer token authentication for HTTP transport
- Request validation and error handling
- Audit logging capabilities

## Architecture Overview

The implementation is organized in layers:

```
Integration Layer (High-level APIs)
    ↓
Protocol Layer (MCP message types and validation)
    ↓  
Transport Layer (Communication abstractions)
    ↓
JSON-RPC 2.0 Foundation
```

### Key Components

- **McpClient**: High-level client API for connecting to MCP servers
- **McpServer**: Server implementation with provider interfaces
- **TransportClient**: Clean request-response interface for communication
- **Message Types**: Complete MCP protocol message definitions
- **Provider Traits**: Extensible interfaces for server capabilities

## Getting Started

Basic client usage:

```rust
use airs_mcp::integration::McpClientBuilder;
use airs_mcp::transport::adapters::stdio::StdioTransportClientBuilder;

// Create transport
let transport = StdioTransportClientBuilder::new()
    .command("your-mcp-server")
    .build()
    .await?;

// Create client
let mut client = McpClientBuilder::new().build(transport);

// Initialize connection
let capabilities = client.initialize().await?;

// Use MCP operations
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
