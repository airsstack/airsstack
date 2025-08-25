# MCP Server Foundation Patterns

**Category**: Integration  
**Complexity**: High  
**Last Updated**: 2025-08-25  
**Maintainer**: Core Development Team  
**Related Task**: task_002_mcp_server_foundation

## Overview
**What is this knowledge about?**

This document captures the technical implementation patterns for building MCP servers using the airs-mcp foundation, specifically focusing on STDIO transport integration, tool provider architecture, and Claude Desktop compatibility patterns.

**Why this knowledge is important**: These patterns provide the foundation for all MCP server implementations in the AIRS ecosystem, ensuring consistent architecture, proper integration with the airs-mcp foundation, and reliable Claude Desktop compatibility.

**Who should read this**: Developers implementing MCP servers, anyone working with MCP protocol integration, and engineers building Claude Desktop compatible tools.

## Context & Background
**When and why was this approach chosen?**

These patterns were developed during task_002 planning to leverage the production-ready airs-mcp foundation while establishing filesystem-specific MCP server capabilities. The approach prioritizes:

- **Foundation Leverage**: Maximum reuse of airs-mcp infrastructure
- **Claude Desktop Compatibility**: Primary deployment target
- **Extensible Architecture**: Support for future tool expansion
- **Security Integration**: Seamless security validation flow

**Related ADRs**: ADR-002 documents the specific architectural decisions for MCP server implementation.

## Technical Details
**How does this work?**

### Core Server Architecture Pattern

#### FilesystemMcpServer Structure
```rust
use airs_mcp::integration::mcp::server::{McpServer, McpServerConfig, ToolProvider};
use airs_mcp::transport::stdio::StdioTransport;
use std::sync::Arc;

pub struct FilesystemMcpServer {
    security_manager: Arc<SecurityManager>,
    config: Arc<Settings>,
    // Additional state as needed
}

impl FilesystemMcpServer {
    pub async fn new(config: Settings) -> Result<Self, McpServerError> {
        let security_manager = Arc::new(SecurityManager::new(&config.security)?);
        
        Ok(Self {
            security_manager,
            config: Arc::new(config),
        })
    }
}
```

**Key Integration Points**:
- **airs-mcp Foundation**: Use `McpServer` as the transport and protocol handler
- **Security Integration**: All operations go through `SecurityManager` validation
- **Configuration Management**: Centralized settings following workspace patterns

### STDIO Transport Pattern

#### Server Initialization
```rust
use airs_mcp::transport::stdio::StdioTransport;

pub async fn start_filesystem_server() -> Result<(), McpServerError> {
    // 1. Initialize STDIO transport (Claude Desktop standard)
    let transport = StdioTransport::new().await
        .map_err(|e| McpServerError::TransportInitialization(e))?;
    
    // 2. Create filesystem-specific server implementation
    let config = Settings::load_from_env()?;
    let filesystem_server = FilesystemMcpServer::new(config).await?;
    
    // 3. Combine transport + server implementation
    let mcp_server = McpServer::new(transport, filesystem_server)?;
    
    // 4. Start message loop
    println!("🚀 Filesystem MCP server ready for Claude Desktop");
    mcp_server.run().await
        .map_err(|e| McpServerError::ServerRuntime(e))
}
```

**Critical Implementation Notes**:
- **STDIO Standard**: Claude Desktop communicates exclusively via STDIO
- **Newline Framing**: Messages are newline-delimited JSON (handled by StdioTransport)
- **Process Lifecycle**: Server starts when Claude Desktop spawns process, exits when connection closes
- **Error Handling**: All transport errors must be properly mapped to MCP error responses

### Tool Provider Implementation Pattern

#### Core Tool Registration
```rust
use airs_mcp::integration::mcp::server::ToolProvider;
use airs_mcp::shared::protocol::{Tool, Content};
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
impl ToolProvider for FilesystemMcpServer {
    async fn list_tools(&self) -> McpResult<Vec<Tool>> {
        Ok(vec![
            Tool {
                name: "read_file".to_string(),
                description: "Read file contents with automatic encoding detection and security validation".to_string(),
                input_schema: self.read_file_schema(),
            },
            Tool {
                name: "write_file".to_string(),
                description: "Write file contents with human approval workflow and security checks".to_string(),
                input_schema: self.write_file_schema(),
            },
            Tool {
                name: "list_directory".to_string(),
                description: "List directory contents with metadata and filtering capabilities".to_string(),
                input_schema: self.list_directory_schema(),
            },
        ])
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> McpResult<Vec<Content>> {
        // Security validation happens here for ALL tool calls
        let operation = FilesystemOperation::from_tool_call(name, &arguments)?;
        self.security_manager.validate_operation(&operation).await?;
        
        match name {
            "read_file" => self.handle_read_file(arguments).await,
            "write_file" => self.handle_write_file(arguments).await,
            "list_directory" => self.handle_list_directory(arguments).await,
            _ => Err(McpError::tool_not_found(name)),
        }
    }
}
```

**Design Benefits**:
- **Automatic Discovery**: Claude Desktop automatically discovers all registered tools
- **Type Safety**: JSON Schema validation prevents invalid parameters
- **Security First**: All operations go through security validation before execution
- **Extensible**: Adding new tools requires only implementing the handler and schema

#### Tool Schema Pattern
```rust
impl FilesystemMcpServer {
    fn read_file_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path to read (relative or absolute)"
                },
                "encoding": {
                    "type": "string",
                    "description": "Text encoding (auto-detected if not specified)",
                    "enum": ["utf-8", "utf-16", "ascii", "auto"]
                },
                "max_size": {
                    "type": "integer",
                    "description": "Maximum file size in bytes (default: 10MB)",
                    "minimum": 1,
                    "maximum": 100_000_000
                }
            },
            "required": ["path"],
            "additionalProperties": false
        })
    }
    
    fn write_file_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path to write (relative or absolute)"
                },
                "content": {
                    "type": "string",
                    "description": "File content to write"
                },
                "encoding": {
                    "type": "string",
                    "description": "Text encoding for writing (default: utf-8)",
                    "enum": ["utf-8", "utf-16", "ascii"]
                },
                "create_dirs": {
                    "type": "boolean",
                    "description": "Create parent directories if they don't exist"
                }
            },
            "required": ["path", "content"],
            "additionalProperties": false
        })
    }
}
```

### Error Handling Pattern

#### Custom Error Mapping
```rust
use airs_mcp::integration::mcp::error::{McpError, McpResult};

#[derive(Debug, thiserror::Error)]
pub enum FilesystemError {
    #[error("Security policy violation: {operation} on {path} - {reason}")]
    SecurityViolation {
        operation: String,
        path: String,
        reason: String,
    },
    
    #[error("Human approval required for {operation}: {context}")]
    ApprovalRequired {
        operation: String,
        context: String,
    },
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Invalid file path: {0}")]
    InvalidPath(String),
}

impl From<FilesystemError> for McpError {
    fn from(err: FilesystemError) -> Self {
        match err {
            FilesystemError::SecurityViolation { operation, path, reason } => {
                McpError::invalid_request(format!(
                    "Security violation: {} operation on {} rejected - {}", 
                    operation, path, reason
                ))
            },
            FilesystemError::ApprovalRequired { operation, context } => {
                McpError::invalid_request(format!(
                    "Human approval required for {}: {}", 
                    operation, context
                ))
            },
            FilesystemError::FileNotFound(path) => {
                McpError::invalid_params(format!("File not found: {}", path))
            },
            FilesystemError::PermissionDenied(path) => {
                McpError::invalid_request(format!("Access denied: {}", path))
            },
            FilesystemError::InvalidPath(path) => {
                McpError::invalid_params(format!("Invalid path: {}", path))
            },
        }
    }
}
```

### Security Integration Pattern

#### Operation Validation Flow
```rust
impl FilesystemMcpServer {
    async fn handle_read_file(&self, arguments: Value) -> McpResult<Vec<Content>> {
        // 1. Parse and validate parameters
        let request: ReadFileRequest = serde_json::from_value(arguments)
            .map_err(|e| McpError::invalid_params(e.to_string()))?;
        
        // 2. Security validation (critical step)
        let operation = FilesystemOperation::ReadFile { 
            path: request.path.clone() 
        };
        self.security_manager.validate_read_access(&request.path).await
            .map_err(|e| FilesystemError::SecurityViolation {
                operation: "read_file".to_string(),
                path: request.path.clone(),
                reason: e.to_string(),
            })?;
        
        // 3. Execute operation
        let content = tokio::fs::read_to_string(&request.path).await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => {
                    FilesystemError::FileNotFound(request.path.clone())
                },
                std::io::ErrorKind::PermissionDenied => {
                    FilesystemError::PermissionDenied(request.path.clone())
                },
                _ => FilesystemError::InvalidPath(format!("{}: {}", request.path, e)),
            })?;
        
        // 4. Return MCP content response
        Ok(vec![Content::text(content)])
    }
}
```

### Claude Desktop Compatibility Pattern

#### Integration Testing Approach
```rust
// Integration test for Claude Desktop compatibility
#[tokio::test]
async fn test_claude_desktop_compatibility() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Start filesystem MCP server process
    let server_path = env!("CARGO_BIN_FILE_airs-mcp-fs");
    let mut server_process = tokio::process::Command::new(server_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    
    // 2. Create MCP client that communicates via STDIO (like Claude Desktop)
    let stdin = server_process.stdin.take().unwrap();
    let stdout = server_process.stdout.take().unwrap();
    
    // 3. Test MCP initialization handshake
    let init_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });
    
    // Send initialization
    let mut writer = BufWriter::new(stdin);
    writer.write_all(init_request.to_string().as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    
    // Read response
    let mut reader = BufReader::new(stdout);
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;
    
    let response: serde_json::Value = serde_json::from_str(&response_line)?;
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["capabilities"]["tools"].is_object());
    
    // 4. Test tool discovery
    let tools_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });
    
    // Continue with tool discovery and execution tests...
    
    Ok(())
}
```

## Implementation Guidelines

### Development Workflow
1. **Foundation First**: Always start with airs-mcp infrastructure
2. **Security Validation**: Every tool call MUST go through security validation
3. **Error Mapping**: Convert all internal errors to appropriate MCP error responses
4. **Schema Validation**: Define comprehensive JSON schemas for all tools
5. **Integration Testing**: Test with actual STDIO communication patterns

### Performance Considerations
- **Async Operations**: All I/O operations must be async (tokio::fs, not std::fs)
- **Memory Management**: Stream large files instead of loading entirely into memory
- **Response Times**: Target sub-100ms for typical filesystem operations
- **Connection Handling**: Graceful shutdown when Claude Desktop disconnects

### Security Requirements
- **Path Validation**: All paths must be canonicalized and validated
- **Access Control**: Security manager validates every operation
- **Audit Logging**: All operations must be logged for security audit
- **Human Approval**: Write operations require explicit user approval

## Code Examples
**Where can I see this in action?**

### Reference Implementations
- **airs-mcp/examples/simple-mcp-server/**: Basic MCP server using airs-mcp foundation
- **airs-mcp/src/integration/mcp/server.rs**: Core McpServer implementation
- **airs-mcp/tests/mcp_ecosystem_tests.rs**: Integration test patterns

### Development Examples
- **task_002**: Will implement these patterns in practice
- **task_003**: Will extend these patterns for specific filesystem operations
- **Claude Desktop Integration**: Real-world validation with actual Claude Desktop

## Troubleshooting

### Common Issues
1. **STDIO Framing**: Ensure all messages end with newline character
2. **JSON Schema**: Validate schemas match tool parameter expectations
3. **Security Integration**: Ensure security validation doesn't block valid operations
4. **Error Responses**: Return proper MCP error codes for different failure types

### Debugging Techniques
- **Message Logging**: Log all MCP messages for protocol debugging
- **Security Tracing**: Trace security validation decisions
- **Transport Monitoring**: Monitor STDIO transport for connection issues
- **Claude Desktop Logs**: Check Claude Desktop logs for integration issues

## Related Documentation
- **ADR-002**: MCP Server Architecture Decisions
- **Security Framework Architecture**: Security integration patterns
- **MCP Integration Patterns**: Broader MCP ecosystem integration
- **task_002**: Implementation progress and decisions
