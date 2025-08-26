# [TASK024] - HTTP Streamable Dynamic Mode Selection

**Status:** pending  
**Added:** 2025-08-26  
**Updated:** 2025-08-26  
**Priority:** MEDIUM - Required for full HTTP Streamable specification compliance

## Original Request
Implement dynamic response mode selection for the `/mcp` endpoint to automatically detect whether to return JSON responses or SSE streaming based on request characteristics and client preferences.

## Thought Process
The HTTP Streamable specification allows a single endpoint to handle different response modes:
1. **JSON Mode**: Direct JSON response for immediate request/response cycles
2. **Streaming Mode**: SSE streaming for long-running operations or client preference

The mode selection should be based on:
- HTTP method (POST = JSON, GET = SSE)
- Accept headers (application/json vs text/event-stream)
- Query parameters or custom headers for explicit mode selection
- Request content type and characteristics

## Implementation Plan
1. **Create ResponseModeSelector** utility for mode detection logic
2. **Enhance route handlers** to use dynamic mode selection
3. **Add mode detection logic** based on headers, method, and content
4. **Implement unified request processing** that works for both modes
5. **Add configuration options** for default mode and selection criteria
6. **Create comprehensive tests** for all mode selection scenarios

## Progress Tracking

**Overall Status:** pending - 0%

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 24.1 | Create ResponseModeSelector utility | not_started | 2025-08-26 | Mode detection logic based on request characteristics |
| 24.2 | Enhance route handlers for mode selection | not_started | 2025-08-26 | Unified handler that routes to JSON or SSE |
| 24.3 | Add Accept header detection | not_started | 2025-08-26 | Support application/json vs text/event-stream |
| 24.4 | Implement query parameter mode override | not_started | 2025-08-26 | ?mode=json or ?mode=stream explicit selection |
| 24.5 | Add configuration for default mode | not_started | 2025-08-26 | Server-wide default response mode setting |
| 24.6 | Write comprehensive mode selection tests | not_started | 2025-08-26 | Test all mode selection scenarios |

## Technical Specifications

### ResponseModeSelector Implementation
```rust
// New utility in axum/response_mode.rs
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseMode {
    Json,    // Direct JSON response
    Stream,  // SSE streaming response
}

pub struct ResponseModeSelector {
    default_mode: ResponseMode,
    allow_mode_override: bool,
}

impl ResponseModeSelector {
    pub fn detect_mode(
        &self,
        method: &Method,
        headers: &HeaderMap,
        query: &str,
    ) -> ResponseMode {
        // 1. Check explicit mode parameter: ?mode=json or ?mode=stream
        if let Some(mode) = self.extract_mode_from_query(query) {
            return mode;
        }
        
        // 2. Check Accept header preference
        if let Some(mode) = self.detect_from_accept_header(headers) {
            return mode;
        }
        
        // 3. Default based on HTTP method
        match method {
            &Method::POST => ResponseMode::Json,    // POST = JSON response
            &Method::GET => ResponseMode::Stream,   // GET = SSE streaming
            _ => self.default_mode.clone(),
        }
    }
}
```

### Unified Handler Architecture
```rust
// Enhanced handler in axum/handlers.rs
async fn handle_mcp_unified(
    State(state): State<ServerState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    method: Method,
    headers: HeaderMap,
    query: String,
    body: Option<String>,
) -> Result<Response<Body>, (StatusCode, String)> {
    // 1. Detect response mode
    let mode = state.response_mode_selector.detect_mode(&method, &headers, &query);
    
    // 2. Common session and connection setup
    let session_id = extract_or_create_session(&state, &headers, addr).await?;
    let connection_id = state.connection_manager.register_connection(addr).await?;
    
    // 3. Route to appropriate response handler
    match mode {
        ResponseMode::Json => handle_json_response(state, session_id, body).await,
        ResponseMode::Stream => handle_sse_response(state, session_id, connection_id).await,
    }
}
```

### Configuration Integration
```rust
// Enhanced ServerState in axum/handlers.rs
pub struct ServerState {
    pub connection_manager: Arc<HttpConnectionManager>,
    pub session_manager: Arc<SessionManager>,
    pub jsonrpc_processor: Arc<ConcurrentProcessor>,
    pub mcp_handlers: Arc<McpHandlers>,
    pub config: HttpTransportConfig,
    pub response_mode_selector: ResponseModeSelector,  // ADD THIS
}
```

### Query Parameter Support
```rust
// Query parameter parsing
// Examples:
// GET /mcp?mode=stream&session_id=abc123
// POST /mcp?mode=json (force JSON even for streaming-capable client)
// GET /mcp (default to streaming mode)
// POST /mcp (default to JSON mode)
```

## Success Criteria
- [ ] Single `/mcp` endpoint handles both JSON and SSE responses
- [ ] Mode selection works based on HTTP method (POST=JSON, GET=SSE)
- [ ] Accept header detection (application/json vs text/event-stream)
- [ ] Query parameter override (?mode=json or ?mode=stream)
- [ ] Configuration option for server default mode
- [ ] Comprehensive test coverage for all mode selection scenarios
- [ ] Backward compatibility maintained for existing clients
- [ ] Zero compilation warnings maintained

## Dependencies
- **Requires**: TASK023 (GET handler implementation)
- **Requires**: Existing POST `/mcp` endpoint
- **Requires**: Existing session and connection management
- **Builds on**: Unified endpoint architecture

## Configuration Examples
```rust
// Server configuration
let mode_selector = ResponseModeSelector::new()
    .default_mode(ResponseMode::Json)           // Server default
    .allow_mode_override(true)                  // Allow ?mode= parameter
    .prefer_accept_header(true);                // Respect Accept headers

// Usage examples:
// curl -X POST /mcp -H "Content-Type: application/json" {...}     → JSON response
// curl -X GET /mcp -H "Accept: text/event-stream"                 → SSE streaming  
// curl -X GET /mcp?mode=json                                      → JSON response (override)
// curl -X POST /mcp?mode=stream                                   → SSE streaming (override)
```

## Progress Log
### 2025-08-26
- Task created with comprehensive mode selection specification
- Architecture defined for unified endpoint handling
- Ready for implementation after TASK023 completion
