# Quick Start Guide

*Get up and running with AIRS MCP in 5 minutes*

## Installation

Add AIRS MCP to your `Cargo.toml`:

```toml
[dependencies]
airs-mcp = "0.1.0"
tokio = { version = "1.40", features = ["full"] }
serde_json = "1.0"
```

## Your First JSON-RPC Client

```rust
use airs_mcp::{JsonRpcClient, StdioTransport};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create transport and client
    let transport = StdioTransport::new().await?;
    let mut client = JsonRpcClient::new(transport).await?;
    
    // Make a method call
    let response = client.call("ping", Some(json!({"message": "hello"}))).await?;
    println!("Response: {:?}", response);
    
    // Send a notification
    client.notify("status", Some(json!({"status": "ready"}))).await?;
    
    // Clean shutdown
    client.shutdown().await?;
    Ok(())
}
```

## Your First MCP Server

```rust
use airs_mcp::{McpServer, ServerCapabilities};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create MCP server with capabilities
    let capabilities = ServerCapabilities::default()
        .with_tools()
        .with_resources();
    
    let server = McpServer::builder()
        .capabilities(capabilities)
        .build()
        .await?;
    
    // Run server (connects to Claude Desktop via STDIO)
    server.run().await?;
    Ok(())
}
```

## Next Steps

- [Basic Examples](./basic_examples.md) - Learn common patterns
- [Claude Integration](./claude_integration.md) - Connect to Claude Desktop
- [Advanced Patterns](./advanced_patterns.md) - High-performance usage
