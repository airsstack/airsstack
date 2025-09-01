//! STDIO Transport Adapter
//!
//! Bridges the legacy StdioTransport (blocking receive) with the new
//! MCP-compliant Transport trait (event-driven MessageHandler).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use serde_json;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

// Layer 3: Internal module imports
use crate::transport::mcp::{
    JsonRpcMessage, MessageContext, MessageHandler, Transport, TransportError,
};
use crate::transport::StdioTransport;

/// Adapter that bridges StdioTransport with MCP-compliant Transport trait
///
/// This adapter wraps the legacy StdioTransport and provides an event-driven
/// interface through MessageHandler callbacks. It runs an internal event loop
/// that converts blocking `receive()` calls into `handle_message()` events.
///
/// # Architecture
///
/// ```text
/// McpServerBuilder -> StdioTransportAdapter -> Event Loop -> Legacy StdioTransport
///                           (MCP Interface)     (Bridge)      (Blocking I/O)
/// ```
///
/// # Event Loop Pattern
///
/// The adapter spawns a background task that:
/// 1. Continuously calls `legacy_transport.receive()`
/// 2. Parses received bytes into JsonRpcMessage
/// 3. Calls `handler.handle_message()` with the parsed message
/// 4. Handles transport errors by calling `handler.handle_error()`
/// 5. Supports graceful shutdown via cancellation token
///
/// # Backward Compatibility
///
/// Existing code using StdioTransport can use this adapter without changes:
/// ```rust,no_run
/// use airs_mcp::transport::StdioTransport;
/// use airs_mcp::transport::adapters::StdioTransportAdapter;
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     // Old code (still works)
///     let transport = StdioTransport::new().await?;
///
///     // New code (same API, event-driven internally)
///     let transport = StdioTransportAdapter::new().await?;
///     Ok(())
/// }
/// ```
///
/// # Examples
///
/// ```rust,no_run
/// # use airs_mcp::transport::adapters::StdioTransportAdapter;
/// # use airs_mcp::transport::mcp::{Transport, MessageHandler, JsonRpcMessage, MessageContext, TransportError};
/// # use async_trait::async_trait;
/// # use std::sync::Arc;
///
/// struct EchoHandler;
///
/// #[async_trait]
/// impl MessageHandler for EchoHandler {
///     async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext) {
///         println!("Received: {:?}", message);
///     }
///
///     async fn handle_error(&self, error: TransportError) {
///         eprintln!("Transport error: {}", error);
///     }
///
///     async fn handle_close(&self) {
///         println!("Transport closed");
///     }
/// }
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// #     example().await
/// # }
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let mut transport = StdioTransportAdapter::new().await?;
///     let handler = Arc::new(EchoHandler);
///     
///     transport.set_message_handler(handler);
///     transport.start().await?;
///     
///     // Transport now processes messages via event-driven callbacks
///     
///     transport.close().await?;
///     Ok(())
/// }
/// ```
pub struct StdioTransportAdapter {
    /// Wrapped legacy transport (None when closed)
    legacy_transport: Option<StdioTransport>,

    /// Event-driven message handler
    message_handler: Option<Arc<dyn MessageHandler>>,

    /// Event loop control
    running: Arc<AtomicBool>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    event_loop_handle: Option<JoinHandle<()>>,

    /// Session context (STDIO is single-session)
    session_id: Option<String>,
}

impl StdioTransportAdapter {
    /// Create a new STDIO transport adapter
    ///
    /// This initializes the adapter with a legacy StdioTransport but does not
    /// start the event loop. Call `start()` to begin event-driven processing.
    ///
    /// # Returns
    ///
    /// * `Ok(StdioTransportAdapter)` - Successfully created adapter
    /// * `Err(TransportError)` - Failed to create underlying StdioTransport
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use airs_mcp::transport::adapters::StdioTransportAdapter;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #     example().await
    /// # }
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let adapter = StdioTransportAdapter::new().await?;
    ///     // Adapter created but not started yet
    ///     Ok(())
    /// }
    /// ```
    pub async fn new() -> Result<Self, TransportError> {
        let legacy_transport =
            StdioTransport::new()
                .await
                .map_err(|e| TransportError::Connection {
                    message: format!("Failed to create StdioTransport: {e}"),
                })?;

        Ok(Self {
            legacy_transport: Some(legacy_transport),
            message_handler: None,
            running: Arc::new(AtomicBool::new(false)),
            shutdown_tx: None,
            event_loop_handle: None,
            session_id: Some("stdio-session".to_string()), // STDIO has a single session
        })
    }

    /// Create adapter with custom configuration
    ///
    /// Allows specifying custom message size limits and other StdioTransport options.
    ///
    /// # Arguments
    ///
    /// * `max_message_size` - Maximum message size in bytes
    ///
    /// # Returns
    ///
    /// * `Ok(StdioTransportAdapter)` - Successfully created adapter with custom config
    /// * `Err(TransportError)` - Failed to create underlying StdioTransport
    pub async fn with_config(max_message_size: usize) -> Result<Self, TransportError> {
        let legacy_transport = StdioTransport::with_max_message_size(max_message_size)
            .await
            .map_err(|e| TransportError::Connection {
                message: format!("Failed to create StdioTransport: {e}"),
            })?;

        Ok(Self {
            legacy_transport: Some(legacy_transport),
            message_handler: None,
            running: Arc::new(AtomicBool::new(false)),
            shutdown_tx: None,
            event_loop_handle: None,
            session_id: Some("stdio-session".to_string()),
        })
    }
}

#[async_trait]
impl Transport for StdioTransportAdapter {
    type Error = TransportError;

    /// Start the transport and begin event-driven message processing
    ///
    /// This method spawns a background event loop task that continuously reads
    /// from the legacy StdioTransport and converts messages to handler events.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Event loop started successfully
    /// * `Err(TransportError)` - Failed to start (no handler set, already running, etc.)
    ///
    /// # Errors
    ///
    /// - `TransportError::Closed` - Transport has been closed
    /// - `TransportError::Connection` - No message handler set
    /// - `TransportError::Connection` - Already running
    async fn start(&mut self) -> Result<(), Self::Error> {
        // Validate state
        if self.legacy_transport.is_none() {
            return Err(TransportError::Closed);
        }

        if self.message_handler.is_none() {
            return Err(TransportError::Connection {
                message: "No message handler set".to_string(),
            });
        }

        if self.running.load(Ordering::Acquire) {
            return Err(TransportError::Connection {
                message: "Transport already running".to_string(),
            });
        }

        // Set up shutdown channel
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        // Extract components for event loop
        let legacy_transport = self.legacy_transport.take().unwrap();
        let handler = self.message_handler.as_ref().unwrap().clone();
        let running = self.running.clone();

        // Start event loop
        let event_loop_handle = tokio::spawn(async move {
            event_loop(legacy_transport, handler, shutdown_rx, running).await;
        });

        self.event_loop_handle = Some(event_loop_handle);
        self.running.store(true, Ordering::Release);

        Ok(())
    }

    /// Close the transport and clean up resources
    ///
    /// This method gracefully shuts down the event loop and closes the underlying
    /// StdioTransport. It is idempotent and safe to call multiple times.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Transport closed successfully
    /// * `Err(TransportError)` - Error during closure (resources may still be cleaned up)
    async fn close(&mut self) -> Result<(), Self::Error> {
        if !self.running.load(Ordering::Acquire) {
            return Ok(()); // Already closed
        }

        self.running.store(false, Ordering::Release);

        // Signal event loop to shutdown
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(()); // Ignore errors, event loop may have exited
        }

        // Wait for event loop to finish
        if let Some(handle) = self.event_loop_handle.take() {
            let _ = handle.await; // Ignore errors from event loop
        }

        // Close legacy transport if it's still available
        if let Some(mut legacy_transport) = self.legacy_transport.take() {
            // Import trait for close method
            use crate::transport::traits::Transport as LegacyTransportTrait;
            let _ = legacy_transport.close().await; // Ignore errors, we're closing anyway
        }

        Ok(())
    }

    /// Send a JSON-RPC message through the transport
    ///
    /// This method sends the message through the underlying StdioTransport.
    /// The message is serialized to JSON and sent via stdout.
    ///
    /// # Arguments
    ///
    /// * `message` - JSON-RPC message to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(TransportError)` - Failed to send message
    ///
    /// # Errors
    ///
    /// - `TransportError::Closed` - Transport has been closed
    /// - `TransportError::Serialization` - Failed to serialize message
    /// - `TransportError::Io` - I/O error during send
    async fn send(&mut self, message: JsonRpcMessage) -> Result<(), Self::Error> {
        if self.legacy_transport.is_none() {
            return Err(TransportError::Closed);
        }

        // Serialize message to JSON bytes
        let _message_bytes = serde_json::to_vec(&message)
            .map_err(|e| TransportError::Serialization { source: e })?;

        // Send through legacy transport
        // Note: We need to access the legacy transport without taking ownership
        // For now, we'll implement a send method that works with the current architecture
        // This is a limitation of the adapter pattern - we may need to redesign this

        // TODO: Implement proper send mechanism
        // This requires either:
        // 1. Modifying StdioTransport to be Send + Sync
        // 2. Using a different approach for sending messages
        // 3. Implementing a message queue system

        // For now, return an error indicating this needs implementation
        Err(TransportError::Connection {
            message: "Send not yet implemented in adapter".to_string(),
        })
    }

    /// Set the message handler for incoming messages
    ///
    /// The transport will call the handler's methods for each incoming message,
    /// transport error, and transport closure event.
    ///
    /// # Arguments
    ///
    /// * `handler` - Handler for incoming messages and events
    fn set_message_handler(&mut self, handler: Arc<dyn MessageHandler>) {
        self.message_handler = Some(handler);
    }

    /// Get the current session ID
    ///
    /// For STDIO transport, this returns a static session identifier since
    /// STDIO represents a single persistent session.
    ///
    /// # Returns
    ///
    /// Session ID string ("stdio-session")
    fn session_id(&self) -> Option<String> {
        self.session_id.clone()
    }

    /// Set session context for the transport
    ///
    /// For STDIO transport, the session context is largely static, but this
    /// method allows customizing the session identifier if needed.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Optional session identifier
    fn set_session_context(&mut self, session_id: Option<String>) {
        self.session_id = session_id.or_else(|| Some("stdio-session".to_string()));
    }

    /// Check if the transport is currently connected
    ///
    /// # Returns
    ///
    /// `true` if transport is running and can send/receive messages
    fn is_connected(&self) -> bool {
        self.running.load(Ordering::Acquire) && self.legacy_transport.is_some()
    }

    /// Get the transport type identifier
    ///
    /// # Returns
    ///
    /// Static string identifying this as a STDIO transport adapter
    fn transport_type(&self) -> &'static str {
        "stdio-adapter"
    }
}

/// Event loop that bridges blocking StdioTransport with event-driven MessageHandler
///
/// This function runs in a background task and continuously reads from the legacy
/// StdioTransport, converting received messages into MessageHandler events.
///
/// # Arguments
///
/// * `transport` - Legacy StdioTransport for blocking I/O
/// * `handler` - MessageHandler for event-driven callbacks  
/// * `shutdown_rx` - Oneshot receiver for graceful shutdown signal
/// * `running` - Atomic boolean to track running state
async fn event_loop(
    mut transport: StdioTransport,
    handler: Arc<dyn MessageHandler>,
    mut shutdown_rx: oneshot::Receiver<()>,
    running: Arc<AtomicBool>,
) {
    // Import trait to enable method calls
    use crate::transport::traits::Transport as LegacyTransportTrait;

    loop {
        tokio::select! {
            // Handle shutdown signal
            _ = &mut shutdown_rx => {
                tracing::debug!("STDIO adapter event loop received shutdown signal");
                handler.handle_close().await;
                break;
            }

            // Handle incoming messages
            result = transport.receive() => {
                match result {
                    Ok(bytes) => {
                        // Parse JSON-RPC message
                        match parse_jsonrpc_message(&bytes) {
                            Ok(message) => {
                                let context = MessageContext::default();
                                handler.handle_message(message, context).await;
                            }
                            Err(e) => {
                                let error = TransportError::Serialization { source: e };
                                handler.handle_error(error).await;
                            }
                        }
                    }
                    Err(e) => {
                        // Convert transport error and notify handler
                        let transport_error = convert_legacy_error(e);
                        let is_closed = matches!(transport_error, TransportError::Closed);
                        handler.handle_error(transport_error).await;

                        // For STDIO, connection errors usually mean shutdown
                        if is_closed {
                            tracing::debug!("STDIO transport closed, shutting down event loop");
                            handler.handle_close().await;
                            break;
                        }
                    }
                }
            }
        }
    }

    running.store(false, Ordering::Release);
    tracing::debug!("STDIO adapter event loop terminated");
}

/// Parse raw bytes into JsonRpcMessage
///
/// This function handles the conversion from legacy byte arrays to the new
/// MCP-compliant JsonRpcMessage type.
///
/// # Arguments
///
/// * `bytes` - Raw message bytes from legacy transport
///
/// # Returns
///
/// * `Ok(JsonRpcMessage)` - Successfully parsed message
/// * `Err(serde_json::Error)` - JSON parsing failed
fn parse_jsonrpc_message(bytes: &[u8]) -> Result<JsonRpcMessage, serde_json::Error> {
    serde_json::from_slice(bytes)
}

/// Convert legacy transport errors to MCP transport errors
///
/// This function maps errors from the legacy StdioTransport to the new
/// MCP-compliant TransportError type.
///
/// # Arguments
///
/// * `legacy_error` - Error from legacy StdioTransport
///
/// # Returns
///
/// Equivalent TransportError for the MCP interface
fn convert_legacy_error(legacy_error: crate::transport::TransportError) -> TransportError {
    match legacy_error {
        crate::transport::TransportError::Io(io_error) => TransportError::Io { source: io_error },
        crate::transport::TransportError::Closed => TransportError::Closed,
        crate::transport::TransportError::BufferOverflow { details } => TransportError::Transport {
            message: format!("Buffer overflow: {details}"),
        },
        crate::transport::TransportError::Format { message } => TransportError::Transport {
            message: format!("Format error: {message}"),
        },
        crate::transport::TransportError::Timeout { duration_ms } => {
            TransportError::Timeout { duration_ms }
        }
        // Map other variants to Transport with descriptive message
        other => TransportError::Transport {
            message: format!("Transport error: {other}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Mock handler for testing
    struct MockHandler {
        messages: Arc<Mutex<Vec<JsonRpcMessage>>>,
        errors: Arc<Mutex<Vec<TransportError>>>,
        close_called: Arc<AtomicBool>,
    }

    impl MockHandler {
        fn new() -> Self {
            Self {
                messages: Arc::new(Mutex::new(Vec::new())),
                errors: Arc::new(Mutex::new(Vec::new())),
                close_called: Arc::new(AtomicBool::new(false)),
            }
        }

        #[allow(dead_code)]
        fn get_messages(&self) -> Vec<JsonRpcMessage> {
            self.messages.lock().unwrap().clone()
        }

        #[allow(dead_code)]
        fn get_errors(&self) -> Vec<String> {
            self.errors
                .lock()
                .unwrap()
                .iter()
                .map(|e| e.to_string())
                .collect()
        }

        #[allow(dead_code)]
        fn was_close_called(&self) -> bool {
            self.close_called.load(Ordering::Acquire)
        }
    }

    #[async_trait]
    impl MessageHandler for MockHandler {
        async fn handle_message(&self, message: JsonRpcMessage, _context: MessageContext) {
            self.messages.lock().unwrap().push(message);
        }

        async fn handle_error(&self, error: TransportError) {
            self.errors.lock().unwrap().push(error);
        }

        async fn handle_close(&self) {
            self.close_called.store(true, Ordering::Release);
        }
    }

    #[tokio::test]
    async fn test_adapter_creation() {
        let adapter = StdioTransportAdapter::new().await;
        assert!(adapter.is_ok());

        let adapter = adapter.unwrap();
        assert_eq!(adapter.transport_type(), "stdio-adapter");
        assert!(!adapter.is_connected()); // Not started yet
        assert_eq!(adapter.session_id(), Some("stdio-session".to_string()));
    }

    #[tokio::test]
    async fn test_adapter_with_config() {
        let adapter = StdioTransportAdapter::with_config(1024).await;
        assert!(adapter.is_ok());
    }

    #[tokio::test]
    async fn test_adapter_lifecycle() {
        let mut adapter = StdioTransportAdapter::new().await.unwrap();
        let handler = Arc::new(MockHandler::new());

        // Test initial state
        assert!(!adapter.is_connected());

        // Test start without handler fails
        let result = adapter.start().await;
        assert!(result.is_err());

        // Set handler and test start
        adapter.set_message_handler(handler.clone());

        // Note: In a real test environment, we'd need a way to provide test input
        // For now, we'll test the error cases and lifecycle management

        // Test close when not running
        let result = adapter.close().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_session_management() {
        let mut adapter = StdioTransportAdapter::new().await.unwrap();

        // Test default session
        assert_eq!(adapter.session_id(), Some("stdio-session".to_string()));

        // Test custom session
        adapter.set_session_context(Some("custom-session".to_string()));
        assert_eq!(adapter.session_id(), Some("custom-session".to_string()));

        // Test None becomes default
        adapter.set_session_context(None);
        assert_eq!(adapter.session_id(), Some("stdio-session".to_string()));
    }

    #[test]
    fn test_parse_jsonrpc_message() {
        // Test valid JSON-RPC message
        let json = r#"{"jsonrpc":"2.0","method":"ping","id":"1"}"#;
        let result = parse_jsonrpc_message(json.as_bytes());
        assert!(result.is_ok());

        let message = result.unwrap();
        assert_eq!(message.method, Some("ping".to_string()));
        assert_eq!(message.id, Some(serde_json::Value::String("1".to_string())));

        // Test invalid JSON
        let invalid_json = b"invalid json";
        let result = parse_jsonrpc_message(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_legacy_error() {
        use std::io;

        // Test I/O error conversion
        let io_error = crate::transport::TransportError::Io(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "pipe broken",
        ));
        let converted = convert_legacy_error(io_error);
        assert!(matches!(converted, TransportError::Io { .. }));

        // Test closed error conversion
        let closed_error = crate::transport::TransportError::Closed;
        let converted = convert_legacy_error(closed_error);
        assert!(matches!(converted, TransportError::Closed));

        // Test buffer overflow conversion
        let buffer_error = crate::transport::TransportError::BufferOverflow {
            details: "too big".to_string(),
        };
        let converted = convert_legacy_error(buffer_error);
        assert!(matches!(converted, TransportError::Transport { .. }));
    }
}
