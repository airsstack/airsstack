# ADR-002: MCP Server Architecture Decisions (MIGRATED)

**Status**: Accepted - Requires Assessment for New Architecture  
**Date**: 2025-08-25  
**Migration Date**: 2025-09-23  
**Deciders**: [@hiraq, GitHub Copilot]  
**Technical Story**: Migrated from airs-mcp-fs - MCP server architecture for Claude Desktop integration

## Migration Notice

**Source**: Migrated from `airs-mcp-fs` ADR registry  
**Assessment Required**: Verify if architectural decisions apply to `airs-mcpserver-fs`  
**Current Implementation**: Review against actual airs-mcpserver-fs architecture

## Context and Problem Statement (Original)

The airs-mcp-fs project required a robust MCP (Model Context Protocol) server implementation to enable Claude Desktop integration for filesystem operations. 

**Assessment Required**: Does this context still apply to airs-mcpserver-fs?

Key architectural forces:
- **Foundation Leverage**: Maximize reuse of existing airs-mcp infrastructure vs. custom implementation
- **Transport Strategy**: STDIO vs. HTTP transport for initial Claude Desktop compatibility
- **Tool Architecture**: Direct message handling vs. trait-based tool provider patterns
- **Security Integration**: Seamless security validation without breaking MCP protocol flow
- **Error Handling**: Protocol-compliant error responses while maintaining debugging capability
- **Future Extensibility**: Support for additional MCP clients beyond Claude Desktop

## Decision Drivers (Assessment Required)

- **Development Velocity**: Leverage proven infrastructure to accelerate implementation
- **Claude Desktop Compatibility**: Primary deployment target requiring STDIO transport
- **Ecosystem Alignment**: Consistency with broader AIRS workspace patterns and standards
- **Security Requirements**: All filesystem operations must go through security validation
- **Maintainability**: Clean separation of concerns between protocol and business logic
- **Testing Strategy**: Enable comprehensive testing including Claude Desktop integration
- **Performance Requirements**: Sub-100ms response times for typical filesystem operations

## Selected Option: airs-mcp Foundation with STDIO Transport

**Original Decision**: Use airs-mcp McpServer with StdioTransport and ToolProvider traits

### Assessment for airs-mcpserver-fs

**Check Current Implementation**:
```rust
// Verify this pattern exists in airs-mcpserver-fs
use airs_mcp::transport::adapters::stdio::StdioTransportBuilder;
use airs_mcpserver_fs::mcp::FilesystemMessageHandler;

let mut transport = StdioTransportBuilder::new()
    .with_message_handler(message_handler)
    .build()
    .await?;
```

### Decision Benefits (If Still Applicable)

**Pros**: 
- Direct Claude Desktop compatibility (STDIO is the standard)
- Proven airs-mcp infrastructure for protocol handling
- Clean trait-based architecture for tool implementation
- Automatic JSON-RPC 2.0 compliance and error handling
- Extensible architecture for future tool additions
- Comprehensive testing infrastructure available

**Cons**: 
- Dependency on airs-mcp foundation evolution
- STDIO debugging more complex than HTTP
- Single client connection model

## Implementation Architecture (Assessment Required)

### Transport Layer
**Pattern to Verify**:
```rust
// Check if this pattern is used in main.rs
let transport = StdioTransportBuilder::new()
    .with_message_handler(filesystem_handler)
    .build()
    .await?;

transport.start().await?;
transport.wait_for_completion().await?;
```

### Tool Provider Pattern
**Pattern to Verify**:
```rust
// Check if FilesystemMessageHandler implements this pattern
pub struct FilesystemMessageHandler {
    server: Arc<DefaultFilesystemMcpServer>,
}

impl MessageHandler for FilesystemMessageHandler {
    async fn handle_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        // Route to appropriate tool handler
    }
}
```

### Security Integration
**Pattern to Verify**:
```rust
// Check if security validation is integrated at this level
pub async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResponse> {
    // Security validation before tool execution
    self.security_manager.validate_operation(&operation).await?;
    
    // Execute tool with validation
    match request.name.as_str() {
        "read_file" => self.handle_read_file(request.arguments).await,
        "write_file" => self.handle_write_file(request.arguments).await,
        "list_directory" => self.handle_list_directory(request.arguments).await,
        _ => Err(McpError::method_not_found(&request.name)),
    }
}
```

## Assessment Checklist

### Architecture Consistency
- [ ] **STDIO Transport**: Is StdioTransportBuilder used in main.rs?
- [ ] **MessageHandler Pattern**: Is FilesystemMessageHandler implemented?
- [ ] **Tool Provider**: Are tools implemented using the expected pattern?
- [ ] **Security Integration**: Is security validation integrated properly?
- [ ] **Error Handling**: Are MCP-compliant errors used?

### Implementation Verification
- [ ] **airs-mcp Dependency**: Is airs-mcp foundation used as expected?
- [ ] **Claude Desktop Compatibility**: Does STDIO transport work with Claude Desktop?
- [ ] **Performance**: Are sub-100ms response times achieved?
- [ ] **Extensibility**: Can new tools be added easily?

## Decision Outcome Assessment

### If Architecture is Consistent
- **Status**: Keep as "Accepted" 
- **Update**: Add migration success notes
- **Documentation**: Reference current implementation details

### If Architecture Has Changed
- **Status**: Update to "Superseded" if major changes
- **New ADR**: Create new ADR documenting architectural changes
- **Migration Notes**: Document what changed and why

### If Implementation Differs
- **Status**: Update to "Modified"
- **Details**: Document specific implementation differences
- **Rationale**: Explain reasons for implementation variations

## Success Metrics (Assessment Required)

### Technical Metrics
- [ ] **Claude Desktop Integration**: Successful connection and tool execution
- [ ] **Response Times**: <100ms for file operations, <200ms for directory listings
- [ ] **Error Handling**: Proper MCP error responses for all failure cases
- [ ] **Security Validation**: All operations go through security checks

### Development Metrics
- [ ] **Implementation Velocity**: Leveraged airs-mcp infrastructure effectively
- [ ] **Code Quality**: Clean separation between MCP protocol and business logic
- [ ] **Testing Coverage**: Comprehensive tests including Claude Desktop integration
- [ ] **Maintainability**: Easy to add new tools and modify existing ones

## Next Steps

1. **Implementation Review**: Compare current airs-mcpserver-fs against this ADR
2. **Update Decision Status**: Based on implementation consistency
3. **Document Changes**: If architecture evolved, document the changes
4. **Validate Success Metrics**: Verify that success criteria are met
5. **Plan Future ADRs**: Identify any new architectural decisions needed