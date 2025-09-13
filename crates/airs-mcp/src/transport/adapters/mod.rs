//! Transport Adapters - Self-Contained Transport Implementations
//!
//! This module provides self-contained transport implementations following the
//! Generic MessageHandler architecture (ADR-012) with clean module organization.
//!
//! # Architecture
//!
//! Each transport module is completely self-contained with:
//! - Transport-specific context structures (e.g., HttpContext for HTTP, () for STDIO)
//! - MessageHandler implementations using transport-specific contexts
//! - Transport and TransportBuilder implementations
//! - Configuration structures and type aliases
//! - No cross-dependencies between transport modules
//!
//! ## Available Transports
//!
//! - **STDIO Transport** (`stdio/`): Self-contained STDIO implementation with MessageHandler<()>
//! - **HTTP Transport** (`http/`): Self-contained HTTP implementation with MessageHandler<HttpContext>
//! - **Future Transports**: WebSocket, TCP, etc. will follow the same pattern
//!
//! # Generic MessageHandler Pattern
//!
//! All transports follow the Generic MessageHandler pattern:
//! 1. **Transport-Specific Context**: Each transport defines its own context type
//! 2. **Generic MessageHandler<T>**: Handlers receive transport-specific context data
//! 3. **Type Aliases**: Convenient aliases like `HttpMessageHandler` for clarity
//! 4. **Self-Contained Modules**: No dependencies between transport implementations
//!
//! # Usage Examples
//!
//! ## STDIO Transport (Simple Context)
//! ```rust
//! use airs_mcp::transport::adapters::{StdioMessageHandler, StdioTransport};
//! use airs_mcp::protocol::{MessageHandler, JsonRpcMessage, MessageContext};
//! use std::sync::Arc;
//!
//! struct MyStdioHandler;
//!
//! #[async_trait::async_trait]
//! impl MessageHandler<()> for MyStdioHandler {
//!     async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<()>) {
//!         // Handle message with unit context (no transport-specific data)
//!     }
//!     async fn handle_error(&self, error: airs_mcp::protocol::TransportError) {}
//!     async fn handle_close(&self) {}
//! }
//! ```
//!
//! ## HTTP Transport (Rich Context)
//! ```rust
//! use airs_mcp::transport::adapters::{HttpContext, HttpMessageHandler, HttpTransport};
//! use airs_mcp::protocol::{MessageHandler, JsonRpcMessage, MessageContext};
//! use std::sync::Arc;
//!
//! struct MyHttpHandler;
//!
//! #[async_trait::async_trait]
//! impl MessageHandler<HttpContext> for MyHttpHandler {
//!     async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<HttpContext>) {
//!         if let Some(http_ctx) = context.transport_data() {
//!             // Access HTTP-specific data: method, path, headers, etc.
//!             println!("HTTP {} {}", http_ctx.method(), http_ctx.path());
//!         }
//!     }
//!     async fn handle_error(&self, error: airs_mcp::protocol::TransportError) {}
//!     async fn handle_close(&self) {}
//! }
//! ```
//! ```

pub mod http;
pub mod stdio;

// Re-exports for convenience and backward compatibility
// Each transport module is self-contained with its own implementations

// STDIO Transport - Self-contained module
pub use stdio::{
    StdioMessageContext, StdioMessageHandler, StdioTransport, StdioTransportBuilder,
    StdioTransportConfig,
};

// HTTP Transport - Self-contained module
pub use http::{
    EchoHttpHandler, HttpConnectionManager, HttpContext, HttpMessageContext, HttpMessageHandler,
    HttpTransport, HttpTransportBuilder, HttpTransportConfig, McpHttpHandler, StaticFileHandler,
};

// HTTP Additional Exports (maintaining backward compatibility)
pub use http::{
    cache_control, content_types, events, headers, BufferPool, BufferPoolStats, BufferStrategy,
    ConnectionHealth, ConnectionId, ConnectionInfo, ConnectionStats, ConnectionStatsSnapshot,
    DeprecationConfig, DeprecationPhase, HealthCheckConfig, HealthCheckResult, HttpSseConfig,
    MigrationMode, PooledBuffer, RequestParser, SseEndpointConfig, DEFAULT_MESSAGES_ENDPOINT,
    DEFAULT_SSE_ENDPOINT,
};

// NOTE: Transport Adapters removed as part of Phase 5.5.6a architectural simplification
// Use direct transport implementations instead of bridge adapters

#[cfg(test)]
mod tests {
    //! Transport module organization validation tests
    //!
    //! These tests validate the self-contained module architecture where:
    //! 1. Each transport is completely self-contained
    //! 2. No cross-dependencies between transport modules
    //! 3. Clean re-exports from each module
    //! 4. Type aliases work correctly for convenience

    #[test]
    fn test_stdio_type_aliases_available() {
        // Verify STDIO type aliases are correctly exposed
        use crate::transport::adapters::{
            StdioMessageHandler, StdioTransport, StdioTransportConfig,
        };

        // These should compile without error, proving the type aliases work
        let _: Option<StdioTransport> = None;
        let _: Option<StdioTransportConfig> = None;
        let _: Option<&StdioMessageHandler> = None; // Use as reference to dyn trait
    }

    #[test]
    fn test_http_type_aliases_available() {
        // Verify HTTP type aliases are correctly exposed
        use crate::transport::adapters::http::axum::{AxumHttpServer, NoAuth};
        use crate::transport::adapters::http::HttpTransport;
        use crate::transport::adapters::{HttpContext, HttpMessageHandler};

        // These should compile without error, proving the type aliases work
        // Note: HttpTransport is now generic, so we use AxumHttpServer for testing
        let _: Option<HttpTransport<AxumHttpServer<NoAuth>>> = None;
        let _: Option<HttpContext> = None;
        let _: Option<&HttpMessageHandler> = None; // Use as reference to dyn trait
    }

    #[test]
    fn test_http_handler_examples_available() {
        // Verify HTTP handler examples are correctly exposed
        use crate::transport::adapters::{EchoHttpHandler, McpHttpHandler, StaticFileHandler};

        // These should compile without error, proving the handlers are available
        let _: Option<McpHttpHandler> = None;
        let _: Option<EchoHttpHandler> = None;
        let _: Option<StaticFileHandler> = None;
    }

    #[test]
    fn test_module_self_containment() {
        // This test validates that modules don't have circular dependencies
        // by ensuring we can use each module independently

        // STDIO module usage (completely independent)
        use crate::transport::adapters::stdio::StdioTransport;
        let _stdio_transport: Option<StdioTransport> = None;

        // HTTP module usage (completely independent)
        // Note: HttpTransport is now generic, so we use AxumHttpServer for testing
        use crate::transport::adapters::http::axum::{AxumHttpServer, NoAuth};
        use crate::transport::adapters::http::HttpTransport;
        let _http_transport: Option<HttpTransport<AxumHttpServer<NoAuth>>> = None;

        // Both modules should be usable without cross-dependencies
    }

    #[test]
    fn test_clean_re_exports() {
        // Verify all expected symbols are re-exported cleanly

        // Core transport types should be available
        use crate::transport::adapters::http::axum::{AxumHttpServer, NoAuth};
        use crate::transport::adapters::http::HttpTransport;
        use crate::transport::adapters::stdio::StdioTransport;
        // Note: HttpTransport is now generic, so we use AxumHttpServer for testing
        let _: Option<HttpTransport<AxumHttpServer<NoAuth>>> = None;
        let _: Option<StdioTransport> = None;

        // Context types should be available
        use crate::transport::adapters::HttpContext;
        let _: Option<HttpContext> = None;

        // Handler examples should be available
        use crate::transport::adapters::{EchoHttpHandler, McpHttpHandler, StaticFileHandler};
        let _: Option<McpHttpHandler> = None;
        let _: Option<EchoHttpHandler> = None;
        let _: Option<StaticFileHandler> = None;
    }

    #[test]
    fn test_backward_compatibility_exports() {
        // Verify backward compatibility exports are maintained
        use crate::transport::adapters::{
            BufferPool, ConnectionId, HttpSseConfig, DEFAULT_MESSAGES_ENDPOINT,
            DEFAULT_SSE_ENDPOINT,
        };

        // These should be available for backward compatibility
        let _: Option<ConnectionId> = None;
        let _: Option<HttpSseConfig> = None;
        let _: Option<BufferPool> = None;

        // Constants should be available
        let _messages_endpoint = DEFAULT_MESSAGES_ENDPOINT;
        let _sse_endpoint = DEFAULT_SSE_ENDPOINT;
    }
}
