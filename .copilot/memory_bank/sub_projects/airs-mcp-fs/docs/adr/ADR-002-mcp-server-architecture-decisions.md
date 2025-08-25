# ADR-002: MCP Server Architecture Decisions

**Status**: Accepted  
**Date**: 2025-08-25  
**Deciders**: [@hiraq, GitHub Copilot]  
**Technical Story**: task_002_mcp_server_foundation - Establishing MCP server architecture for Claude Desktop integration

## Context and Problem Statement

The airs-mcp-fs project requires a robust MCP (Model Context Protocol) server implementation to enable Claude Desktop integration for filesystem operations. Multiple architectural approaches exist for MCP server implementation, ranging from custom protocol handling to leveraging existing foundations, each with different trade-offs for development velocity, maintainability, and ecosystem compatibility.

Key architectural forces:
- **Foundation Leverage**: Maximize reuse of existing airs-mcp infrastructure vs. custom implementation
- **Transport Strategy**: STDIO vs. HTTP transport for initial Claude Desktop compatibility
- **Tool Architecture**: Direct message handling vs. trait-based tool provider patterns
- **Security Integration**: Seamless security validation without breaking MCP protocol flow
- **Error Handling**: Protocol-compliant error responses while maintaining debugging capability
- **Future Extensibility**: Support for additional MCP clients beyond Claude Desktop

## Decision Drivers

- **Development Velocity**: Leverage proven infrastructure to accelerate implementation
- **Claude Desktop Compatibility**: Primary deployment target requiring STDIO transport
- **Ecosystem Alignment**: Consistency with broader AIRS workspace patterns and standards
- **Security Requirements**: All filesystem operations must go through security validation
- **Maintainability**: Clean separation of concerns between protocol and business logic
- **Testing Strategy**: Enable comprehensive testing including Claude Desktop integration
- **Performance Requirements**: Sub-100ms response times for typical filesystem operations

## Considered Options

### Option 1: Custom MCP Protocol Implementation
- **Approach**: Implement MCP protocol handling from scratch
- **Pros**: 
  - Maximum control over protocol implementation details
  - No external dependencies on airs-mcp foundation
  - Optimized specifically for filesystem operations
- **Cons**: 
  - Significant development effort for protocol compliance
  - Potential protocol bugs and edge cases
  - Maintenance burden for MCP specification updates
  - No reuse of existing airs-mcp infrastructure
- **Implementation effort**: High
- **Risk**: High - protocol implementation complexity

### Option 2: airs-mcp Foundation with HTTP Transport
- **Approach**: Use airs-mcp server foundation with HTTP transport
- **Pros**: 
  - Proven transport infrastructure
  - Better debugging and monitoring capabilities
  - Supports multiple concurrent clients
  - Web-based testing tools available
- **Cons**: 
  - Claude Desktop requires STDIO transport for MCP servers
  - Additional configuration complexity for Claude Desktop integration
  - Network layer adds latency for local filesystem operations
- **Implementation effort**: Medium
- **Risk**: Medium - Claude Desktop compatibility issues

### Option 3: airs-mcp Foundation with STDIO Transport (Selected)
- **Approach**: Leverage airs-mcp McpServer with StdioTransport and ToolProvider traits
- **Pros**: 
  - Direct Claude Desktop compatibility (STDIO is the standard)
  - Proven airs-mcp infrastructure for protocol handling
  - Clean trait-based architecture for tool implementation
  - Automatic JSON-RPC 2.0 compliance and error handling
  - Extensible architecture for future tool additions
  - Comprehensive testing infrastructure available
- **Cons**: 
  - Dependency on airs-mcp foundation evolution
  - STDIO debugging more complex than HTTP
  - Single client connection model
- **Implementation effort**: Low-Medium
- **Risk**: Low - proven infrastructure with Claude Desktop compatibility

### Option 4: Hybrid Approach with Multiple Transports
- **Approach**: Support both STDIO and HTTP transports simultaneously
- **Pros**: 
  - Maximum flexibility for different MCP clients
  - STDIO for Claude Desktop, HTTP for web clients
  - Enhanced debugging capabilities
- **Cons**: 
  - Significant complexity increase
  - Double the testing burden
  - Authentication and security model complications
  - Over-engineering for current requirements
- **Implementation effort**: High
- **Risk**: Medium - complexity management challenges

## Decision Outcome

**Chosen option**: "Option 3: airs-mcp Foundation with STDIO Transport"

### Rationale

**Primary Factors**:
1. **Claude Desktop Compatibility**: STDIO transport is the standard for Claude Desktop MCP server integration
2. **Foundation Leverage**: airs-mcp provides production-ready infrastructure for protocol handling
3. **Development Velocity**: Trait-based architecture enables rapid tool implementation
4. **Proven Architecture**: Existing examples demonstrate successful integration patterns

**Implementation Strategy**:
```rust
// Core server structure leveraging airs-mcp foundation
pub struct FilesystemMcpServer {
    security_manager: Arc<SecurityManager>,
    config: Arc<Settings>,
}

// Tool Provider trait implementation for filesystem operations
#[async_trait]
impl ToolProvider for FilesystemMcpServer {
    async fn list_tools(&self) -> McpResult<Vec<Tool>> { /* ... */ }
    async fn call_tool(&self, name: &str, arguments: Value) -> McpResult<Vec<Content>> { /* ... */ }
}

// Server initialization with STDIO transport
pub async fn start_server() -> Result<(), McpServerError> {
    let transport = StdioTransport::new().await?;
    let filesystem_server = FilesystemMcpServer::new(config).await?;
    let mcp_server = McpServer::new(transport, filesystem_server)?;
    mcp_server.run().await
}
```

### Trade-offs Accepted

**Benefits Gained**:
- ✅ **Rapid Development**: Leverage existing airs-mcp infrastructure
- ✅ **Claude Desktop Ready**: STDIO transport works immediately with Claude Desktop
- ✅ **Protocol Compliance**: Automatic JSON-RPC 2.0 and MCP specification compliance
- ✅ **Security Integration**: Clean integration points for security validation
- ✅ **Extensible Architecture**: Easy to add new tools via ToolProvider trait
- ✅ **Testing Infrastructure**: Comprehensive testing patterns available

**Compromises Made**:
- ⚠️ **Single Transport**: STDIO only (can be extended later if needed)
- ⚠️ **Foundation Dependency**: Coupled to airs-mcp evolution (acceptable trade-off)
- ⚠️ **Debugging Complexity**: STDIO debugging less convenient than HTTP (mitigated by logging)

## Implementation Details

### Tool Provider Architecture
```rust
// Security-first tool implementation pattern
async fn call_tool(&self, name: &str, arguments: Value) -> McpResult<Vec<Content>> {
    // 1. Parse and validate parameters
    let operation = FilesystemOperation::from_tool_call(name, &arguments)?;
    
    // 2. Security validation (critical)
    self.security_manager.validate_operation(&operation).await?;
    
    // 3. Execute operation
    match name {
        "read_file" => self.handle_read_file(arguments).await,
        "write_file" => self.handle_write_file(arguments).await,
        "list_directory" => self.handle_list_directory(arguments).await,
        _ => Err(McpError::tool_not_found(name)),
    }
}
```

### Error Handling Strategy
```rust
// Convert internal errors to MCP-compliant responses
impl From<FilesystemError> for McpError {
    fn from(err: FilesystemError) -> Self {
        match err {
            FilesystemError::SecurityViolation { .. } => {
                McpError::invalid_request("Security policy violation")
            },
            FilesystemError::FileNotFound(path) => {
                McpError::invalid_params(format!("File not found: {}", path))
            },
            // ... other error mappings
        }
    }
}
```

### Security Integration Points
```rust
// All operations go through security validation
impl FilesystemMcpServer {
    async fn handle_write_file(&self, arguments: Value) -> McpResult<Vec<Content>> {
        let request: WriteFileRequest = serde_json::from_value(arguments)?;
        
        // Security validation with human approval workflow
        let approval_token = self.security_manager
            .validate_write_access(&request.path, &request.content).await?;
        
        // Execute with approval token
        self.filesystem.write_file_with_approval(request, approval_token).await?;
        Ok(vec![Content::text("File written successfully")])
    }
}
```

## Consequences

### Positive Consequences
- **Accelerated Development**: Proven infrastructure reduces implementation time from weeks to days
- **Immediate Claude Desktop Integration**: STDIO transport enables immediate testing and deployment
- **Security-First Architecture**: All operations flow through security validation by design
- **Extensible Foundation**: Adding new filesystem tools becomes straightforward
- **Production Ready**: Built on proven airs-mcp infrastructure with comprehensive testing

### Negative Consequences
- **STDIO Debugging Challenges**: Requires structured logging for protocol debugging
- **Foundation Coupling**: Evolution dependent on airs-mcp maintenance and updates
- **Single Transport Limitation**: May need HTTP transport for future web-based clients

### Mitigation Strategies
1. **Comprehensive Logging**: Implement detailed MCP message logging for debugging
2. **Integration Testing**: Automated tests with actual Claude Desktop communication patterns
3. **Documentation**: Detailed patterns for future developers and troubleshooting guides
4. **Future Transport Support**: Architecture designed to support additional transports if needed

## Validation Strategy

### Claude Desktop Integration Test
```rust
#[tokio::test]
async fn test_claude_desktop_protocol_compliance() {
    // 1. Start MCP server with STDIO transport
    let server = start_filesystem_server().await?;
    
    // 2. Simulate Claude Desktop initialization
    let capabilities = client.initialize().await?;
    assert!(capabilities.tools.is_some());
    
    // 3. Test tool discovery
    let tools = client.list_tools().await?;
    assert!(tools.iter().any(|t| t.name == "read_file"));
    
    // 4. Test tool execution with security validation
    let result = client.call_tool("read_file", json!({"path": "test.txt"})).await?;
    assert!(!result.is_empty());
}
```

### Performance Validation
- **Response Time**: All filesystem operations under 100ms for typical files
- **Memory Usage**: Streaming for large files, bounded memory consumption
- **Error Handling**: Graceful degradation for all failure scenarios

### Security Validation
- **Operation Validation**: All tool calls go through security manager validation
- **Path Traversal Prevention**: Canonical path validation for all file operations
- **Audit Logging**: Comprehensive audit trail for all filesystem operations

## Future Considerations

### Potential Evolution Paths
1. **Multi-Transport Support**: Add HTTP transport for web clients while maintaining STDIO
2. **Performance Optimization**: Implement advanced caching and streaming for large files
3. **Extended Tool Set**: Binary processing tools, advanced file operations
4. **Cloud Integration**: Remote filesystem access through MCP protocol

### Architecture Review Triggers
- **Performance Issues**: If response times exceed 100ms consistently
- **Claude Desktop Changes**: If Claude Desktop changes MCP integration requirements
- **Security Requirements**: If additional security validation layers are needed
- **Scale Requirements**: If multiple concurrent MCP clients become necessary

## Related Documentation
- **MCP Server Foundation Patterns**: Implementation patterns and code examples
- **Security Framework Architecture**: Security integration requirements and patterns
- **task_002**: Implementation progress and specific technical decisions
- **MCP Integration Patterns**: Broader ecosystem integration approaches
