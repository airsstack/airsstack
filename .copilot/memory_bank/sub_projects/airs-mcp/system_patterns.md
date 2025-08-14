# system_patterns.md

## Architecture Objectives
- Protocol-first design: 100% MCP spec compliance, built on JSON-RPC 2.0
- Type safety & memory safety: Rust type system, zero unsafe code, ownership-based resource management
- Async-native performance: Tokio-based async, sub-ms latency, high throughput
- Operational requirements: Structured logging, metrics, error handling, connection recovery, 24/7 stability
- **Single Responsibility Principle**: Each module focuses on exactly one responsibility (MANDATORY STANDARD)

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
