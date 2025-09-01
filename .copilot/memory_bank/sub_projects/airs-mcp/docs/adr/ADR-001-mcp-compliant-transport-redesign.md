# ADR-001: MCP-Compliant Transport Redesign

**Status**: Proposed  
**Date**: 2025-09-01  
**Deciders**: Core Team  
**Technical Story**: [DEBT-001: HTTP Transport Trait Impedance Mismatch](../debts/DEBT-001-http-transport-trait-impedance-mismatch.md)

## Context

### Problem Statement
Our current Transport trait design is fundamentally misaligned with the official MCP specification, causing architectural complexity and limiting extensibility.

### Current Architecture Issues
1. **Sequential vs Event-Driven**: Our Transport trait uses blocking `receive()` calls, but MCP specification requires event-driven message handling
2. **Single Connection Assumption**: Transport trait assumes persistent single connection, but HTTP requires multi-session concurrent handling
3. **Manual Correlation**: We implemented oneshot channels and session tracking to force HTTP semantics into incompatible Transport interface
4. **Specification Non-Compliance**: Official TypeScript/Python SDKs use event-driven patterns that we don't support

### Research Findings

**Official MCP Specification Transport Interface:**
```typescript
interface Transport {
  // Event handlers for transport lifecycle
  onclose?: () => void;
  onerror?: (error: Error) => void;
  onmessage?: (message: JSONRPCMessage) => void;
  
  // Optional session identifier
  sessionId?: string;
  
  // Core transport methods
  start?(): Promise<void>;
  close(): Promise<void>;
  send(message: JSONRPCMessage | JSONRPCMessage[]): Promise<void>;
}
```

**Key Insights from Official SDKs:**
- Clear separation between "data layer" (JSON-RPC protocol) and "transport layer" (communication mechanisms)
- Event-driven message handling via callbacks
- Transport-specific session management
- Pluggable transport architecture
- No manual correlation mechanisms needed

## Decision

### Proposed Solution: MCP-Specification-Compliant Transport Redesign

Implement a complete Transport trait redesign that aligns with the official MCP specification:

```rust
// Core MCP message type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcMessage {
    pub jsonrpc: String,
    pub id: Option<JsonValue>,
    pub method: Option<String>,
    pub params: Option<JsonValue>,
    pub result: Option<JsonValue>,
    pub error: Option<JsonRpcError>,
}

// Event-driven message handler (separation of concerns)
#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext);
    async fn handle_error(&self, error: TransportError);
    async fn handle_close(&self);
}

// MCP-compliant Transport trait
#[async_trait]
pub trait Transport: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    // Lifecycle Management (MCP Spec)
    async fn start(&mut self) -> Result<(), Self::Error>;
    async fn close(&mut self) -> Result<(), Self::Error>;
    
    // Message Exchange (MCP Spec)
    async fn send(&mut self, message: JsonRpcMessage) -> Result<(), Self::Error>;
    
    // Event Handling (MCP Spec)
    fn set_message_handler(&mut self, handler: Arc<dyn MessageHandler>);
    
    // Session Support
    fn session_id(&self) -> Option<String>;
    fn set_session_context(&mut self, session_id: Option<String>);
    
    // Transport State
    fn is_connected(&self) -> bool;
    fn transport_type(&self) -> &'static str;
}
```

### Architecture Benefits

1. **Event-Driven Messaging**: Natural MCP semantics, no artificial correlation
2. **Clean Separation**: Transport handles delivery, MessageHandler handles protocol
3. **HTTP Transport Simplification**: Eliminates oneshot channels and session tracking complexity
4. **Specification Compliance**: Matches official TypeScript/Python SDK patterns
5. **Extensibility**: Easy to add WebSocket, SSE, custom transports
6. **Testability**: Transport and protocol logic can be tested independently

### Migration Strategy

**Phase 1: Foundation (Week 1)**
- Implement new MCP-compliant Transport trait
- Create JsonRpcMessage and MessageHandler types  
- Design MessageContext for session/metadata handling
- Build HttpEngine trait abstraction for pluggable HTTP frameworks

**Phase 2: Core Components & HTTP Engines (Week 1-2)**
- Implement transport trait with lifecycle management (start/close)
- Add event-driven message handling via MessageHandler callbacks
- Create AxumHttpEngine, RocketHttpEngine, WarpHttpEngine implementations
- Implement pluggable HTTP engine architecture with response modes (JSON, SSE, streaming)

**Phase 3: Legacy Compatibility (Week 2)**
- Create LegacyTransportAdapter for existing StdioTransport
- Implement event loop to convert blocking receive() to message events
- Ensure backward compatibility with existing stdio-based examples
- Test adapter with current McpServerBuilder integration

**Phase 4: HTTP Transport Redesign (Week 2-3)**
- Complete rewrite of HttpServerTransport using pluggable engine pattern
- Eliminate oneshot channels and manual correlation mechanisms
- Implement natural HTTP request/response flow with message events
- Add proper session context management for concurrent HTTP requests
- Integrate OAuth2 middleware with all HTTP engines (Axum, Rocket, Warp)

**Phase 5: Protocol Layer & OAuth Integration (Week 3)**
- Implement McpServer as MessageHandler for protocol logic
- Update McpServerBuilder to work with new Transport interface
- Enhance MessageContext with OAuth AuthContext integration
- Add protocol-level authorization checking with scope mappings
- Maintain backward compatibility during transition period

**Phase 6: Testing and Validation (Week 3-4)**
- Comprehensive unit tests for new Transport trait implementations
- Integration tests for HTTP engines (Axum, Rocket, Warp) with OAuth
- Performance validation comparing old vs new architecture
- Stress testing for concurrent HTTP sessions with different engines
- Security testing for session isolation and OAuth integration

**Phase 7: Migration and Documentation (Week 4)**
- Create migration guides for existing Transport implementations
- Update all examples to demonstrate pluggable HTTP engines
- Comprehensive documentation for MessageHandler and HttpEngine patterns
- OAuth integration examples for all supported HTTP frameworks
- Performance benchmarks and comparison with old implementation

**Phase 8: Cleanup and Optimization (Week 4)**
- Remove deprecated Transport trait and compatibility adapters
- Performance optimization based on testing results with different engines
- Final security review and code audit
- Documentation review and developer guide updates
- Preparation for future transport implementations (WebSocket, custom engines)

## Consequences

### Positive
- **Simplified HTTP Transport**: Eliminates architectural complexity and correlation mechanisms
- **MCP Compliance**: Aligns with official specification and SDK patterns
- **Better Developer Experience**: Intuitive, matches MCP documentation
- **Extensibility**: Foundation for WebSocket, SSE, and custom transports
- **Performance**: No artificial correlation overhead
- **Maintainability**: Clear separation of concerns

### Negative
- **Breaking Changes**: Existing Transport implementations need migration
- **Development Time**: 3-4 weeks of focused development effort
- **Complexity During Migration**: Temporary dual interfaces during transition
- **Testing Overhead**: Comprehensive testing required for new architecture

### Mitigation Strategies
- **Compatibility Layer**: Adapters during migration period
- **Incremental Migration**: Phase-by-phase implementation with fallbacks
- **Comprehensive Testing**: Unit, integration, and performance tests
- **Documentation**: Clear migration guides and examples

## Implementation Details

### Extended AuthContext Strategy

**Current State**: Our existing `AuthContext` in `src/oauth2/context.rs` is well-designed for OAuth2 but needs evolution to support multiple authentication methods as required by the MCP specification.

**Evolution Approach**: Extend existing `AuthContext` to support OAuth, API keys, and username/password combinations while maintaining 100% backward compatibility.

```rust
// Extended AuthContext (maintains backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub user_id: String,                          // Unified user identifier
    pub auth_method: AuthMethod,                  // Authentication method used
    pub scopes: Vec<String>,                      // Normalized permissions
    pub expires_at: Option<DateTime<Utc>>,        // Existing field
    pub created_at: DateTime<Utc>,                // Existing field
    pub request_id: Option<String>,               // Existing field
    pub metadata: AuthMetadata,                   // Existing field
    pub auth_data: AuthData,                      // Method-specific data
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    OAuth2 { provider: String, token_type: String },
    ApiKey { key_type: String, scope: Option<String> },
    BasicAuth { realm: Option<String> },
    Custom { scheme: String, version: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthData {
    OAuth2(JwtClaims),                                    // Existing OAuth2 claims
    ApiKey { key_id: String, permissions: Vec<String> }, // API key data
    BasicAuth { username: String, groups: Vec<String> }, // Basic auth data
    Custom(HashMap<String, serde_json::Value>),          // Custom schemes
}
```

**Backward Compatibility**: All existing OAuth2 functionality preserved through dedicated constructor and accessor methods.

### Authentication Strategy Pattern Integration

```rust
#[async_trait]
pub trait AuthenticationStrategy: Send + Sync {
    async fn authenticate(&self, request: &AuthenticationRequest) -> Result<AuthContext, AuthError>;
    fn extract_from_request(&self, headers: &HeaderMap, body: Option<&[u8]>) -> Result<AuthenticationRequest, AuthError>;
    fn strategy_name(&self) -> &'static str;
}

pub struct AuthenticationManager {
    strategies: HashMap<String, Box<dyn AuthenticationStrategy>>,
    strategy_order: Vec<String>, // Try in order for fallback support
}
```

### HTTP Engine Integration

Enhanced HTTP engines to support multiple authentication strategies:

```rust
#[async_trait]
pub trait HttpEngine: Send + Sync {
    // ... existing methods ...
    
    /// Register authentication manager (replaces single OAuth2 config)
    fn register_authentication(&mut self, auth_manager: AuthenticationManager) -> Result<(), Self::Error>;
}
```

### HTTP Engine Abstraction Strategy

**Pluggable HTTP Engine Pattern**: Support multiple HTTP frameworks through abstraction

```rust
#[async_trait]
pub trait HttpEngine: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    type Config: Clone + Send + Sync;
    
    // Lifecycle management
    async fn bind(&mut self, addr: SocketAddr) -> Result<(), Self::Error>;
    async fn start(&mut self) -> Result<(), Self::Error>;
    async fn shutdown(&mut self) -> Result<(), Self::Error>;
    
    // MCP request handling
    fn register_mcp_handler(&mut self, handler: Arc<dyn McpRequestHandler>);
    fn register_oauth_middleware(&mut self, oauth_config: OAuth2Config) -> Result<(), Self::Error>;
    
    // Server state
    fn is_bound(&self) -> bool;
    fn local_addr(&self) -> Option<SocketAddr>;
    fn engine_type(&self) -> &'static str;
}

// Engine implementations
pub struct AxumHttpEngine { /* Axum-specific implementation */ }
pub struct RocketHttpEngine { /* Rocket-specific implementation */ }
pub struct WarpHttpEngine { /* Warp-specific implementation */ }
```

**Benefits:**
- Framework choice flexibility for teams
- Performance optimization based on engine selection
- Migration capability between HTTP frameworks
- Consistent Transport interface regardless of engine

### Module Structure Redesign

```
src/
├── transport/
│   ├── core/                           # Transport abstractions
│   │   ├── traits.rs                   # Transport, MessageHandler traits
│   │   ├── message.rs                  # JsonRpcMessage, MessageContext
│   │   └── handler.rs                  # Event-driven message handling
│   │
│   ├── implementations/
│   │   ├── http/                       # HTTP Transport (Unified)
│   │   │   ├── transport.rs            # HttpServerTransport<E: HttpEngine>
│   │   │   ├── session.rs              # HTTP session management
│   │   │   │
│   │   │   ├── engines/                # Pluggable HTTP Engines
│   │   │   │   ├── mod.rs               # HttpEngine trait
│   │   │   │   ├── axum/               # Axum engine (default)
│   │   │   │   ├── rocket/             # Rocket engine (alternative)
│   │   │   │   └── warp/               # Warp engine (alternative)
│   │   │   │
│   │   │   ├── responses/              # HTTP Response Modes
│   │   │   │   ├── json.rs             # JSON responses
│   │   │   │   ├── sse.rs              # Server-Sent Events responses
│   │   │   │   └── streaming.rs        # Streaming responses
│   │   │   │
│   │   │   └── middleware/             # HTTP middleware
│   │   │       ├── cors.rs
│   │   │       ├── auth.rs
│   │   │       └── logging.rs
│   │   │
│   │   ├── stdio/                      # STDIO transport
│   │   └── websocket/                  # WebSocket transport (future)
│   │
│   └── legacy/                         # Backward compatibility
│       ├── adapter.rs                  # LegacyTransportAdapter
│       └── migration.rs                # Migration utilities
│
├── protocol/                           # MCP Protocol Logic (NEW)
│   ├── core/
│   │   ├── server.rs                   # McpServer (MessageHandler impl)
│   │   ├── client.rs                   # McpClient
│   │   └── builder.rs                  # McpServerBuilder (updated)
│   │
│   ├── handlers/                       # MCP method handlers
│   │   ├── tools.rs                    # tools/list, tools/call handlers
│   │   ├── resources.rs                # resources/list, resources/read handlers
│   │   └── prompts.rs                  # prompts/list, prompts/get handlers
│   │
│   └── types/                          # MCP-specific types
│       ├── requests.rs                 # MCP request types
│       ├── responses.rs                # MCP response types
│       └── errors.rs                   # MCP-specific errors
│
└── oauth2/                             # Multi-Method Authentication (Enhanced)
    ├── strategies/                     # Authentication strategies (NEW)
    │   ├── mod.rs                      # Strategy trait and manager
    │   ├── oauth2.rs                   # OAuth2/JWT strategy (existing logic)
    │   ├── api_key.rs                  # API key authentication strategy
    │   ├── basic_auth.rs               # Username/password authentication
    │   └── custom.rs                   # Custom authentication schemes
    ├── stores/                         # Authentication data stores (NEW)
    │   ├── mod.rs                      # Store traits
    │   ├── api_key.rs                  # API key storage interface
    │   └── user.rs                     # User storage interface
    ├── middleware/                     # HTTP framework middleware
    │   ├── axum.rs                     # Axum auth integration
    │   ├── rocket.rs                   # Rocket auth integration (NEW)
    │   └── warp.rs                     # Warp auth integration (NEW)
    └── context.rs                      # Extended AuthContext (multi-method support)
```

### HTTP Transport Example
```rust
impl Transport for HttpServerTransport {
    async fn send(&mut self, message: JsonRpcMessage) -> Result<(), Self::Error> {
        // Send to current session - no correlation needed!
        if let Some(session_id) = &self.current_session {
            self.send_to_session(session_id, message).await?;
        }
        Ok(())
    }
    
    // HTTP requests naturally trigger message events
    async fn handle_http_request(&self, session_id: String, request_data: Vec<u8>) {
        let message = parse_jsonrpc(request_data)?;
        
        self.set_session_context(Some(session_id));
        if let Some(handler) = &self.message_handler {
            handler.handle_message(message, context).await;
        }
    }
}
```

### McpServer as MessageHandler
```rust
impl MessageHandler for McpServer {
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext) {
        match message.method.as_deref() {
            Some("tools/call") => {
                let result = self.execute_tool(&message).await;
                let response = create_response(message.id, result);
                
                let mut transport = self.transport.lock().await;
                transport.set_session_context(context.session_id);
                transport.send(response).await.unwrap();
            }
            _ => { /* handle other methods */ }
        }
    }
}
```

## Related Decisions
- [DEBT-001: HTTP Transport Trait Impedance Mismatch](../debts/DEBT-001-http-transport-trait-impedance-mismatch.md)
- Future: WebSocket Transport Implementation
- Future: Server-Sent Events Transport Implementation

## Review Schedule
- **Initial Review**: 2025-09-02
- **Implementation Review**: After Phase 1 completion
- **Final Review**: After full migration completion
- **Post-Implementation Review**: 3 months after deployment

## References
- [MCP Official Specification](https://modelcontextprotocol.io/docs)
- [TypeScript SDK Transport Interface](https://github.com/modelcontextprotocol/typescript-sdk)
- [Python SDK Transport Patterns](https://github.com/modelcontextprotocol/python-sdk)
- [DEBT-001: HTTP Transport Trait Impedance Mismatch](../debts/DEBT-001-http-transport-trait-impedance-mismatch.md)
