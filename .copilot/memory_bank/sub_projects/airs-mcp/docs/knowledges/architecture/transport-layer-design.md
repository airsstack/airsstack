# Transport Layer Design Architecture

**Category**: Architecture  
**Complexity**: High  
**Last Updated**: 2025-08-21  
**Maintainer**: Core Development Team

## Overview
**What is this knowledge about?**

This document captures the architectural design and implementation patterns for the transport layer in airs-mcp. The transport layer provides protocol-agnostic communication abstractions that enable MCP (Model Context Protocol) to operate over various network transports including HTTP, WebSocket, and potentially TCP or other protocols.

**Why this knowledge is important**: The transport layer is the foundation for all MCP communication and its design affects performance, reliability, and extensibility of the entire system.

**Who should read this**: Anyone implementing new transport types, debugging transport issues, or understanding the MCP communication architecture.

## Context & Background
**When and why was this approach chosen?**

The transport abstraction was designed during Phase 2 development when HTTP transport implementation revealed the need for role-specific transport semantics. The architecture balances several competing requirements:

- **Protocol Independence**: MCP should work over multiple transport protocols
- **Role Clarity**: Client and server transport patterns are fundamentally different
- **Performance**: Transport implementations should enable protocol-specific optimizations
- **Testability**: Clear interfaces enable comprehensive testing strategies

**Alternative approaches considered**:
- **Single Transport Interface**: Rejected due to role confusion and complex conditional logic
- **Protocol-Specific APIs**: Rejected due to lack of abstraction and code duplication
- **Factory Pattern**: Evaluated but deemed overly complex for current requirements

**Related ADRs**: ADR-001 (Transport Role-Specific Architecture), ADR-002 (Transport Abstraction Strategy)

## Technical Details
**How does this work?**

### Core Transport Trait
```rust
#[async_trait]
pub trait Transport: Send + Sync + 'static {
    async fn send(&self, data: Vec<u8>) -> Result<(), TransportError>;
    async fn receive(&self) -> Result<Vec<u8>, TransportError>;
    async fn close(&self) -> Result<(), TransportError>;
    fn is_connected(&self) -> bool;
}
```

### Role-Specific Implementations

**Client Transport Pattern**:
```rust
pub struct HttpClientTransport {
    client: Client,                    // HTTP client for outbound requests
    target_url: Option<Url>,          // Server endpoint URL
    message_queue: Arc<Mutex<VecDeque<Vec<u8>>>>, // Response message queue
    session_id: Option<String>,       // Session correlation identifier
    config: HttpTransportConfig,      // Transport configuration
}

impl Transport for HttpClientTransport {
    async fn send(&self, data: Vec<u8>) -> Result<(), TransportError> {
        // Sends HTTP request to server, handles response queuing
    }
    
    async fn receive(&self) -> Result<Vec<u8>, TransportError> {
        // Dequeues responses from message queue
    }
}
```

**Server Transport Pattern**:
```rust
pub struct HttpServerTransport {
    bind_address: SocketAddr,         // Server listen address
    config: HttpTransportConfig,      // Server configuration
    request_parser: RequestParser,    // HTTP request parsing
    // Phase 3: connection_pool, session_manager, request_router
}
```

### Key Design Patterns

1. **Async-First Design**: All transport operations are async to support high-concurrency scenarios
2. **Message Queue Abstraction**: Client transports use queues for response correlation
3. **Configuration Injection**: Transport behavior controlled through config objects
4. **Error Type Specialization**: Transport-specific error types with common trait bounds

## Code Examples
**Practical implementation examples**

### Basic Client Transport Usage
```rust
use airs_mcp::transport::http::HttpClientTransport;
use airs_mcp::transport::Transport;

// Create and configure client transport
let mut transport = HttpClientTransport::new(
    "http://localhost:8080/mcp".parse()?,
    HttpTransportConfig::default()
).await?;

// Send MCP message
let message = serde_json::to_vec(&mcp_request)?;
transport.send(message).await?;

// Receive response
let response_data = transport.receive().await?;
let mcp_response: McpResponse = serde_json::from_slice(&response_data)?;
```

### Transport Factory Pattern
```rust
pub enum TransportType {
    Http { url: Url, config: HttpTransportConfig },
    WebSocket { url: Url, config: WsTransportConfig },
}

pub async fn create_client_transport(
    transport_type: TransportType
) -> Result<Box<dyn Transport>, TransportError> {
    match transport_type {
        TransportType::Http { url, config } => {
            Ok(Box::new(HttpClientTransport::new(url, config).await?))
        }
        TransportType::WebSocket { url, config } => {
            Ok(Box::new(WebSocketClientTransport::new(url, config).await?))
        }
    }
}
```

## Performance Characteristics
**How does this perform?**

### Time Complexity
- **Send Operation**: O(1) for message queuing + O(network) for HTTP request
- **Receive Operation**: O(1) for queue dequeue operations
- **Connection Setup**: O(network) for initial HTTP client setup

### Memory Usage
- **Message Queue**: O(n) where n = number of pending responses
- **Connection Pool**: O(c) where c = number of configured connections
- **Session State**: O(s) where s = number of active sessions

### Throughput Characteristics
- **HTTP Transport**: Limited by HTTP connection limits and server capacity
- **Concurrent Requests**: Bounded by HTTP client connection pool size
- **Message Correlation**: Correlation ID lookup is O(1) with HashMap-based queuing

**Benchmarking Results**: Reference `crates/airs-mcp/benches/transport_performance.rs` for detailed performance analysis.

## Trade-offs & Limitations
**What are the constraints and compromises?**

### Design Trade-offs
- **Role Separation vs API Complexity**: Chose role clarity over simpler single-type API
- **Generic Transport vs Protocol Optimization**: Balances abstraction with performance
- **Memory vs Latency**: Message queuing trades memory usage for response correlation

### Current Limitations
- **Connection Pooling**: HTTP client doesn't yet optimize connection reuse
- **Backpressure**: No automatic backpressure handling for message queue overflow
- **Compression**: No built-in compression support (can be added at HTTP layer)
- **Multiplexing**: HTTP transport doesn't support request multiplexing

### Scalability Considerations
- **Memory Growth**: Message queues can grow unbounded without proper cleanup
- **Connection Limits**: HTTP client connection limits may bottleneck high-concurrency scenarios
- **Session Management**: Server transport needs session cleanup for long-running connections

## Dependencies
**What does this rely on?**

### External Crates
- **reqwest**: HTTP client implementation with async support
- **axum**: HTTP server framework for server transport implementation
- **tokio**: Async runtime for all transport operations
- **serde**: Message serialization for transport protocol

### Internal Modules
- **correlation**: Correlation ID generation and validation
- **shared::protocol**: MCP protocol definitions and message types
- **base**: Common error types and utility functions

### Configuration Requirements
- **HttpTransportConfig**: Timeout, retry, and connection pool configuration
- **Runtime**: Tokio async runtime required for all transport operations

## Testing Strategy
**How is this tested?**

### Unit Testing Approach
```rust
#[tokio::test]
async fn test_http_client_transport_send_receive() {
    let mock_server = setup_mock_http_server().await;
    let transport = HttpClientTransport::new(
        mock_server.url(),
        HttpTransportConfig::default()
    ).await.unwrap();
    
    let test_message = b"test message".to_vec();
    transport.send(test_message.clone()).await.unwrap();
    
    let received = transport.receive().await.unwrap();
    assert_eq!(received, test_message);
}
```

### Integration Testing
- **Transport Compatibility**: Tests ensuring all transport types implement Transport trait correctly
- **Error Handling**: Comprehensive error scenario testing (network failures, timeouts, malformed data)
- **Performance Testing**: Benchmarks in `benches/` directory for throughput and latency analysis

### Edge Cases and Error Conditions
- **Network Failures**: Connection drops, DNS failures, timeout scenarios
- **Protocol Errors**: Malformed HTTP responses, unexpected status codes
- **Resource Exhaustion**: Message queue overflow, connection pool exhaustion
- **Concurrent Access**: Multiple thread safety and message ordering guarantees

## Common Pitfalls
**What should developers watch out for?**

### Implementation Pitfalls
- **Blocking Operations**: Ensure all transport operations remain async (no blocking I/O)
- **Error Handling**: Don't ignore transport errors - they often indicate network or protocol issues
- **Resource Cleanup**: Always call `close()` to properly cleanup transport resources
- **Message Correlation**: Client transports require proper correlation ID handling for response matching

### Debugging Tips
- **Connection Issues**: Check `is_connected()` status before send/receive operations
- **Queue Overflow**: Monitor message queue depth for client transports
- **Timeout Configuration**: Adjust timeout values based on network characteristics and usage patterns
- **Error Logging**: Transport errors provide detailed context for debugging network issues

### Performance Gotchas
- **Connection Reuse**: HTTP client benefits from connection pooling for repeated requests
- **Message Size**: Large messages may require streaming or chunking for optimal performance
- **Concurrent Limits**: Respect HTTP client connection limits to avoid blocking

## Related Knowledge
**What else should I read?**

### Related Architecture Documents
- **patterns/async-error-handling.md**: Error handling patterns used in transport implementations
- **domain/mcp-protocol-compliance.md**: MCP protocol requirements that transport layer must support

### Relevant Design Patterns
- **Factory Pattern**: For transport type selection and configuration
- **Command Pattern**: For message queuing and correlation
- **Strategy Pattern**: For protocol-specific optimizations

### Performance Analysis
- **performance/streaming-benchmarks.md**: Performance characteristics of different transport types
- **performance/connection-pooling-analysis.md**: HTTP connection optimization strategies

## Evolution History
**How has this changed over time?**

### Major Revisions
- **2025-08-14**: Initial role-specific architecture implementation
  - Separated HttpClientTransport and HttpServerTransport
  - Established Transport trait abstraction
  - Implemented backward compatibility with deprecated HttpStreamableTransport

### Deprecated Approaches
- **Single HttpStreamableTransport**: Replaced due to role confusion and architectural complexity
- **Synchronous Transport API**: Replaced with async-first design for better concurrency support

### Future Evolution Plans
- **WebSocket Transport**: Planned implementation following established role-specific pattern
- **Connection Pooling**: Enhanced HTTP client connection management
- **Compression Support**: Transport-level compression for improved bandwidth efficiency
- **Streaming Support**: Large message streaming capabilities for file transfer scenarios

## Examples in Codebase
**Where can I see this in action?**

### Reference Implementations
- **crates/airs-mcp/src/transport/http/client.rs**: Complete HttpClientTransport implementation
- **crates/airs-mcp/src/transport/http/server.rs**: HttpServerTransport foundation (Phase 3)
- **crates/airs-mcp/examples/http_transport_usage.rs**: Complete usage examples

### Test Files
- **crates/airs-mcp/tests/http_transport_integration.rs**: Integration test suite
- **crates/airs-mcp/src/transport/http/tests.rs**: Unit test examples
- **crates/airs-mcp/benches/transport_performance.rs**: Performance benchmarking

### Documentation Examples
- **crates/airs-mcp/examples/simple-mcp-client/**: Complete MCP client using HTTP transport
- **crates/airs-mcp/examples/axum_server_with_handlers.rs**: Server-side transport usage patterns
