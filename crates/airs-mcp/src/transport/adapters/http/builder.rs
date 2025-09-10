//! HTTP Transport Builder Implementation
//!
//! This module provides the TransportBuilder implementation for HTTP transports,
//! enabling pre-configured transport creation per ADR-011 Transport Configuration
//! Separation Architecture with generic MessageHandler<HttpContext> pattern.

// Layer 1: Standard library imports
use std::fmt::Debug;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use super::config::HttpTransportConfig;
use super::context::HttpContext;
use crate::protocol::{
    JsonRpcMessage, MessageContext, MessageHandler, Transport, TransportBuilder, TransportError,
};

/// HTTP Transport implementing the new protocol::Transport trait
///
/// This transport implements the pre-configured pattern where the message
/// handler is set during construction, eliminating dangerous post-creation
/// handler modifications. Uses the generic MessageHandler<HttpContext> pattern
/// to provide HTTP-specific context information to handlers.
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
/// use airs_mcp::protocol::{TransportBuilder, MessageHandler, MessageContext, JsonRpcMessage, TransportError};
/// use airs_mcp::transport::adapters::http::{HttpTransportBuilder, HttpTransportConfig, HttpContext};
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
/// # impl MessageHandler<HttpContext> for MyHandler {
/// #     async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<HttpContext>) {}
/// #     async fn handle_error(&self, error: TransportError) {}
/// #     async fn handle_close(&self) {}
/// # }
/// ```
pub struct HttpTransport {
    /// Pre-configured message handler for HTTP context
    #[allow(dead_code)] // TODO: Will be used when HTTP server implementation is completed
    message_handler: Arc<dyn MessageHandler<HttpContext>>,
    /// HTTP transport configuration
    config: HttpTransportConfig,
    /// Connection state
    is_connected: bool,
    /// Current session context (for HTTP request/response cycles)
    session_id: Option<String>,
}

impl Debug for HttpTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpTransport")
            .field("message_handler", &"Arc<dyn MessageHandler<HttpContext>>")
            .field("config", &self.config)
            .field("is_connected", &self.is_connected)
            .field("session_id", &self.session_id)
            .finish()
    }
}

impl HttpTransport {
    /// Create a new HTTP transport with pre-configured handler
    ///
    /// This constructor enforces the pre-configured pattern - the transport
    /// is created with its message handler already set.
    pub fn new(
        config: HttpTransportConfig,
        message_handler: Arc<dyn MessageHandler<HttpContext>>,
    ) -> Self {
        Self {
            message_handler,
            config,
            is_connected: false,
            session_id: None,
        }
    }

    /// Handle an incoming HTTP request by parsing it into HttpContext and dispatching to handler
    ///
    /// This method demonstrates how HTTP requests are converted into the generic
    /// MessageHandler pattern with HttpContext.
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `path` - Request path
    /// * `headers` - HTTP headers
    /// * `query_params` - Query parameters
    /// * `body` - Request body (if any)
    /// * `remote_addr` - Client address
    ///
    /// # Returns
    ///
    /// Result indicating success or transport error
    #[allow(dead_code)] // Will be used when integrated with HTTP server
    async fn handle_http_request(
        &self,
        method: String,
        path: String,
        headers: std::collections::HashMap<String, String>,
        query_params: std::collections::HashMap<String, String>,
        body: Option<String>,
        remote_addr: Option<String>,
    ) -> Result<(), TransportError> {
        // Create HTTP context from request details
        let mut http_context = HttpContext::new(method, path)
            .with_headers(headers)
            .with_query_params(query_params);

        if let Some(addr) = remote_addr {
            http_context = http_context.with_remote_addr(addr);
        }

        // Parse JSON-RPC message from request body
        if let Some(body_str) = body {
            match serde_json::from_str::<JsonRpcMessage>(&body_str) {
                Ok(message) => {
                    // Create MessageContext with HttpContext as transport data
                    let session_id = http_context
                        .session_id()
                        .unwrap_or("http-session")
                        .to_string();

                    let message_context =
                        MessageContext::new_with_transport_data(session_id, http_context);

                    // Dispatch to the pre-configured message handler
                    self.message_handler
                        .handle_message(message, message_context)
                        .await;
                }
                Err(e) => {
                    // Handle JSON parsing errors
                    let error = TransportError::Serialization { source: e };
                    self.message_handler.handle_error(error).await;
                }
            }
        } else {
            // Handle missing body error
            let error = TransportError::Protocol {
                message: "HTTP request body is required for JSON-RPC messages".to_string(),
            };
            self.message_handler.handle_error(error).await;
        }

        Ok(())
    }
}

#[async_trait]
impl Transport for HttpTransport {
    type Error = TransportError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        // TODO: Implement full HTTP server startup using config
        // This should:
        // 1. Create HTTP server with self.config.bind_address
        // 2. Set up routes for MCP endpoints
        // 3. Integrate with existing AxumHttpServer infrastructure
        // 4. Use self.message_handler for incoming requests
        // 5. Parse HTTP requests into HttpContext for handlers

        self.is_connected = true;
        tracing::info!("HTTP transport started on {}", self.config.bind_address);

        // Note: This is a placeholder implementation for Phase 5.5.3
        // Full HTTP server integration will be completed in subsequent phases
        // when we integrate with the existing Axum infrastructure

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
/// Uses the generic MessageHandler<HttpContext> pattern.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::{TransportBuilder, MessageHandler, MessageContext, JsonRpcMessage, TransportError};
/// use airs_mcp::transport::adapters::http::{HttpTransportBuilder, HttpTransportConfig, HttpContext};
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # struct MyHandler;
/// # #[async_trait::async_trait]
/// # impl MessageHandler<HttpContext> for MyHandler {
/// #     async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<HttpContext>) {}
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
    /// Message handler for processing incoming messages with HTTP context
    message_handler: Option<Arc<dyn MessageHandler<HttpContext>>>,
}

impl std::fmt::Debug for HttpTransportBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpTransportBuilder")
            .field("config", &self.config)
            .field(
                "message_handler",
                &self
                    .message_handler
                    .as_ref()
                    .map(|_| "Arc<dyn MessageHandler<HttpContext>>"),
            )
            .finish()
    }
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

impl TransportBuilder<HttpContext> for HttpTransportBuilder {
    type Transport = HttpTransport;
    type Error = TransportError;

    /// Set the message handler for the transport
    ///
    /// This is the key method that implements the pre-configured pattern.
    /// The handler must be set before building the transport.
    fn with_message_handler(mut self, handler: Arc<dyn MessageHandler<HttpContext>>) -> Self {
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

    struct TestHandler;

    #[async_trait]
    impl MessageHandler<HttpContext> for TestHandler {
        async fn handle_message(
            &self,
            _message: JsonRpcMessage,
            _context: MessageContext<HttpContext>,
        ) {
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

    #[tokio::test]
    async fn test_phase_5_5_3_http_context_integration() {
        // Test HttpContext creation and HTTP-specific methods
        let context = HttpContext::new("POST".to_string(), "/mcp/request".to_string());

        assert!(context.is_post());
        assert_eq!(context.method(), "POST");
        assert_eq!(context.path(), "/mcp/request");
        assert!(context.headers().is_empty());
        assert!(context.query_params().is_empty());
    }

    #[tokio::test]
    async fn test_phase_5_5_3_http_context_builder_pattern() {
        // Test HttpContext builder pattern with headers and query parameters
        let context = HttpContext::new("GET".to_string(), "/mcp/status".to_string())
            .with_header("Content-Type", "application/json")
            .with_header("Authorization", "Bearer token123")
            .with_query_param("format", "json")
            .with_query_param("version", "1.0")
            .with_remote_addr("127.0.0.1:8080".to_string());

        assert_eq!(context.method(), "GET");
        assert_eq!(context.path(), "/mcp/status");
        assert_eq!(context.remote_addr(), Some("127.0.0.1:8080"));

        // Test case-insensitive header access
        assert_eq!(context.get_header("content-type"), Some("application/json"));
        assert_eq!(context.get_header("CONTENT-TYPE"), Some("application/json"));
        assert_eq!(context.get_header("Content-Type"), Some("application/json"));

        assert_eq!(context.get_query_param("format"), Some("json"));
        assert_eq!(context.get_query_param("version"), Some("1.0"));
    }

    #[tokio::test]
    async fn test_phase_5_5_3_json_content_detection() {
        // Test is_json() method for Content-Type detection
        let json_context = HttpContext::new("POST".to_string(), "/api/data".to_string())
            .with_header("Content-Type", "application/json");

        let xml_context = HttpContext::new("POST".to_string(), "/api/data".to_string())
            .with_header("Content-Type", "application/xml");

        let no_content_type = HttpContext::new("POST".to_string(), "/api/data".to_string());

        assert!(json_context.is_json());
        assert!(!xml_context.is_json());
        assert!(!no_content_type.is_json());
    }

    #[tokio::test]
    async fn test_phase_5_5_3_session_extraction() {
        // Test session ID extraction from different sources
        let header_session = HttpContext::new("GET".to_string(), "/mcp/status".to_string())
            .with_header("X-Session-ID", "session123");

        let cookie_session = HttpContext::new("GET".to_string(), "/mcp/status".to_string())
            .with_header("Cookie", "sessionId=cookie456; other=value");

        let query_session = HttpContext::new("GET".to_string(), "/mcp/status".to_string())
            .with_query_param("sessionId", "query789");

        assert_eq!(header_session.session_id(), Some("session123"));
        assert_eq!(cookie_session.session_id(), Some("cookie456"));
        assert_eq!(query_session.session_id(), Some("query789"));
    }

    #[tokio::test]
    async fn test_phase_5_5_3_generic_handler_pattern() {
        use crate::protocol::{JsonRpcMessage, MessageContext, MessageHandler, RequestId};
        use async_trait::async_trait;

        // Test handler implementation with HttpContext
        struct TestHttpHandler;

        #[async_trait]
        impl MessageHandler<HttpContext> for TestHttpHandler {
            async fn handle_message(
                &self,
                _message: JsonRpcMessage,
                context: MessageContext<HttpContext>,
            ) {
                // Access HTTP-specific context data
                let http_context = context
                    .transport_data()
                    .expect("HttpContext should be present");
                assert_eq!(http_context.method(), "POST");
                assert_eq!(http_context.path(), "/test");
            }

            async fn handle_error(&self, _error: TransportError) {
                // Test error handling
            }

            async fn handle_close(&self) {
                // Test close handling
            }
        }

        // Create handler and test with HttpContext
        let handler = TestHttpHandler;
        let http_context = HttpContext::new("POST".to_string(), "/test".to_string());

        let message_context = MessageContext::new_with_transport_data(
            "test-correlation-id".to_string(),
            http_context,
        );

        // Create a test message for the handler
        let test_message =
            JsonRpcMessage::from_request("test_method", None, RequestId::new_number(1));

        handler.handle_message(test_message, message_context).await;
    }

    #[tokio::test]
    async fn test_phase_5_5_3_transport_builder_with_http_context() {
        // Test HttpTransportBuilder with MessageHandler<HttpContext>
        use crate::protocol::{JsonRpcMessage, MessageContext, MessageHandler};
        use async_trait::async_trait;

        struct HttpContextHandler;

        #[async_trait]
        impl MessageHandler<HttpContext> for HttpContextHandler {
            async fn handle_message(
                &self,
                _message: JsonRpcMessage,
                context: MessageContext<HttpContext>,
            ) {
                let http_ctx = context
                    .transport_data()
                    .expect("HttpContext should be present");
                tracing::info!(
                    "Handling HTTP {} request to {}",
                    http_ctx.method(),
                    http_ctx.path()
                );
            }

            async fn handle_error(&self, _error: TransportError) {
                // Test error handling
            }

            async fn handle_close(&self) {
                // Test close handling
            }
        }

        let handler = Arc::new(HttpContextHandler);
        let builder = HttpTransportBuilder::new().with_message_handler(handler);
        let transport = builder.build().await.unwrap();

        // Verify the transport is properly configured
        assert_eq!(transport.transport_type(), "http");
        assert!(!transport.is_connected());
    }
}
