//! STDIO Transport Implementation
//!
//! This module provides a modern STDIO transport implementation using the
//! unified protocol module Transport trait for event-driven message handling.

// Layer 1: Standard library imports
use std::fmt::Debug;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Stdin, Stdout};
use tokio::sync::broadcast;

// Layer 3: Internal module imports
use crate::protocol::{JsonRpcMessage, MessageContext, MessageHandler, Transport, TransportError};

//
// Type Aliases for Default I/O Streams and Convenience
//

/// Default stdin type for production use
pub type DefaultStdin = BufReader<Stdin>;

/// Default stdout type for production use  
pub type DefaultStdout = Stdout;

/// Production STDIO transport with default I/O streams
pub type ProductionStdioTransport = StdioTransport<DefaultStdin, DefaultStdout>;

//
// Legacy Type Aliases for STDIO Transport Convenience
//
// Note: Since type aliases cannot have generic parameters, we define these
// for the specific case of STDIO transport (which uses () as context type)

/// Type alias for STDIO message context (no transport-specific data)
pub type StdioMessageContext = MessageContext<()>;

/// Modern Generic STDIO transport implementation
///
/// This transport reads JSON-RPC messages from a generic reader (R) and writes responses
/// to a generic writer (W), using the event-driven Transport trait for clean separation
/// of concerns.
///
/// # Type Parameters
///
/// * `R` - Reader type implementing AsyncBufReadExt + Unpin + Send + 'static (defaults to DefaultStdin)
/// * `W` - Writer type implementing AsyncWriteExt + Unpin + Send + 'static (defaults to DefaultStdout)
///
/// # Architecture
///
/// ```text
/// reader -> StdioTransport<R,W> -> MessageHandler -> writer
///          (event-driven)        (protocol logic)   (responses)
/// ```
///
/// # Examples
///
/// ```rust,no_run
/// use airs_mcp::protocol::{MessageHandler, JsonRpcMessage, TransportError, MessageContext};
/// use airs_mcp::transport::adapters::stdio::{StdioTransport, StdioTransportBuilder};
/// use async_trait::async_trait;
/// use std::sync::Arc;
///
/// struct EchoHandler;
///
/// #[async_trait]
/// impl MessageHandler<()> for EchoHandler {
///     async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<()>) {
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
///     use airs_mcp::protocol::Transport;
///     
///     let handler = Arc::new(EchoHandler);
///     
///     // Pre-configured transport pattern - no dangerous set_message_handler() calls
///     let mut transport = StdioTransportBuilder::new()
///         .with_message_handler(handler)
///         .build()
///         .await?;
///     
///     transport.start().await?;
///     transport.close().await?;
///     Ok(())
/// }
/// ```
pub struct StdioTransport<R = DefaultStdin, W = DefaultStdout>
where
    R: AsyncBufReadExt + Unpin + Send + Sync + 'static,
    W: AsyncWriteExt + Unpin + Send + Sync + 'static,
{
    /// Event-driven message handler (STDIO uses no transport-specific context)
    message_handler: Option<Arc<dyn MessageHandler<()>>>,

    /// Shutdown signal broadcaster
    shutdown_tx: Option<broadcast::Sender<()>>,

    /// Background task handle for proper shutdown synchronization
    task_handle: Option<tokio::task::JoinHandle<()>>,

    /// Session context (STDIO is single-session)
    session_id: String,

    /// Connection state
    is_running: bool,

    /// Reader for incoming messages (generic for testing)
    reader: Option<R>,

    /// Writer for outgoing messages (generic for testing)
    writer: Option<W>,
}

impl StdioTransport<DefaultStdin, DefaultStdout> {
    /// Create a new StdioTransport with maximum message size configuration
    ///
    /// This function is removed - use new() and configure separately if needed
    pub fn with_max_message_size(_max_message_size: usize) -> Self {
        Self::new()
    }

    /// Create a new production STDIO transport using default stdin/stdout
    ///
    /// This is the standard constructor for production use that maintains
    /// backward compatibility with existing code.
    pub fn new() -> Self {
        Self {
            message_handler: None,
            shutdown_tx: None,
            task_handle: None,
            session_id: "stdio-session".to_string(),
            is_running: false,
            reader: None,
            writer: None,
        }
    }

    /// Create transport with custom session ID using default stdin/stdout
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
            task_handle: None,
            session_id,
            is_running: false,
            reader: None,
            writer: None,
        }
    }
}

impl<R, W> StdioTransport<R, W>
where
    R: AsyncBufReadExt + Unpin + Send + Sync + 'static,
    W: AsyncWriteExt + Unpin + Send + Sync + 'static,
{
    /// Create transport with custom I/O streams
    ///
    /// This constructor enables dependency injection for testing and
    /// alternative I/O configurations.
    ///
    /// # Arguments
    ///
    /// * `reader` - Custom reader implementing AsyncBufReadExt
    /// * `writer` - Custom writer implementing AsyncWriteExt
    /// * `session_id` - Optional session identifier
    ///
    /// # Returns
    ///
    /// A new StdioTransport with custom I/O streams
    pub fn with_custom_io(reader: R, writer: W, session_id: Option<String>) -> Self {
        Self {
            message_handler: None,
            shutdown_tx: None,
            task_handle: None,
            session_id: session_id.unwrap_or_else(|| "stdio-session".to_string()),
            is_running: false,
            reader: Some(reader),
            writer: Some(writer),
        }
    }
}

impl Default for StdioTransport<DefaultStdin, DefaultStdout> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R, W> Debug for StdioTransport<R, W>
where
    R: AsyncBufReadExt + Unpin + Send + Sync + 'static,
    W: AsyncWriteExt + Unpin + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StdioTransport")
            .field(
                "message_handler",
                &self
                    .message_handler
                    .as_ref()
                    .map(|_| "Arc<dyn MessageHandler<()>>"),
            )
            .field(
                "shutdown_tx",
                &self.shutdown_tx.as_ref().map(|_| "broadcast::Sender<()>"),
            )
            .field("session_id", &self.session_id)
            .field("is_running", &self.is_running)
            .field("reader", &self.reader.as_ref().map(|_| "R"))
            .field("writer", &self.writer.as_ref().map(|_| "W"))
            .finish()
    }
}

impl<R, W> StdioTransport<R, W>
where
    R: AsyncBufReadExt + Unpin + Send + Sync + 'static,
    W: AsyncWriteExt + Unpin + Send + Sync + 'static,
{
    /// Wait for the background reader task to complete
    ///
    /// This method allows waiting for the transport to finish processing
    /// without polling. When reader reaches EOF, the background task will
    /// complete and this method will return.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Background task completed successfully
    /// * `Err(TransportError)` - Task completed with an error
    pub async fn wait_for_completion(&mut self) -> Result<(), TransportError> {
        if let Some(task_handle) = self.task_handle.take() {
            task_handle.await.map_err(|e| TransportError::Connection {
                message: format!("Background task failed: {e}"),
            })?;
            self.is_running = false;
        }
        Ok(())
    }
}

#[async_trait]
impl<R, W> Transport for StdioTransport<R, W>
where
    R: AsyncBufReadExt + Unpin + Send + Sync + 'static,
    W: AsyncWriteExt + Unpin + Send + Sync + 'static,
{
    type Error = TransportError;

    /// Start the transport and begin event-driven message processing
    ///
    /// This spawns a background task that reads from the configured reader and processes
    /// messages through the configured MessageHandler.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Transport started successfully
    /// * `Err(TransportError)` - Failed to start transport
    ///
    /// # Errors
    ///
    /// - `TransportError::Connection` - Already running
    /// - `TransportError::Connection` - No message handler configured (should not happen with pre-configured pattern)
    async fn start(&mut self) -> Result<(), Self::Error> {
        if self.is_running {
            return Err(TransportError::Connection {
                message: "Transport already running".to_string(),
            });
        }

        // With pre-configured pattern, handler should always be set
        let handler = self.message_handler.as_ref()
            .ok_or_else(|| TransportError::Connection {
                message: "No message handler configured. Use StdioTransportBuilder for pre-configured setup.".to_string(),
            })?
            .clone();

        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let session_id = self.session_id.clone();

        // Handle different cases: custom I/O provided vs production default
        if let Some(reader) = self.reader.take() {
            // Custom I/O case - use generic reader loop
            let task_handle = tokio::spawn(async move {
                generic_reader_loop(reader, handler, session_id, shutdown_rx).await;
            });
            self.task_handle = Some(task_handle);
        } else {
            // Production case - use stdin directly (this will be specialized for production types)
            let task_handle = tokio::spawn(async move {
                stdin_reader_loop(handler, session_id, shutdown_rx).await;
            });
            self.task_handle = Some(task_handle);
        }

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

        // Wait for background task to complete
        if let Some(task_handle) = self.task_handle.take() {
            let _ = task_handle.await;
        }

        self.is_running = false;
        self.shutdown_tx = None;

        Ok(())
    }

    /// Send a JSON-RPC message through the configured writer
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

        if let Some(writer) = self.writer.as_mut() {
            // Use custom writer if provided
            writer
                .write_all(json.as_bytes())
                .await
                .map_err(|e| TransportError::Io { source: e })?;
            writer
                .write_all(b"\n")
                .await
                .map_err(|e| TransportError::Io { source: e })?;
            writer
                .flush()
                .await
                .map_err(|e| TransportError::Io { source: e })?;
        } else {
            // Default to stdout for backward compatibility
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
        }

        Ok(())
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

/// Generic background task that reads from any reader and processes messages
///
/// This function runs the main event loop for generic I/O transport, reading
/// line-delimited JSON messages from the provided reader and dispatching them to the
/// configured MessageHandler. This enables dependency injection for testing.
///
/// # Arguments
///
/// * `reader` - Generic reader implementing AsyncBufReadExt
/// * `handler` - MessageHandler for event callbacks
/// * `session_id` - Session identifier for message context
/// * `shutdown_rx` - Shutdown signal receiver
async fn generic_reader_loop<R>(
    mut reader: R,
    handler: Arc<dyn MessageHandler<()>>,
    session_id: String,
    mut shutdown_rx: broadcast::Receiver<()>,
) where
    R: AsyncBufReadExt + Unpin + Send + 'static,
{
    let mut line = String::new();

    loop {
        tokio::select! {
            // Handle shutdown signal
            _ = shutdown_rx.recv() => {
                handler.handle_close().await;
                break;
            }

            // Read from generic reader
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => {
                        // EOF reached, reader closed
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
    handler: Arc<dyn MessageHandler<()>>,
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

/// Builder for creating pre-configured STDIO transports
///
/// This builder implements the pre-configured transport pattern where
/// transports are created with their message handlers already set,
/// eliminating the dangerous `set_message_handler()` pattern.
///
/// # Examples
///
/// ```rust,no_run
/// use airs_mcp::protocol::{MessageHandler, JsonRpcMessage, MessageContext, TransportError, Transport};
/// use airs_mcp::transport::adapters::stdio::StdioTransportBuilder;
/// use async_trait::async_trait;
/// use std::sync::Arc;
///
/// struct EchoHandler;
///
/// #[async_trait]
/// impl MessageHandler<()> for EchoHandler {
///     async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<()>) {
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
///     let handler = Arc::new(EchoHandler);
///     
///     // Pre-configured transport pattern - no dangerous set_message_handler() calls
///     let mut transport = StdioTransportBuilder::new()
///         .with_message_handler(handler)
///         .build()
///         .await?;
///     
///     transport.start().await?;
///     transport.close().await?;
///     Ok(())
/// }
/// ```
pub struct StdioTransportBuilder<R = DefaultStdin, W = DefaultStdout>
where
    R: AsyncBufReadExt + Unpin + Send + Sync + 'static,
    W: AsyncWriteExt + Unpin + Send + Sync + 'static,
{
    /// Message handler for the transport (set via with_message_handler)
    message_handler: Option<Arc<dyn MessageHandler<()>>>,

    /// Custom reader for dependency injection (None means use default stdin)
    custom_reader: Option<R>,

    /// Custom writer for dependency injection (None means use default stdout)
    custom_writer: Option<W>,

    /// Custom session ID (None means use default "stdio-session")
    session_id: Option<String>,
}

impl<R, W> StdioTransportBuilder<R, W>
where
    R: AsyncBufReadExt + Unpin + Send + Sync + 'static,
    W: AsyncWriteExt + Unpin + Send + Sync + 'static,
{
    /// Create a new STDIO transport builder with custom I/O streams
    pub fn new_with_custom_io(reader: R, writer: W) -> Self {
        Self {
            message_handler: None,
            custom_reader: Some(reader),
            custom_writer: Some(writer),
            session_id: None,
        }
    }

    /// Set the message handler for the transport
    ///
    /// This implements the transport-specific construction pattern.
    /// The handler must be set before building the transport.
    pub fn with_message_handler(mut self, handler: Arc<dyn MessageHandler<()>>) -> Self {
        self.message_handler = Some(handler);
        self
    }

    /// Set a custom session ID for the transport
    ///
    /// If not set, defaults to "stdio-session"
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Build the transport with the configured message handler and I/O streams
    ///
    /// This creates a fully configured transport that is ready to start.
    /// The transport will have its message handler pre-configured.
    pub async fn build(self) -> Result<StdioTransport<R, W>, TransportError> {
        let handler = self
            .message_handler
            .ok_or_else(|| TransportError::Connection {
                message: "Message handler must be set before building transport".to_string(),
            })?;

        let session_id = self
            .session_id
            .unwrap_or_else(|| "stdio-session".to_string());

        Ok(StdioTransport {
            message_handler: Some(handler),
            shutdown_tx: None,
            task_handle: None,
            session_id,
            is_running: false,
            reader: self.custom_reader,
            writer: self.custom_writer,
        })
    }
}

// Production-specific builder implementation for default I/O streams
impl StdioTransportBuilder<DefaultStdin, DefaultStdout> {
    /// Create a new STDIO transport builder for production (default stdin/stdout)
    pub fn new() -> Self {
        Self {
            message_handler: None,
            custom_reader: None,
            custom_writer: None,
            session_id: None,
        }
    }

    /// Convenience method for adding custom I/O streams to production builder
    ///
    /// This allows fluent transition from production to custom I/O:
    /// ```rust,no_run
    /// use airs_mcp::transport::adapters::stdio::StdioTransportBuilder;
    /// use airs_mcp::protocol::{MessageHandler, JsonRpcMessage, MessageContext, TransportError};
    /// use async_trait::async_trait;
    /// use std::sync::Arc;
    /// use std::io::Cursor;
    /// use tokio::io::BufReader;
    ///
    /// struct EchoHandler;
    /// #[async_trait]
    /// impl MessageHandler<()> for EchoHandler {
    ///     async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<()>) {}
    ///     async fn handle_error(&self, error: TransportError) {}
    ///     async fn handle_close(&self) {}
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let handler = Arc::new(EchoHandler);
    /// let reader = BufReader::new(Cursor::new(b"test"));
    /// let writer = Vec::new();
    ///
    /// let transport = StdioTransportBuilder::new()
    ///     .with_message_handler(handler)
    ///     .with_custom_io(reader, writer)
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_custom_io<NewR, NewW>(
        self,
        reader: NewR,
        writer: NewW,
    ) -> StdioTransportBuilder<NewR, NewW>
    where
        NewR: AsyncBufReadExt + Unpin + Send + Sync + 'static,
        NewW: AsyncWriteExt + Unpin + Send + Sync + 'static,
    {
        StdioTransportBuilder {
            message_handler: self.message_handler,
            custom_reader: Some(reader),
            custom_writer: Some(writer),
            session_id: self.session_id,
        }
    }
}

impl Default for StdioTransportBuilder<DefaultStdin, DefaultStdout> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R, W> std::fmt::Debug for StdioTransportBuilder<R, W>
where
    R: AsyncBufReadExt + Unpin + Send + Sync + 'static,
    W: AsyncWriteExt + Unpin + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StdioTransportBuilder")
            .field(
                "message_handler",
                &self
                    .message_handler
                    .as_ref()
                    .map(|_| "Arc<dyn MessageHandler<()>>"),
            )
            .field("custom_reader", &self.custom_reader.as_ref().map(|_| "R"))
            .field("custom_writer", &self.custom_writer.as_ref().map(|_| "W"))
            .field("session_id", &self.session_id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::RequestId;
    use std::io::Cursor;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Mutex;
    use tokio::io::BufReader;
    use tokio::time::Duration;

    // TransportBuilder trait is already imported above

    //
    // Mock I/O Testing Infrastructure
    //

    /// Mock reader that can simulate various I/O scenarios
    pub struct MockReader {
        data: Cursor<Vec<u8>>,
        read_delay: Option<Duration>,
        error_after_bytes: Option<usize>,
        bytes_read: usize,
    }

    impl MockReader {
        /// Create a new mock reader with JSON-RPC messages
        pub fn new(messages: &[&str]) -> Self {
            let mut data = Vec::new();
            for message in messages {
                data.extend_from_slice(message.as_bytes());
                data.push(b'\n'); // Line delimiter
            }

            Self {
                data: Cursor::new(data),
                read_delay: None,
                error_after_bytes: None,
                bytes_read: 0,
            }
        }

        /// Add a delay to each read operation (for testing timeouts)
        #[allow(dead_code)] // Available for future timeout testing
        pub fn with_delay(mut self, delay: Duration) -> Self {
            self.read_delay = Some(delay);
            self
        }

        /// Inject an error after reading specific number of bytes
        #[allow(dead_code)] // Available for future error injection testing
        pub fn with_error_after(mut self, bytes: usize) -> Self {
            self.error_after_bytes = Some(bytes);
            self
        }
    }

    impl tokio::io::AsyncRead for MockReader {
        fn poll_read(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            // Check if we should inject an error
            if let Some(error_after) = self.error_after_bytes {
                if self.bytes_read >= error_after {
                    return std::task::Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        "Mock error injection",
                    )));
                }
            }

            // Simulate delay if configured
            if let Some(_delay) = self.read_delay {
                // For simplicity, we'll skip actual delay in tests
                // In real scenarios, this would use tokio::time::sleep
            }

            let initial_filled = buf.filled().len();
            let result = std::pin::Pin::new(&mut self.data).poll_read(cx, buf);

            if let std::task::Poll::Ready(Ok(())) = result {
                let bytes_read_now = buf.filled().len() - initial_filled;
                self.bytes_read += bytes_read_now;
            }

            result
        }
    }

    /// Mock writer that captures written data for inspection
    #[derive(Clone)]
    pub struct MockWriter {
        data: Arc<Mutex<Vec<u8>>>,
        #[allow(dead_code)] // Available for future delay testing
        write_delay: Option<Duration>,
        should_fail: Arc<AtomicBool>,
    }

    impl MockWriter {
        /// Create a new mock writer
        pub fn new() -> Self {
            Self {
                data: Arc::new(Mutex::new(Vec::new())),
                write_delay: None,
                should_fail: Arc::new(AtomicBool::new(false)),
            }
        }

        /// Add a delay to each write operation
        #[allow(dead_code)] // Available for future timeout testing
        pub fn with_delay(mut self, delay: Duration) -> Self {
            self.write_delay = Some(delay);
            self
        }

        /// Configure writer to fail on next write
        pub fn set_should_fail(&self, should_fail: bool) {
            self.should_fail.store(should_fail, Ordering::Release);
        }

        /// Get all data written to this mock writer
        pub fn get_data(&self) -> Vec<u8> {
            self.data.lock().unwrap().clone()
        }

        /// Get data as string (useful for JSON inspection)
        pub fn get_data_as_string(&self) -> String {
            String::from_utf8_lossy(&self.get_data()).to_string()
        }

        /// Get individual JSON-RPC messages written (split by newlines)
        pub fn get_messages(&self) -> Vec<String> {
            let data = self.get_data_as_string();
            data.lines()
                .filter(|line| !line.trim().is_empty())
                .map(|s| s.to_string())
                .collect()
        }

        /// Clear all captured data
        #[allow(dead_code)] // Available for future test scenarios
        pub fn clear(&self) {
            self.data.lock().unwrap().clear();
        }
    }

    impl tokio::io::AsyncWrite for MockWriter {
        fn poll_write(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<Result<usize, std::io::Error>> {
            if self.should_fail.load(Ordering::Acquire) {
                return std::task::Poll::Ready(Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "Mock write failure",
                )));
            }

            self.data.lock().unwrap().extend_from_slice(buf);
            std::task::Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), std::io::Error>> {
            std::task::Poll::Ready(Ok(()))
        }

        fn poll_shutdown(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), std::io::Error>> {
            std::task::Poll::Ready(Ok(()))
        }
    }

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
    impl MessageHandler<()> for MockHandler {
        async fn handle_message(&self, message: JsonRpcMessage, _context: StdioMessageContext) {
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
        let handler = Arc::new(MockHandler::new());

        // Test creating transport with pre-configured pattern
        let transport_result = StdioTransportBuilder::new()
            .with_message_handler(handler.clone())
            .build()
            .await;

        assert!(transport_result.is_ok());
        let mut transport = transport_result.unwrap();

        // Test initial state
        assert!(!transport.is_connected());

        // Test that transport without handler (created with new()) fails to start
        let mut basic_transport = StdioTransport::new();
        let start_result = basic_transport.start().await;
        assert!(start_result.is_err());
        assert!(start_result
            .unwrap_err()
            .to_string()
            .contains("No message handler configured"));

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

    #[tokio::test]
    async fn test_transport_builder() {
        let handler = Arc::new(MockHandler::new());

        // Test successful build with handler
        let transport_result = StdioTransportBuilder::new()
            .with_message_handler(handler.clone())
            .build()
            .await;

        assert!(transport_result.is_ok());
        let transport = transport_result.unwrap();
        assert_eq!(transport.transport_type(), "stdio");
        assert_eq!(transport.session_id(), Some("stdio-session".to_string()));

        // Test builder without handler fails
        let builder_without_handler = StdioTransportBuilder::new();
        let result = builder_without_handler.build().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Message handler must be set"));
    }

    #[tokio::test]
    async fn test_generic_builder_with_custom_io() {
        use std::io::Cursor;
        use tokio::io::BufReader;

        let handler = Arc::new(MockHandler::new());

        // Create mock I/O streams
        let input_data = r#"{"jsonrpc":"2.0","method":"test","id":1}"#;
        let reader = BufReader::new(Cursor::new(input_data.as_bytes()));
        let writer = Vec::new();

        // Test builder with custom I/O
        let transport_result = StdioTransportBuilder::new_with_custom_io(reader, writer)
            .with_message_handler(handler.clone())
            .with_session_id("test-session".to_string())
            .build()
            .await;

        assert!(transport_result.is_ok());
        let transport = transport_result.unwrap();
        assert_eq!(transport.transport_type(), "stdio");
        assert_eq!(transport.session_id(), Some("test-session".to_string()));
        assert!(!transport.is_connected());

        // Test fluent API with production builder transitioning to custom I/O
        let input_data2 = r#"{"jsonrpc":"2.0","method":"test2","id":2}"#;
        let reader2 = BufReader::new(Cursor::new(input_data2.as_bytes()));
        let writer2 = Vec::new();

        let transport_result2 = StdioTransportBuilder::new()
            .with_message_handler(handler.clone())
            .with_custom_io(reader2, writer2)
            .build()
            .await;

        assert!(transport_result2.is_ok());
        let transport2 = transport_result2.unwrap();
        assert_eq!(transport2.transport_type(), "stdio");
    }

    #[tokio::test]
    async fn test_lifecycle_with_mock_io() {
        let handler = Arc::new(MockHandler::new());

        // Create mock I/O streams with multiple JSON-RPC messages
        let messages = [
            r#"{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}"#,
            r#"{"jsonrpc":"2.0","method":"ping","id":2}"#,
            r#"{"jsonrpc":"2.0","method":"close","id":3}"#,
        ];

        let reader = BufReader::new(MockReader::new(&messages));
        let writer = MockWriter::new();
        let _writer_clone = writer.clone();

        // Build transport with custom I/O
        let mut transport = StdioTransportBuilder::new_with_custom_io(reader, writer)
            .with_message_handler(handler.clone())
            .with_session_id("lifecycle-test".to_string())
            .build()
            .await
            .expect("Failed to build transport");

        // Test lifecycle without blocking on stdin
        assert!(!transport.is_connected());
        assert_eq!(transport.session_id(), Some("lifecycle-test".to_string()));

        // Start transport - this should begin processing mock messages
        transport.start().await.expect("Failed to start transport");
        assert!(transport.is_connected());

        // Give it a moment to process the messages
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Close transport gracefully
        transport.close().await.expect("Failed to close transport");
        assert!(!transport.is_connected());

        // Wait for completion
        transport
            .wait_for_completion()
            .await
            .expect("Transport completion failed");

        // Verify messages were processed
        let received_messages = handler.get_messages();
        assert_eq!(
            received_messages.len(),
            3,
            "Should have received 3 messages"
        );

        // Verify close was called
        assert!(
            handler.was_close_called(),
            "Handler close should have been called"
        );
    }

    #[tokio::test]
    async fn test_bidirectional_communication() {
        let handler = Arc::new(MockHandler::new());

        // Create mock input with a request
        let input_messages = [r#"{"jsonrpc":"2.0","method":"ping","id":42}"#];
        let reader = BufReader::new(MockReader::new(&input_messages));
        let writer = MockWriter::new();
        let writer_clone = writer.clone();

        // Build and start transport
        let mut transport = StdioTransportBuilder::new_with_custom_io(reader, writer)
            .with_message_handler(handler.clone())
            .build()
            .await
            .expect("Failed to build transport");

        transport.start().await.expect("Failed to start transport");

        // Send a response message
        let response = JsonRpcMessage::from_response(
            Some(serde_json::json!("pong")),
            None,
            Some(RequestId::new_number(42)),
        );

        transport
            .send(&response)
            .await
            .expect("Failed to send response");

        // Close and verify
        transport.close().await.expect("Failed to close transport");

        // Check what was written
        let written_messages = writer_clone.get_messages();
        assert_eq!(written_messages.len(), 1, "Should have written 1 message");

        let written_json: serde_json::Value = serde_json::from_str(&written_messages[0])
            .expect("Written message should be valid JSON");

        assert_eq!(written_json["jsonrpc"], "2.0");
        assert_eq!(written_json["result"], "pong");
        assert_eq!(written_json["id"], 42);
    }

    #[tokio::test]
    async fn test_error_handling_scenarios() {
        let handler = Arc::new(MockHandler::new());

        // Test 1: Invalid JSON handling
        let invalid_json = [r#"{"invalid":"json"#]; // Missing closing brace
        let reader = BufReader::new(MockReader::new(&invalid_json));
        let writer = MockWriter::new();

        let mut transport = StdioTransportBuilder::new_with_custom_io(reader, writer)
            .with_message_handler(handler.clone())
            .build()
            .await
            .expect("Failed to build transport");

        transport.start().await.expect("Failed to start transport");
        tokio::time::sleep(Duration::from_millis(10)).await;
        transport.close().await.expect("Failed to close transport");

        // Should have received an error for invalid JSON
        let errors = handler.get_errors();
        assert!(
            !errors.is_empty(),
            "Should have received JSON parsing error"
        );
    }

    #[tokio::test]
    async fn test_write_failure_handling() {
        let handler = Arc::new(MockHandler::new());

        let reader = BufReader::new(MockReader::new(&[])); // Empty reader
        let writer = MockWriter::new();
        writer.set_should_fail(true); // Configure writer to fail

        let mut transport = StdioTransportBuilder::new_with_custom_io(reader, writer)
            .with_message_handler(handler.clone())
            .build()
            .await
            .expect("Failed to build transport");

        transport.start().await.expect("Failed to start transport");

        // Try to send a message - this should fail
        let message = JsonRpcMessage::from_notification("test", None);
        let result = transport.send(&message).await;
        assert!(
            result.is_err(),
            "Send should fail with mock writer configured to fail"
        );

        transport.close().await.expect("Failed to close transport");
    }

    #[tokio::test]
    async fn test_concurrent_message_processing() {
        let handler = Arc::new(MockHandler::new());

        // Create many messages to test concurrent processing
        let messages: Vec<String> = (0..10)
            .map(|i| format!(r#"{{"jsonrpc":"2.0","method":"test_{i}","id":{i}}}"#))
            .collect();

        let message_refs: Vec<&str> = messages.iter().map(|s| s.as_str()).collect();

        let reader = BufReader::new(MockReader::new(&message_refs));
        let writer = MockWriter::new();

        let mut transport = StdioTransportBuilder::new_with_custom_io(reader, writer)
            .with_message_handler(handler.clone())
            .build()
            .await
            .expect("Failed to build transport");

        transport.start().await.expect("Failed to start transport");

        // Give time for all messages to be processed
        tokio::time::sleep(Duration::from_millis(50)).await;

        transport.close().await.expect("Failed to close transport");
        transport
            .wait_for_completion()
            .await
            .expect("Transport completion failed");

        // Verify all messages were processed
        let received_messages = handler.get_messages();
        assert_eq!(
            received_messages.len(),
            10,
            "Should have received all 10 messages"
        );

        // Verify message ordering (should be preserved)
        for (i, msg) in received_messages.iter().enumerate() {
            if let JsonRpcMessage::Request(req) = msg {
                assert_eq!(req.method, format!("test_{i}"));
            }
        }
    }

    #[tokio::test]
    async fn test_transport_without_blocking_stdin() {
        // This test demonstrates that we can now test transport lifecycle
        // without any dependency on actual stdin/stdout blocking behavior

        let handler = Arc::new(MockHandler::new());

        // Simulate a complete MCP session with multiple message types
        let session_messages = [
            r#"{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}}},"id":1}"#,
            r#"{"jsonrpc":"2.0","method":"tools/list","id":2}"#,
            r#"{"jsonrpc":"2.0","method":"tools/call","params":{"name":"test_tool","arguments":{}},"id":3}"#,
            r#"{"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":3}}"#,
        ];

        let reader = BufReader::new(MockReader::new(&session_messages));
        let writer = MockWriter::new();
        let writer_clone = writer.clone();

        let mut transport = StdioTransportBuilder::new_with_custom_io(reader, writer)
            .with_message_handler(handler.clone())
            .with_session_id("mcp-session".to_string())
            .build()
            .await
            .expect("Failed to build transport");

        // Complete lifecycle test that runs without any stdin blocking
        let start_time = std::time::Instant::now();

        transport.start().await.expect("Failed to start transport");

        // Send some responses
        let init_response = JsonRpcMessage::from_response(
            Some(serde_json::json!({"protocolVersion": "2024-11-05", "capabilities": {}})),
            None,
            Some(RequestId::new_number(1)),
        );
        transport
            .send(&init_response)
            .await
            .expect("Failed to send init response");

        let tools_response = JsonRpcMessage::from_response(
            Some(serde_json::json!({"tools": []})),
            None,
            Some(RequestId::new_number(2)),
        );
        transport
            .send(&tools_response)
            .await
            .expect("Failed to send tools response");

        // Allow message processing
        tokio::time::sleep(Duration::from_millis(20)).await;

        transport.close().await.expect("Failed to close transport");
        transport
            .wait_for_completion()
            .await
            .expect("Transport completion failed");

        let elapsed = start_time.elapsed();

        // Verify the test completed quickly (not blocked on stdin)
        assert!(
            elapsed < Duration::from_secs(1),
            "Test should complete quickly without stdin blocking"
        );

        // Verify messages were processed
        let received = handler.get_messages();
        assert!(
            received.len() >= 3,
            "Should have processed request messages"
        ); // notification might not increment count

        // Verify responses were sent
        let sent_messages = writer_clone.get_messages();
        assert_eq!(
            sent_messages.len(),
            2,
            "Should have sent 2 response messages"
        );

        println!("✅ Lifecycle test completed in {elapsed:?} without stdin blocking");
    }
}
