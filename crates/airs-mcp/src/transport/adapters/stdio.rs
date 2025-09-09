//! STDIO Transport Implementation
//!
//! This module provides a modern STDIO transport implementation using the
//! unified protocol module Transport trait for event-driven message handling.

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;

// Layer 3: Internal module imports
use crate::protocol::{JsonRpcMessage, MessageContext, MessageHandler, Transport, TransportError};

/// Modern STDIO transport implementation
///
/// This transport reads JSON-RPC messages from stdin and writes responses to stdout,
/// using the event-driven Transport trait for clean separation of concerns.
///
/// # Architecture
///
/// ```text
/// stdin -> StdioTransport -> MessageHandler -> stdout
///          (event-driven)   (protocol logic)  (responses)
/// ```
///
/// # Examples
///
/// ```rust,no_run
/// use airs_mcp::protocol::{Transport, MessageHandler, JsonRpcMessage, MessageContext, TransportError};
/// use airs_mcp::transport::adapters::StdioTransport;
/// use async_trait::async_trait;
/// use std::sync::Arc;
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
///     let mut transport = StdioTransport::new();
///     let handler = Arc::new(EchoHandler);
///     
///     transport.set_message_handler(handler);
///     transport.start().await?;
///     
///     // Transport processes messages via event-driven callbacks
///     
///     transport.close().await?;
///     Ok(())
/// }
/// ```
pub struct StdioTransport {
    /// Event-driven message handler
    message_handler: Option<Arc<dyn MessageHandler>>,

    /// Shutdown signal broadcaster
    shutdown_tx: Option<broadcast::Sender<()>>,

    /// Session context (STDIO is single-session)
    session_id: String,

    /// Connection state
    is_running: bool,
}

impl StdioTransport {
    /// Create a new StdioTransport with maximum message size configuration
    ///
    /// This function is removed - use new() and configure separately if needed
    pub fn with_max_message_size(_max_message_size: usize) -> Self {
        Self::new()
    }
    pub fn new() -> Self {
        Self {
            message_handler: None,
            shutdown_tx: None,
            session_id: "stdio-session".to_string(),
            is_running: false,
        }
    }

    /// Create transport with custom session ID
    ///
    /// # Arguments
    ///
    /// * `session_id` - Custom session identifier
    ///
    /// # Returns
    ///
    /// A new StdioTransport with the specified session ID
    pub fn with_session_id(session_id: String) -> Self {
        Self {
            message_handler: None,
            shutdown_tx: None,
            session_id,
            is_running: false,
        }
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for StdioTransport {
    type Error = TransportError;

    /// Start the transport and begin event-driven message processing
    ///
    /// This spawns a background task that reads from stdin and processes
    /// messages through the configured MessageHandler.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Transport started successfully
    /// * `Err(TransportError)` - Failed to start transport
    ///
    /// # Errors
    ///
    /// - `TransportError::Connection` - No message handler set
    /// - `TransportError::Connection` - Already running
    async fn start(&mut self) -> Result<(), Self::Error> {
        if self.message_handler.is_none() {
            return Err(TransportError::Connection {
                message: "No message handler set".to_string(),
            });
        }

        if self.is_running {
            return Err(TransportError::Connection {
                message: "Transport already running".to_string(),
            });
        }

        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let handler = self.message_handler.as_ref().unwrap().clone();
        let session_id = self.session_id.clone();

        // Spawn stdin reader task
        tokio::spawn(async move {
            stdin_reader_loop(handler, session_id, shutdown_rx).await;
        });

        self.is_running = true;
        Ok(())
    }

    /// Close the transport and clean up resources
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Transport closed successfully
    async fn close(&mut self) -> Result<(), Self::Error> {
        if !self.is_running {
            return Ok(());
        }

        // Signal shutdown
        if let Some(shutdown_tx) = &self.shutdown_tx {
            let _ = shutdown_tx.send(());
        }

        self.is_running = false;
        self.shutdown_tx = None;

        Ok(())
    }

    /// Send a JSON-RPC message through stdout
    ///
    /// # Arguments
    ///
    /// * `message` - JSON-RPC message to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(TransportError)` - Failed to send message
    async fn send(&mut self, message: &JsonRpcMessage) -> Result<(), Self::Error> {
        // Serialize message to JSON
        let json = serde_json::to_string(message)
            .map_err(|e| TransportError::Serialization { source: e })?;

        // Write to stdout with newline delimiter
        let mut stdout = tokio::io::stdout();
        stdout
            .write_all(json.as_bytes())
            .await
            .map_err(|e| TransportError::Io { source: e })?;
        stdout
            .write_all(b"\n")
            .await
            .map_err(|e| TransportError::Io { source: e })?;
        stdout
            .flush()
            .await
            .map_err(|e| TransportError::Io { source: e })?;

        Ok(())
    }

    /// Set the message handler for incoming messages
    ///
    /// # Arguments
    ///
    /// * `handler` - Handler for incoming messages and events
    fn set_message_handler(&mut self, handler: Arc<dyn MessageHandler>) {
        self.message_handler = Some(handler);
    }

    /// Get the current session ID
    ///
    /// # Returns
    ///
    /// The STDIO session identifier
    fn session_id(&self) -> Option<String> {
        Some(self.session_id.clone())
    }

    /// Set session context for the transport
    ///
    /// # Arguments
    ///
    /// * `session_id` - Optional session identifier
    fn set_session_context(&mut self, session_id: Option<String>) {
        self.session_id = session_id.unwrap_or_else(|| "stdio-session".to_string());
    }

    /// Check if the transport is currently connected
    ///
    /// # Returns
    ///
    /// `true` if transport is running and ready for I/O
    fn is_connected(&self) -> bool {
        self.is_running
    }

    /// Get the transport type identifier
    ///
    /// # Returns
    ///
    /// Static string identifying this as a STDIO transport
    fn transport_type(&self) -> &'static str {
        "stdio"
    }
}

/// Background task that reads from stdin and processes messages
///
/// This function runs the main event loop for STDIO transport, reading
/// line-delimited JSON messages from stdin and dispatching them to the
/// configured MessageHandler.
///
/// # Arguments
///
/// * `handler` - MessageHandler for event callbacks
/// * `session_id` - Session identifier for message context
/// * `shutdown_rx` - Shutdown signal receiver
async fn stdin_reader_loop(
    handler: Arc<dyn MessageHandler>,
    session_id: String,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        tokio::select! {
            // Handle shutdown signal
            _ = shutdown_rx.recv() => {
                handler.handle_close().await;
                break;
            }

            // Read from stdin
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => {
                        // EOF reached, stdin closed
                        handler.handle_close().await;
                        break;
                    }
                    Ok(_) => {
                        // Process the line
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            match serde_json::from_str::<JsonRpcMessage>(trimmed) {
                                Ok(message) => {
                                    let context = MessageContext::new(session_id.clone());
                                    handler.handle_message(message, context).await;
                                }
                                Err(e) => {
                                    let error = TransportError::Serialization { source: e };
                                    handler.handle_error(error).await;
                                }
                            }
                        }
                        line.clear();
                    }
                    Err(e) => {
                        let error = TransportError::Io { source: e };
                        handler.handle_error(error).await;
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Mutex;

    // Mock handler for testing
    struct MockHandler {
        messages: Arc<Mutex<Vec<JsonRpcMessage>>>,
        errors: Arc<Mutex<Vec<String>>>,
        close_called: Arc<AtomicBool>,
        message_count: Arc<AtomicUsize>,
    }

    impl MockHandler {
        fn new() -> Self {
            Self {
                messages: Arc::new(Mutex::new(Vec::new())),
                errors: Arc::new(Mutex::new(Vec::new())),
                close_called: Arc::new(AtomicBool::new(false)),
                message_count: Arc::new(AtomicUsize::new(0)),
            }
        }

        #[allow(dead_code)]
        fn get_messages(&self) -> Vec<JsonRpcMessage> {
            self.messages.lock().unwrap().clone()
        }

        #[allow(dead_code)]
        fn get_errors(&self) -> Vec<String> {
            self.errors.lock().unwrap().clone()
        }

        #[allow(dead_code)]
        fn was_close_called(&self) -> bool {
            self.close_called.load(Ordering::Acquire)
        }

        #[allow(dead_code)]
        fn message_count(&self) -> usize {
            self.message_count.load(Ordering::Acquire)
        }
    }

    #[async_trait]
    impl MessageHandler for MockHandler {
        async fn handle_message(&self, message: JsonRpcMessage, _context: MessageContext) {
            self.messages.lock().unwrap().push(message);
            self.message_count.fetch_add(1, Ordering::Release);
        }

        async fn handle_error(&self, error: TransportError) {
            self.errors.lock().unwrap().push(error.to_string());
        }

        async fn handle_close(&self) {
            self.close_called.store(true, Ordering::Release);
        }
    }

    #[tokio::test]
    async fn test_transport_creation() {
        let transport = StdioTransport::new();
        assert_eq!(transport.transport_type(), "stdio");
        assert!(!transport.is_connected());
        assert_eq!(transport.session_id(), Some("stdio-session".to_string()));
    }

    #[tokio::test]
    async fn test_transport_with_custom_session() {
        let transport = StdioTransport::with_session_id("custom".to_string());
        assert_eq!(transport.session_id(), Some("custom".to_string()));
    }

    #[tokio::test]
    async fn test_transport_lifecycle() {
        let mut transport = StdioTransport::new();
        let handler = Arc::new(MockHandler::new());

        // Test initial state
        assert!(!transport.is_connected());

        // Test start without handler fails
        let result = transport.start().await;
        assert!(result.is_err());

        // Set handler and test operations
        transport.set_message_handler(handler.clone());

        // Note: We can't easily test actual stdin reading in unit tests,
        // but we can test the lifecycle management

        // Test close when not running
        let result = transport.close().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_session_management() {
        let mut transport = StdioTransport::new();

        // Test default session
        assert_eq!(transport.session_id(), Some("stdio-session".to_string()));

        // Test custom session
        transport.set_session_context(Some("custom-session".to_string()));
        assert_eq!(transport.session_id(), Some("custom-session".to_string()));

        // Test None becomes default
        transport.set_session_context(None);
        assert_eq!(transport.session_id(), Some("stdio-session".to_string()));
    }

    #[tokio::test]
    async fn test_send_message() {
        let mut transport = StdioTransport::new();

        // Create a test message
        let message = JsonRpcMessage::from_notification("test_method", None);

        // Test send (this will write to actual stdout in test environment)
        let result = transport.send(&message).await;
        assert!(result.is_ok());
    }
}
