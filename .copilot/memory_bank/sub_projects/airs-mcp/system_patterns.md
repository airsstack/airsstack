# system_patterns.md

## Architecture Objectives
- Protocol-first design: 100% MCP spec compliance, built on JSON-RPC 2.0
- Type safety & memory safety: Rust type system, zero unsafe code, ownership-based resource management
- Async-native performance: Tokio-based async, sub-ms latency, high throughput
- Operational requirements: Structured logging, metrics, error handling, connection recovery, 24/7 stability

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

## Transport Abstraction
- Transport trait for async send/receive/close operations
- STDIO transport: newline-delimited JSON, streaming parser, buffer management
- Future transports: HTTP, WebSocket, TCP

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
