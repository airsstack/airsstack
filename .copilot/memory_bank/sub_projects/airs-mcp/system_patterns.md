# system_patterns.md

## Architecture Objectives
- Protocol-first design: 100% MCP spec compliance, built on JSON-RPC 2.0
- Type safety & memory safety: Rust type system, zero unsafe code, ownership-based resource management
- Async-native performance: Tokio-based async, sub-ms latency, high throughput
- Operational requirements: Structured logging, metrics, error handling, connection recovery, 24/7 stability

## Core Component Design
- JSON-RPC 2.0 message processor
- Request tracking and correlation (DashMap, async primitives)
- Message validation and error handling
- Advanced Correlation Manager: lock-free concurrency, timeout management, memory safety, error propagation

## Data Flow Architecture
- Client-server and server-client message flow patterns
- Processing phases: request validation, routing, response correlation
- Bidirectional communication flow: request ID generation, registration, sending, correlation, resolution, cleanup

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
