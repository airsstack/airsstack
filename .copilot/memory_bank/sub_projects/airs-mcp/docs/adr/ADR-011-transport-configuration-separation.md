# ADR-011: Transport Configuration Separation Architecture

**Status**: Proposed  
**Date**: 2025-09-09  
**Decision Made By**: @hiraq + GitHub Copilot collaborative design  
**Impact Level**: Critical  
**Next Review**: 2026-03-09

## Context

The current `McpServer` architecture has several critical design issues that violate clean architecture principles and create dangerous coupling between transport and configuration concerns:

### Current Problems

1. **Handler Overwriting**: `McpServer::run()` calls `transport.set_message_handler()`, which overwrites any existing handler that the transport may have already configured
2. **Mixed Responsibilities**: `McpServer` handles both transport management AND MCP configuration, violating single responsibility principle
3. **Inconsistent Configuration**: `McpServerConfig` tries to be "one-size-fits-all" for different transport types (STDIO, HTTP, WebSocket) which have vastly different configuration needs
4. **Unused Logic**: The comprehensive `handle_request` method exists but isn't being used by HTTP transports which have their own `McpHandlers`
5. **Architectural Confusion**: Unclear whether transport or `McpServer` should own MCP message processing logic

### User's Architectural Insight

> "I think before a transport object injected to McpServer, they should set their message handler right? Meaning that McpServer should not care about the message handler at all"

This insight revealed the fundamental issue: **transport should be fully configured before being passed to `McpServer`**.

## Decision

We will implement a **Transport Configuration Separation Architecture** with the following design:

### 1. McpCoreConfig - Universal MCP Requirements

```rust
/// Core MCP protocol configuration required by all transports
#[derive(Debug, Clone)]
pub struct McpCoreConfig {
    /// Server information to send during initialization
    pub server_info: ServerInfo,
    /// Server capabilities to advertise
    pub capabilities: ServerCapabilities,
    /// Protocol version to support
    pub protocol_version: ProtocolVersion,
    /// Optional instructions to provide to clients during initialization
    pub instructions: Option<String>,
}
```

### 2. Pure MCP Transport Trait

```rust
/// Core MCP transport trait - focuses only on MCP protocol compliance
#[async_trait]
pub trait Transport: Send + Sync {
    /// Start the transport
    async fn start(&mut self) -> Result<(), TransportError>;
    
    /// Stop the transport
    async fn close(&mut self) -> Result<(), TransportError>;
    
    /// Set message handler for processing MCP messages
    fn set_message_handler(&mut self, handler: Arc<dyn MessageHandler>);
    
    /// Send an MCP message through this transport
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<(), TransportError>;
    
    /// Check if transport is connected/active
    fn is_connected(&self) -> bool;
}
```

### 3. Separate Transport Configuration Trait

```rust
/// Transport configuration trait - handles transport-specific configuration
pub trait TransportConfig: Send + Sync {
    /// Set MCP core configuration
    fn set_mcp_core_config(&mut self, config: McpCoreConfig) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Get MCP core configuration
    fn mcp_core_config(&self) -> Option<&McpCoreConfig>;
    
    /// Get effective MCP capabilities for this transport
    fn effective_capabilities(&self) -> ServerCapabilities;
    
    /// Get server info from core config
    fn server_info(&self) -> Option<&ServerInfo> {
        self.mcp_core_config().map(|c| &c.server_info)
    }
    
    /// Get protocol version from core config
    fn protocol_version(&self) -> Option<&ProtocolVersion> {
        self.mcp_core_config().map(|c| &c.protocol_version)
    }
}
```

### 4. Transport-Specific Configuration Structures

Each transport will have its own configuration structure that includes `McpCoreConfig` plus transport-specific settings:

```rust
/// STDIO-specific configuration
#[derive(Debug, Clone)]
pub struct StdioTransportConfig {
    pub mcp_core: McpCoreConfig,
    pub buffer_size: usize,
    pub flush_on_response: bool,
    pub strict_validation: bool,
    pub log_operations: bool,
    pub log_file_path: Option<PathBuf>,
}

/// HTTP-specific configuration
#[derive(Debug, Clone)]
pub struct HttpTransportConfig {
    pub mcp_core: McpCoreConfig,
    pub cors_origins: Vec<String>,
    pub auth_config: Option<OAuth2Config>,
    pub rate_limiting: Option<RateLimitConfig>,
    pub request_timeout: Duration,
    pub max_request_size: usize,
}
```

### 5. Simplified McpServer

```rust
/// Simplified MCP server that wraps a pre-configured transport
pub struct McpServer<T: ConfigurableTransport> {
    transport: T,
}

impl<T: ConfigurableTransport> McpServer<T> {
    /// Create a new MCP server with a pre-configured transport
    pub fn new(transport: T) -> Self {
        Self { transport }
    }
    
    /// Start the server (delegates to transport)
    pub async fn run(&mut self) -> McpResult<()> {
        self.transport.start().await.map_err(|e| {
            McpError::Integration(super::error::IntegrationError::Other {
                message: format!("Failed to start transport: {}", e),
            })
        })
    }
    
    // Convenience methods that delegate to transport configuration
    pub fn server_info(&self) -> Option<&ServerInfo> {
        self.transport.server_info()
    }
    
    pub fn capabilities(&self) -> ServerCapabilities {
        self.transport.effective_capabilities()
    }
}
```

## Rationale

### Why This Approach is Superior

1. **Single Responsibility**: 
   - `Transport` trait = Pure MCP protocol compliance
   - `TransportConfig` trait = Configuration management
   - `McpServer` = Simple transport wrapper

2. **No Handler Overwriting**: Transport comes pre-configured with its message handler

3. **Transport Specialization**: Each transport can have configuration optimized for its specific needs

4. **Type Safety**: Impossible to misconfigure transport-specific settings

5. **Clean Separation**: MCP core requirements are separate from transport-specific behavior

6. **Extensibility**: Easy to add new transports without affecting existing code

### Eliminated Problems

- ❌ **Handler Overwriting**: Transport is pre-configured, no overwriting
- ❌ **Mixed Responsibilities**: Clear separation between transport, config, and server
- ❌ **Generic Configuration**: Each transport has its own optimized config
- ❌ **Unused Logic**: Transport handles its own MCP processing
- ❌ **Architectural Confusion**: Clear ownership model

## Implementation Strategy

### Phase 1: Extract McpCoreConfig
- Create `McpCoreConfig` struct with universal MCP requirements
- Keep current `McpServerConfig` for backward compatibility

### Phase 2: Enhance Transport Trait
- Add `TransportConfig` trait with configuration methods
- Update existing transports to implement both traits

### Phase 3: Create Transport-Specific Configurations
- Implement `StdioTransportConfig`, `HttpTransportConfig`, etc.
- Each includes `McpCoreConfig` plus transport-specific settings

### Phase 4: Simplify McpServer
- Remove complex builder pattern and message handler management
- Make `McpServer` a simple wrapper around pre-configured transport

### Phase 5: Migration Support
- Provide adapters for existing code
- Deprecate old APIs with clear migration path

## Usage Examples

### STDIO Transport
```rust
let mcp_core = McpCoreConfig {
    server_info: ServerInfo::new("my-server", "1.0.0"),
    capabilities: ServerCapabilities::default(),
    protocol_version: ProtocolVersion::current(),
    instructions: Some("STDIO MCP server".to_string()),
};

let stdio_config = StdioTransportConfig {
    mcp_core,
    buffer_size: 8192,
    strict_validation: true,
    log_operations: false,
    log_file_path: Some("/tmp/mcp.log".into()),
};

let transport = StdioTransport::new_with_config(stdio_config)?;
let server = McpServer::new(transport);
server.run().await?;
```

### HTTP Transport
```rust
let mcp_core = McpCoreConfig {
    server_info: ServerInfo::new("my-server", "1.0.0"),
    capabilities: ServerCapabilities::default(),
    protocol_version: ProtocolVersion::current(),
    instructions: Some("HTTP MCP server".to_string()),
};

let http_config = HttpTransportConfig {
    mcp_core,
    cors_origins: vec!["https://myapp.com".to_string()],
    auth_config: Some(oauth2_config),
    rate_limiting: Some(rate_limit_config),
    request_timeout: Duration::from_secs(30),
    max_request_size: 1024 * 1024,
};

let transport = HttpTransport::new_with_config("0.0.0.0:8080".parse()?, http_config)?;
let server = McpServer::new(transport);
server.run().await?;
```

## Consequences

### Positive
- ✅ **Clean Architecture**: Clear separation of concerns
- ✅ **Type Safety**: Transport-specific configurations prevent misconfigurations  
- ✅ **No Handler Conflicts**: Pre-configured transports eliminate overwriting issues
- ✅ **Simplified API**: `McpServer` becomes much simpler to use
- ✅ **Transport Independence**: Each transport optimized for its use case
- ✅ **Backward Compatibility**: Migration path for existing code

### Negative
- ⚠️ **Breaking Changes**: Will require code updates for users of current API
- ⚠️ **Initial Complexity**: More types and traits to understand initially
- ⚠️ **Migration Effort**: Existing transports need to be updated

### Neutral
- 📝 **More Code**: Additional trait and config structures, but cleaner overall
- 📝 **Documentation**: Need to update all examples and documentation

## Alternatives Considered

### Alternative 1: Keep Current Architecture
**Rejected**: Fundamental design flaws cannot be fixed without architectural changes

### Alternative 2: Single Generic Configuration
**Rejected**: Different transports have fundamentally different configuration needs

### Alternative 3: Configuration in McpServer Only
**Rejected**: Violates principle that transport should be pre-configured

## Related Decisions

- **ADR-002**: Transport Role-Specific Architecture - This builds on the transport specialization principle
- **ADR-008**: MCP Protocol Architecture - This ensures MCP compliance is maintained
- **ADR-009**: Zero-Cost Generic Authorization - This architecture supports flexible authorization patterns

## References

- GitHub discussion thread: Integration Server Architecture Review
- User insight: "Transport should set their message handler right?"
- Current code analysis: `src/integration/server.rs` handler overwriting issues
- Examples analysis: Different usage patterns in STDIO vs HTTP transports

---

**Next Steps**: Implementation planning and timeline discussion with development team.
