//! HTTP Server Transport Adapter
//!
//! This module provides an adapter that bridges the legacy HttpServerTransport
//! to the new MCP-compliant Transport interface. It implements the event-driven
//! MessageHandler pattern while maintaining full compatibility with existing
//! HTTP server functionality.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{mpsc, Mutex};

use crate::transport::adapters::http::config::HttpTransportConfig;
use crate::transport::adapters::http::server::HttpServerTransport;
use crate::transport::mcp::{
    JsonRpcMessage, MessageContext, MessageHandler, Transport, TransportError,
};
use crate::transport::traits::Transport as LegacyTransport;

/// HTTP Server Transport Adapter
///
/// Bridges the legacy HttpServerTransport to the new MCP-compliant Transport interface.
/// This adapter implements the event-driven MessageHandler pattern, allowing gradual
/// migration from blocking I/O to event-driven message processing.
///
/// ## Architecture
///
/// The adapter maintains a background event loop that:
/// - Polls the legacy HTTP server for incoming requests
/// - Converts legacy transport errors to MCP TransportError format
/// - Routes messages through the MessageHandler interface
/// - Manages session state and graceful shutdown
///
/// ## Usage
///
/// ```rust
/// use crate::transport::adapters::http::{HttpServerTransportAdapter, HttpTransportConfig};
/// use crate::transport::mcp::transport::Transport;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = HttpTransportConfig::new();
///     let mut adapter = HttpServerTransportAdapter::new(config).await?;
///     
///     // Set message handler
///     adapter.set_message_handler(my_handler);
///     
///     // Start the transport
///     adapter.start().await?;
///     
///     // Send responses
///     adapter.send(my_response).await?;
///     
///     Ok(())
/// }
/// ```
pub struct HttpServerTransportAdapter {
    /// Legacy HTTP server transport (thread-safe for background loop)
    legacy_transport: Arc<Mutex<HttpServerTransport>>,

    /// Message handler for event-driven processing
    message_handler: Option<Arc<dyn MessageHandler>>,

    /// Shutdown signal for graceful termination
    shutdown_tx: Option<mpsc::Sender<()>>,

    /// Session ID for this transport instance
    session_id: Option<String>,

    /// Connection state tracking
    is_connected: bool,
}

impl HttpServerTransportAdapter {
    /// Create a new HTTP server transport adapter
    ///
    /// This creates the adapter and initializes the underlying HTTP server transport.
    /// The legacy transport is wrapped in Arc<Mutex<>> to enable safe access from
    /// the background event loop.
    ///
    /// # Arguments
    ///
    /// * `config` - HTTP transport configuration
    ///
    /// # Returns
    ///
    /// * `Ok(HttpServerTransportAdapter)` - Successfully created adapter
    /// * `Err(TransportError)` - Failed to initialize transport
    ///
    /// # Examples
    ///
    /// ```rust
    /// let config = HttpTransportConfig::new();
    /// let adapter = HttpServerTransportAdapter::new(config).await?;
    /// ```
    pub async fn new(config: HttpTransportConfig) -> Result<Self, TransportError> {
        let legacy_transport = HttpServerTransport::new(config)
            .await
            .map_err(Self::convert_legacy_error)?;

        Ok(Self {
            legacy_transport: Arc::new(Mutex::new(legacy_transport)),
            message_handler: None,
            shutdown_tx: None,
            session_id: None,
            is_connected: false,
        })
    }

    /// Start the background event loop
    ///
    /// This method spawns a background task that continuously polls the legacy
    /// HTTP transport for incoming requests and routes them through the MessageHandler.
    ///
    /// The event loop handles:
    /// - Request reception and parsing
    /// - Error conversion and propagation
    /// - Graceful shutdown coordination
    /// - Session context management
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Event loop started successfully
    /// * `Err(TransportError)` - Failed to start event loop
    async fn start_event_loop(&mut self) -> Result<(), TransportError> {
        if self.message_handler.is_none() {
            return Err(TransportError::transport("Message handler not set"));
        }

        if self.shutdown_tx.is_some() {
            return Err(TransportError::transport("Event loop already running"));
        }

        let handler = self.message_handler.as_ref().unwrap().clone();
        let legacy_transport = Arc::clone(&self.legacy_transport);
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        self.shutdown_tx = Some(shutdown_tx);
        self.is_connected = true;

        // Bind the server first - but this is handled by the HttpServerTransport constructor
        // so we don't need to call bind() separately

        // Spawn background event loop
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle shutdown signal
                    _ = shutdown_rx.recv() => {
                        break;
                    }

                    // Poll for incoming requests
                    result = async {
                        let mut transport = legacy_transport.lock().await;
                        transport.receive().await
                    } => {
                        match result {
                            Ok(message_bytes) => {
                                // Parse message and create context
                                match Self::parse_message_and_create_context(&message_bytes, &legacy_transport).await {
                                    Ok((message, context)) => {
                                        // Route through message handler
                                        handler.handle_message(message, context).await;
                                    }
                                    Err(parse_error) => {
                                        // Convert parsing error to transport error
                                        let transport_error = TransportError::Serialization { source: parse_error };
                                        handler.handle_error(transport_error).await;
                                    }
                                }
                            }
                            Err(transport_error) => {
                                // Convert legacy transport error to MCP format
                                let mcp_error = Self::convert_legacy_error(transport_error);

                                // Check if this is a connection closure before handling
                                let is_closed = matches!(mcp_error, TransportError::Closed);
                                handler.handle_error(mcp_error).await;

                                if is_closed {
                                    handler.handle_close().await;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Parse message bytes and create message context
    ///
    /// This method converts raw message bytes from the legacy transport into
    /// a JsonRpcMessage and creates appropriate MessageContext with session
    /// and transport metadata.
    async fn parse_message_and_create_context(
        message_bytes: &[u8],
        legacy_transport: &Arc<Mutex<HttpServerTransport>>,
    ) -> Result<(JsonRpcMessage, MessageContext), serde_json::Error> {
        // Parse JSON-RPC message
        let message_str = std::str::from_utf8(message_bytes).map_err(|e| {
            serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid UTF-8: {e}"),
            ))
        })?;

        let message: JsonRpcMessage = serde_json::from_str(message_str)?;

        // Create message context with server-specific information
        let bind_address = {
            let transport = legacy_transport.lock().await;
            transport.bind_address().to_string()
        };

        let context = MessageContext::new(format!("http-{bind_address}"))
            .with_metadata("transport_type".to_string(), "http-server".to_string())
            .with_metadata("bind_address".to_string(), bind_address.clone())
            .with_metadata("server_ready".to_string(), "true".to_string());

        Ok((message, context))
    }

    /// Convert legacy transport errors to MCP TransportError format
    ///
    /// This method provides a mapping between the legacy transport error types
    /// and the new MCP TransportError enum, ensuring consistent error handling
    /// across the transport layer.
    fn convert_legacy_error(
        legacy_error: crate::transport::error::TransportError,
    ) -> TransportError {
        match legacy_error {
            crate::transport::error::TransportError::Io(io_error) => {
                TransportError::Io { source: io_error }
            }
            crate::transport::error::TransportError::Timeout { duration_ms } => {
                TransportError::Timeout { duration_ms }
            }
            _ => TransportError::transport(format!("Legacy transport error: {legacy_error}")),
        }
    }
}

#[async_trait]
impl Transport for HttpServerTransportAdapter {
    type Error = TransportError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        // The legacy transport doesn't have a start method,
        // it's already bound and ready from the constructor
        // Start our event loop
        self.start_event_loop().await?;

        Ok(())
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        // Signal event loop to shut down
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(()).await;
        }

        // Close the legacy transport
        {
            let mut legacy_transport = self.legacy_transport.lock().await;
            legacy_transport
                .close()
                .await
                .map_err(Self::convert_legacy_error)?;
        }

        self.is_connected = false;
        Ok(())
    }

    async fn send(&mut self, message: JsonRpcMessage) -> Result<(), Self::Error> {
        if !self.is_connected {
            return Err(TransportError::transport("Transport not connected"));
        }

        // Serialize message to bytes
        let message_bytes = serde_json::to_vec(&message)
            .map_err(|e| TransportError::Serialization { source: e })?;

        // Send through legacy transport
        {
            let mut legacy_transport = self.legacy_transport.lock().await;
            legacy_transport
                .send(&message_bytes)
                .await
                .map_err(Self::convert_legacy_error)?;
        }

        Ok(())
    }

    fn set_message_handler(&mut self, handler: Arc<dyn MessageHandler>) {
        self.message_handler = Some(handler);
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
        "http-server"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::adapters::http::config::HttpTransportConfig;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::time::{sleep, Duration};

    /// Mock message handler for testing event loop functionality
    struct TestMessageHandler {
        messages: Arc<Mutex<Vec<JsonRpcMessage>>>,
        errors: Arc<Mutex<Vec<TransportError>>>,
        close_calls: Arc<Mutex<u32>>,
    }

    impl TestMessageHandler {
        fn new() -> Self {
            Self {
                messages: Arc::new(Mutex::new(Vec::new())),
                errors: Arc::new(Mutex::new(Vec::new())),
                close_calls: Arc::new(Mutex::new(0)),
            }
        }

        async fn get_message_count(&self) -> usize {
            self.messages.lock().await.len()
        }

        async fn get_error_count(&self) -> usize {
            self.errors.lock().await.len()
        }

        async fn get_close_count(&self) -> u32 {
            *self.close_calls.lock().await
        }
    }

    #[async_trait]
    impl MessageHandler for TestMessageHandler {
        async fn handle_message(&self, message: JsonRpcMessage, _context: MessageContext) {
            self.messages.lock().await.push(message);
        }

        async fn handle_error(&self, error: TransportError) {
            self.errors.lock().await.push(error);
        }

        async fn handle_close(&self) {
            *self.close_calls.lock().await += 1;
        }
    }

    #[tokio::test]
    async fn test_adapter_creation() {
        let config = HttpTransportConfig::new();
        let adapter = HttpServerTransportAdapter::new(config).await;
        assert!(adapter.is_ok());

        let adapter = adapter.unwrap();
        assert!(!adapter.is_connected());
        assert_eq!(adapter.transport_type(), "http-server");
        assert!(adapter.session_id().is_none());
    }

    #[tokio::test]
    async fn test_session_management() {
        let config = HttpTransportConfig::new();
        let mut adapter = HttpServerTransportAdapter::new(config).await.unwrap();

        // Test setting session context
        adapter.set_session_context(Some("test-session-456".to_string()));
        assert_eq!(adapter.session_id(), Some("test-session-456".to_string()));

        // Test clearing session
        adapter.set_session_context(None);
        assert!(adapter.session_id().is_none());
    }

    #[tokio::test]
    async fn test_message_handler_requirement() {
        let config = HttpTransportConfig::new();
        let mut adapter = HttpServerTransportAdapter::new(config).await.unwrap();

        // Should fail to start without message handler
        let result = adapter.start().await;
        assert!(result.is_err());

        if let Err(TransportError::Transport { message }) = result {
            assert!(message.contains("Message handler not set"));
        } else {
            panic!("Expected Transport error with message handler message");
        }
    }

    #[tokio::test]
    async fn test_start_event_loop_success() {
        let config = HttpTransportConfig::new();
        let mut adapter = HttpServerTransportAdapter::new(config).await.unwrap();

        // Set up message handler
        let handler = Arc::new(TestMessageHandler::new());
        adapter.set_message_handler(handler.clone());

        // Start event loop should succeed
        let result = adapter.start_event_loop().await;
        assert!(result.is_ok());

        // Verify adapter state
        assert!(adapter.is_connected());
        assert!(adapter.shutdown_tx.is_some());

        // Clean up
        adapter.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_start_event_loop_without_handler() {
        let config = HttpTransportConfig::new();
        let mut adapter = HttpServerTransportAdapter::new(config).await.unwrap();

        // Don't set message handler

        // Start event loop should fail
        let result = adapter.start_event_loop().await;
        assert!(result.is_err());

        if let Err(TransportError::Transport { message }) = result {
            assert!(message.contains("Message handler not set"));
        } else {
            panic!("Expected Transport error about missing message handler");
        }

        // Verify adapter state unchanged
        assert!(!adapter.is_connected());
        assert!(adapter.shutdown_tx.is_none());
    }

    #[tokio::test]
    async fn test_start_event_loop_already_running() {
        let config = HttpTransportConfig::new();
        let mut adapter = HttpServerTransportAdapter::new(config).await.unwrap();

        // Set up message handler
        let handler = Arc::new(TestMessageHandler::new());
        adapter.set_message_handler(handler.clone());

        // Start event loop first time - should succeed
        let result1 = adapter.start_event_loop().await;
        assert!(result1.is_ok());

        // Try to start again - should fail
        let result2 = adapter.start_event_loop().await;
        assert!(result2.is_err());

        if let Err(TransportError::Transport { message }) = result2 {
            assert!(message.contains("Event loop already running"));
        } else {
            panic!("Expected Transport error about event loop already running");
        }

        // Clean up
        adapter.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_event_loop_shutdown_signal() {
        let config = HttpTransportConfig::new();
        let mut adapter = HttpServerTransportAdapter::new(config).await.unwrap();

        // Set up message handler
        let handler = Arc::new(TestMessageHandler::new());
        adapter.set_message_handler(handler.clone());

        // Start event loop
        adapter.start_event_loop().await.unwrap();

        // Verify it's running
        assert!(adapter.is_connected());
        assert!(adapter.shutdown_tx.is_some());

        // Send shutdown signal
        if let Some(shutdown_tx) = adapter.shutdown_tx.take() {
            shutdown_tx.send(()).await.unwrap();
        }

        // Give event loop time to process shutdown
        sleep(Duration::from_millis(10)).await;

        // Event loop should have stopped (we can't directly verify this, but we can verify
        // that the shutdown channel was consumed)
        assert!(adapter.shutdown_tx.is_none());

        // Clean up
        adapter.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_event_loop_message_handler_integration() {
        let config = HttpTransportConfig::new();
        let mut adapter = HttpServerTransportAdapter::new(config).await.unwrap();

        // Set up message handler
        let handler = Arc::new(TestMessageHandler::new());
        adapter.set_message_handler(handler.clone());

        // Start event loop
        adapter.start_event_loop().await.unwrap();

        // Verify handler is set up correctly
        assert_eq!(handler.get_message_count().await, 0);
        assert_eq!(handler.get_error_count().await, 0);
        assert_eq!(handler.get_close_count().await, 0);

        // Clean up
        adapter.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_parse_message_and_create_context_success() {
        let config = HttpTransportConfig::new();
        let server_transport = HttpServerTransport::new(config).await.unwrap();
        let legacy_transport = Arc::new(Mutex::new(server_transport));

        // Test valid JSON-RPC message
        let message_bytes =
            br#"{"jsonrpc":"2.0","method":"test_method","params":{"key":"value"},"id":42}"#;

        let result = HttpServerTransportAdapter::parse_message_and_create_context(
            message_bytes,
            &legacy_transport,
        )
        .await;

        assert!(result.is_ok());
        let (message, context) = result.unwrap();

        // Verify message parsing
        assert_eq!(message.method.as_ref().unwrap(), "test_method");
        assert_eq!(message.id.as_ref().unwrap().as_u64().unwrap(), 42);
        assert!(message.params.is_some());

        // Verify context creation
        assert!(context.session_id().is_some());
        assert_eq!(context.get_metadata("transport_type"), Some("http-server"));
        assert!(context.get_metadata("bind_address").is_some());
        assert_eq!(context.get_metadata("server_ready"), Some("true"));
    }

    #[tokio::test]
    async fn test_parse_message_and_create_context_invalid_utf8() {
        let config = HttpTransportConfig::new();
        let server_transport = HttpServerTransport::new(config).await.unwrap();
        let legacy_transport = Arc::new(Mutex::new(server_transport));

        // Test invalid UTF-8 bytes
        let message_bytes = &[0xFF, 0xFE, 0xFD];

        let result = HttpServerTransportAdapter::parse_message_and_create_context(
            message_bytes,
            &legacy_transport,
        )
        .await;

        assert!(result.is_err());
        // Should be a serde_json::Error about invalid UTF-8
    }

    #[tokio::test]
    async fn test_parse_message_and_create_context_invalid_json() {
        let config = HttpTransportConfig::new();
        let server_transport = HttpServerTransport::new(config).await.unwrap();
        let legacy_transport = Arc::new(Mutex::new(server_transport));

        // Test invalid JSON
        let message_bytes = b"not valid json {";

        let result = HttpServerTransportAdapter::parse_message_and_create_context(
            message_bytes,
            &legacy_transport,
        )
        .await;

        assert!(result.is_err());
        // Should be a serde_json::Error about invalid JSON
    }

    #[tokio::test]
    async fn test_convert_legacy_error_io() {
        use std::io;
        let io_error = io::Error::new(io::ErrorKind::ConnectionRefused, "Connection refused");
        let legacy_error = crate::transport::error::TransportError::Io(io_error);

        let mcp_error = HttpServerTransportAdapter::convert_legacy_error(legacy_error);

        match mcp_error {
            TransportError::Io { source } => {
                assert_eq!(source.kind(), io::ErrorKind::ConnectionRefused);
            }
            _ => panic!("Expected Io error variant"),
        }
    }

    #[tokio::test]
    async fn test_convert_legacy_error_timeout() {
        let legacy_error = crate::transport::error::TransportError::Timeout { duration_ms: 5000 };

        let mcp_error = HttpServerTransportAdapter::convert_legacy_error(legacy_error);

        match mcp_error {
            TransportError::Timeout { duration_ms } => {
                assert_eq!(duration_ms, 5000);
            }
            _ => panic!("Expected Timeout error variant"),
        }
    }

    #[tokio::test]
    async fn test_convert_legacy_error_other() {
        use crate::transport::error::TransportError as LegacyError;
        let legacy_error = LegacyError::Other {
            details: "Test protocol error".to_string(),
        };

        let mcp_error = HttpServerTransportAdapter::convert_legacy_error(legacy_error);

        match mcp_error {
            TransportError::Transport { message } => {
                assert!(message.contains("Legacy transport error"));
                assert!(message.contains("Test protocol error"));
            }
            _ => panic!("Expected Transport error variant"),
        }
    }

    #[tokio::test]
    async fn test_context_creation() {
        let config = HttpTransportConfig::new();
        let server_transport = HttpServerTransport::new(config).await.unwrap();
        let legacy_transport = Arc::new(Mutex::new(server_transport));

        let message_bytes = br#"{"jsonrpc":"2.0","method":"test","id":1}"#;

        let result = HttpServerTransportAdapter::parse_message_and_create_context(
            message_bytes,
            &legacy_transport,
        )
        .await;

        assert!(result.is_ok());
        let (message, context) = result.unwrap();

        // Check message parsing
        assert_eq!(message.method.as_ref().unwrap(), "test");

        // Check context metadata
        assert!(context.session_id().is_some());
        assert_eq!(context.get_metadata("transport_type"), Some("http-server"));
        assert!(context.get_metadata("bind_address").is_some());
    }
}
