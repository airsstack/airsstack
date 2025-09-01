# KNOWLEDGE-003: MCP Transport Architecture Patterns

**Category**: Architecture  
**Created**: 2025-09-01  
**Updated**: 2025-09-01  
**Status**: Active  
**Priority**: Critical

## Overview

Comprehensive analysis of Model Context Protocol (MCP) transport layer architecture based on official specification research and comparison with TypeScript/Python SDKs.

## Key Insights

### MCP Specification Transport Interface

The official MCP specification defines transport as an event-driven interface:

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

### Separation of Concerns

**Official MCP Specification Principle:**
> "Clear separation between the 'data layer' (JSON-RPC protocol, lifecycle management, core primitives) and the 'transport layer' (communication mechanisms)"

**Layer Responsibilities:**
- **Protocol Logic**: Handles MCP semantics (tools, resources, prompts)
- **Transport Logic**: Handles message delivery mechanics

### Event-Driven vs Sequential Patterns

**MCP-Compliant (Event-Driven):**
```typescript
// Natural message flow
transport.onmessage = (message) => {
  handleMcpMessage(message);
};

// Simple sending
transport.send(responseMessage);
```

**Current Implementation (Sequential - Non-Compliant):**
```rust
// Blocking/polling pattern
loop {
    let message = transport.receive().await?;  // ← Blocking
    let response = process(message).await;
    transport.send(&response).await?;          // ← Sequential
}
```

## Transport Implementation Patterns

### Official SDK Transport Types

**TypeScript SDK Implementations:**
- `StdioServerTransport` - STDIO communication
- `StreamableHTTPServerTransport` - HTTP-based communication  
- `SSEServerTransport` - Server-Sent Events (deprecated)
- `WebSocketClientTransport` - WebSocket communication
- `InMemoryTransport` - In-process communication

**Common Pattern:**
```typescript
class HttpTransport implements Transport {
  private messageHandler?: (message: JSONRPCMessage) => void;
  
  // Natural HTTP request handling
  private handleRequest(req: Request, sessionId: string) {
    const message = parseJsonRpc(req.body);
    this.onmessage?.(message);  // ← Event emission
  }
  
  // Simple send - no correlation needed
  async send(message: JSONRPCMessage): Promise<void> {
    this.sendToSession(this.currentSession, message);
  }
}
```

### Key Architecture Principles

1. **Event-Driven**: Messages trigger callbacks, not blocking receives
2. **Asynchronous**: No sequential request/response assumptions
3. **Session-Aware**: Transport manages its own session semantics
4. **Pluggable**: Clean interface allows custom transport implementations
5. **Correlation-Free**: JSON-RPC IDs handle message correlation naturally

## HTTP Transport Specific Insights

### Natural HTTP Semantics
```rust
// HTTP request naturally creates message event
async fn handle_http_request(session_id: String, request_data: Vec<u8>) {
    let message = parse_jsonrpc(request_data)?;
    
    // Set session context
    transport.set_session_context(Some(session_id));
    
    // Emit message event (no correlation needed)
    if let Some(handler) = &transport.message_handler {
        handler.handle_message(message, context).await;
    }
}

// Response sent naturally via transport
impl MessageHandler for McpServer {
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext) {
        let response = process_message(message).await;
        
        // Send response to same session
        let mut transport = self.transport.lock().await;
        transport.set_session_context(context.session_id);
        transport.send(response).await?;  // ← Natural response
    }
}
```

### No Artificial Correlation Needed
- **oneshot channels**: Unnecessary with event-driven design
- **Session tracking maps**: Transport handles session routing internally  
- **Manual correlation**: JSON-RPC message IDs provide natural correlation

## Bidirectional Communication Examples

### Client Request → Server Response
```rust
// 1. HTTP request triggers message event
let request = JsonRpcMessage {
    id: Some(json!(1)),
    method: Some("tools/call".to_string()),
    params: Some(json!({"name": "read_file"})),
    // ...
};

// 2. MessageHandler processes request
async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext) {
    let result = execute_tool(message.params).await;
    let response = JsonRpcMessage {
        id: message.id,  // ← Same ID for correlation
        result: Some(result),
        // ...
    };
    
    // 3. Send response via transport
    transport.send(response).await?;
}
```

### Server-Initiated Notification
```rust
// Server sends notification (no response expected)
let notification = JsonRpcMessage {
    id: None,  // ← No ID = notification
    method: Some("notifications/resources/updated".to_string()),
    params: Some(json!({"uri": "file://updated.txt"})),
    // ...
};

transport.send(notification).await?;
```

### Bidirectional Request (Server → Client)
```rust
// Server requests client capabilities
let request = JsonRpcMessage {
    id: Some(json!(42)),  // ← Expects response
    method: Some("tools/list".to_string()),
    // ...
};

transport.send(request).await?;

// Client responds via same transport/different handler
async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext) {
    if message.method == Some("tools/list") {
        let response = JsonRpcMessage {
            id: message.id,  // ← Same ID
            result: Some(json!({"tools": [...]})),
            // ...
        };
        transport.send(response).await?;
    }
}
```

## Implementation Recommendations

### Rust MCP-Compliant Transport Trait
```rust
#[async_trait]
pub trait Transport: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    // Lifecycle (matches MCP spec)
    async fn start(&mut self) -> Result<(), Self::Error>;
    async fn close(&mut self) -> Result<(), Self::Error>;
    
    // Message exchange (matches MCP spec)
    async fn send(&mut self, message: JsonRpcMessage) -> Result<(), Self::Error>;
    
    // Event handling (matches MCP spec)
    fn set_message_handler(&mut self, handler: Arc<dyn MessageHandler>);
    
    // Session support
    fn session_id(&self) -> Option<String>;
    fn set_session_context(&mut self, session_id: Option<String>);
    
    // Transport state
    fn is_connected(&self) -> bool;
    fn transport_type(&self) -> &'static str;
}

#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext);
    async fn handle_error(&self, error: TransportError);
    async fn handle_close(&self);
}
```

### Benefits of MCP-Compliant Design
1. **Specification Alignment**: Matches official TypeScript/Python patterns
2. **Natural HTTP Support**: Event-driven eliminates correlation complexity
3. **Extensibility**: Easy WebSocket, SSE, custom transport addition
4. **Performance**: No artificial correlation overhead
5. **Developer Experience**: Intuitive, matches MCP documentation
6. **Testing**: Clean separation enables independent transport/protocol testing

## Migration Impact

### Current Problems Solved
- **HTTP Transport Complexity**: Eliminates oneshot channels, session tracking
- **Architectural Confusion**: Clear separation of transport vs protocol concerns
- **Limited Extensibility**: Foundation for additional transport types
- **Non-Standard Interface**: Brings compliance with official MCP specification

### Breaking Changes Required
- Transport trait interface complete redesign
- MessageHandler trait introduction
- McpServerBuilder integration updates
- Example and documentation updates

## Related Documentation
- [ADR-001: MCP-Compliant Transport Redesign](../adr/ADR-001-mcp-compliant-transport-redesign.md)
- [DEBT-001: HTTP Transport Trait Impedance Mismatch](../debts/DEBT-001-http-transport-trait-impedance-mismatch.md)

## External References
- [MCP Official Specification](https://modelcontextprotocol.io/docs)
- [TypeScript SDK Transport Interface](https://github.com/modelcontextprotocol/typescript-sdk)
- [Python SDK Transport Patterns](https://github.com/modelcontextprotocol/python-sdk)

## Implementation Notes

### Testing Strategy
```rust
// Transport testing (delivery mechanics)
#[tokio::test]
async fn test_http_transport_message_delivery() {
    let mut transport = HttpServerTransport::new(config).await?;
    let mock_handler = Arc::new(MockMessageHandler::new());
    transport.set_message_handler(mock_handler.clone());
    
    // Test message delivery without protocol complexity
}

// Protocol testing (MCP logic)
#[tokio::test]
async fn test_mcp_server_tool_execution() {
    let server = McpServer::new().with_tool("test", TestTool::new());
    let message = create_tool_call_message("test", json!({}));
    
    server.handle_message(message, context).await;
    // Test MCP protocol without transport complexity
}
```

### Performance Considerations
- Event-driven design eliminates blocking operations
- No correlation map lookups or oneshot channel overhead
- Natural HTTP request/response flow
- Minimal memory allocation for message passing

### Security Implications
- Transport-level session isolation
- Clear message flow audit trail
- No artificial correlation state to compromise
- Natural timeout and error handling patterns
