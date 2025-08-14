# Decision Record: HTTP Transport Architecture Strategy

**Decision Status:** PENDING  
**Decision Required By:** 2025-08-20  
**Impact Level:** HIGH - Affects entire HTTP transport implementation strategy  
**Created:** 2025-08-14T15:45:00Z  
**Context:** Phase 2 HTTP implementation reveals Transport trait architectural mismatch

## Decision Required

**Should we redesign the HTTP transport architecture to address the fundamental mismatch between the symmetric Transport trait and asymmetric HTTP protocol?**

## Context & Problem Statement

### Current Implementation Analysis
Phase 2 HTTP transport implementation (`HttpStreamableTransport`) reveals a fundamental architectural tension:

```rust
// Current implementation forces HTTP into symmetric interface
pub struct HttpStreamableTransport {
    client: Client,           // Client-side perspective
    target_url: Option<Url>,  // Where to send requests
    message_queue: Arc<Mutex<VecDeque<Vec<u8>>>>, // Artificial receive() support
}

impl Transport for HttpStreamableTransport {
    async fn send(&mut self, message: &[u8]) -> Result<(), Self::Error> {
        // POST message to target_url, queue response
    }
    async fn receive(&mut self) -> Result<Vec<u8>, Self::Error> {
        // Return queued response from previous send() - NOT true peer messaging
    }
}
```

### The Core Problem

**Transport Trait Assumptions (Works for STDIO):**
```rust
// Perfect for bidirectional pipes
stdin.write(message)  → send()
stdout.read()         → receive()  // True peer messaging
```

**HTTP Reality (Asymmetric):**
```rust
// Client side:
POST /mcp + body      → send()
response.body()       → receive()  // Response to OUR request, not peer message

// Server side:
listen_for_requests() → receive()  // TRUE peer messaging  
write_response()      → send()     // Response to THEIR request
```

### Technical Debt Identified

1. **Semantic Violation**: `receive()` doesn't receive peer messages, only responses
2. **Role Confusion**: Client implementation in what should be server transport
3. **Scalability Issues**: Single-session design incompatible with production HTTP
4. **Architecture Mismatch**: HTTP fundamentally different from bidirectional transports

## Options Analysis

### Option A: Role-Specific Transports (Recommended)

**Implementation:**
```rust
pub struct HttpClientTransport {
    client: Client,
    target_url: Url,
    session_id: Option<String>,
}

pub struct HttpServerTransport {
    listener: TcpListener,
    connection_pool: Pool<ConnectionManager>,
    session_manager: SessionManager,
}

// Both implement Transport trait appropriately for their role
```

**Pros:**
- ✅ Clear role separation (client vs server)
- ✅ Each implementation semantically correct for its role
- ✅ Allows proper HTTP server features (connection pooling, session mgmt)
- ✅ Maintains Transport trait compatibility

**Cons:**
- ❌ Two separate types instead of one
- ❌ Requires client vs server decision at creation time
- ❌ Some code duplication between implementations

### Option B: Mode-Based Unified Transport

**Implementation:**
```rust
pub enum HttpTransportMode {
    Client { target_url: Url },
    Server { bind_address: SocketAddr },
}

pub struct HttpStreamableTransport {
    mode: HttpTransportMode,
    // Runtime dispatch to appropriate implementation
}
```

**Pros:**
- ✅ Single type handles both roles
- ✅ Mode can be configured at runtime
- ✅ Maintains current API surface

**Cons:**
- ❌ Runtime dispatch overhead
- ❌ Complex internal implementation
- ❌ Some methods only valid for some modes

### Option C: Abandon Transport Trait for HTTP

**Implementation:**
```rust
// Dedicated HTTP APIs that don't force symmetric interface
pub struct HttpClient {
    pub async fn post_message(&self, message: &[u8]) -> Result<Vec<u8>, Error>;
    pub async fn start_streaming_session(&self) -> Result<HttpStream, Error>;
}

pub struct HttpServer {
    pub async fn listen(&self) -> Result<HttpRequestStream, Error>;
    pub async fn respond(&self, request_id: u64, response: &[u8]) -> Result<(), Error>;
}
```

**Pros:**
- ✅ Perfect semantic match for HTTP protocol
- ✅ No forced abstractions
- ✅ Optimal APIs for each use case

**Cons:**
- ❌ Breaks Transport trait compatibility
- ❌ Requires different integration patterns
- ❌ More complex for library users

## Recommendation

**Preferred Option: A (Role-Specific Transports)**

### Rationale

1. **Semantic Correctness**: Each implementation correctly models its role
2. **Architectural Clarity**: Clear separation of concerns
3. **Future Extensibility**: Allows proper server-side features
4. **Compatibility**: Maintains Transport trait interface

### Implementation Strategy

1. **Phase 2 Completion**: Keep current `HttpStreamableTransport` as `HttpClientTransport`
2. **Phase 3 Planning**: Design `HttpServerTransport` with proper server architecture
3. **Migration Path**: Provide type aliases for compatibility if needed

## Impact Assessment

### If We Proceed With Current Implementation
- ✅ **Short-term**: Phase 2 complete, functional client implementation
- ❌ **Medium-term**: Technical debt compounds in Phase 3
- ❌ **Long-term**: Confusing APIs, scalability issues

### If We Redesign Now
- ❌ **Short-term**: Additional refactoring work required
- ✅ **Medium-term**: Clean foundation for Phase 3 server implementation  
- ✅ **Long-term**: Maintainable, semantically correct architecture

## Decision Criteria

**Proceed with redesign if:**
- User values architectural excellence over shipping speed
- Phase 3 server implementation is planned
- Long-term maintainability is prioritized

**Proceed with current implementation if:**
- Immediate Phase 2 completion is critical
- Only client-side HTTP usage is needed
- Technical debt can be addressed in future major version

## Next Steps

1. **User Decision**: Choose between shipping vs. architectural excellence
2. **If Redesign**: Create detailed refactoring plan
3. **If Proceed**: Document technical debt for Phase 3 planning
4. **Either Way**: Update memory bank with decision rationale

---

**Status**: Awaiting user decision on architectural strategy
