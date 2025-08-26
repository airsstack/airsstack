# [TASK023] - HTTP Streamable GET Handler Implementation

**Status:** pending  
**Added:** 2025-08-26  
**Updated:** 2025-08-26  
**Priority:** HIGH - Required for HTTP Streamable specification compliance

## Original Request
Implement GET `/mcp` handler for HTTP Streamable transport to enable SSE streaming responses and complete the dynamic response mode selection architecture.

## Thought Process
The HTTP Streamable specification requires a single `/mcp` endpoint that can handle both:
1. **POST requests**: Return JSON responses directly
2. **GET requests**: Return SSE streaming responses with event-stream content-type

This completes the HTTP Streamable transport by enabling the streaming mode while reusing all existing infrastructure (session management, connection tracking, JSON-RPC processing).

## Implementation Plan
1. **Add GET route handler** to existing `/mcp` endpoint in `axum/handlers.rs`
2. **Implement SSE streaming response** using axum_streams::StreamBodyAs
3. **Reuse session management** from existing POST handler infrastructure
4. **Integrate event streaming** with existing JsonRpcProcessor for response delivery
5. **Add proper SSE headers** (text/event-stream, cache-control, keep-alive)
6. **Handle connection recovery** using `Last-Event-ID` from session context

## Progress Tracking

**Overall Status:** pending - 0%

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 23.1 | Add GET route to `/mcp` endpoint | not_started | 2025-08-26 | Extend existing router with get() handler |
| 23.2 | Implement handle_mcp_get function | not_started | 2025-08-26 | SSE streaming response with session management |
| 23.3 | Add SSE response headers | not_started | 2025-08-26 | text/event-stream, cache-control, connection keep-alive |
| 23.4 | Integrate with JsonRpcProcessor | not_started | 2025-08-26 | Route streaming responses through existing processor |
| 23.5 | Add connection recovery support | not_started | 2025-08-26 | Use Last-Event-ID for session resume |
| 23.6 | Write integration tests | not_started | 2025-08-26 | Test GET /mcp SSE streaming functionality |

## Technical Specifications

### Router Enhancement
```rust
// Extend existing router in axum/handlers.rs
Router::new()
    .route("/mcp", post(handle_mcp_request))
    .route("/mcp", get(handle_mcp_get))  // ADD THIS
    // ... existing routes
```

### SSE Handler Implementation
```rust
// New handler function in axum/handlers.rs
async fn handle_mcp_get(
    State(state): State<ServerState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Response<Body>, (StatusCode, String)> {
    // 1. Extract/create session (reuse existing function)
    let session_id = extract_or_create_session(&state, &headers, addr).await?;
    
    // 2. Register connection (reuse existing logic)
    let connection_id = state.connection_manager.register_connection(addr).await?;
    
    // 3. Create SSE stream with proper headers
    let stream = create_sse_event_stream(state, session_id, connection_id).await?;
    
    // 4. Return SSE response
    Ok(Response::builder()
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .header("connection", "keep-alive")
        .header("Mcp-Session-Id", session_id.to_string())
        .body(Body::from_stream(stream))?)
}
```

### Event Stream Integration
```rust
// Stream creation function
async fn create_sse_event_stream(
    state: ServerState, 
    session_id: SessionId, 
    connection_id: ConnectionId
) -> Result<impl Stream<Item = Result<Bytes, Error>>, TransportError> {
    // Create event stream that:
    // 1. Connects to JsonRpcProcessor response channel
    // 2. Formats responses as SSE events
    // 3. Handles connection recovery via Last-Event-ID
    // 4. Maintains session activity tracking
}
```

## Success Criteria
- [ ] GET `/mcp` endpoint returns proper SSE streaming response
- [ ] SSE headers (text/event-stream, cache-control) correctly set
- [ ] Session management integrated with streaming responses
- [ ] Connection recovery works with `Last-Event-ID`
- [ ] All existing functionality preserved (POST `/mcp` continues working)
- [ ] Integration tests demonstrate SSE streaming capability
- [ ] Zero compilation warnings maintained

## Dependencies
- **Requires**: Existing session management (`SessionManager`)
- **Requires**: Existing connection management (`HttpConnectionManager`)
- **Requires**: Existing JSON-RPC processing (`ConcurrentProcessor`)
- **Builds on**: Existing `/mcp` POST endpoint infrastructure

## Progress Log
### 2025-08-26
- Task created with technical specifications
- Implementation plan developed based on existing infrastructure
- Ready for implementation with clear success criteria
