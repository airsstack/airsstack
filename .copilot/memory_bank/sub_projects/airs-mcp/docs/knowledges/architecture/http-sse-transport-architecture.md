# HTTP SSE Transport Architecture

**Document Type**: Knowledge Documentation  
**Category**: Architecture  
**Complexity**: High  
**Created**: 2025-08-26  
**Updated**: 2025-08-26  
**Status**: Active  
**Related Tasks**: TASK013 (HTTP SSE Implementation)  
**Related ADRs**: ADR-001 (Transport abstraction), ADR-002 (HTTP architecture)  

## Executive Summary

HTTP Server-Sent Events (SSE) transport implementation designed as **legacy compatibility layer** for MCP ecosystem transition. Built on shared infrastructure with HTTP Streamable transport while providing dual-endpoint SSE architecture for existing clients requiring Server-Sent Events support.

**Strategic Position**: Transitional technology to support ecosystem migration from SSE to superior HTTP Streamable transport.

## Architectural Overview

### Design Philosophy

**Core Principles**:
- **Shared Infrastructure**: Maximum reuse of HTTP Streamable foundation
- **Legacy Compatibility**: Support existing MCP implementations using SSE
- **Built-in Deprecation**: Migration guidance and sunset planning
- **Simplified Implementation**: Intentionally less optimized than HTTP Streamable

**Strategic Context**:
- **Ecosystem Bridge**: Enables gradual migration without breaking integrations
- **Educational Value**: Demonstrates proper SSE patterns for learning
- **Performance Trade-off**: Deliberately conservative performance targets

### Dual-Endpoint Architecture

```
┌─────────────────────┐    POST /messages     ┌─────────────────────┐
│                     │ ─────────────────────► │                     │
│   MCP Client        │                        │   HTTP SSE Server   │
│   (Legacy)          │ ◄───────────────────── │                     │
└─────────────────────┘    GET /sse            └─────────────────────┘
                           (SSE Stream)

Flow:
1. Client posts JSON-RPC message to /messages endpoint
2. Server processes request and creates/updates session
3. Server broadcasts response via SSE to /sse endpoint
4. Client receives response through SSE event stream
```

**Endpoint Responsibilities**:
- **POST /messages**: JSON request/response with session creation/correlation
- **GET /sse**: Server-Sent Events streaming with session-based event delivery

## Module Structure Design

### Directory Architecture
```
crates/airs-mcp/src/transport/http/
├── mod.rs                    # Add SSE exports
├── config.rs                 # Extend with SSE config
├── session.rs                # Reuse existing session management
├── sse/                      # New SSE-specific module
│   ├── mod.rs               # SSE module exports
│   ├── config.rs            # SSE-specific configuration
│   ├── transport.rs         # HttpSseTransport implementation
│   ├── broadcaster.rs       # SseBroadcaster for event distribution
│   ├── endpoints.rs         # Dual-endpoint handlers
│   ├── migration.rs         # Migration helper utilities
│   ├── deprecation.rs       # Deprecation tracking and warnings
│   └── events.rs           # SSE event formatting and types
└── axum/
    ├── mod.rs
    ├── server.rs
    └── sse_handlers.rs       # New SSE endpoint handlers for Axum
```

### Component Architecture

#### Core Components
```rust
// Primary transport implementation
pub struct HttpSseTransport {
    config: HttpSseConfig,                    // SSE-specific configuration
    session_manager: Arc<SessionManager>,     // Shared with HTTP Streamable
    correlation_manager: Arc<CorrelationManager>, // Request/response matching
    broadcaster: Arc<SseBroadcaster>,         // Event distribution system
    deprecation_tracker: DeprecationTracker, // Migration guidance
}

// Event broadcasting system
pub struct SseBroadcaster {
    sessions: Arc<DashMap<SessionId, SseSession>>, // Active SSE connections
    formatter: EventFormatter,                     // SSE protocol formatting
    stats: Arc<BroadcastStats>,                    // Monitoring metrics
}

// Configuration extending HTTP Streamable
pub struct HttpSseConfig {
    base_config: HttpTransportConfig,     // Shared foundation
    sse_endpoint: SseEndpointConfig,      // SSE-specific settings
    messages_endpoint: String,            // JSON endpoint path
    deprecation: DeprecationConfig,       // Sunset planning
    migration_mode: MigrationMode,        // Migration assistance level
}
```

## Shared Infrastructure Integration

### Reused Components

**SessionManager**: Complete reuse of HTTP Streamable session management
```rust
// Shared session management infrastructure
let session_manager = Arc::new(SessionManager::new(
    config.base_config.session_config.clone()
));

// Sessions work identically between transports
pub struct SessionContext {
    pub session_id: SessionId,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub client_info: ClientInfo,
    pub metadata: SessionMetadata,
}
```

**CorrelationManager**: Shared request/response correlation system
```rust
// Same correlation patterns as HTTP Streamable
let correlation_manager = Arc::new(CorrelationManager::new(
    config.base_config.correlation_config.clone()
));
```

**Configuration Base**: Extension rather than duplication
```rust
// SSE config extends HTTP Streamable config
impl HttpTransportConfig {
    pub fn to_sse_config(self) -> HttpSseConfig {
        HttpSseConfig {
            base_config: self,  // Complete HTTP foundation
            // ... SSE-specific additions
        }
    }
}
```

### Integration Points

**Error Handling**: Consistent error types across transports
**JSON Processing**: Same StreamingParser (per-request, no pooling)
**Metrics Collection**: Shared monitoring and statistics infrastructure
**Workspace Standards**: Same 3-layer import organization, chrono DateTime<Utc>

## Performance Architecture

### Intentional Performance Characteristics

**Conservative Targets** (Legacy compatibility focus):
```
HTTP SSE (Legacy)          vs    HTTP Streamable (Modern)
~10,000 req/sec           vs    ~100,000 req/sec
~1,000 concurrent         vs    ~10,000+ concurrent  
~1-2ms latency           vs    ~<100μs latency
~50MB base memory        vs    ~20MB base memory
```

**Performance Design Decisions**:
- **No Buffer Pooling**: Simpler per-request allocation
- **Basic Session Management**: Lower optimization than HTTP Streamable
- **Traditional Patterns**: Standard SSE implementation without advanced optimizations
- **Resource Overhead**: Dual-endpoint connection pools and persistent SSE connections

### Resource Management Strategy

**Connection Management**:
- Lower default connection limits than HTTP Streamable
- Aggressive timeout configuration for resource conservation
- Persistent SSE connections require careful cleanup

**Memory Allocation**:
- Per-request JSON parsing (no pooled buffers)
- Event buffer management per SSE session
- Session correlation overhead between endpoints

## Migration Strategy Architecture

### Built-in Migration Support

**Migration Helper System**:
```rust
pub struct MigrationHelper {
    config_translator: ConfigTranslator,      // Config conversion
    performance_analyzer: PerformanceAnalyzer, // Benefit analysis
    compatibility_checker: CompatibilityChecker, // Client assessment
}

// Automatic configuration translation
pub fn generate_streamable_config(sse_config: &HttpSseConfig) -> HttpTransportConfig {
    let mut streamable_config = sse_config.base_config.clone();
    
    // Optimize for streamable performance
    streamable_config.max_connections *= 10;  // Higher capacity
    streamable_config.enable_buffer_pool = true;  // Performance optimization
    
    streamable_config
}
```

**Deprecation Management**:
```rust
pub struct DeprecationConfig {
    warnings_enabled: bool,                   // Response header warnings
    sunset_date: Option<DateTime<Utc>>,       // Planned deprecation date
    migration_docs_url: String,               // Migration documentation
    warning_frequency: Duration,              // Warning throttling
}

pub enum MigrationMode {
    Silent,        // No migration assistance
    Passive,       // Headers/response hints
    Active,        // Migration suggestions
    Aggressive,    // Strong migration promotion
}
```

## Risk Mitigation Architecture

### Technical Risk Management

**Resource Leak Prevention**:
- Aggressive timeout configuration for SSE connections
- Automatic session cleanup with conservative intervals
- Connection limit enforcement below HTTP Streamable

**Load Balancer Compatibility**:
- Session affinity requirement documentation
- Sticky session configuration guides
- Dual-endpoint session correlation patterns

**Legacy Client Edge Cases**:
- Comprehensive compatibility testing framework
- Fallback mechanisms for problematic clients
- SSE reconnection handling

### Architectural Risk Controls

**Performance Isolation**:
- Separate resource pools from HTTP Streamable
- Independent metrics collection
- Isolated configuration management

**Maintenance Burden Limitation**:
- Clear sunset timeline documentation
- Minimal feature set (no advanced optimizations)
- Built-in migration incentives

## Implementation Quality Standards

### Code Quality Requirements

**Workspace Standards Compliance**:
- 3-layer import organization (std → third-party → internal)
- chrono DateTime<Utc> for all time operations
- Clean mod.rs organization (exports only, no implementation)
- AIRS foundation crate prioritization in dependencies

**Testing Requirements**:
- Unit tests for all SSE-specific components
- Integration tests with legacy client simulation
- Migration scenario validation testing
- Resource leak detection and cleanup verification

**Documentation Standards**:
- Clear deprecation messaging and timeline
- Migration guides with step-by-step instructions
- Performance comparison documentation
- Example implementations for common scenarios

## Future Considerations

### Sunset Planning

**Timeline Management**:
- Built-in sunset date tracking in configuration
- Automatic warning escalation as sunset approaches
- Migration progress monitoring and reporting

**Ecosystem Transition Support**:
- Client compatibility analysis tools
- Migration assistance automation
- Performance benefit documentation

### Maintenance Strategy

**Minimal Evolution**:
- No advanced feature additions (encourage migration instead)
- Security updates only as needed
- Documentation maintenance for migration support

**Success Metrics**:
- Migration rate to HTTP Streamable transport
- Legacy client compatibility coverage
- Resource usage efficiency compared to targets

---

This architectural documentation provides the complete technical foundation for HTTP SSE transport implementation, emphasizing shared infrastructure reuse, migration support, and strategic positioning as a transitional technology in the AIRS MCP ecosystem.
