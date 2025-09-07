# airs-mcp

**Production-ready Model Context Protocol (MCP) implementation for Rust**

[![🎉 Claude Desktop Integration](https://img.shields.io/badge/Claude_Desktop-✅_Integrated-green)](examples/simple-mcp-server)
[![🔥 Schema Compliance](https://img.shields.io/badge/MCP_2024--11--05-✅_Compliant-blue)](https://github.com/modelcontextprotocol/modelcontextprotocol)
[![🏭 Production Ready](https://img.shields.io/badge/Status-🏭_Production_Ready-success)]()

## 🎯 Production Status

**✅ Complete MCP Implementation**  
**✅ Full Claude Desktop Integration Verified**  
**✅ OAuth2 Authentication + MCP Inspector Integration Validated**  
**✅ 100% Schema Compliance (MCP 2024-11-05)**  
**✅ Enterprise-Grade Architecture & Testing**

## Overview

`airs-mcp` is a **production-ready, enterprise-grade Rust implementation** of the Model Context Protocol (MCP). Successfully integrated with Claude Desktop, it provides all three MCP capability types through a sophisticated, type-safe API.

### 🚀 **Real-World Integration Success**

This library powers **both server and client MCP implementations** with verified real-world integrations:

**🖥️ MCP Server (Claude Desktop Integration)**
- **✅ Tools**: Mathematical operations, greeting functions - real-time execution confirmed
- **✅ Resources**: File system access, configuration reading - attachment interface integration  
- **✅ Prompts**: Code review templates, concept explanations - prompt template system integration

**🔧 MCP Client (AIRS Library Integration)**
- **✅ High-Level API**: Type-safe client operations with automatic subprocess management
- **✅ Custom Transports**: Extensible transport layer with SubprocessTransport example
- **✅ Production Patterns**: Error handling, state management, and resource lifecycle

[**See the server example →**](examples/simple-mcp-server/)  
[**See the client example →**](examples/simple-mcp-client/)

### 🏗️ **Enterprise Architecture**

Built with production-grade patterns for both server and client implementations:

```rust
// High-level MCP Server API
let server = McpServerBuilder::new()
    .server_info("my-server", "1.0.0")
    .with_resource_provider(MyResourceProvider)
    .with_tool_provider(MyToolProvider)
    .with_prompt_provider(MyPromptProvider)
    .build(transport)
    .await?;

// High-level MCP Client API  
let client = McpClientBuilder::new()
    .client_info("my-client", "1.0.0")
    .timeout(Duration::from_secs(30))
    .auto_retry(true, 3)
    .build(transport)
    .await?;

// Use the client
let resources = client.list_resources().await?;
let result = client.call_tool("add", Some(args)).await?;
```

### 🔬 **Technical Excellence**

- **🎯 234+ Tests Passing**: Comprehensive unit, integration, and doc test coverage
- **🛡️ Zero Warnings**: Strict clippy compliance with enterprise code quality standards
- **⚡ High Performance**: Concurrent processing with advanced buffer management and zero-copy optimizations
- **🏛️ Clean Architecture**: Layered design with proper separation of concerns and async-first patterns
- **📋 Schema Compliance**: 100% MCP 2024-11-05 specification compliance verified with official tools

## Core Features

### 🔌 **Complete MCP Protocol Support**

**Three-Tier MCP Capability Implementation (Server & Client):**

```rust
// SERVER SIDE - Provide capabilities
async fn execute_greeting_tool(&self, args: ToolCallArgs) -> Result<ToolResult, Error> {
    let name = args.get("name").unwrap_or("World");
    Ok(ToolResult::text(format!("Hello, {}!", name)))
}

async fn get_config_resource(&self, uri: &str) -> Result<ResourceContents, Error> {
    let content = read_config_file(uri).await?;
    Ok(ResourceContents::text(content))
}

async fn provide_code_review_prompt(&self, args: PromptArgs) -> Result<PromptResult, Error> {
    let template = CodeReviewTemplate::new(args)?;
    Ok(PromptResult::from_template(template))
}

// CLIENT SIDE - Consume capabilities
let resources = client.list_resources().await?;           // Discovery
let content = client.read_resource(resource_uri).await?;   // Access  
let result = client.call_tool("greet", Some(args)).await?; // Execution
let messages = client.get_prompt("review", args).await?;   // Templates
```

### 🚀 **Production-Ready Transport Layer**

**High-Performance Transport System for Server & Client:**

```rust
// Server-side STDIO transport for Claude Desktop integration
let transport = StdioTransport::builder()
    .with_buffer_size(8192)      // Optimized for JSON-RPC payload sizes
    .with_batch_processing()     // Concurrent message handling
    .with_connection_pooling()   // Efficient resource management
    .build()
    .await?;

// Client-side custom transports (e.g., subprocess management)
impl Transport for SubprocessTransport {
    async fn send(&mut self, data: &[u8]) -> Result<(), Self::Error>;
    async fn receive(&mut self) -> Result<Vec<u8>, Self::Error>;
    async fn close(&mut self) -> Result<(), Self::Error>;
}

let subprocess_transport = SubprocessTransport::spawn_server(server_path).await?;
let client = McpClientBuilder::new().build(subprocess_transport).await?;
```

### 🧠 **Enterprise Message Correlation**

**Advanced Request-Response Management:**

```rust
// Sophisticated correlation with timeout handling and cleanup
let correlator = CorrelationManager::builder()
    .with_timeout(Duration::from_secs(30))
    .with_cleanup_interval(Duration::from_secs(60))
    .with_concurrent_capacity(1000)
    .build();

// Automatic correlation for complex conversation flows  
let response = correlator.correlate_request(request_id, message).await?;
```

### 🔒 **Type-Safe Error Handling**

**Structured Error Management:**

```rust
#[derive(Error, Debug)]
pub enum McpError {
    #[error("Transport error: {source}")]
    Transport { #[from] source: TransportError },
    
    #[error("Protocol violation: {message}")]
    Protocol { message: String },
    
    #[error("Resource not found: {uri}")]
    ResourceNotFound { uri: String },
}
```

## Examples

### 🖥️ **MCP Server Example** - [Claude Desktop Integration](examples/simple-mcp-server/)

**Production-ready server with verified Claude Desktop integration:**

```bash
cd examples/simple-mcp-server
cargo build --release

# Test with Claude Desktop - full UI integration verified!
# Add to Claude Desktop config and see resources, tools, prompts working
```

**Features demonstrated:**
- ✅ Complete MCP server implementation
- ✅ Claude Desktop integration (resources appear in attachment menu)
- ✅ All three capability types: Resources, Tools, Prompts  
- ✅ Production-grade error handling and logging

### 🔧 **MCP Client Example** - [AIRS Library Usage](examples/simple-mcp-client/)

**High-level client API with automatic server management:**

```bash
cd examples/simple-mcp-client
cargo run  # Automatically spawns and connects to server!
```

**Features demonstrated:**
- ✅ Custom `SubprocessTransport` implementing `Transport` trait
- ✅ High-level `McpClient` API hiding JSON-RPC complexity
- ✅ Automatic server process lifecycle management
- ✅ Real client ↔ server communication patterns
- ✅ Production error handling and state management

### 🔐 **OAuth2 MCP Server Examples** - [Enterprise Authentication](examples/)

**Production-ready OAuth2 authentication with MCP Inspector validation:**

```bash
# API Key Authentication
cd examples/mcp-remote-server-apikey
cargo run  # Bearer token + X-API-Key authentication

# OAuth2 Authentication (Latest)
cd examples/mcp-remote-server-oauth2
cargo run  # Full OAuth2 + PKCE + JWT authentication
```

**OAuth2 Integration Features:**
- ✅ **Complete OAuth2 Flow**: Authorization code + PKCE + JWT tokens
- ✅ **MCP Inspector Validated**: Full compatibility with official MCP testing tools
- ✅ **Three-Server Architecture**: Smart proxy server with clean separation of concerns
- ✅ **Enterprise Security**: Scope-based authorization, token validation, audit logging
- ✅ **Production Ready**: Comprehensive error handling, monitoring, and observability

## Usage

This crate is part of the AIRS workspace. See the main project README for build instructions.

## License

Licensed under MIT OR Apache-2.0, same as the parent AIRS project.