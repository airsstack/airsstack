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

## Your First MCP Client

```rust
use airs_mcp::integration::mcp::{McpClientBuilder, McpError};
use airs_mcp::transport::stdio::StdioTransport;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create transport and MCP client
    let transport = StdioTransport::spawn_server("./path/to/mcp-server").await?;
    let mut client = McpClientBuilder::new()
        .client_info("my-client", "1.0.0")
        .build(transport)
        .await?;
    
    // Initialize connection
    client.initialize().await?;
    
    // List available tools
    let tools = client.list_tools().await?;
    println!("Available tools: {:?}", tools);
    
    // Call a tool
    if let Some(tool) = tools.first() {
        let result = client.call_tool(&tool.name, json!({"input": "test"})).await?;
        println!("Tool result: {:?}", result);
    }
    
    // Clean shutdown
    client.shutdown().await?;
    Ok(())
}
```

## Your First MCP Server

```rust
use airs_mcp::integration::mcp::{McpServerBuilder, ToolProvider, McpError, McpResult};
use airs_mcp::shared::protocol::{Tool, Content};
use airs_mcp::transport::stdio::StdioTransport;
use serde_json::Value;
use async_trait::async_trait;

// Implement a simple tool provider
struct EchoTools;

#[async_trait]
impl ToolProvider for EchoTools {
    async fn list_tools(&self) -> McpResult<Vec<Tool>> {
        Ok(vec![Tool {
            name: "echo".to_string(),
            description: Some("Echo back the input".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {"type": "string"}
                }
            }),
        }])
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> McpResult<Vec<Content>> {
        match name {
            "echo" => {
                let message = arguments.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No message provided");
                
                Ok(vec![Content::text(format!("Echo: {}", message))])
            }
            _ => Err(McpError::method_not_found(format!("Unknown tool: {}", name)))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create MCP server with tool provider
    let mut server = McpServerBuilder::new()
        .server_info("echo-server", "1.0.0")
        .tool_provider(EchoTools)
        .build();
    
    // Run server (connects to Claude Desktop via STDIO)
    server.run_stdio().await?;
    Ok(())
}
```

## Next Steps

- [Basic Examples](./basic_examples.md) - Learn common patterns
- [Claude Integration](./claude_integration.md) - Connect to Claude Desktop
- [Advanced Patterns](./advanced_patterns.md) - High-performance usage
