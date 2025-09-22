# Performance Benchmarks

This document contains performance benchmarks for airs-mcp v0.2.0.

## Running Benchmarks

**Note**: These benchmarks are designed for resource-limited environments.

```bash
# Run all lightweight benchmarks
cargo bench --package airs-mcp --bench lightweight_benchmarks

# Run specific benchmark groups
cargo bench --package airs-mcp --bench lightweight_benchmarks jsonrpc_serialization
cargo bench --package airs-mcp --bench lightweight_benchmarks message_creation
cargo bench --package airs-mcp --bench lightweight_benchmarks payload_sizes
cargo bench --package airs-mcp --bench lightweight_benchmarks async_operations
```

## Benchmark Categories

### 1. JSON-RPC Serialization
- **simple_request_serialize**: Time to serialize basic JSON-RPC request
- **simple_request_deserialize**: Time to deserialize basic JSON-RPC request  
- **simple_response_serialize**: Time to serialize basic JSON-RPC response
- **notification_serialize**: Time to serialize JSON-RPC notification

### 2. Message Creation
- **request_creation**: Time to create JsonRpcRequest objects
- **response_creation**: Time to create JsonRpcResponse objects

### 3. Payload Sizes
- **small payload**: ~50 bytes JSON payload
- **medium payload**: ~100 characters payload  
- **large payload**: ~1KB payload

### 4. Async Operations
- **simple_async_task**: Basic async task performance

## Performance Baselines (v0.2.0)

**Benchmark Date**: 2025-09-22  
**Environment**: Resource-limited (CI-friendly configuration)

### JSON-RPC Message Processing

#### Serialization Performance
- **Simple Request**: 79.7ns avg (12.5M ops/sec)
- **Simple Response**: 81.4ns avg (12.3M ops/sec)  
- **Notification**: 91.1ns avg (11.0M ops/sec)

#### Message Creation
- **Request Creation**: 194.7ns avg (5.1M ops/sec)
- **Response Creation**: 152.6ns avg (6.6M ops/sec)

#### Payload Size Impact
- **Small Payload** (~50 bytes): 100.9ns avg
- **Medium Payload** (~100 chars): 195.9ns avg
- **Large Payload** (~1KB): 530.8ns avg

#### Async Operations
- **Simple Async Task**: 97.3ns avg (10.3M ops/sec)

### Performance Characteristics
- **Excellent Sub-Microsecond Performance**: All basic operations <1μs
- **Linear Payload Scaling**: Performance scales predictably with payload size
- **High Throughput**: >10M operations/sec for simple JSON-RPC processing
- **Memory Efficient**: Minimal allocation overhead for basic operations

### Performance Commitments
- No regression >20% between minor versions  
- Sub-microsecond performance for basic JSON-RPC operations
- Linear scaling with payload complexity
- Regular performance regression testing

## Resource Configuration

The benchmarks are configured for limited resource environments:

- **Warm-up time**: 100ms (minimal)
- **Measurement time**: 300-500ms (short)
- **Sample size**: 25-50 samples (small)
- **Payload sizes**: Capped at 1KB for large tests

## Performance Targets

For v0.2.0, we aim to maintain:

- **Serialization latency**: <10μs for typical messages
- **Memory allocation**: Minimal unnecessary allocations
- **Async overhead**: <1μs for basic async operations
- **Scaling**: Linear performance with payload size

## Interpreting Results

Benchmark results will vary significantly based on:
- Hardware specifications
- System load during testing
- Rust compiler optimizations
- Available memory

Focus on relative performance and trends rather than absolute numbers.

## Future Improvements

Areas for performance optimization in future versions:
- Zero-copy deserialization where possible
- Custom serialization for hot paths
- Memory pool allocation for frequent objects
- SIMD optimizations for large payloads