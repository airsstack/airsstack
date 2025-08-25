# task_002 - MCP Server Foundation

**Status:** pending  
**Added:** 2025-08-16  
**Updated:** 2025-08-16

## Original Request
Implement the basic MCP server infrastructure with STDIO transport, JSON-RPC 2.0 message handling, tool registration framework, and Claude Desktop integration validation.

## Thought Process
This task builds on the project foundation to create the core MCP server that enables Claude Desktop integration. The implementation follows the documented multi-layer architecture:

1. **STDIO Transport**: Claude Desktop communicates via STDIO, requiring proper message framing and async handling without blocking operations.

2. **Tool Registration Framework**: Need a flexible system for registering filesystem operation tools that can be discovered by MCP clients.

3. **Message Routing**: JSON-RPC 2.0 message handling with proper error responses and async operation support.

4. **Integration Validation**: Must verify successful connection and tool discovery with Claude Desktop to ensure the foundation works correctly.

This task establishes the communication layer that all filesystem operations will depend on. Success here enables Phase 1 file operation development.

### Available Infrastructure Analysis
The task leverages the **airs-mcp** foundation which provides:
- ✅ **High-level MCP Server API** (`airs_mcp::integration::mcp::server`)
- ✅ **STDIO Transport** (`airs_mcp::transport::stdio::StdioTransport`) 
- ✅ **JSON-RPC Message Handling** with automatic routing
- ✅ **Provider Traits** for tools, resources, and prompts
- ✅ **Production-ready Examples** in `examples/simple-mcp-server/`

### Architecture Pattern
```
┌─────────────────────────────────────────────────────────────┐
│                    Claude Desktop                           │
│                   (MCP Client)                              │
└─────────────────────┬───────────────────────────────────────┘
                      │ STDIO Transport (JSON-RPC 2.0)
┌─────────────────────▼───────────────────────────────────────┐
│              AIRS MCP-FS Server                             │
│  ┌─────────────────────────────────────────────────────────┤
│  │ McpServer (airs-mcp foundation)                         │
│  │  ├─ STDIO Transport + Message Routing                  │
│  │  ├─ Tool Registration Framework                         │
│  │  └─ JSON-RPC Response Handling                         │
│  └─────────────────────────────────────────────────────────┤
│  │ FilesystemToolProvider (task_002)                      │
│  │  ├─ Tool Discovery: read_file, write_file, list_dir   │
│  │  ├─ Parameter Validation                               │
│  │  └─ Security Integration Points                        │
└──┴─────────────────────────────────────────────────────────┘
```

## Implementation Plan
### Detailed Technical Implementation (6 Subtasks)

#### **2.1: Create MCP Server Struct** 
**Technical Approach:**
```rust
// In src/mcp/server.rs
use airs_mcp::integration::mcp::server::{McpServer, McpServerConfig, ToolProvider};
use airs_mcp::transport::stdio::StdioTransport;

pub struct FilesystemMcpServer {
    security_manager: Arc<SecurityManager>,
    config: Arc<Settings>,
}

impl FilesystemMcpServer {
    pub async fn new(config: Settings) -> Result<Self, McpServerError> {
        // Initialize security manager and other components
    }
}
```

**Key Integration Points:**
- Use `airs_mcp::integration::mcp::server::McpServer` as the foundation
- Implement `ToolProvider` trait for filesystem operations
- Integrate with existing `SecurityManager` from task_001

#### **2.2: Implement STDIO Transport**
**Technical Approach:**
```rust
// Leverage existing StdioTransport from airs-mcp
use airs_mcp::transport::stdio::StdioTransport;

pub async fn start_server() -> Result<(), McpServerError> {
    let transport = StdioTransport::new().await?;
    let filesystem_server = FilesystemMcpServer::new(config).await?;
    
    let mcp_server = McpServer::new(
        transport,
        filesystem_server, // implements ToolProvider
    )?;
    
    mcp_server.run().await
}
```

**Critical for Claude Desktop:**
- STDIO communication is Claude Desktop's standard integration method
- Newline-delimited JSON message framing (handled by `StdioTransport`)
- Automatic process lifecycle management

#### **2.3: Tool Registration Framework**
**Implementation:**
```rust
#[async_trait]
impl ToolProvider for FilesystemMcpServer {
    async fn list_tools(&self) -> McpResult<Vec<Tool>> {
        Ok(vec![
            Tool {
                name: "read_file".to_string(),
                description: "Read file contents with security validation".to_string(),
                input_schema: read_file_schema(),
            },
            Tool {
                name: "write_file".to_string(), 
                description: "Write file with human approval workflow".to_string(),
                input_schema: write_file_schema(),
            },
            Tool {
                name: "list_directory".to_string(),
                description: "List directory contents with metadata".to_string(),
                input_schema: list_directory_schema(),
            },
        ])
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> McpResult<Vec<Content>> {
        match name {
            "read_file" => self.handle_read_file(arguments).await,
            "write_file" => self.handle_write_file(arguments).await,
            "list_directory" => self.handle_list_directory(arguments).await,
            _ => Err(McpError::tool_not_found(name)),
        }
    }
}
```

**Design Benefits:**
- Automatic tool discovery by Claude Desktop
- Type-safe parameter validation using JSON Schema
- Extensible architecture for Phase 2 binary processing tools

#### **2.4: JSON-RPC Message Routing**
**Handled by AIRS MCP Foundation:**
- ✅ Automatic JSON-RPC 2.0 compliance
- ✅ Request/response correlation management  
- ✅ Error handling with proper error codes
- ✅ Message validation and parsing

**Custom Error Handling:**
```rust
// In src/mcp/error.rs
use airs_mcp::integration::mcp::error::{McpError, McpResult};

pub enum FilesystemError {
    SecurityViolation { operation: String, path: String, reason: String },
    ApprovalRequired { operation: String, context: String },
    FileNotFound(String),
    PermissionDenied(String),
}

impl From<FilesystemError> for McpError {
    fn from(err: FilesystemError) -> Self {
        match err {
            FilesystemError::SecurityViolation { .. } => {
                McpError::invalid_request("Security policy violation")
            },
            // ... other mappings
        }
    }
}
```

#### **2.5: Claude Desktop Integration Test**
**Test Strategy:**
```rust
// In tests/integration/claude_desktop_integration.rs
#[tokio::test]
async fn test_claude_desktop_compatibility() {
    // 1. Start filesystem MCP server 
    let server_process = start_filesystem_server().await?;
    
    // 2. Simulate Claude Desktop connection via STDIO
    let mut client = McpClient::connect_stdio(server_process).await?;
    
    // 3. Test initialization handshake
    let capabilities = client.initialize().await?;
    assert!(capabilities.tools.is_some());
    
    // 4. Test tool discovery
    let tools = client.list_tools().await?;
    assert!(tools.iter().any(|t| t.name == "read_file"));
    
    // 5. Test basic tool execution
    let result = client.call_tool("list_directory", json!({"path": "."})).await?;
    assert!(!result.is_empty());
}
```

#### **2.6: Health Check and Tool Discovery** 
**Implementation Focus:**
- Server status reporting for Claude Desktop
- Tool capability advertisement
- Connection health monitoring
- Graceful shutdown handling

### Success Criteria

#### **Functional Requirements**
1. ✅ **Claude Desktop Connection**: Server successfully accepts STDIO connections
2. ✅ **Tool Discovery**: Claude Desktop can discover all registered filesystem tools  
3. ✅ **Message Handling**: Proper JSON-RPC 2.0 request/response handling
4. ✅ **Error Handling**: Graceful error responses for invalid requests
5. ✅ **Security Integration**: All tool calls route through security validation

#### **Technical Requirements**
- **Zero Warnings**: Clean compilation with `cargo check`, `cargo clippy`
- **Test Coverage**: Integration tests demonstrating Claude Desktop compatibility
- **Workspace Standards**: Compliance with all workspace standards (§2.1, §3.2, §4.3, §5.1)
- **Documentation**: Clear examples and usage patterns for tool providers

### Risk Assessment

#### **Low Risk Items** ✅
- **AIRS MCP Foundation**: Production-ready with comprehensive examples
- **STDIO Transport**: Well-tested in existing examples
- **Tool Provider Pattern**: Proven architecture with clear trait implementations

#### **Medium Risk Items** ⚠️
- **Claude Desktop Compatibility**: Need to validate message format expectations
- **Security Integration**: Ensure security validation doesn't break MCP flow
- **Error Message Quality**: User-friendly error messages for Claude Desktop users

#### **Mitigation Strategies**
- **Early Testing**: Create Claude Desktop integration test first
- **Reference Examples**: Use `simple-mcp-server` example as validation baseline
- **Incremental Implementation**: Start with minimal tool set, expand iteratively

## Progress Tracking

**Overall Status:** not_started - 0%

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 2.1 | Create MCP server struct using airs-mcp foundation | not_started | 2025-08-25 | Leverage existing AIRS infrastructure with FilesystemMcpServer |
| 2.2 | Implement STDIO transport with async message handling | not_started | 2025-08-25 | Critical for Claude Desktop integration - use StdioTransport |
| 2.3 | Set up tool registration framework for filesystem tools | not_started | 2025-08-25 | Foundation for all file operations - implement ToolProvider trait |
| 2.4 | Add JSON-RPC 2.0 message routing and error handling | not_started | 2025-08-25 | Proper protocol compliance with custom FilesystemError mapping |
| 2.5 | Create Claude Desktop integration test | not_started | 2025-08-25 | Validate end-to-end communication with real Claude Desktop |
| 2.6 | Implement basic health check and tool discovery | not_started | 2025-08-25 | Ensure Claude can discover tools and monitor server health |

## Progress Log
### 2025-08-25
- **Technical Plan Documentation**: Comprehensive technical implementation plan documented
- **Infrastructure Analysis**: Detailed analysis of available airs-mcp foundation infrastructure
- **Architecture Design**: Complete architecture pattern with STDIO transport integration
- **Implementation Strategy**: 6-subtask breakdown with detailed code examples and approaches
- **Risk Assessment**: Identified low/medium risk items with specific mitigation strategies
- **Success Criteria**: Defined functional and technical requirements for task completion

### 2025-08-16
- Task created as part of Phase 1 foundation development
- Depends on completion of task_001 (project foundation setup)
- Architecture and patterns clearly documented for implementation
- Integration approach with airs-mcp foundation established
