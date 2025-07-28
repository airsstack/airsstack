# JSON-RPC Core Implementation Plan

**Created**: 2025-07-28  
**Strategy**: Core-First Implementation  
**Phase**: Foundation Building  

## Core Implementation Scope

### What We're Building Now (Phase 1)
Focus on the fundamental JSON-RPC 2.0 message handling without advanced features:

#### 1. Core Message Types
```rust
// Essential structures for JSON-RPC 2.0 compliance
pub struct JsonRpcRequest {
    pub jsonrpc: String,        // Always "2.0"
    pub method: String,         // Method name
    pub params: Option<Value>,  // Optional parameters
    pub id: RequestId,          // Request identifier
}

pub struct JsonRpcResponse {
    pub jsonrpc: String,              // Always "2.0"  
    pub result: Option<Value>,        // Success result
    pub error: Option<JsonRpcError>,  // Error information
    pub id: Option<RequestId>,        // Request ID (null for parse errors)
}

pub struct JsonRpcNotification {
    pub jsonrpc: String,        // Always "2.0"
    pub method: String,         // Method name
    pub params: Option<Value>,  // Optional parameters
    // No ID field - notifications don't expect responses
}
```

#### 2. Request ID Support
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
}

impl RequestId {
    pub fn new_string(id: impl Into<String>) -> Self;
    pub fn new_number(id: i64) -> Self;
}
```

#### 3. JSON-RPC Error Types
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

impl JsonRpcError {
    pub fn parse_error(message: impl Into<String>) -> Self;      // -32700
    pub fn invalid_request(message: impl Into<String>) -> Self;  // -32600
    pub fn method_not_found(method: impl Into<String>) -> Self;  // -32601
    pub fn invalid_params(message: impl Into<String>) -> Self;   // -32602
    pub fn internal_error(message: impl Into<String>) -> Self;   // -32603
}
```

#### 4. Basic Serialization/Deserialization
```rust
impl JsonRpcRequest {
    pub fn to_json(&self) -> Result<String, serde_json::Error>;
    pub fn from_json(json: &str) -> Result<Self, JsonRpcError>;
}

impl JsonRpcResponse {
    pub fn to_json(&self) -> Result<String, serde_json::Error>;
    pub fn from_json(json: &str) -> Result<Self, JsonRpcError>;
}

impl JsonRpcNotification {
    pub fn to_json(&self) -> Result<String, serde_json::Error>;
    pub fn from_json(json: &str) -> Result<Self, JsonRpcError>;
}
```

#### 5. Message Validation
```rust
pub trait JsonRpcMessage {
    fn validate(&self) -> Result<(), JsonRpcError>;
}

// Validation rules:
// - jsonrpc field must be exactly "2.0"
// - method field must be non-empty string
// - id field format validation
// - params field type validation (null, object, or array)
```

### What We're NOT Building Yet
These advanced features are documented but deferred:

- ❌ **Correlation Manager**: Bidirectional request/response matching
- ❌ **Transport Abstraction**: STDIO, HTTP, WebSocket implementations  
- ❌ **High-Level Client**: Async request/response handling
- ❌ **Connection Management**: Lifecycle, timeouts, cleanup
- ❌ **Performance Optimizations**: Zero-copy, buffer pooling
- ❌ **Concurrent Processing**: Multi-threaded message handling

## Implementation Structure

### Module Organization
```
src/base/jsonrpc/
├── mod.rs              # Public API exports
├── message.rs          # Core message types
├── error.rs            # JSON-RPC error handling
├── id.rs               # Request ID implementation
└── validation.rs       # Message validation logic
```

### Dependencies (Minimal Core Set)
```toml
serde = { workspace = true }        # Serialization framework
serde_json = { workspace = true }   # JSON serialization
thiserror = { workspace = true }    # Structured error types
```

### Testing Strategy (Core Focus)
```rust
#[cfg(test)]
mod tests {
    // Message serialization/deserialization
    // Error response creation
    // Request ID handling  
    // Validation edge cases
    // JSON-RPC 2.0 compliance
}
```

## Success Criteria (Phase 1)

### Functional Requirements
- ✅ Parse valid JSON-RPC 2.0 requests, responses, notifications
- ✅ Generate compliant JSON-RPC 2.0 messages
- ✅ Validate message structure and required fields
- ✅ Handle all standard JSON-RPC error codes
- ✅ Support both string and numeric request IDs

### Quality Requirements  
- ✅ 100% JSON-RPC 2.0 specification compliance
- ✅ Comprehensive unit test coverage (>95%)
- ✅ Property-based testing for edge cases
- ✅ Clean, documented public API
- ✅ Zero unsafe code

### Performance Baseline
- ✅ Message parsing/generation < 100μs (establishes baseline)
- ✅ Memory usage < 1KB per message (reasonable baseline)
- ✅ Zero memory leaks in long-running tests

## Integration Points (Future Phases)

### Phase 2 Integration: Correlation Layer
```rust
// Future integration point
impl JsonRpcRequest {
    pub fn with_correlation_id(self, id: RequestId) -> Self;  // Future
}

impl JsonRpcResponse {
    pub fn correlate_with(result: Value, request: &JsonRpcRequest) -> Self;  // Future
}
```

### Phase 3 Integration: Transport Layer
```rust
// Future integration point  
pub trait TransportMessage {
    fn serialize_for_transport(&self) -> Result<Bytes, TransportError>;  // Future
    fn deserialize_from_transport(data: &[u8]) -> Result<Self, TransportError>;  // Future
}
```

## Development Process (Core Phase)

### 1. Core Message Types (Week 1)
- Implement JsonRpcRequest, JsonRpcResponse, JsonRpcNotification
- Add comprehensive serde serialization support
- Create RequestId enum with string/number variants

### 2. Error Handling (Week 1)  
- Implement JsonRpcError with standard codes
- Add structured error creation methods
- Ensure JSON-RPC 2.0 error format compliance

### 3. Validation System (Week 2)
- Add message validation for required fields
- Implement JSON-RPC version checking
- Add parameter type validation

### 4. Testing & Documentation (Week 2)
- Comprehensive unit test suite
- Property-based tests for edge cases
- Complete API documentation with examples
- JSON-RPC 2.0 specification compliance validation

This core implementation provides the foundation for all advanced features while maintaining focus and preventing architectural creep.