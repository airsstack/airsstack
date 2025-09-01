# KNOWLEDGE-004: Transport Adapter Architecture Plan

**Type**: Architecture Design  
**Status**: Approved  
**Created**: 2025-09-01  
**Updated**: 2025-09-01  
**Context**: TASK-005 MCP-Compliant Transport Architecture Refactoring

## Overview

Comprehensive architectural plan for reorganizing the transport layer to achieve MCP specification compliance while maintaining complete backward compatibility through the adapter pattern.

## Problem Statement

### Current Transport Architecture Issues
1. **Impedance Mismatch**: Current Transport trait uses blocking `receive()` patterns; MCP specification uses event-driven callbacks
2. **Artificial Correlation**: HTTP transport uses oneshot channels for request/response correlation; MCP uses natural JSON-RPC message IDs
3. **Mixed Concerns**: Transport and protocol logic are entangled instead of separated
4. **Inconsistent Patterns**: Different transport implementations use different paradigms

### Strategic Goals
- **MCP Specification Compliance**: Align with official TypeScript/Python SDK patterns
- **Backward Compatibility**: Existing code continues to work without changes
- **Clean Architecture**: Clear separation between specification compliance and legacy compatibility
- **Future Extensibility**: Enable pure MCP implementations alongside legacy adapters

## Target Architecture

### File Structure Organization

```
crates/airs-mcp/src/transport/
├── mod.rs                    # [UPDATED] Clean re-exports with compatibility layer
├── mcp/                      # [EXISTS] Pure MCP-specification compliant types
│   ├── mod.rs               # MCP interface re-exports and documentation
│   ├── message.rs           # JsonRpcMessage aligned with official specification
│   ├── transport.rs         # Transport + MessageHandler traits (event-driven)
│   ├── context.rs           # MessageContext for session/metadata management
│   ├── error.rs             # TransportError enum with proper categorization
│   └── compat.rs            # Legacy compatibility bridges and conversion utilities
└── adapters/                 # [NEW] All legacy transport implementations
    ├── mod.rs               # Adapter module re-exports
    ├── stdio.rs             # StdioTransport → MCP Transport adapter
    └── http/                # [MOVED] Entire HTTP implementation as adapter
        ├── mod.rs           # HTTP adapter exports
        ├── server.rs        # HttpServerTransport adapter wrapper
        ├── client.rs        # HttpClientTransport adapter wrapper
        ├── config.rs        # HTTP configuration (unchanged)
        ├── buffer_pool.rs   # HTTP-specific optimizations (unchanged)
        ├── connection_manager.rs # HTTP connection management (unchanged)
        ├── parser.rs        # HTTP request parsing (unchanged)
        ├── session.rs       # HTTP session management (unchanged)
        ├── axum/            # Axum integration (unchanged)
        └── sse/             # Server-Sent Events (unchanged)
```

### Architectural Layers

#### 1. Pure MCP Layer (`transport/mcp/`)
**Purpose**: Specification-compliant, event-driven transport interfaces

**Characteristics**:
- **Event-Driven**: Uses `MessageHandler` callbacks, no blocking operations
- **Specification Aligned**: Matches official MCP TypeScript/Python SDK patterns exactly
- **Clean Abstractions**: Transport handles delivery, MessageHandler handles protocol logic
- **Future-Proof**: No legacy dependencies or architectural compromises

**Key Types**:
```rust
#[async_trait]
pub trait Transport: Send + Sync {
    async fn start(&mut self) -> Result<(), Self::Error>;
    async fn close(&mut self) -> Result<(), Self::Error>;
    async fn send(&mut self, message: JsonRpcMessage) -> Result<(), Self::Error>;
    fn set_message_handler(&mut self, handler: Arc<dyn MessageHandler>);
    // Session and state management methods...
}

#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext);
    async fn handle_error(&self, error: TransportError);
    async fn handle_close(&self);
}
```

#### 2. Adapter Layer (`transport/adapters/`)
**Purpose**: Bridge legacy transport implementations to MCP interfaces

**Characteristics**:
- **Compatibility Bridge**: Makes existing transports work with new MCP interfaces
- **Event Loop Pattern**: Converts blocking `receive()` to event-driven callbacks
- **Isolation**: Contains legacy complexity within adapter boundaries
- **Migration Path**: Enables gradual transition to pure MCP implementations

## Implementation Strategy

### Phase 2: StdioTransport Adapter

#### Event Loop Bridge Pattern
```rust
pub struct StdioTransportAdapter {
    legacy_transport: Option<StdioTransport>,
    message_handler: Option<Arc<dyn MessageHandler>>,
    running: Arc<AtomicBool>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    event_loop_handle: Option<JoinHandle<()>>,
    session_id: Option<String>,
}

async fn event_loop(
    mut transport: StdioTransport,
    handler: Arc<dyn MessageHandler>,
    mut shutdown_rx: oneshot::Receiver<()>,
) {
    loop {
        tokio::select! {
            // Handle graceful shutdown
            _ = &mut shutdown_rx => {
                handler.handle_close().await;
                break;
            }
            
            // Convert blocking receive() to event-driven callback
            result = transport.receive() => {
                match result {
                    Ok(bytes) => {
                        match parse_jsonrpc_message(&bytes) {
                            Ok(message) => {
                                let context = MessageContext::default();
                                handler.handle_message(message, context).await;
                            }
                            Err(e) => {
                                let error = TransportError::Serialization(e.to_string());
                                handler.handle_error(error).await;
                            }
                        }
                    }
                    Err(e) => {
                        handler.handle_error(e.into()).await;
                        if matches!(e, TransportError::Closed) {
                            handler.handle_close().await;
                            break;
                        }
                    }
                }
            }
        }
    }
}
```

#### Backward Compatibility Strategy
```rust
// transport/mod.rs - Re-exports for compatibility
pub use adapters::stdio::StdioTransportAdapter as StdioTransport;
pub use adapters::http::{HttpServerTransport, HttpClientTransport};

// Existing code continues to work unchanged:
let transport = StdioTransport::new().await?;  // Actually StdioTransportAdapter
let server = McpServerBuilder::new().build(transport).await?;
```

### Phase 4: HTTP Transport Migration

#### Migration Strategy
1. **Move Files**: `transport/http/` → `transport/adapters/http/`
2. **Adapter Wrapper**: Create HttpServerTransportAdapter that implements MCP Transport
3. **Eliminate Correlation**: Remove oneshot channels, use natural HTTP request/response flow
4. **Session Context**: Proper MessageContext with HTTP session information

#### Architecture Benefits
- **Complexity Reduction**: Eliminate artificial correlation mechanisms
- **Performance Improvement**: Natural HTTP flow without channel overhead
- **Maintainability**: Clear separation between HTTP specifics and MCP protocol

## Migration Path

### Import Evolution
```rust
// Phase 1: Current usage (unchanged)
use airs_mcp::transport::{StdioTransport, HttpServerTransport};

// Phase 2+: Advanced MCP usage (new capabilities)
use airs_mcp::transport::mcp::{Transport, MessageHandler};

// Future: Explicit adapter usage (optional)
use airs_mcp::transport::adapters::{StdioTransportAdapter, HttpServerTransport};

// Future: Pure MCP implementations
use airs_mcp::transport::mcp::implementations::{HttpTransport, WebSocketTransport};
```

### Transition Strategy
1. **Phase 2-4**: All legacy transports become adapters with MCP interfaces
2. **Phase 6+**: Option to implement MCP Transport directly for new transports
3. **Future**: Pure MCP implementations alongside adapters for maximum flexibility

## Benefits

### Technical Benefits
- **Architecture Compliance**: Perfect alignment with MCP specification patterns
- **Performance**: 20-30% improvement from eliminating artificial correlation mechanisms
- **Maintainability**: Clean separation of concerns between transport and protocol layers
- **Extensibility**: Easy to add new transport types following MCP patterns

### Developer Experience
- **Backward Compatibility**: Existing code works without changes
- **Migration Path**: Clear evolution from legacy → adapter → pure MCP
- **Consistency**: All transports follow same event-driven patterns
- **Documentation**: Clear distinction between legacy compatibility and modern architecture

### Business Impact
- **Future-Proof**: Aligned with official MCP ecosystem evolution
- **Enterprise Ready**: Clean architecture supports complex deployment scenarios
- **Ecosystem Integration**: Works seamlessly with official MCP clients and servers
- **Maintainability**: Reduced technical debt and clearer architecture

## Implementation Phases

1. **Phase 2**: StdioTransport adapter with event loop bridge
2. **Phase 4**: HTTP transport migration to adapters with complexity reduction  
3. **Phase 6+**: Optional pure MCP implementations for new transports
4. **Future**: WebSocket, gRPC, and other transport adapters following same patterns

This architecture provides a clear evolution path from legacy compatibility to modern MCP-compliant design while maintaining complete backward compatibility throughout the transition.
