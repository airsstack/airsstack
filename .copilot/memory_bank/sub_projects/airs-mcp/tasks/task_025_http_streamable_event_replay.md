# [TASK025] - HTTP Streamable Event Replay & Connection Recovery

**Status:** pending  
**Added:** 2025-08-26  
**Updated:** 2025-08-26  
**Priority:** MEDIUM - Required for production-grade HTTP Streamable reliability

## Original Request
Implement event replay and connection recovery mechanisms for HTTP Streamable transport using `Last-Event-ID` headers to enable seamless reconnection and message delivery guarantees.

## Thought Process
HTTP Streamable specification includes connection recovery to handle:
1. **Network interruptions**: Client reconnects after temporary network issues
2. **Server restarts**: Client reconnects with last known event ID
3. **Message delivery guarantees**: Replay missed events during disconnection
4. **Session continuity**: Maintain session state across reconnections

The implementation should:
- Store recent events with sequence IDs for replay capability
- Use `Last-Event-ID` header to determine replay starting point
- Integrate with existing session management for state continuity
- Provide configurable replay buffer size and retention policies

## Implementation Plan
1. **Create EventBuffer** for storing recent events with sequence IDs
2. **Enhance session management** to track last delivered event ID
3. **Implement replay logic** for connection recovery scenarios
4. **Add Last-Event-ID header processing** in GET handler
5. **Create event sequence management** for reliable delivery tracking
6. **Add buffer configuration** and cleanup policies
7. **Write comprehensive recovery tests** for all reconnection scenarios

## Progress Tracking

**Overall Status:** pending - 0%

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 25.1 | Create EventBuffer for replay storage | not_started | 2025-08-26 | Ring buffer with configurable size and retention |
| 25.2 | Enhance SessionContext for event tracking | not_started | 2025-08-26 | Track last_delivered_event_id in session state |
| 25.3 | Implement Last-Event-ID header processing | not_started | 2025-08-26 | Extract and validate event ID for replay start |
| 25.4 | Add event sequence management | not_started | 2025-08-26 | Generate and track sequential event IDs |
| 25.5 | Create replay logic for missed events | not_started | 2025-08-26 | Query buffer and stream missed events on reconnect |
| 25.6 | Add buffer cleanup and retention policies | not_started | 2025-08-26 | Configurable buffer size and event expiration |
| 25.7 | Write connection recovery tests | not_started | 2025-08-26 | Test reconnection scenarios and event replay |

## Technical Specifications

### EventBuffer Implementation
```rust
// New module: transport/http/event_buffer.rs
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event_id: u64,           // Sequential event ID
    pub event_type: String,      // SSE event type
    pub data: String,            // Event data (JSON)
    pub retry: Option<u32>,      // Reconnect delay hint
    pub timestamp: DateTime<Utc>, // Event creation time
}

pub struct EventBuffer {
    buffer: Arc<RwLock<VecDeque<SseEvent>>>,
    max_events: usize,           // Maximum events to retain
    retention_duration: Duration, // Maximum event age
    next_event_id: Arc<AtomicU64>, // Next event ID to assign
}

impl EventBuffer {
    pub async fn store_event(&self, event_type: String, data: String) -> u64 {
        // Store event with sequential ID
    }
    
    pub async fn get_events_since(&self, last_event_id: Option<u64>) -> Vec<SseEvent> {
        // Return events after the specified ID for replay
    }
    
    pub async fn cleanup_old_events(&self) {
        // Remove events older than retention_duration
    }
}
```

### Enhanced Session Context
```rust
// Update transport/http/session.rs
#[derive(Debug, Clone)]
pub struct SessionContext {
    pub session_id: SessionId,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub client_info: ClientInfo,
    pub metadata: SessionMetadata,
    pub last_delivered_event_id: Option<u64>,  // ADD THIS
}

impl SessionManager {
    pub fn update_last_delivered_event(&self, session_id: SessionId, event_id: u64) -> Result<(), TransportError> {
        // Update session with last successfully delivered event ID
    }
    
    pub fn get_last_delivered_event(&self, session_id: SessionId) -> Option<u64> {
        // Get last event ID for replay starting point
    }
}
```

### Connection Recovery Logic
```rust
// Enhanced GET handler in axum/handlers.rs
async fn handle_mcp_get_with_recovery(
    State(state): State<ServerState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Response<Body>, (StatusCode, String)> {
    // 1. Extract session and Last-Event-ID
    let session_id = extract_or_create_session(&state, &headers, addr).await?;
    let last_event_id = extract_last_event_id(&headers); // Existing function
    
    // 2. Determine replay starting point
    let replay_from = match last_event_id {
        Some(id) => Some(id.parse::<u64>().unwrap_or(0)),
        None => state.session_manager.get_last_delivered_event(session_id),
    };
    
    // 3. Get missed events for replay
    let missed_events = state.event_buffer.get_events_since(replay_from).await;
    
    // 4. Create stream that replays missed events first, then continues with live events
    let stream = create_recovery_stream(state, session_id, missed_events).await?;
    
    // 5. Return SSE response with proper headers
    Ok(Response::builder()
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache") 
        .header("connection", "keep-alive")
        .header("Mcp-Session-Id", session_id.to_string())
        .body(Body::from_stream(stream))?)
}
```

### Event Sequence Management
```rust
// Event streaming with sequence tracking
async fn create_recovery_stream(
    state: ServerState,
    session_id: SessionId,
    missed_events: Vec<SseEvent>,
) -> Result<impl Stream<Item = Result<Bytes, Error>>, TransportError> {
    // 1. Create stream that first replays missed events
    // 2. Then transitions to live event streaming
    // 3. Updates session last_delivered_event_id for each event
    // 4. Handles connection cleanup on stream termination
}
```

### Configuration Options
```rust
// Enhanced HttpTransportConfig
pub struct HttpTransportConfig {
    // ... existing fields
    pub event_buffer_size: usize,           // Default: 1000 events
    pub event_retention_duration: Duration, // Default: 1 hour
    pub enable_event_replay: bool,          // Default: true
}
```

## Success Criteria
- [ ] EventBuffer stores recent events with sequential IDs
- [ ] Last-Event-ID header processing works correctly
- [ ] Connection recovery replays missed events on reconnect
- [ ] Session state tracks last delivered event ID
- [ ] Buffer cleanup removes old events based on retention policy
- [ ] Comprehensive tests for all reconnection scenarios:
  - [ ] Client reconnects after network interruption
  - [ ] Client reconnects after server restart
  - [ ] Event replay starts from correct sequence point
  - [ ] No duplicate event delivery
  - [ ] Buffer overflow handling
- [ ] Configurable buffer size and retention settings
- [ ] Zero compilation warnings maintained

## Dependencies
- **Requires**: TASK023 (GET handler implementation)
- **Requires**: Existing session management (`SessionManager`)
- **Requires**: Existing `extract_last_event_id` function
- **Builds on**: HTTP Streamable SSE streaming infrastructure

## Recovery Scenarios
```rust
// Example recovery scenarios:

// Scenario 1: Clean reconnection
// Client: GET /mcp -H "Last-Event-ID: 42"
// Server: Replay events 43, 44, 45... then continue live

// Scenario 2: New session
// Client: GET /mcp (no Last-Event-ID header)
// Server: Start fresh stream, no replay needed

// Scenario 3: Invalid Last-Event-ID
// Client: GET /mcp -H "Last-Event-ID: invalid"
// Server: Start from session's last known event or fresh stream

// Scenario 4: Very old Last-Event-ID
// Client: GET /mcp -H "Last-Event-ID: 1" (buffer only has events 100+)
// Server: Start from oldest available event (100) with gap warning
```

## Progress Log
### 2025-08-26
- Task created with comprehensive connection recovery specification
- Architecture defined for event buffer and replay mechanisms
- Ready for implementation after TASK023 completion
