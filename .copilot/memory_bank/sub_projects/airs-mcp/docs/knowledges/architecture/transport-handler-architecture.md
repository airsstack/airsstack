# Transport Handler Architecture - Generic MessageHandler Pattern

**Category**: Architecture / Transport Layer  
**Created**: 2025-09-10  
**Status**: Finalized  
**Impact**: High - Fundamental transport architecture design

## Overview

This document captures the architectural discovery and design decisions for implementing a unified, generic transport handler pattern across all transport types in the AIRS MCP system.

## Architectural Discovery

### Problem Analysis
- **Initial Challenge**: How to integrate HTTP transport objects with existing transport architecture
- **Root Issue**: Confusion about correlation between HttpTransport and HttpEngine objects
- **Existing Solution Discovery**: STDIO transport already implemented elegant event-driven pattern

### Key Insight
The STDIO transport implementation revealed that complex bridge patterns are unnecessary. Each transport can use the same event-driven `MessageHandler` pattern with transport-specific context data.

## Architectural Design

### 1. Generic MessageHandler Pattern

```rust
#[async_trait]
pub trait MessageHandler<T>: Send + Sync {
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<T>);
    async fn handle_error(&self, error: TransportError);
    async fn handle_close(&self);
}
```

**Key Benefits**:
- **Type Safety**: Generic ensures compile-time validation of context data
- **Transport Flexibility**: Each transport defines its own context structure
- **No Enum Maintenance**: No central `TransportData` enum to extend

### 2. Generic MessageContext

```rust
pub struct MessageContext<T = ()> {
    session_id: String,
    transport_data: T,
}

impl<T> MessageContext<T> {
    pub fn new(session_id: String, transport_data: T) -> Self {
        Self { session_id, transport_data }
    }

    pub fn transport_data(&self) -> &T {
        &self.transport_data
    }
}
```

**Design Principles**:
- **Engineer Freedom**: Engineers define their own context structures
- **Zero Overhead**: Generic compiles to concrete types
- **Helper Methods**: Simple access to transport-specific data

### 3. Transport-Specific Module Organization

```
src/
├── protocol/                    # Transport-agnostic core
│   ├── message_handler.rs       # Generic MessageHandler<T> trait
│   ├── message_context.rs       # Generic MessageContext<T>
│   └── transport.rs             # Generic Transport trait
│
├── transport/adapters/
│   ├── stdio/                   # STDIO-specific everything
│   │   ├── transport.rs         # StdioTransport implementation
│   │   ├── handlers.rs          # EchoHandler, LoggingHandler, etc.
│   │   └── mod.rs
│   │
│   ├── http/                    # HTTP-specific everything
│   │   ├── transport.rs         # HttpTransport implementation
│   │   ├── handlers.rs          # McpHttpHandler, StaticFileHandler, etc.
│   │   ├── context.rs           # HttpContext definition
│   │   └── mod.rs
│   │
│   └── websocket/               # WebSocket-specific everything
│       ├── transport.rs         # WebSocketTransport implementation
│       ├── handlers.rs          # WebSocketEchoHandler, etc.
│       └── context.rs           # WebSocketContext definition
```

**Organizational Principles**:
- **Self-Contained Modules**: Each transport contains all its specific implementations
- **Clean Core**: Protocol module remains transport-agnostic
- **No Cross-Dependencies**: Transport modules don't know about each other

## Implementation Patterns

### 1. STDIO Transport Pattern (Reference Implementation)

```rust
// Type alias for clarity
pub type StdioMessageHandler = dyn MessageHandler<()>;

// Handler implementation
pub struct EchoHandler;

#[async_trait]
impl MessageHandler<()> for EchoHandler {
    async fn handle_message(&self, message: JsonRpcMessage, _context: MessageContext<()>) {
        println!("Echo: {:?}", message);
    }
    // ... other methods
}

// Usage
let handler = Arc::new(EchoHandler);
let transport = StdioTransportBuilder::new()
    .with_message_handler(handler)
    .build()
    .await?;
```

### 2. HTTP Transport Pattern (Planned Implementation)

```rust
// HTTP-specific context (engineer-defined)
pub struct HttpContext {
    pub method: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub remote_addr: Option<String>,
    pub request_id: String,
}

// Type alias for clarity
pub type HttpMessageHandler = dyn MessageHandler<HttpContext>;

// Handler implementation
pub struct McpHttpHandler;

#[async_trait]
impl MessageHandler<HttpContext> for McpHttpHandler {
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<HttpContext>) {
        let http_ctx = context.transport_data();
        
        // Handler responsible for HTTP response
        let response = process_mcp_message(message).await;
        send_http_response(http_ctx.request_id, response).await;
    }
    // ... other methods
}
```

### 3. Event-Driven Flow Pattern

```
Transport receives data → Parses to JsonRpcMessage → Creates context → Calls MessageHandler
                                                                              ↓
                                                         Handler processes and responds directly
```

**Key Characteristics**:
- **No Response Waiting**: Handlers do their work immediately
- **Direct Response**: Handlers responsible for transport-specific responses
- **Error Handling**: Handlers produce transport-specific error responses
- **Performance**: Engineers choose their own optimization strategies

## Design Decisions

### Decision 1: Generic MessageHandler vs Enum TransportData
**Decision**: Use generic `MessageHandler<T>` trait  
**Rationale**: 
- Eliminates enum maintenance burden
- Provides compile-time type safety
- Gives engineers complete freedom in context design
- No central coordination required for new transports

### Decision 2: Handler Scope and Responsibility
**Decision**: Handlers responsible for transport-specific responses  
**Rationale**:
- HTTP handlers send HTTP responses directly
- STDIO handlers print to stdout directly
- Eliminates complex response coordination
- Simplifies transport implementation

### Decision 3: Module Organization
**Decision**: Transport-specific handlers in transport modules  
**Rationale**:
- Protocol module remains transport-agnostic
- Self-contained transport modules
- Clear separation of concerns
- Easy extension without core modifications

### Decision 4: Performance Strategy
**Decision**: Leave performance decisions to engineers  
**Rationale**:
- Handler pooling optional (engineer's choice)
- HTTP server framework choice (engineer's choice)
- Optimization strategies (engineer's choice)
- Flexible architecture supports all approaches

## Technical Benefits

### 1. Architectural Consistency
- Same pattern across all transports (STDIO, HTTP, WebSocket)
- Consistent event-driven flow
- Uniform error handling approach

### 2. Implementation Simplicity
- No complex bridge patterns needed
- No event loops or response coordination
- Direct handler-to-transport communication

### 3. Engineer Freedom
- Define custom context structures
- Choose HTTP frameworks
- Implement custom performance optimizations
- Add new transports without core changes

### 4. Type Safety
- Compile-time validation of context data
- No runtime type casting
- Clear API contracts

## Integration with Existing Architecture

### ADR-011 Compliance
- Maintains pre-configured transport pattern
- No dangerous `set_message_handler()` calls
- Transport builders create fully configured transports

### Workspace Standards Compliance
- 3-layer import organization in all modules
- chrono DateTime<Utc> standard for time operations
- Module architecture patterns followed
- Zero warning policy maintained

## Future Extensions

### New Transport Types
1. **WebSocket Transport**:
   - `WebSocketContext` with connection info
   - `WebSocketMessageHandler` implementations
   - Self-contained in `websocket/` module

2. **gRPC Transport**:
   - `GrpcContext` with metadata and streaming info
   - `GrpcMessageHandler` implementations
   - Self-contained in `grpc/` module

3. **Custom Transports**:
   - Engineers define their own context structures
   - Implement `MessageHandler<CustomContext>`
   - Follow same modular organization

### Performance Enhancements
- Handler pooling (optional, engineer-implemented)
- Async handler factories
- Connection pooling
- Load balancing

## Validation

### Proven Patterns
- **STDIO Implementation**: Already working and tested
- **Event-Driven Flow**: Proven in production systems
- **Generic Traits**: Standard Rust pattern for flexibility

### Testing Strategy
- Unit tests for each handler implementation
- Integration tests for transport-handler coordination
- Performance benchmarks for high-throughput scenarios

## Conclusion

This generic MessageHandler architecture provides a clean, consistent, and flexible foundation for all transport types while maintaining the simplicity and elegance demonstrated by the STDIO implementation. It eliminates complex bridge patterns while giving engineers maximum freedom in implementation choices.
