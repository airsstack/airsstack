# ADR-012: Generic MessageHandler Architecture for Transport Layer

**Status**: Accepted  
**Date**: 2025-09-10  
**Deciders**: Engineering Team  
**Technical Story**: Unified transport handler architecture design

## Context

During HTTP transport implementation, confusion arose about the correlation between HttpTransport and HttpEngine objects. Investigation revealed that the existing STDIO transport already implemented an elegant event-driven pattern that could be generalized across all transport types.

### Problem Statement
- Need consistent architecture across all transport types (STDIO, HTTP, WebSocket, etc.)
- Avoid complex bridge patterns between transport protocols and MCP message handling
- Eliminate enum maintenance burden for transport-specific context data
- Provide maximum flexibility for engineers implementing transport-specific handlers

### Existing Solution Discovery
The STDIO transport demonstrated a clean pattern:
```
Transport receives data → Parses to JsonRpcMessage → Calls MessageHandler → Handler responds directly
```

This eliminated the need for complex HttpToMcpBridge patterns or event-driven response coordination.

## Decision

We will implement a **Generic MessageHandler Architecture** with the following design:

### 1. Generic MessageHandler Trait
```rust
#[async_trait]
pub trait MessageHandler<T>: Send + Sync {
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<T>);
    async fn handle_error(&self, error: TransportError);
    async fn handle_close(&self);
}
```

### 2. Generic MessageContext
```rust
pub struct MessageContext<T = ()> {
    session_id: String,
    transport_data: T,
}
```

### 3. Transport-Specific Module Organization
- Each transport module (stdio/, http/, websocket/) contains all its specific implementations
- Protocol module remains transport-agnostic with only generic traits
- No cross-dependencies between transport modules

### 4. Handler Responsibility Scope
- Handlers are responsible for transport-specific responses (HTTP status codes, STDIO output, WebSocket frames)
- No response waiting or coordination - handlers act immediately
- Error handling produces transport-appropriate error responses

## Alternatives Considered

### Alternative 1: Enum-Based TransportData
```rust
pub enum TransportData {
    Stdio,
    Http(HttpContext),
    WebSocket(WebSocketContext),
}
```

**Rejected because**:
- Requires central enum maintenance for every new transport
- Creates coupling between transport types
- Forces all transports to be known at compile time

### Alternative 2: Complex Bridge Patterns
```rust
pub struct HttpToMcpBridge {
    // Event-driven bridge between HTTP and MCP protocols
}
```

**Rejected because**:
- STDIO implementation proves bridges are unnecessary
- Adds complexity without benefits
- Creates additional abstraction layers

### Alternative 3: Trait Objects with Any
```rust
pub trait MessageHandler: Send + Sync {
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<Box<dyn Any>>);
}
```

**Rejected because**:
- Loses compile-time type safety
- Requires runtime type casting
- Less clear API contracts

## Consequences

### Positive
- **Consistency**: Same pattern across all transport types
- **Simplicity**: No complex bridge patterns or event coordination
- **Flexibility**: Engineers define their own context structures and performance strategies
- **Type Safety**: Compile-time validation of context data
- **Extensibility**: New transports added without modifying core protocol
- **Self-Contained**: Each transport module contains all its specific implementations

### Negative
- **Generic Complexity**: Some engineers may find generics more complex than concrete types
- **Code Duplication**: Similar patterns repeated across transport modules (acceptable trade-off for flexibility)

### Neutral
- **Learning Curve**: Engineers need to understand generic pattern (standard Rust practice)
- **Performance**: Performance decisions delegated to engineers (intentional design choice)

## Implementation Plan

### Phase 1: Core Generic Types
- [ ] Update `MessageContext<T>` to be generic
- [ ] Update `MessageHandler<T>` trait to be generic  
- [ ] Update STDIO transport to use `MessageHandler<()>`
- [ ] Verify STDIO functionality with generic pattern

### Phase 2: HTTP Transport Foundation
- [ ] Define `HttpContext` structure in `http/context.rs`
- [ ] Implement `HttpTransport` with `MessageHandler<HttpContext>`
- [ ] Create `HttpTransportBuilder` following ADR-011 pattern
- [ ] Implement basic HTTP request parsing and handler dispatch

### Phase 3: HTTP Handler Examples
- [ ] `McpHttpHandler` - MCP protocol over HTTP
- [ ] `StaticFileHandler` - Static file serving example
- [ ] `EchoHttpHandler` - Simple request/response echo

### Phase 4: Documentation and Examples
- [ ] Complete documentation with usage examples
- [ ] Integration tests demonstrating pattern
- [ ] Performance benchmarks for validation

## Compliance

### ADR-011 Integration
This decision maintains the pre-configured transport pattern established in ADR-011:
- Transport builders create fully configured transports
- No dangerous `set_message_handler()` calls
- Clean separation between transport and handler lifecycle

### Workspace Standards
- 3-layer import organization in all modules
- chrono DateTime<Utc> standard maintained
- Module architecture patterns followed
- Zero warning policy compliance

## Review

This decision should be reviewed when:
- Performance issues arise that cannot be solved within current pattern
- New transport types require capabilities not supported by generic pattern
- Maintenance burden of generic pattern exceeds benefits
- Alternative patterns emerge that provide significant advantages

**Next Review Date**: 2025-12-10 (3 months)

## References

- [ADR-011: Pre-configured Transport Pattern](./ADR-011-pre-configured-transport.md)
- [STDIO Transport Implementation](../../../src/transport/adapters/stdio/transport.rs)
- [Transport Handler Architecture Knowledge Doc](../knowledges/architecture/transport-handler-architecture.md)
