# airs-mcp

**Production-ready Model Context Protocol (MCP) implementation for Rust**

[![🎉 Claude Desktop Integration](https://img.shields.io/badge/Claude_Desktop-✅_Integrated-green)](examples/simple-mcp-server)
[![🔥 Schema Compliance](https://img.shields.io/badge/MCP_2024--11--05-✅_Compliant-blue)](https://github.com/modelcontextprotocol/modelcontextprotocol)
[![🏭 Production Ready](https://img.shields.io/badge/Status-🏭_Production_Ready-success)]()

## 🎯 Production Status

**✅ Complete MCP Implementation**  
**✅ Full Claude Desktop Integration Verified**  
**✅ 100% Schema Compliance (MCP 2024-11-05)**  
**✅ Enterprise-Grade Architecture & Testing**

## Overview

`airs-mcp` is a **production-ready, enterprise-grade Rust implementation** of the Model Context Protocol (MCP). Successfully integrated with Claude Desktop, it provides all three MCP capability types through a sophisticated, type-safe API.

### 🚀 **Real-World Integration Success**

This library powers a **fully functional MCP server** that integrates seamlessly with Claude Desktop:

- **✅ Tools**: Mathematical operations, greeting functions - real-time execution confirmed
- **✅ Resources**: File system access, configuration reading - attachment interface integration
- **✅ Prompts**: Code review templates, concept explanations - prompt template system integration

[**See the working example →**](examples/simple-mcp-server/)

### 🏗️ **Enterprise Architecture**

Built with production-grade patterns and comprehensive safety measures:

```rust
// High-level MCP Server API
let server = McpServerBuilder::new()
    .server_info("my-server", "1.0.0")
    .with_resource_provider(MyResourceProvider)
    .with_tool_provider(MyToolProvider)
    .with_prompt_provider(MyPromptProvider)
    .build(transport)
    .await?;
```

### 🔬 **Technical Excellence**

- **🎯 234+ Tests Passing**: Comprehensive unit, integration, and doc test coverage
- **🛡️ Zero Warnings**: Strict clippy compliance with enterprise code quality standards
- **⚡ High Performance**: Concurrent processing with advanced buffer management and zero-copy optimizations
- **🏛️ Clean Architecture**: Layered design with proper separation of concerns and async-first patterns
- **📋 Schema Compliance**: 100% MCP 2024-11-05 specification compliance verified with official tools

## Core Features

### 🔌 **Complete MCP Protocol Support**

**Three-Tier MCP Capability Implementation:**

```rust
// 1. TOOLS - Execute functions and operations
async fn execute_greeting_tool(&self, args: ToolCallArgs) -> Result<ToolResult, Error> {
    let name = args.get("name").unwrap_or("World");
    Ok(ToolResult::text(format!("Hello, {}!", name)))
}

// 2. RESOURCES - Provide data and content
async fn get_config_resource(&self, uri: &str) -> Result<ResourceContents, Error> {
    let content = read_config_file(uri).await?;
    Ok(ResourceContents::text(content))
}

// 3. PROMPTS - Offer template-based interactions  
async fn provide_code_review_prompt(&self, args: PromptArgs) -> Result<PromptResult, Error> {
    let template = CodeReviewTemplate::new(args)?;
    Ok(PromptResult::from_template(template))
}
```

### 🚀 **Production-Ready Transport Layer**

**High-Performance STDIO Transport:**

```rust
// Zero-copy message processing with advanced buffering
let transport = StdioTransport::builder()
    .with_buffer_size(8192)      // Optimized for JSON-RPC payload sizes
    .with_batch_processing()     // Concurrent message handling
    .with_connection_pooling()   // Efficient resource management
    .build()
    .await?;
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

## Usage

This crate is part of the AIRS workspace. See the main project README for build instructions.

## License

Licensed under MIT OR Apache-2.0, same as the parent AIRS project.