//! HTTP Transport Builder Implementation
//!
//! This module provides the TransportBuilder implementation for HTTP transports,
//! enabling pre-configured transport creation per ADR-011 Transport Configuration
//! Separation Architecture.

use std::sync::Arc;

use async_trait::async_trait;

use super::config::HttpTransportConfig;
use crate::protocol::transport::{MessageHandler, Transport, TransportBuilder};
use crate::protocol::{JsonRpcMessage, TransportError};

/// HTTP Transport implementing the new protocol::Transport trait
///
/// This transport implements the pre-configured pattern where the message
/// handler is set during construction, eliminating dangerous post-creation
/// handler modifications.
///
/// # Architecture
///
/// ```text
/// HttpTransportBuilder -> HttpTransport (pre-configured)
/// (builder pattern)       (ready to start)
/// ```
///
/// # Usage
///
/// ```rust
/// use airs_mcp::protocol::{TransportBuilder, MessageHandler};
/// use airs_mcp::transport::adapters::http::{HttpTransportBuilder, HttpTransportConfig};
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = HttpTransportConfig::new()
///     .bind_address("127.0.0.1:3000".parse().unwrap());
///
/// let handler = Arc::new(MyHandler);
/// let transport = HttpTransportBuilder::new()
///     .with_config(config)
///     .with_message_handler(handler)
///     .build()
///     .await?;
///
/// // Transport is now fully configured and ready to start
/// # Ok(())
/// # }
/// # struct MyHandler;
/// # #[async_trait::async_trait]
/// # impl MessageHandler for MyHandler {
/// #     async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext) {}
/// #     async fn handle_error(&self, error: TransportError) {}
/// #     async fn handle_close(&self) {}
/// # }
/// ```
pub struct HttpTransport {
    /// Pre-configured message handler
    #[allow(dead_code)] // TODO: Will be used when HTTP server implementation is completed
    message_handler: Arc<dyn MessageHandler>,
    /// HTTP transport configuration
    config: HttpTransportConfig,
    /// Connection state
    is_connected: bool,
    /// Current session context (for HTTP request/response cycles)
    session_id: Option<String>,
}

impl HttpTransport {
    /// Create a new HTTP transport with pre-configured handler
    ///
    /// This constructor enforces the pre-configured pattern - the transport
    /// is created with its message handler already set.
    pub fn new(config: HttpTransportConfig, message_handler: Arc<dyn MessageHandler>) -> Self {
        Self {
            message_handler,
            config,
            is_connected: false,
            session_id: None,
        }
    }
}

#[async_trait]
impl Transport for HttpTransport {
    type Error = TransportError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        // TODO: Implement HTTP server startup using config
        // This should:
        // 1. Create HTTP server with self.config.bind_address
        // 2. Set up routes for MCP endpoints
        // 3. Integrate with existing AxumHttpServer infrastructure
        // 4. Use self.message_handler for incoming requests

        self.is_connected = true;
        tracing::info!("HTTP transport started on {}", self.config.bind_address);
        Ok(())
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        // TODO: Implement HTTP server shutdown
        // This should:
        // 1. Gracefully shutdown HTTP server
        // 2. Close active connections
        // 3. Clean up resources

        self.is_connected = false;
        self.session_id = None;
        tracing::info!("HTTP transport closed");
        Ok(())
    }

    async fn send(&mut self, message: &JsonRpcMessage) -> Result<(), Self::Error> {
        // TODO: Implement HTTP message sending
        // This should:
        // 1. Serialize message to JSON
        // 2. Send HTTP response (if in request/response cycle)
        // 3. Handle SSE streaming (if in streaming mode)

        tracing::debug!("Sending HTTP message: {:?}", message);
        Ok(())
    }

    fn session_id(&self) -> Option<String> {
        self.session_id.clone()
    }

    fn set_session_context(&mut self, session_id: Option<String>) {
        self.session_id = session_id;
    }

    fn is_connected(&self) -> bool {
        self.is_connected
    }

    fn transport_type(&self) -> &'static str {
        "http"
    }
}

/// Builder for creating pre-configured HTTP transports
///
/// This builder implements the pre-configured transport pattern where
/// the transport is created with its message handler already set,
/// eliminating dangerous post-creation handler modifications.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::{TransportBuilder, MessageHandler};
/// use airs_mcp::transport::adapters::http::{HttpTransportBuilder, HttpTransportConfig};
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # struct MyHandler;
/// # #[async_trait::async_trait]
/// # impl MessageHandler for MyHandler {
/// #     async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext) {}
/// #     async fn handle_error(&self, error: TransportError) {}
/// #     async fn handle_close(&self) {}
/// # }
/// let config = HttpTransportConfig::new()
///     .bind_address("127.0.0.1:3000".parse().unwrap())
///     .max_connections(1000);
///
/// let handler = Arc::new(MyHandler);
/// let transport = HttpTransportBuilder::new()
///     .with_config(config)
///     .with_message_handler(handler)
///     .build()
///     .await?;
///
/// // Transport is pre-configured and ready to start
/// # Ok(())
/// # }
/// ```
pub struct HttpTransportBuilder {
    /// HTTP transport configuration
    config: HttpTransportConfig,
    /// Message handler for processing incoming messages
    message_handler: Option<Arc<dyn MessageHandler>>,
}

impl HttpTransportBuilder {
    /// Create a new HTTP transport builder
    ///
    /// This creates a builder with default HTTP configuration.
    /// Use `with_config()` to customize the configuration.
    pub fn new() -> Self {
        Self {
            config: HttpTransportConfig::new(),
            message_handler: None,
        }
    }

    /// Set the HTTP transport configuration
    ///
    /// This allows customizing the HTTP-specific settings like bind address,
    /// connection limits, timeouts, etc.
    pub fn with_config(mut self, config: HttpTransportConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for HttpTransportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TransportBuilder for HttpTransportBuilder {
    type Transport = HttpTransport;
    type Error = TransportError;

    /// Set the message handler for the transport
    ///
    /// This is the key method that implements the pre-configured pattern.
    /// The handler must be set before building the transport.
    fn with_message_handler(mut self, handler: Arc<dyn MessageHandler>) -> Self {
        self.message_handler = Some(handler);
        self
    }

    /// Build the transport with the configured message handler
    ///
    /// This creates a fully configured transport that is ready to start.
    /// The transport will have its message handler pre-configured.
    async fn build(self) -> Result<Self::Transport, Self::Error> {
        let handler = self
            .message_handler
            .ok_or_else(|| TransportError::Connection {
                message: "Message handler must be set before building HTTP transport".to_string(),
            })?;

        Ok(HttpTransport::new(self.config, handler))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{JsonRpcMessage, MessageContext};

    struct TestHandler;

    #[async_trait]
    impl MessageHandler for TestHandler {
        async fn handle_message(&self, _message: JsonRpcMessage, _context: MessageContext) {
            // Test handler implementation
        }

        async fn handle_error(&self, _error: TransportError) {
            // Test error handling
        }

        async fn handle_close(&self) {
            // Test close handling
        }
    }

    #[tokio::test]
    async fn test_http_transport_builder() {
        let handler = Arc::new(TestHandler);
        let config = HttpTransportConfig::new().bind_address("127.0.0.1:8080".parse().unwrap());

        let transport_result = HttpTransportBuilder::new()
            .with_config(config)
            .with_message_handler(handler)
            .build()
            .await;

        assert!(transport_result.is_ok());
        let transport = transport_result.unwrap();
        assert_eq!(transport.transport_type(), "http");
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_http_transport_builder_requires_handler() {
        let config = HttpTransportConfig::new();

        let builder_without_handler = HttpTransportBuilder::new().with_config(config);

        let result = builder_without_handler.build().await;
        assert!(result.is_err());

        if let Err(TransportError::Connection { message }) = result {
            assert!(message.contains("Message handler must be set"));
        } else {
            panic!("Expected Connection error with message about handler");
        }
    }

    #[tokio::test]
    async fn test_pre_configured_pattern() {
        let handler = Arc::new(TestHandler);
        let transport = HttpTransport::new(HttpTransportConfig::new(), handler.clone());

        // Pre-configured pattern: transport is created with handler already set
        // No way to accidentally overwrite the handler after creation
        // This is the key safety improvement of the pre-configured pattern

        // Transport should work normally with pre-configured handler
        assert_eq!(transport.transport_type(), "http");
        assert!(!transport.is_connected());

        // The message_handler field is private and can only be set during construction
        // This prevents the dangerous handler overwriting pattern
    }
}
