//! Transport Adapters
//!
//! This module provides adapters that bridge between different transport
//! paradigms, enabling gradual migration and backward compatibility.
//!
//! # Architecture
//!
//! Adapters bridge legacy transport implementations (blocking receive/send patterns)
//! with MCP-compliant transport interfaces (event-driven MessageHandler callbacks).
//!
//! ## Available Adapters
//!
//! - **StdioTransportAdapter**: Bridges legacy StdioTransport with event-driven MCP Transport
//! - **HTTP Adapters**: HttpServerTransport and HttpClientTransport adapters
//! - **WebSocket Adapters** (Future): WebSocket transport adapters
//!
//! # Design Pattern
//!
//! All adapters follow the same pattern:
//! 1. **Wrap Legacy Transport**: Contains the original transport implementation
//! 2. **Event Loop Bridge**: Background task converts blocking operations to events
//! 3. **MCP Interface**: Implements the MCP-compliant Transport trait
//! 4. **Backward Compatibility**: Existing APIs continue to work unchanged
//!
//! # Usage
//!
//! ```rust
//! use airs_mcp::transport::adapters::StdioTransportAdapter;
//! use airs_mcp::transport::mcp::{Transport, MessageHandler};
//! use std::sync::Arc;
//!
//! # struct MyHandler;
//! # #[async_trait::async_trait]
//! # impl MessageHandler for MyHandler {
//! #     async fn handle_message(&self, message: airs_mcp::transport::mcp::JsonRpcMessage, context: airs_mcp::transport::mcp::MessageContext) {}
//! #     async fn handle_error(&self, error: airs_mcp::transport::mcp::TransportError) {}
//! #     async fn handle_close(&self) {}
//! # }
//! #
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create adapter (same API as legacy StdioTransport)
//!     let mut transport = StdioTransportAdapter::new().await?;
//!     
//!     // Set event-driven message handler
//!     let handler = Arc::new(MyHandler);
//!     transport.set_message_handler(handler);
//!     
//!     // Start event-driven processing
//!     transport.start().await?;
//!     
//!     // Send messages (same API as before)
//!     let message = airs_mcp::transport::mcp::JsonRpcMessage::new_notification("ping", None);
//!     transport.send(message).await?;
//!     
//!     // Transport automatically calls handler.handle_message() for incoming messages
//!     // No blocking receive() calls needed
//!     
//!     // Clean shutdown
//!     transport.close().await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod http;
pub mod stdio;

// Re-exports for convenience
pub use stdio::{StdioTransport, StdioTransportConfig};

// HTTP transport adapters (MCP-compliant event-driven interfaces)
pub use http::{HttpClientTransportAdapter, HttpServerTransportAdapter};

// HTTP transport re-exports (maintaining backward compatibility)
// Note: Complex types temporarily disabled for MCP-compliant simplification
pub use http::{
    cache_control, content_types, events, extract_last_event_id, extract_session_id, headers,
    /*AxumHttpServer,*/ BufferPool, BufferPoolStats, BufferStrategy, ConnectionHealth,
    ConnectionId, ConnectionInfo, ConnectionStats, ConnectionStatsSnapshot, DeprecationConfig,
    DeprecationPhase, HealthCheckConfig, HealthCheckResult, HttpClientTransport,
    HttpConnectionManager, HttpSseConfig, HttpTransportConfig, MigrationMode, PooledBuffer,
    RequestParser, ServerState, SessionConfig, SessionId, SessionManager, SessionStatsSnapshot,
    SseEndpointConfig, DEFAULT_MESSAGES_ENDPOINT, DEFAULT_SSE_ENDPOINT,
};

// Legacy transport re-exports (will be replaced by adapters)
pub use http::HttpServerTransport;
