# system_patterns.md

## Architecture Objectives
- Protocol-first design: 100% MCP spec compliance, built on JSON-RPC 2.0
- Type safety & memory safety: Rust type system, zero unsafe code, ownership-based resource management
- Async-native performance: Tokio-based async, sub-ms latency, high throughput
- Operational requirements: Structured logging, metrics, error handling, connection recovery, 24/7 stability
- **Single Responsibility Principle**: Each module focuses on exactly one responsibility (MANDATORY STANDARD)
- **Standards Compliance**: OAuth 2.1 + MCP protocol conformance patterns (see: `oauth2_rfc_specifications.md`, `mcp_official_specification.md`)
- **Zero-Cost Abstractions**: Eliminate abstractions that don't provide clear value (MANDATORY STANDARD per workspace §1)

## TransportBuilder Abstraction Analysis (TASK-033 ARCHITECTURAL DISCOVERY)

**CRITICAL FINDING**: TransportBuilder trait identified as over-abstraction violating workspace standards.

### Evidence of Over-Abstraction:
1. **Not Used in Practice**: Real examples bypass TransportBuilder trait entirely
2. **Abstraction Leakage**: Cannot hide transport-specific configuration differences 
3. **Violates Zero-Cost Principle**: Adds complexity without solving actual problems
4. **Pattern Inconsistency**: STDIO and HTTP use completely different construction patterns

### Current Reality vs. Intended Design:

**Intended Pattern (TransportBuilder trait):**
```rust
// ❌ Not actually used in practice
let transport = builder.with_message_handler(handler).build().await?;
```

**Actual Usage Patterns:**
```rust
// ✅ STDIO - Simple, consistent
let transport = StdioTransportBuilder::new()
    .with_message_handler(handler)
    .build().await?;

// ✅ HTTP - Complex, bypasses trait entirely  
let transport = HttpTransportBuilder::with_engine(engine)?
    .bind(addr)?.await?.build().await?;
```

### Architectural Decision:
**RECOMMENDATION**: Remove TransportBuilder trait, preserve individual builders.

**Rationale**:
- Each transport has evolved more sophisticated construction patterns than generic trait allows
- Transport-specific optimization is more valuable than forced consistency
- Eliminates unused abstraction per workspace "zero-cost abstractions" principle
- Allows each transport to optimize for its specific use case and constraints

**Migration Strategy**: Keep StdioTransportBuilder, HttpTransportBuilder<E> with their transport-specific convenience methods.

## Standards Compliance Architecture Pattern
**STANDARDS FIRST DESIGN**: All implementations must follow documented RFC and protocol specifications for interoperability and security.

**Reference Documentation**:
- **OAuth 2.1 Standards**: `oauth2_rfc_specifications.md` - Complete RFC implementation guides
- **MCP Protocol**: `mcp_official_specification.md` - Official specification with security requirements
- **Integration Patterns**: Documented convergence of OAuth 2.1 + MCP requirements for production deployment

**Compliance Requirements**:
- **RFC Adherence**: Mandatory compliance with documented OAuth 2.1 RFCs (9728, 7636, 8707, 6749)
- **MCP Specification**: 100% adherence to MCP 2025-06-18 specification requirements
- **Security Boundaries**: Client-host-server isolation, token audience validation, PKCE implementation
- **Transport Integration**: OAuth middleware patterns with HTTP Streamable transport compatibility

## HTTP Transport Architecture Patterns

### Zero-Dyn Generic Architecture (TASK-030)

**Pattern**: Eliminate all `Arc<dyn Trait>` patterns using associated types and generic constraints for zero-cost abstractions.

**Implementation**:
```rust
// Associated Types Pattern
trait HttpEngine {
    type Handler: McpRequestHandler;  // No dyn!
    fn register_mcp_handler(&mut self, handler: Self::Handler);
}

// Generic Transport  
struct HttpTransport<E: HttpEngine> {
    engine: E,  // Concrete type
}

// Generic Handler with Provider Types
struct AxumMcpRequestHandler<R, T, P, L> 
where R: ResourceProvider, T: ToolProvider, P: PromptProvider, L: LoggingHandler
{
    resource_provider: Option<R>,  // Concrete types
    tool_provider: Option<T>,
    prompt_provider: Option<P>, 
    logging_handler: Option<L>,
}
```

**Benefits**: Compile-time optimization, zero dynamic dispatch, type safety, workspace standards compliance.

### Direct MCP Integration Pattern 

**Pattern**: Eliminate JSON-RPC intermediary layer for direct HTTP → MCP processing.

**Current (Problematic)**:
```
HTTP Request → JSON-RPC Parse → mcp_operations.rs → MCP Response
```

**New (Direct)**:
```
HTTP Request → AxumMcpRequestHandler → MCP Response
```

**Implementation**: Single `handle_request()` method processes HTTP directly to MCP without serialization overhead.

### Engine-Layer Authentication Pattern

**Pattern**: Keep authentication/authorization at concrete engine implementations, not in generic abstractions.

**Structure**:
- **HttpEngine Trait**: Core lifecycle only (bind, start, shutdown)
- **AxumHttpServer**: OAuth2, custom auth via builder patterns
- **HttpTransportBuilder**: Delegates engine-specific configuration

**Usage**:
```rust
// OAuth2 engine configuration
let transport = HttpTransportBuilder::with_oauth2_engine(
    connection_manager, oauth2_adapter, auth_config
).await?.build().await?;

// Manual engine configuration
let server = AxumHttpServer::new(deps).await?
    .with_oauth2_authorization(adapter, config);
let transport = HttpTransportBuilder::with_custom_engine(server).build().await?;
```

### McpServer Integration Pattern

**Pattern**: Ensure HTTP transport implements `Transport` trait for high-level `McpServer<T>` wrapper compatibility.

**Application Flow**:
```rust
// 1. Configure providers and handlers
let handler = AxumMcpRequestHandlerBuilder::new()
    .with_resource_provider(provider).build();

// 2. Configure transport with auth
let mut transport = HttpTransportBuilder::with_oauth2_engine(...).build().await?;
transport.register_mcp_handler(handler);
transport.bind(addr).await?;

// 3. High-level server abstraction
let server = McpServer::new(transport);  // Transport trait
server.start().await?;  // Delegates to transport.start()
```

**Integration Requirements**: `HttpTransport<E>` implements `Transport`, proper error mapping, lifecycle delegation.

## Transport Architecture Patterns

### Generic MessageHandler Architecture Pattern
**ARCHITECTURAL ACHIEVEMENT**: Unified transport layer architecture using generic MessageHandler pattern for all transport types.

**Core Generic Pattern**:
```rust
// Generic MessageHandler trait for all transports
#[async_trait]
pub trait MessageHandler<T>: Send + Sync {
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<T>);
    async fn handle_error(&self, error: TransportError);
    async fn handle_close(&self);
}

// Generic MessageContext with transport-specific data
pub struct MessageContext<T = ()> {
    session_id: String,
    transport_data: T,
}
```

**Transport-Specific Module Organization**:
```
src/protocol/                    # Transport-agnostic core
├── message_handler.rs           # Generic MessageHandler<T> trait
├── message_context.rs           # Generic MessageContext<T>
└── transport.rs                 # Generic Transport trait

src/transport/adapters/
├── stdio/                       # STDIO-specific everything
│   ├── transport.rs             # StdioTransport implementation
│   ├── handlers.rs              # EchoHandler, LoggingHandler
│   └── mod.rs                   # type StdioMessageHandler = dyn MessageHandler<()>
│
├── http/                        # HTTP-specific everything  
│   ├── transport.rs             # HttpTransport implementation
│   ├── handlers.rs              # McpHttpHandler, StaticFileHandler
│   ├── context.rs               # HttpContext definition
│   └── mod.rs                   # type HttpMessageHandler = dyn MessageHandler<HttpContext>
│
└── websocket/                   # WebSocket-specific everything
    ├── transport.rs             # WebSocketTransport implementation
    ├── handlers.rs              # WebSocketEchoHandler
    └── context.rs               # WebSocketContext definition
```

**Event-Driven Flow Pattern**:
```
Transport receives data → Parses to JsonRpcMessage → Creates context → Calls MessageHandler
                                                                           ↓
                                            Handler processes and responds directly through transport
```

**Key Design Principles**:
- **No Bridge Patterns**: Direct handler-to-transport communication eliminates complex bridges
- **Transport Autonomy**: Each transport module contains all its specific implementations  
- **Handler Responsibility**: Handlers produce transport-specific responses (HTTP status codes, STDIO output)
- **Generic Type Safety**: Compile-time validation of transport context data
- **Engineer Freedom**: Engineers define custom context structures and performance strategies

**Reference**: ADR-012 - Generic MessageHandler Architecture for Transport Layer

## Authentication System Architecture ✅ COMPLETE (2025-09-02)

### Zero-Cost Authentication Abstraction Pattern
**ARCHITECTURAL ACHIEVEMENT**: Complete authentication system foundation with generic design, strategy pattern, and workspace standards compliance.

**Core Architecture Pattern**:
```rust
// Generic Authentication Manager with Strategy Pattern
AuthenticationManager<S, T, D>
where
    S: AuthenticationStrategy<T, D>,  // Strategy type (OAuth2Strategy, ApiKeyStrategy)
    T: Send + Sync,                   // Request type (HttpRequest, etc.)
    D: Send + Sync + 'static,         // Auth data type (OAuth2Data, ApiKeyData)
```

**Module Organization (Single Responsibility)**:
```rust
authentication/
├── mod.rs              // API coordination only
├── method.rs           // AuthMethod wrapper for extensible method identification
├── metadata.rs         // AuthMetadata HashMap wrapper with convenience methods
├── context.rs          // AuthContext<D> with timestamps and validation
├── error.rs            // AuthError with thiserror integration
├── request.rs          // AuthRequest<T> trait for abstraction
├── strategy.rs         // AuthenticationStrategy<T, D> async trait
└── manager.rs          // AuthenticationManager<S, T, D> with timeout support
```

**Strategy Pattern Implementation**:
```rust
#[async_trait]
pub trait AuthenticationStrategy<T, D>: Send + Sync + 'static {
    fn method(&self) -> AuthMethod;
    async fn authenticate(&self, request: &impl AuthRequest<T>) -> AuthResult<AuthContext<D>>;
    async fn validate(&self, context: &AuthContext<D>) -> AuthResult<bool>;
}
```

**Technical Excellence Achieved**:
- **Zero-Cost Abstractions**: Compile-time dispatch with generic type parameters
- **Strategy Pattern**: Extensible authentication methods without runtime overhead
- **Workspace Standards**: §2.1 import organization, §3.2 chrono DateTime<Utc>, zero warnings
- **Type Safety**: Generic parameters ensure compile-time correctness across auth strategies
- **Async Support**: Full async/await with timeout support and error handling

**HTTP Integration Foundation**:
```rust
// HTTP-specific request implementation
impl AuthRequest<HttpRequest> for HttpAuthRequest {
    fn custom_attribute(&self, key: &str) -> Option<String>;
    fn custom_attributes(&self) -> HashMap<String, String>;
}

// Engine integration with generic authentication
fn register_authentication<S, T, D>(
    &mut self,
    auth_manager: AuthenticationManager<S, T, D>,
) -> Result<(), HttpEngineError>;
```

**Standards Compliance**:
- **Error Handling**: `thiserror` integration for modern Rust error patterns
- **Import Organization**: 3-layer import structure (std → third-party → internal)
- **No FQN Usage**: Clean imports instead of `crate::` fully qualified names
- **Const Functions**: Strategic const constructors for performance where applicable

### Authentication Strategy Implementation Pattern
**NEXT PHASE**: OAuth2 and API Key strategy implementations following established architecture.

**Strategy Structure Template**:
```rust
authentication/strategies/
├── oauth2/
│   ├── mod.rs          // OAuth2 strategy exports
│   ├── strategy.rs     // OAuth2Strategy implementation
│   ├── data.rs         // OAuth2Data auth context
│   └── config.rs       // OAuth2 configuration
└── apikey/
    ├── mod.rs          // API Key strategy exports
    ├── strategy.rs     // ApiKeyStrategy implementation
    ├── data.rs         // ApiKeyData auth context
    └── config.rs       // API Key configuration
```

## HTTP Transport Adapter Pattern ✅ COMPLETE (2025-09-01)

### Transport Trait Adapter Architecture
**ARCHITECTURAL ACHIEVEMENT**: Complete implementation of adapter pattern bridging AxumHttpServer to Transport trait with session coordination.

**Implementation Pattern**:
```rust
// Adapter Pattern: HttpServerTransport bridges HTTP to Transport
McpServerBuilder -> HttpServerTransport -> AxumHttpServer -> HTTP Clients
                        (Adapter)           (Component)
```

**Session Coordination Architecture**:
```rust
pub struct HttpServerTransport {
    // Core HTTP server component integration
    axum_server: Option<AxumHttpServer>,
    
    // Phase 2: Session-aware message coordination
    incoming_requests: Arc<Mutex<mpsc::UnboundedReceiver<(SessionId, Vec<u8>)>>>,
    incoming_sender: mpsc::UnboundedSender<(SessionId, Vec<u8>)>,
    outgoing_responses: Arc<Mutex<HashMap<SessionId, oneshot::Sender<Vec<u8>>>>>,
    current_session: Option<SessionId>,
}
```

**Technical Benefits Achieved**:
- **Multi-Session Support**: Concurrent HTTP sessions with proper isolation and correlation
- **Transport Compliance**: Full Transport trait implementation enabling McpServerBuilder integration
- **Session Context**: HTTP request/response lifecycle properly mapped to Transport send/receive semantics
- **Production Ready**: 6/6 tests passing, zero warnings, full workspace standards compliance

**Integration Interfaces**:
```rust
// HTTP handlers coordinate with Transport trait through adapter
pub fn get_request_sender(&self) -> mpsc::UnboundedSender<(SessionId, Vec<u8>)>
pub async fn handle_http_request(&self, session_id: SessionId, request_data: Vec<u8>) -> Result<Vec<u8>, TransportError>
pub fn get_session_manager(&self) -> &Arc<SessionManager>
```

**Reference Documentation**: `architecture/phase2-session-coordination-implementation.md` - Complete implementation guide

## Single Responsibility Principle Standard (MANDATORY - 2025-08-14) ✅

### Module Organization Standard
**TECHNICAL STANDARD**: Every module must follow Single Responsibility Principle with clear boundaries and focused purpose.

**Implementation Requirements**:
- **One Purpose Per Module**: Each file has exactly one reason to change
- **Clear Separation of Concerns**: Implementation logic separated from organization logic  
- **Test Co-location**: Tests live with their implementations, not in coordinator modules
- **API Coordination**: `mod.rs` files focus purely on module organization and public API

**Established Pattern (HTTP Transport)**:
```rust
// mod.rs - API coordination ONLY
pub mod client;
pub mod server;
pub use client::HttpClientTransport;
pub use server::HttpServerTransport;

// client.rs - Client implementation ONLY
impl Transport for HttpClientTransport { ... }
#[cfg(test)] mod tests { /* client-specific tests */ }

// server.rs - Server implementation ONLY  
impl Transport for HttpServerTransport { ... }
#[cfg(test)] mod tests { /* server-specific tests */ }
```

**Benefits Demonstrated**:
- **Maintainability**: Clear module boundaries reduce cognitive load
- **Testability**: Focused tests eliminate redundancy and improve coverage clarity
- **Team Development**: Concurrent development enabled by clear separation
- **Code Quality**: Eliminated duplicate code and improved architectural clarity

### Role-Specific Transport Architecture ✅
**ARCHITECTURAL PATTERN**: HTTP transport demonstrates correct application of Single Responsibility through role-specific implementations.

**Implementation**:
- **HttpClientTransport**: Single responsibility = client-side HTTP communication
- **HttpServerTransport**: Single responsibility = server-side HTTP communication  
- **Module Organization**: Each transport in dedicated file with focused testing

**Technical Excellence Results**:
- **259 Unit Tests + 6 Integration Tests + 130 Doc Tests**: All passing
- **Zero Test Redundancy**: Eliminated duplicate coverage between modules
- **Clear Semantics**: Role-specific APIs eliminate confusion
- **Future-Ready**: Clean foundation for Phase 3 server features

## MCP Protocol Compliance Patterns (CRITICAL ARCHITECTURE)

### Field Naming Convention Compliance ✅ RESOLVED 2025-08-07
- **JSON Serialization Standard**: All compound fields must serialize to camelCase per MCP specification
- **Rust Implementation Pattern**: Use snake_case internally with `#[serde(rename = "camelCase")]` attributes
- **Specification Alignment**: Direct mapping to official MCP TypeScript schema definitions
- **Client Compatibility**: Ensures compatibility with Claude Desktop and other MCP clients

**Field Mapping Standards:**
```rust
// Protocol message fields requiring camelCase serialization
#[serde(rename = "protocolVersion")]  // initialization
#[serde(rename = "clientInfo")]       // initialization  
#[serde(rename = "serverInfo")]       // initialization
#[serde(rename = "mimeType")]         // resources
#[serde(rename = "uriTemplate")]      // resources
#[serde(rename = "nextCursor")]       // pagination (resources, tools, prompts)
#[serde(rename = "inputSchema")]      // tools
#[serde(rename = "isError")]          // tools
#[serde(rename = "progressToken")]    // tools
```

**Structural Compliance:**
- `display_name` → `title` (field renamed to match official MCP specification)
- All `title` fields are `Option<String>` per specification requirements
- Maintains Rust ergonomics with internal snake_case while ensuring JSON compatibility

### Protocol Message Architecture Patterns
- **JSON-RPC 2.0 Foundation**: Complete message type system with serialization/deserialization ✅
- **Correlation Manager**: Production-ready request/response correlation with DashMap, timeout management, background cleanup ✅
- **Message validation and error handling**: Structured error system with 6 error variants and context ✅
- **Advanced Concurrency**: Lock-free DashMap, oneshot channels, atomic operations, Arc shared ownership ✅
- **MCP Protocol Compliance**: Field naming consistency with official specification, camelCase JSON serialization ✅

## Data Flow Architecture (IMPLEMENTED)
- **Request Registration**: Unique ID generation → oneshot channel creation → DashMap storage ✅
- **Response Correlation**: ID lookup → channel notification → automatic cleanup ✅
- **Timeout Management**: Background task → expired request detection → timeout error delivery ✅
- **Graceful Shutdown**: Signal propagation → task cleanup → pending request cancellation ✅

## Correlation Manager Implementation Details ✅
- **Thread-Safe Access**: DashMap for lock-free concurrent operations
- **Background Processing**: Tokio spawn task with configurable cleanup intervals
- **Memory Safety**: Automatic cleanup prevents leaks, RAII patterns for resource management
- **Error Propagation**: Structured CorrelationError with context (ID, duration, details)
- **Configuration**: CorrelationConfig with timeout, capacity, interval, tracing controls
- **API Design**: 9 public methods covering all correlation scenarios with comprehensive documentation

## Transport Abstraction & Remote Server Architecture

### Current Transport Implementation ✅
- Transport trait for async send/receive/close operations
- STDIO transport: newline-delimited JSON, streaming parser, buffer management
- SubprocessTransport: Custom transport for client-server lifecycle management

### Remote Server Transport Architecture (PLANNED)
**HTTP Streamable Transport** - Critical Foundation:
```rust
#[async_trait]
pub trait McpTransport: Send + Sync {
    async fn start(&mut self) -> Result<(), TransportError>;
    async fn send(&self, message: JsonRpcMessage) -> Result<(), TransportError>;
    async fn receive(&self) -> Result<JsonRpcMessage, TransportError>;
    async fn close(&mut self) -> Result<(), TransportError>;
}

// Streamable HTTP implementation
pub struct StreamableHttpTransport {
    config: HttpTransportConfig,
    server: Arc<HttpServer>,
    sessions: Arc<RwLock<SessionManager>>,
    metrics: MetricsCollector,
}
```

**Session Management Architecture**:
```rust
// Session lifecycle management
pub struct SessionManager {
    sessions: HashMap<SessionId, Session>,
    cleanup_scheduler: CleanupScheduler,
    recovery_manager: RecoveryManager,
}

pub struct Session {
    id: SessionId,
    transport_context: TransportContext,
    security_context: Option<SecurityContext>,
    protocol_state: ProtocolState,
    capabilities: NegotiatedCapabilities,
    activity_tracker: ActivityTracker,
}
```

**OAuth 2.1 Security Architecture**:
```rust
// OAuth 2.1 + PKCE implementation
pub struct OAuth2Security {
    config: OAuth2Config,
    authorization_server: AuthorizationServerClient,
    token_manager: TokenManager,
    approval_workflow: ApprovalWorkflow,
}

// Human-in-the-loop approval
#[async_trait]
pub trait ApprovalHandler: Send + Sync {
    async fn request_approval(
        &self,
        operation: Operation,
        context: SecurityContext,
    ) -> Result<ApprovalDecision, ApprovalError>;
}
```

## Integration Architecture
- High-level JsonRpcClient interface: correlation manager, transport, message handler
- Message processing pipeline: parsing, routing, handler isolation

## Error Handling Architecture
- Structured error hierarchy: transport, correlation, parse, protocol errors
- Error context preservation: chaining, request/transport/timeout context

## Performance Architecture
- Zero-copy optimizations: Bytes type, buffer pools, streaming JSON
- Concurrent processing: request parallelism, non-blocking correlation, handler isolation, backpressure management
- Memory management: bounded queues, timeout cleanup, connection pooling, metric collection

## Security Standards & Compliance
- Security audit framework: static/dynamic analysis, compliance checking, vulnerability scanning
- Extensible analyzers and reporting
- Robust security practices and auditability

## Architectural Decision Records

### Core Design Decisions
- **[SOLID Principles Standard](decision_single_responsibility_principle_standard.md)** - Mandatory SRP implementation standard (2025-08-14)
- **[HTTP Transport Architecture](decision_http_transport_architecture.md)** - Role-specific transport layer design
- **[MCP Protocol Field Naming](decision_mcp_protocol_field_naming_compliance.md)** - camelCase JSON compliance standard

### Module Architecture Decisions  
- **[Axum Modular Architecture Refactor](decision_axum_modular_architecture_refactor.md)** - Strategic aliasing & facade elimination (2025-08-15)

### Implementation Decisions
- **[MCP Protocol Architecture](technical_decision_mcp_protocol_architecture.md)** - Protocol-first design approach
