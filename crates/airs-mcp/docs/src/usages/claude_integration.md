# Claude Desktop Integration

*Complete guide for integrating AIRS MCP servers with Claude Desktop*

## Overview

AIRS MCP provides **production-ready Claude Desktop integration** with comprehensive automation, safety measures, and enterprise-grade deployment tooling. This integration has been **successfully tested and verified** with Claude Desktop's sophisticated UI.

## Quick Integration

### Automated Setup (Recommended)

The fastest way to integrate with Claude Desktop:

```bash
# Clone the example server
git clone https://github.com/yourusername/airs
cd airs/crates/airs-mcp/examples/simple-mcp-server

# Complete automated integration
./scripts/integrate.sh
```

This script provides:
- ✅ **Complete Automation**: End-to-end integration with a single command
- ✅ **Safety First**: Automatic configuration backups with timestamp recovery  
- ✅ **Schema Compliance**: 100% MCP 2024-11-05 specification compliance validated
- ✅ **Multi-Modal Testing**: MCP Inspector browser testing + Claude Desktop verification

### Manual Configuration

If you prefer manual control:

```bash
# 1. Build release binary
./scripts/build.sh

# 2. Test with MCP Inspector
./scripts/test_inspector.sh

# 3. Configure Claude Desktop
./scripts/configure_claude.sh

# 4. Restart Claude Desktop and verify
./scripts/debug_integration.sh
```

## Claude Desktop Configuration

### Configuration File Location

Claude Desktop reads MCP servers from:
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`  
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

### Example Configuration

```json
{
  "mcpServers": {
    "simple-mcp-server": {
      "command": "/path/to/your/simple-mcp-server",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Advanced Configuration

```json
{
  "mcpServers": {
    "airs-mcp-server": {
      "command": "/usr/local/bin/airs-mcp-server",
      "args": ["--config", "/etc/airs-mcp/config.json"],
      "env": {
        "RUST_LOG": "debug",
        "MCP_SERVER_NAME": "AIRS Production Server",
        "MAX_CONNECTIONS": "10",
        "REQUEST_TIMEOUT": "30"
      }
    }
  }
}
```

## MCP Capabilities Integration

### Tools Integration

Tools appear in Claude Desktop's **MCP Tools interface** and can be executed in real-time:

```rust
use airs_mcp::integration::mcp::{ToolProvider, McpError};
use airs_mcp::shared::protocol::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};

#[derive(Debug)]
struct CalculatorTool;

#[async_trait]
impl ToolProvider for CalculatorTool {
    async fn list_tools(&self) -> Result<Vec<Tool>, McpError> {
        Ok(vec![
            Tool {
                name: "add".to_string(),
                description: "Add two numbers".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number", "description": "First number"},
                        "b": {"type": "number", "description": "Second number"}
                    },
                    "required": ["a", "b"]
                }),
            }
        ])
    }

    async fn call_tool(&self, name: &str, args: Value) -> Result<Vec<Content>, McpError> {
        match name {
            "add" => {
                let a = args["a"].as_f64().unwrap_or(0.0);
                let b = args["b"].as_f64().unwrap_or(0.0);
                let result = a + b;
                
                Ok(vec![Content::Text {
                    text: format!("Result: {} + {} = {}", a, b, result),
                }])
            }
            _ => Err(McpError::InvalidRequest {
                message: format!("Unknown tool: {}", name),
            })
        }
    }
}
```

### Resources Integration

Resources are accessible through Claude Desktop's **attachment menu**:

```rust
use airs_mcp::integration::mcp::{ResourceProvider, McpError};
use airs_mcp::shared::protocol::{Resource, Content, Uri, MimeType};
use async_trait::async_trait;

#[derive(Debug)]
struct FileSystemProvider {
    base_path: String,
}

#[async_trait]
impl ResourceProvider for FileSystemProvider {
    async fn list_resources(&self) -> Result<Vec<Resource>, McpError> {
        Ok(vec![
            Resource {
                uri: Uri::new("file:///docs/readme.txt")?,
                name: "Project README".to_string(),
                description: Some("Main project documentation".to_string()),
                mime_type: Some(MimeType::new("text/plain")?),
            },
            Resource {
                uri: Uri::new("file:///config/app.json")?,
                name: "App Configuration".to_string(),
                description: Some("Application configuration file".to_string()),
                mime_type: Some(MimeType::new("application/json")?),
            },
        ])
    }

    async fn read_resource(&self, uri: &str) -> Result<Vec<Content>, McpError> {
        match uri {
            "file:///docs/readme.txt" => {
                Ok(vec![Content::Text {
                    text: std::fs::read_to_string(&format!("{}/docs/readme.txt", self.base_path))
                        .map_err(|e| McpError::InvalidRequest {
                            message: format!("Failed to read file: {}", e),
                        })?,
                }])
            }
            "file:///config/app.json" => {
                let content = std::fs::read_to_string(&format!("{}/config/app.json", self.base_path))
                    .map_err(|e| McpError::InvalidRequest {
                        message: format!("Failed to read file: {}", e),
                    })?;
                
                Ok(vec![Content::Text { text: content }])
            }
            _ => Err(McpError::InvalidRequest {
                message: format!("Resource not found: {}", uri),
            })
        }
    }
}
```

### Prompts Integration

Prompts appear in Claude Desktop's **prompt template interface**:

```rust
use airs_mcp::integration::mcp::{PromptProvider, McpError};
use airs_mcp::shared::protocol::{Prompt, PromptArgument, PromptMessage};
use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug)]
struct CodeReviewPrompts;

#[async_trait]
impl PromptProvider for CodeReviewPrompts {
    async fn list_prompts(&self) -> Result<Vec<Prompt>, McpError> {
        Ok(vec![
            Prompt {
                name: "code-review".to_string(),
                description: "Perform comprehensive code review".to_string(),
                arguments: vec![
                    PromptArgument {
                        name: "language".to_string(),
                        description: "Programming language".to_string(),
                        required: true,
                    },
                    PromptArgument {
                        name: "code".to_string(),
                        description: "Code to review".to_string(),
                        required: true,
                    }
                ],
            }
        ])
    }

    async fn get_prompt(&self, name: &str, args: Value) -> Result<PromptMessage, McpError> {
        match name {
            "code-review" => {
                let language = args["language"].as_str().unwrap_or("unknown");
                let code = args["code"].as_str().unwrap_or("");
                
                Ok(PromptMessage {
                    role: "user".to_string(),
                    content: format!(
                        "Please review this {} code for:\n\
                         • Code quality and best practices\n\
                         • Potential bugs or issues\n\
                         • Performance optimizations\n\
                         • Security considerations\n\n\
                         ```{}\n{}\n```",
                        language, language, code
                    ),
                })
            }
            _ => Err(McpError::InvalidRequest {
                message: format!("Unknown prompt: {}", name),
            })
        }
    }
}
```

## Server Implementation

### Complete MCP Server

```rust
use airs_mcp::integration::mcp::McpServerBuilder;
use airs_mcp::shared::protocol::ServerCapabilities;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging (file-based to avoid STDIO contamination)
    init_logging()?;
    
    info!("🚀 Starting AIRS MCP Server for Claude Desktop");

    // Create server capabilities
    let capabilities = ServerCapabilities::default()
        .with_tools()
        .with_resources()
        .with_prompts()
        .with_logging();

    // Build server with providers
    let server = McpServerBuilder::new()
        .capabilities(capabilities)
        .tool_provider(Box::new(CalculatorTool))
        .resource_provider(Box::new(FileSystemProvider {
            base_path: "/tmp".to_string(),
        }))
        .prompt_provider(Box::new(CodeReviewPrompts))
        .build()?;

    info!("✅ Server built successfully, starting JSON-RPC communication");

    // Run server (connects to Claude Desktop via STDIO)
    server.run().await?;

    info!("🔄 Server shutdown complete");
    Ok(())
}

fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::{fmt, EnvFilter};
    use tracing_appender::rolling::{RollingFileAppender, Rotation};
    
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        "/tmp/airs-mcp-server",
        "server.log",
    );

    fmt()
        .with_writer(file_appender)
        .with_env_filter(EnvFilter::new("info"))
        .init();

    Ok(())
}
```

## Testing and Validation

### MCP Inspector Testing

Test your server with the browser-based MCP Inspector:

```bash
# Run MCP Inspector tests
./scripts/test_inspector.sh

# Manual testing
npx @modelcontextprotocol/inspector /path/to/your/server
```

### Claude Desktop Verification

1. **Tools Testing**: Look for your tools in the MCP icon in Claude's chat interface
2. **Resources Testing**: Check the attachment menu for "Add from [your-server-name]"
3. **Prompts Testing**: Find your prompts in the prompt template interface

### Real-time Monitoring

```bash
# Monitor server logs
tail -f /tmp/airs-mcp-server/server.log

# Debug integration status
./scripts/debug_integration.sh

# Watch Claude Desktop logs (macOS)
tail -f ~/Library/Logs/Claude/claude-desktop.log
```

## Production Deployment

### Security Considerations

```rust
// Secure file system access
impl FileSystemProvider {
    fn validate_path(&self, path: &str) -> Result<(), McpError> {
        let canonical = std::fs::canonicalize(path)
            .map_err(|_| McpError::InvalidRequest {
                message: "Invalid file path".to_string(),
            })?;
        
        if !canonical.starts_with(&self.base_path) {
            return Err(McpError::InvalidRequest {
                message: "Path outside allowed directory".to_string(),
            });
        }
        
        Ok(())
    }
}
```

### Performance Optimization

```rust
use airs_mcp::base::jsonrpc::concurrent::ConcurrentJsonRpcConfig;
use std::time::Duration;

let config = ConcurrentJsonRpcConfig::builder()
    .request_timeout(Duration::from_secs(30))
    .max_concurrent_requests(50)
    .buffer_size(16384)
    .enable_correlation_tracking(true)
    .build();

let server = McpServerBuilder::new()
    .with_config(config)
    .build()?;
```

### Error Recovery

```rust
use tokio::signal;

async fn run_with_graceful_shutdown() -> Result<(), Box<dyn std::error::Error>> {
    let server = build_server()?;
    
    let shutdown = async {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        info!("Received shutdown signal");
    };
    
    tokio::select! {
        result = server.run() => {
            match result {
                Ok(_) => info!("Server completed normally"),
                Err(e) => {
                    error!("Server error: {}", e);
                    return Err(e.into());
                }
            }
        }
        _ = shutdown => {
            info!("Shutting down gracefully");
        }
    }
    
    Ok(())
}
```

## Troubleshooting

### Common Issues

1. **Server Not Appearing in Claude**: Check configuration file path and syntax
2. **Permission Denied**: Ensure server binary has execute permissions
3. **Connection Refused**: Verify server starts without errors
4. **Tools Not Working**: Check tool schema validation

### Debug Commands

```bash
# Validate configuration
jq empty claude_desktop_config.json

# Test server directly
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | your-server

# Check file permissions
ls -la /path/to/your/server

# Monitor Claude Desktop process
ps aux | grep -i claude
```

---

*Next: [Advanced Patterns](./advanced_patterns.md) | Return to [Usages Overview](../usages.md)*

Check back soon for comprehensive Claude Desktop integration guidance.
