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

**Phase 2: Adapter Layer (Week 1-2)**
- Create compatibility adapter for existing StdioTransport
- Implement McpServer as MessageHandler for existing functionality
- Ensure backward compatibility with McpServerBuilder

**Phase 3: HTTP Transport Redesign (Week 2-3)**
- Rewrite HttpServerTransport using new interface
- Eliminate oneshot channels and manual correlation
- Implement natural HTTP request/response flow
- Update integration tests

**Phase 4: Integration (Week 3)**
- Update McpServerBuilder to use new Transport interface
- Migrate examples and documentation
- Performance validation and optimization

**Phase 5: Cleanup (Week 4)**
- Deprecate old Transport trait
- Remove compatibility adapters
- Final documentation and examples

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
