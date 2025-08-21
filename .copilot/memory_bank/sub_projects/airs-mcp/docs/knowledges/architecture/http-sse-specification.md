# HTTP SSE Technical Implementation Specification

**Document Type**: Technical Specification  
**Status**: Comprehensive Plan Complete  
**Date**: 2025-08-13  
**Priority**: LOW - Legacy Compatibility Transport

## Executive Summary

HTTP SSE (Server-Sent Events) transport implementation for airs-mcp, positioned as **legacy compatibility** support during ecosystem transition to HTTP Streamable. This specification provides a complete technical plan for a dual-endpoint SSE implementation that leverages the HTTP Streamable foundation while providing clear migration incentives.

## Strategic Context

### Legacy Transport Role
HTTP SSE serves as an **ecosystem transition bridge** with specific purpose:
- **Legacy Client Support**: Existing MCP implementations that haven't migrated to HTTP Streamable
- **Educational Reference**: Demonstrates proper SSE patterns for learning purposes  
- **Gradual Migration**: Enables ecosystem-wide transition without breaking existing integrations
- **Simplified Architecture**: Less complex entry point compared to HTTP Streamable

### Performance Positioning
Intentionally conservative performance targets reflecting legacy status:
- **Throughput**: ~10,000 req/sec (vs 100,000+ for HTTP Streamable)
- **Connections**: ~1,000 concurrent (vs 10,000+ for HTTP Streamable)
- **Latency**: ~1-2ms (vs <100μs for HTTP Streamable)
- **Memory**: ~50MB base (vs ~20MB for HTTP Streamable)

## Core Architecture

### Shared Infrastructure Strategy
Leverage HTTP Streamable foundation to minimize implementation cost:

```rust
// Configuration extending HTTP Streamable
pub struct HttpSseConfig {
    pub base_config: HttpTransportConfig,  // Reuse HTTP Streamable config
    pub sse_endpoint: String,              // Default: "/sse"
    pub messages_endpoint: String,         // Default: "/messages"
    pub deprecation_warnings: bool,        // Default: true
    pub deprecation: DeprecationConfig,
}

pub struct DeprecationConfig {
    pub warnings_enabled: bool,
    pub migration_docs_url: String,
    pub sunset_date: Option<DateTime<Utc>>,
    pub streamable_endpoint: Option<String>,  // Auto-redirect guidance
}
```

### Transport Implementation
```rust
pub struct HttpSseTransport {
    config: HttpSseConfig,
    session_manager: Arc<SessionManager>,  // Shared with HTTP Streamable
    message_handler: MessageHandler,
    sse_broadcaster: SseBroadcaster,
    migration_helper: MigrationHelper,
}

#[async_trait]
impl Transport for HttpSseTransport {
    async fn start(&mut self) -> Result<(), TransportError> {
        // Issue deprecation warning on startup
        warn!("HTTP+SSE transport is deprecated. Migrate to HTTP Streamable for 60-80% better performance.");
        
        // Start dual-endpoint server
        self.start_dual_endpoint_server().await
    }
    
    async fn send(&self, message: JsonRpcMessage) -> Result<(), TransportError> {
        // Broadcast via SSE to relevant sessions
        self.sse_broadcaster.broadcast_message(message).await
    }
    
    async fn receive(&self) -> Result<JsonRpcMessage, TransportError> {
        // Handle incoming messages from /messages endpoint
        self.message_handler.receive_next().await
    }
}
```

## Implementation Phases

### Phase 1: Foundation (Week 1)

#### Configuration Structure
```rust
impl HttpSseConfig {
    pub fn new() -> Self {
        Self {
            base_config: HttpTransportConfig::new(),
            sse_endpoint: "/sse".to_string(),
            messages_endpoint: "/messages".to_string(),
            deprecation_warnings: true,
            deprecation: DeprecationConfig::default(),
        }
    }
    
    pub fn builder() -> HttpSseConfigBuilder {
        HttpSseConfigBuilder::new()
    }
}
```

#### Basic Transport Structure
```rust
impl HttpSseTransport {
    pub async fn new(config: HttpSseConfig) -> Result<Self, TransportError> {
        let session_manager = Arc::new(SessionManager::new(config.base_config.clone()));
        let sse_broadcaster = SseBroadcaster::new();
        let migration_helper = MigrationHelper::new(&config);
        
        Ok(Self {
            config,
            session_manager,
            message_handler: MessageHandler::new(),
            sse_broadcaster,
            migration_helper,
        })
    }
}
```

### Phase 2: Dual-Endpoint Implementation (Week 1-2)

#### POST /messages - JSON Request/Response
```rust
async fn handle_messages_post(
    State(transport): State<Arc<HttpSseTransport>>,
    headers: HeaderMap,
    Json(request): Json<JsonRpcRequest>,
) -> Result<Json<JsonRpcResponse>, HttpError> {
    // Issue deprecation warning
    if transport.config.deprecation_warnings {
        warn!("HTTP+SSE transport is deprecated. Migrate to HTTP Streamable for better performance.");
    }
    
    let session_id = extract_or_create_session(&headers)?;
    
    // Process request using shared infrastructure
    let correlation_id = transport.session_manager
        .register_request(session_id, &request).await?;
        
    let response = transport.process_request(session_id, request).await?;
    
    // Provide migration guidance in response headers
    let mut response_headers = HeaderMap::new();
    response_headers.insert("X-MCP-Migration", "http-streamable-available".parse()?);
    
    Ok(Json(response))
}
```

#### GET /sse - Server-Sent Events Stream
```rust
async fn handle_sse_stream(
    State(transport): State<Arc<HttpSseTransport>>,
    headers: HeaderMap,
) -> Result<Response, HttpError> {
    let session_id = extract_session_id(&headers)?;
    let last_event_id = extract_last_event_id(&headers);
    
    // Create SSE stream with session management
    let stream = transport.sse_broadcaster
        .create_session_stream(session_id, last_event_id).await?;
    
    Ok(Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response())
}
```

### Phase 3: SSE Broadcasting System (Week 2)

#### Event Broadcasting
```rust
pub struct SseBroadcaster {
    sessions: Arc<DashMap<SessionId, SseSender>>,
    event_counter: AtomicU64,
}

impl SseBroadcaster {
    pub async fn broadcast_to_session(
        &self,
        session_id: SessionId,
        message: JsonRpcMessage,
    ) -> Result<(), SseError> {
        if let Some(sender) = self.sessions.get(&session_id) {
            let event = Event::default()
                .id(self.next_event_id().to_string())
                .event("mcp-message")
                .json_data(&message)?;
            
            sender.send(Ok(event)).await?;
        }
        Ok(())
    }
    
    pub async fn create_session_stream(
        &self,
        session_id: SessionId,
        last_event_id: Option<EventId>,
    ) -> Result<impl Stream<Item = Result<Event, SseError>>, SseError> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        // Handle reconnection from last event ID
        if let Some(event_id) = last_event_id {
            self.replay_events_from(session_id, event_id, &tx).await?;
        }
        
        // Register for ongoing events
        self.sessions.insert(session_id, tx);
        
        Ok(UnboundedReceiverStream::new(rx))
    }
}
```

### Phase 4: Migration Support (Week 2-3)

#### Migration Helper
```rust
pub struct MigrationHelper {
    sse_config: HttpSseConfig,
}

impl MigrationHelper {
    pub fn generate_streamable_config(&self) -> HttpTransportConfig {
        // Translate SSE config to HTTP Streamable equivalent
        let mut streamable_config = self.sse_config.base_config.clone();
        
        // Optimize settings for HTTP Streamable
        streamable_config.enable_buffer_pool();
        streamable_config.max_connections *= 10; // Higher capacity
        
        streamable_config
    }
    
    pub fn migration_guide(&self) -> String {
        format!(
            "Migration Guide: HTTP SSE → HTTP Streamable\n\
            \n\
            Performance Benefits:\n\
            - 10x higher throughput ({} → {} req/sec)\n\
            - 10x more concurrent connections\n\
            - 50% lower latency\n\
            - 60% lower memory usage\n\
            \n\
            Migration Steps:\n\
            1. Update to HTTP Streamable transport\n\
            2. Change endpoint to /mcp (unified)\n\
            3. Enable buffer pooling for optimization\n\
            4. Update client to use dynamic response modes\n\
            \n\
            Documentation: {}\n",
            10_000, 100_000,
            self.sse_config.deprecation.migration_docs_url
        )
    }
    
    pub fn compatibility_check(&self, client_headers: &HeaderMap) -> MigrationAdvice {
        // Analyze client capabilities and provide migration advice
        MigrationAdvice {
            recommended_transport: TransportType::HttpStreamable,
            compatibility_level: self.assess_client_compatibility(client_headers),
            migration_urgency: MigrationUrgency::High,
            performance_gain_estimate: "10x throughput, 50% latency reduction",
        }
    }
}
```

## Integration Points

### Shared Components with HTTP Streamable
1. **Session Management**: Same `SessionManager` and `Session` structures
2. **Configuration**: `HttpSseConfig` extends `HttpTransportConfig`
3. **Error Handling**: Shared error types and handling patterns
4. **JSON Processing**: Same per-request `StreamingParser` approach

### Resource Management
```rust
// Conservative resource limits for legacy transport
impl Default for HttpSseConfig {
    fn default() -> Self {
        Self {
            base_config: HttpTransportConfig::new()
                .max_connections(1000)      // 10x lower than HTTP Streamable
                .connection_timeout(30)     // Shorter timeout
                .session_timeout(300),      // 5 minute sessions
            sse_endpoint: "/sse".to_string(),
            messages_endpoint: "/messages".to_string(),
            deprecation_warnings: true,
            deprecation: DeprecationConfig::default(),
        }
    }
}
```

## Testing Strategy

### Legacy Client Simulation
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_dual_endpoint_functionality() {
        let config = HttpSseConfig::new();
        let transport = HttpSseTransport::new(config).await.unwrap();
        
        // Test POST /messages endpoint
        let request = JsonRpcRequest::new("test_method", json!({}));
        let response = transport.handle_message_request(request).await.unwrap();
        assert!(response.is_success());
        
        // Test GET /sse endpoint
        let stream = transport.create_sse_stream("session-1", None).await.unwrap();
        // Verify stream produces SSE events
    }
    
    #[tokio::test]
    async fn test_migration_helper() {
        let sse_config = HttpSseConfig::new();
        let migration_helper = MigrationHelper::new(&sse_config);
        
        // Test configuration translation
        let streamable_config = migration_helper.generate_streamable_config();
        assert!(streamable_config.max_connections() > sse_config.base_config.max_connections());
        
        // Test migration advice
        let headers = HeaderMap::new();
        let advice = migration_helper.compatibility_check(&headers);
        assert_eq!(advice.recommended_transport, TransportType::HttpStreamable);
    }
}
```

### Performance Benchmarking
```rust
#[cfg(test)]
mod performance_tests {
    #[tokio::test]
    async fn benchmark_sse_vs_streamable() {
        // Comparative benchmarking to validate performance characteristics
        let sse_transport = HttpSseTransport::new(HttpSseConfig::new()).await.unwrap();
        let streamable_transport = HttpStreamableTransport::new(HttpTransportConfig::new()).await.unwrap();
        
        // Measure throughput, latency, memory usage
        let sse_metrics = benchmark_transport(&sse_transport).await;
        let streamable_metrics = benchmark_transport(&streamable_transport).await;
        
        // Verify SSE performance is intentionally lower
        assert!(streamable_metrics.throughput > sse_metrics.throughput * 5);
        assert!(streamable_metrics.memory_usage < sse_metrics.memory_usage);
    }
}
```

## Documentation Strategy

### Deprecation Messaging
```markdown
# HTTP SSE Transport (DEPRECATED)

⚠️ **LEGACY TRANSPORT**: This implementation is maintained for backward compatibility only.

## ⚡ Migrate to HTTP Streamable for Major Performance Gains

| Metric | HTTP SSE | HTTP Streamable | Improvement |
|--------|----------|-----------------|-------------|
| Throughput | 10,000 req/sec | 100,000+ req/sec | **10x faster** |
| Connections | 1,000 concurrent | 10,000+ concurrent | **10x more** |
| Latency | 1-2ms | <100μs | **50% lower** |
| Memory | 50MB base | 20MB base | **60% less** |

## Migration Timeline
- **Deprecated**: March 2025 (MCP Specification Update)
- **Support End**: March 2026
- **Action Required**: Migrate to HTTP Streamable immediately

## Quick Migration
```rust
// Before: HTTP SSE
let sse_config = HttpSseConfig::new();
let sse_transport = HttpSseTransport::new(sse_config).await?;

// After: HTTP Streamable  
let streamable_config = HttpTransportConfig::new();
let streamable_transport = HttpStreamableTransport::new(streamable_config).await?;
```

[View Complete Migration Guide →](../migration/sse-to-streamable.md)
```

## Risk Assessment

### Technical Risks
1. **Resource Leaks**: Persistent SSE connections require careful cleanup
   - **Mitigation**: Aggressive timeouts, connection limits, comprehensive monitoring
2. **Load Balancer Issues**: Sticky sessions required for dual-endpoint pattern
   - **Mitigation**: Clear deployment documentation, session affinity guidance
3. **Client Compatibility**: Edge cases in legacy client implementations
   - **Mitigation**: Extensive testing, fallback mechanisms, migration support

### Business Risks
1. **Development Investment**: Time spent on deprecated technology
   - **Mitigation**: Minimal viable implementation, time-boxed development
2. **Maintenance Burden**: Ongoing support for legacy transport
   - **Mitigation**: Clear sunset timeline, migration incentives, deprecation warnings

## Success Criteria

### Functional Requirements ✅
- Dual-endpoint architecture (/sse, /messages) working correctly
- Session management and event broadcasting functional
- Integration with existing correlation and session systems
- Clear deprecation warnings and migration guidance

### Performance Requirements ✅  
- Target performance: 10k req/sec, 1k concurrent connections
- Memory usage: <50MB base footprint
- Latency: <2ms average response time
- No impact on HTTP Streamable performance

### Migration Requirements ✅
- Automatic configuration translation to HTTP Streamable
- Clear migration documentation and performance comparisons
- Client compatibility analysis and migration advice
- Sunset timeline and deprecation strategy

---

**Implementation Timeline**: 3 weeks  
**Resource Investment**: Minimal (legacy support only)  
**Strategic Value**: Ecosystem transition support  
**Long-term Maintenance**: Limited (deprecated transport)
