# DEBT-001: HTTP Transport Trait Impedance Mismatch

**Status**: Active  
**Priority**: High  
**Category**: Architecture  
**Created**: 2025-09-01  
**Updated**: 2025-09-01  
**Estimated Effort**: 2-3 Weeks

## Problem Description
**What is the technical debt?**
- HttpServerTransport implements Transport trait despite fundamental semantic mismatch
- Transport trait assumes single persistent connection (send/receive sequential)
- HTTP reality involves multiple concurrent sessions with natural request/response correlation
- Current solution uses dual correlation systems: oneshot channels + session tracking
- Architecture forces HTTP semantics into incompatible abstraction

**Impact on current development velocity:**
- Complex debugging due to dual correlation mechanisms
- Difficult to reason about message flow and session management
- New HTTP features require working around Transport trait limitations
- Code review complexity due to architectural confusion

**Impact on code maintainability:**
- Mixed correlation responsibilities between HTTP layer and Transport layer
- Session tracking (`current_session`) feels hacky and error-prone
- oneshot channels mask natural HTTP request/response semantics
- Future HTTP features (WebSocket upgrade, streaming) will compound the mismatch

## Context & Reason
**Why was this debt incurred?**
- Business requirement: Integrate HTTP transport with existing McpServerBuilder
- McpServerBuilder expects Transport trait interface
- Transport trait was designed for persistent connections (WebSocket, TCP)
- Time constraint: Needed working HTTP integration for Phase 2 delivery

**Technology limitations:**
- Rust trait system doesn't allow easy retrofitting of multi-session semantics
- Existing codebase heavily invested in single Transport trait pattern

## Technical Details

### Current Architecture Problems
```rust
// Transport trait assumes this pattern (single connection):
loop {
    let request = transport.receive().await?;  // Sequential
    let response = process(request).await;     
    transport.send(&response).await?;          // Correlated by sequence
}

// HTTP reality (multi-session):
async fn handle_request_1() { /* session_1 */ }
async fn handle_request_2() { /* session_2 */ }  
async fn handle_request_3() { /* session_1 again */ }
```

### Forced Correlation Mechanisms
1. **oneshot::channel per request** - Creates artificial correlation
2. **current_session tracking** - Bridges Transport.send() to correct session
3. **HashMap<SessionId, Sender>** - Manual response routing

### Code Complexity Examples
```rust
// This should be simple HTTP request/response:
pub async fn handle_http_request(&self, session_id: SessionId, request_data: Vec<u8>) -> Result<Vec<u8>, TransportError> {
    let (response_tx, response_rx) = oneshot::channel(); // ← Artificial correlation
    self.outgoing_responses.insert(session_id, response_tx); // ← Manual routing
    self.incoming_sender.send((session_id, request_data))?; // ← Bridge to Transport
    response_rx.await // ← Wait for Transport.send() to respond
}
```

## Proposed Solutions

### Option A: HTTP-Native Transport (Recommended)
**Approach**: Abandon Transport trait for HTTP, design HTTP-specific interface
```rust
pub trait HttpMcpTransport {
    async fn handle_mcp_request(&mut self, session_id: SessionId, request: McpRequest) -> Result<McpResponse, Error>;
}
```

**Pros:**
- Natural HTTP semantics
- No artificial correlation mechanisms
- Clear separation of concerns
- Easy to add HTTP-specific features (streaming, WebSocket upgrade)

**Cons:**
- Requires McpServerBuilder refactoring
- Different interface pattern from other transports

### Option B: Multi-Session Transport Trait
**Approach**: Redesign Transport trait to handle multi-session natively
```rust
pub trait Transport {
    async fn send_to_session(&mut self, session_id: SessionId, message: &[u8]) -> Result<(), Error>;
    async fn receive_from_any_session(&mut self) -> Result<(SessionId, Vec<u8>), Error>;
}
```

**Pros:**
- Maintains Transport trait pattern
- Works for both HTTP and future multi-session transports
- Clear session semantics

**Cons:**
- Breaking change to existing Transport implementations
- Complex migration path

### Option C: Connection Factory Pattern
**Approach**: Transport creates per-session connections
```rust
pub trait TransportFactory {
    async fn create_connection(&self, session_id: SessionId) -> Result<Box<dyn Transport>, Error>;
}
```

**Pros:**
- Maintains single-connection Transport semantics
- Clear session isolation
- Compatible with existing code

**Cons:**
- Resource overhead (connection per session)
- Complex lifecycle management

## Impact Assessment

### Current State Risk
- **Development Velocity**: Medium risk - complex debugging slows feature development
- **Code Quality**: High risk - architectural confusion leads to bugs
- **Maintainability**: High risk - dual correlation systems are brittle
- **Extensibility**: Critical risk - HTTP features (streaming, WebSocket) will be severely limited

### Migration Complexity
- **Option A (HTTP-Native)**: High complexity, high value
- **Option B (Multi-Session Trait)**: Critical complexity, highest value
- **Option C (Factory Pattern)**: Medium complexity, medium value

## Remediation Plan

### Phase 1: Analysis & Design (1 week)
1. Analyze McpServerBuilder dependencies on Transport trait
2. Design HTTP-native interface (Option A)
3. Create migration strategy document
4. Stakeholder review and decision

### Phase 2: Implementation (1-2 weeks)
1. Implement new HTTP-native interface
2. Refactor McpServerBuilder to support both patterns
3. Migrate HTTP transport to new interface
4. Update tests and documentation

### Phase 3: Cleanup (1 week)
1. Remove oneshot channel correlation system
2. Remove session tracking mechanisms
3. Simplify HTTP request/response flow
4. Performance validation

## Dependencies
- McpServerBuilder refactoring
- Transport trait evolution decision
- HTTP feature roadmap priorities

## Related Technical Debt
- None identified (this is foundational architectural debt)

## Monitoring Metrics
- Code complexity metrics in transport/http module
- HTTP request/response latency
- Session management memory overhead
- Developer productivity on HTTP features

## Business Impact
**Immediate**: Slower HTTP feature development, complex debugging  
**Long-term**: Limited HTTP capabilities, maintenance burden, developer frustration

## Notes
This debt represents a fundamental architectural decision that affects the entire HTTP transport subsystem. Resolution requires careful planning and stakeholder alignment, but is essential for long-term HTTP transport success.
