//! HTTP Client Transport Adapter
//!
//! This module provides an adapter that bridges the legacy HttpClientTransport
//! to the new MCP-compliant Transport interface. It implements the event-driven
//! MessageHandler pattern while maintaining full compatibility with existing
//! HTTP client functionality.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{mpsc, Mutex};

use crate::transport::adapters::http::client::HttpClientTransport;
use crate::transport::adapters::http::config::HttpTransportConfig;
use crate::transport::mcp::{
    JsonRpcMessage, MessageContext, MessageHandler, Transport, TransportError,
};
use crate::transport::traits::Transport as LegacyTransport;

/// HTTP Client Transport Adapter
///
/// Bridges the legacy HttpClientTransport to the new MCP-compliant Transport interface.
/// This adapter implements the event-driven MessageHandler pattern, allowing gradual
/// migration from blocking I/O to event-driven message processing.
///
/// ## Architecture
///
/// The adapter maintains a background event loop that:
/// - Polls the legacy HTTP client for incoming messages
/// - Converts legacy transport errors to MCP TransportError format
/// - Routes messages through the MessageHandler interface
/// - Manages session state and graceful shutdown
///
/// ## Usage
///
/// ```rust
/// use crate::transport::adapters::http::{HttpClientTransportAdapter, HttpTransportConfig};
/// use crate::transport::mcp::transport::Transport;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = HttpTransportConfig::new();
///     let mut adapter = HttpClientTransportAdapter::new(config).await?;
///     
///     // Set message handler
///     adapter.set_message_handler(my_handler);
///     
///     // Start the transport
///     adapter.start().await?;
///     
///     // Send messages
///     adapter.send(my_message).await?;
///     
///     Ok(())
/// }
/// ```
pub struct HttpClientTransportAdapter {
    /// Legacy HTTP client transport (thread-safe for background loop)
    legacy_transport: Arc<Mutex<HttpClientTransport>>,

    /// Message handler for event-driven processing
    message_handler: Option<Arc<dyn MessageHandler>>,

    /// Shutdown signal for graceful termination
    shutdown_tx: Option<mpsc::Sender<()>>,

    /// Session ID for this transport instance
    session_id: Option<String>,

    /// Connection state tracking
    is_connected: bool,
}

impl HttpClientTransportAdapter {
    /// Create a new HTTP client transport adapter
    ///
    /// This creates the adapter and initializes the underlying HTTP client transport.
    /// The legacy transport is wrapped in Arc<Mutex<>> to enable safe access from
    /// the background event loop.
    ///
    /// # Arguments
    ///
    /// * `config` - HTTP transport configuration
    ///
    /// # Returns
    ///
    /// * `Ok(HttpClientTransportAdapter)` - Successfully created adapter
    /// * `Err(TransportError)` - Failed to initialize transport
    ///
    /// # Examples
    ///
    /// ```rust
    /// let config = HttpTransportConfig::new();
    /// let adapter = HttpClientTransportAdapter::new(config).await?;
    /// ```
    pub async fn new(config: HttpTransportConfig) -> Result<Self, TransportError> {
        let legacy_transport = HttpClientTransport::new(config);

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
    /// HTTP transport for incoming messages and routes them through the MessageHandler.
    ///
    /// The event loop handles:
    /// - Message reception and parsing
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

        // Spawn background event loop
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle shutdown signal
                    _ = shutdown_rx.recv() => {
                        break;
                    }

                    // Poll for incoming messages
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
        _legacy_transport: &Arc<Mutex<HttpClientTransport>>,
    ) -> Result<(JsonRpcMessage, MessageContext), serde_json::Error> {
        // Parse JSON-RPC message
        let message_str = std::str::from_utf8(message_bytes).map_err(|e| {
            serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid UTF-8: {e}"),
            ))
        })?;

        let message: JsonRpcMessage = serde_json::from_str(message_str)?;

        // Create message context with client-specific information
        let context = MessageContext::new(format!("http-client-{}", std::process::id()))
            .with_metadata("transport_type".to_string(), "http-client".to_string())
            .with_metadata("client_id".to_string(), std::process::id().to_string());

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
impl Transport for HttpClientTransportAdapter {
    type Error = TransportError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        // The legacy transport doesn't have a start method
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
        "http-client"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::adapters::http::config::HttpTransportConfig;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_adapter_creation() {
        let config = HttpTransportConfig::new();
        let adapter = HttpClientTransportAdapter::new(config).await;
        assert!(adapter.is_ok());

        let adapter = adapter.unwrap();
        assert!(!adapter.is_connected());
        assert_eq!(adapter.transport_type(), "http-client");
        assert!(adapter.session_id().is_none());
    }

    #[tokio::test]
    async fn test_session_management() {
        let config = HttpTransportConfig::new();
        let mut adapter = HttpClientTransportAdapter::new(config).await.unwrap();

        // Test setting session context
        adapter.set_session_context(Some("test-session-123".to_string()));
        assert_eq!(adapter.session_id(), Some("test-session-123".to_string()));

        // Test clearing session
        adapter.set_session_context(None);
        assert!(adapter.session_id().is_none());
    }

    #[tokio::test]
    async fn test_message_handler_requirement() {
        let config = HttpTransportConfig::new();
        let mut adapter = HttpClientTransportAdapter::new(config).await.unwrap();

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
    async fn test_context_creation() {
        let config = HttpTransportConfig::new();
        let legacy_transport = Arc::new(Mutex::new(HttpClientTransport::new(config)));

        let message_bytes = br#"{"jsonrpc":"2.0","method":"test","id":1}"#;

        let result = HttpClientTransportAdapter::parse_message_and_create_context(
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
        assert_eq!(context.get_metadata("transport_type"), Some("http-client"));
        assert!(context.get_metadata("client_id").is_some());
    }
}
