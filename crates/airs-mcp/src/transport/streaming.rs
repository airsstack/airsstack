//! Enhanced transport integration with streaming JSON parser
//!
//! This module provides transport helpers that integrate with the
//! streaming JSON parser for efficient message processing.

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use tokio::sync::Mutex;

// Layer 3: Internal module imports
use crate::protocol::{
    BufferStats, ParsedMessage, StreamingConfig, StreamingError, StreamingParser,
};
use crate::transport::buffer::{BufferConfig, BufferManager};
use crate::transport::{Transport, TransportError, ZeroCopyTransport};

/// Enhanced transport that uses streaming JSON parsing
///
/// This transport wrapper adds streaming JSON parsing capabilities
/// to any existing transport implementation.
///
/// # Features
///
/// - Streaming JSON parsing for memory-efficient processing
/// - Zero-copy buffer management
/// - Configurable buffer sizes and message limits
///
/// # Examples
///
/// ```rust
/// use airs_mcp::transport::streaming::StreamingTransport;
/// use airs_mcp::protocol::StreamingConfig;
/// use airs_mcp::transport::stdio::StdioTransport;
///
/// # tokio_test::block_on(async {
/// let base_transport = StdioTransport::new().await.unwrap();
/// let config = StreamingConfig::default();
/// let transport = StreamingTransport::new(base_transport, config);
/// # });
/// ```
pub struct StreamingTransport<T> {
    inner: Arc<Mutex<T>>,
    parser: Arc<Mutex<StreamingParser>>,
    buffer_manager: Arc<BufferManager>,
}

impl<T> StreamingTransport<T>
where
    T: Transport + ZeroCopyTransport + Send + Sync,
{
    /// Create a new streaming transport wrapping an existing transport
    pub fn new(inner: T, config: StreamingConfig) -> Self {
        let buffer_config = BufferConfig::default();
        let buffer_manager = Arc::new(BufferManager::new(buffer_config));
        Self {
            inner: Arc::new(Mutex::new(inner)),
            parser: Arc::new(Mutex::new(StreamingParser::new(config))),
            buffer_manager,
        }
    }

    /// Create a streaming transport with default configuration
    pub fn new_default(inner: T) -> Self {
        Self::new(inner, StreamingConfig::default())
    }

    /// Parse a JSON message from bytes using the streaming parser
    pub async fn parse_message(&self, data: &[u8]) -> Result<ParsedMessage, StreamingError> {
        let mut parser = self.parser.lock().await;
        // For now, we'll implement a simple parse method
        // This will be expanded when we fully migrate the StreamingParser implementation
        todo!("Parse implementation needs to be completed in StreamingParser")
    }

    /// Receive and parse a message using streaming
    pub async fn receive_parsed(&self) -> Result<ParsedMessage, TransportError> {
        // Receive raw message from underlying transport
        let mut transport = self.inner.lock().await;
        let message_bytes = transport
            .receive()
            .await
            .map_err(|e| TransportError::other(format!("Transport receive error: {e}")))?;

        // Parse using streaming parser
        let parsed = self
            .parse_message(&message_bytes)
            .await
            .map_err(|e| TransportError::other(format!("Streaming parse error: {e}")))?;

        Ok(parsed)
    }

    /// Get buffer usage statistics
    pub async fn buffer_stats(&self) -> BufferStats {
        // Return basic stats for now
        BufferStats {
            capacity: 1024,
            used: 0,
            max_size: 16 * 1024 * 1024,
        }
    }

    /// Get buffer manager for advanced buffer operations
    pub fn buffer_manager(&self) -> &Arc<BufferManager> {
        &self.buffer_manager
    }
}

// Implement Transport trait for StreamingTransport
impl<T: Transport> Transport for StreamingTransport<T> {
    type Error = T::Error;

    async fn send(&mut self, message: &[u8]) -> Result<(), Self::Error> {
        let mut transport = self.inner.lock().await;
        transport.send(message).await
    }

    async fn receive(&mut self) -> Result<Vec<u8>, Self::Error> {
        let mut transport = self.inner.lock().await;
        transport.receive().await
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        let mut transport = self.inner.lock().await;
        transport.close().await
    }
}

/// Statistics for streaming performance monitoring
#[derive(Debug, Clone, Default)]
pub struct StreamingStats {
    /// Number of messages parsed successfully
    pub messages_parsed: u64,
    /// Number of parse errors encountered
    pub parse_errors: u64,
    /// Total bytes processed through streaming parser
    pub bytes_processed: u64,
}

impl StreamingStats {
    /// Create new streaming stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate parsing success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        let total = self.messages_parsed + self.parse_errors;
        if total == 0 {
            1.0
        } else {
            self.messages_parsed as f64 / total as f64
        }
    }
}

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use crate::transport::buffer::{BufferConfig, BufferManager};
use crate::transport::{Transport, TransportError, ZeroCopyTransport};

/// Configuration for streaming JSON parser
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Maximum buffer size for incoming messages
    pub max_buffer_size: usize,
    /// Maximum number of messages to buffer
    pub max_messages: usize,
    /// Enable compression for large messages
    pub enable_compression: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 1024 * 1024, // 1MB
            max_messages: 1000,
            enable_compression: false,
        }
    }
}

/// Errors that can occur during streaming JSON parsing
#[derive(Debug, thiserror::Error)]
pub enum StreamingError {
    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// I/O error during streaming
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Buffer overflow - message too large
    #[error("Buffer overflow - message exceeds maximum size: {0}")]
    BufferOverflow(usize),

    /// Too many messages in buffer
    #[error("Too many messages in buffer: {0}")]
    TooManyMessages(usize),
}

/// A parsed JSON-RPC message that can be processed by the streaming parser
#[derive(Debug, Clone)]
pub enum ParsedMessage {
    /// JSON-RPC request message
    Request(JsonRpcRequest),
    /// JSON-RPC response message
    Response(JsonRpcResponse),
    /// JSON-RPC notification message
    Notification(JsonRpcNotification),
}

impl ParsedMessage {
    /// Get the message type as a string
    pub fn message_type(&self) -> &'static str {
        match self {
            ParsedMessage::Request(_) => "request",
            ParsedMessage::Response(_) => "response",
            ParsedMessage::Notification(_) => "notification",
        }
    }
}

/// Simple streaming parser for JSON-RPC messages
///
/// This is a simplified version that uses the new protocol module types
#[derive(Debug)]
pub struct StreamingParser {
    config: StreamingConfig,
    buffer: Vec<u8>,
}

impl StreamingParser {
    /// Create a new streaming parser with the given configuration
    pub fn new(config: StreamingConfig) -> Self {
        Self {
            config,
            buffer: Vec::with_capacity(config.max_buffer_size),
        }
    }

    /// Create a new streaming parser with default configuration
    pub fn new_default() -> Self {
        Self::new(StreamingConfig::default())
    }

    /// Parse a complete JSON message from bytes
    pub fn parse_message(&mut self, data: &[u8]) -> Result<ParsedMessage, StreamingError> {
        let json_str = std::str::from_utf8(data).map_err(|e| {
            StreamingError::JsonError(serde_json::Error::custom(format!("Invalid UTF-8: {}", e)))
        })?;

        // Try to parse as different message types
        if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(json_str) {
            return Ok(ParsedMessage::Request(request));
        }

        if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(json_str) {
            return Ok(ParsedMessage::Response(response));
        }

        if let Ok(notification) = serde_json::from_str::<JsonRpcNotification>(json_str) {
            return Ok(ParsedMessage::Notification(notification));
        }

        Err(StreamingError::JsonError(serde_json::Error::custom(
            "Could not parse as any JSON-RPC message type",
        )))
    }
}

/// Statistics for buffer performance monitoring
#[derive(Debug, Clone)]
pub struct BufferStats {
    /// Total messages processed
    pub messages_processed: u64,
    /// Current buffer size
    pub buffer_size: usize,
    /// Peak buffer size
    pub peak_buffer_size: usize,
    /// Number of buffer overflows
    pub buffer_overflows: u64,
}

impl Default for BufferStats {
    fn default() -> Self {
        Self {
            messages_processed: 0,
            buffer_size: 0,
            peak_buffer_size: 0,
            buffer_overflows: 0,
        }
    }
}

/// Enhanced transport that uses streaming JSON parsing
///
/// This transport wrapper adds streaming JSON parsing capabilities
/// to any existing transport implementation.
pub struct StreamingTransport<T> {
    /// Inner transport
    inner: T,
    /// Streaming parser instance
    parser: Arc<Mutex<StreamingParser>>,
    /// Buffer manager for efficient memory usage
    buffer_manager: BufferManager,
}

impl<T> StreamingTransport<T> {
    pub fn new(inner: T, config: StreamingConfig) -> Self {
        let buffer_config = BufferConfig {
            initial_capacity: config.max_buffer_size / 4,
            max_capacity: config.max_buffer_size,
            pool_size: 10,
        };

        Self {
            inner,
            parser: Arc::new(Mutex::new(StreamingParser::new(config))),
            buffer_manager: BufferManager::new(buffer_config),
        }
    }

    pub fn new_default(inner: T) -> Self {
        Self::new(inner, StreamingConfig::default())
    }

    /// Parse a message from the given data
    pub async fn parse_message(&self, data: &[u8]) -> Result<ParsedMessage, StreamingError> {
        let mut parser = self.parser.lock().await;
        parser.parse_message(data)
    }

    /// Parse multiple messages from a buffer
    pub async fn parse_multiple(&self, data: &[u8]) -> Result<Vec<ParsedMessage>, StreamingError> {
        let mut parser = self.parser.lock().await;
        let mut messages = Vec::new();

        // Simple implementation - split by newlines and parse each
        for line in data.split(|&b| b == b'\n') {
            if !line.is_empty() {
                match parser.parse_message(line) {
                    Ok(message) => messages.push(message),
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(messages)
    }

    /// Receive and parse a message from the transport
    pub async fn receive_parsed(&self) -> Result<ParsedMessage, TransportError> {
        match &self.inner.receive().await {
            Ok(data) => self.parse_message(data).await.map_err(|e| {
                TransportError::MessageParsing(format!("Streaming parse error: {}", e))
            }),
            Err(e) => Err(e.clone()),
        }
    }

    /// Send a parsed message through the transport
    pub async fn send_parsed(&self, message: &ParsedMessage) -> Result<(), TransportError> {
        let json_data = match message {
            ParsedMessage::Request(req) => serde_json::to_vec(req),
            ParsedMessage::Response(resp) => serde_json::to_vec(resp),
            ParsedMessage::Notification(notif) => serde_json::to_vec(notif),
        }
        .map_err(|e| TransportError::MessageParsing(format!("Serialization error: {}", e)))?;

        self.inner.send(&json_data).await
    }

    /// Get buffer statistics for performance monitoring
    pub async fn buffer_stats(&self) -> BufferStats {
        // Simplified stats - in real implementation this would track actual metrics
        BufferStats::default()
    }
}

// Implement Transport trait for StreamingTransport
impl<T: Transport> Transport for StreamingTransport<T> {
    async fn send(&self, data: &[u8]) -> Result<(), TransportError> {
        self.inner.send(data).await
    }

    async fn receive(&self) -> Result<&[u8], TransportError> {
        self.inner.receive().await
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        self.inner.close().await
    }
}

/// Statistics for streaming performance monitoring
#[derive(Debug, Clone, Default)]
pub struct StreamingStats {
    /// Total messages processed
    pub messages_processed: u64,
    /// Parse errors encountered
    pub parse_errors: u64,
    /// Average message size
    pub avg_message_size: u64,
}

// Re-export key types for backward compatibility
pub use ParsedMessage as LegacyParsedMessage;
pub use StreamingConfig as LegacyStreamingConfig;
pub use StreamingParser as LegacyStreamingParser;

use std::sync::Arc;

use tokio::sync::Mutex;

// Layer 3: Internal module imports
use crate::protocol::{
    BufferStats, ParsedMessage, StreamingConfig, StreamingError, StreamingParser,
};
use crate::transport::buffer::{BufferConfig, BufferManager};
use crate::transport::{Transport, TransportError, ZeroCopyTransport};

/// Enhanced transport that uses streaming JSON parsing
///
/// This transport wrapper adds streaming JSON parsing capabilities
/// to any existing transport implementation.
///
/// # Features
///
/// - Streaming JSON parsing for memory-efficient processing
/// - Zero-copy buffer management
/// - Configurable buffer sizes and message limits
///
/// # Examples
///
/// ```rust
/// use airs_mcp::transport::streaming::StreamingTransport;
/// use airs_mcp::base::jsonrpc::streaming::StreamingConfig;
/// use airs_mcp::transport::stdio::StdioTransport;
///
/// # tokio_test::block_on(async {
/// let base_transport = StdioTransport::new().await.unwrap();
/// let config = StreamingConfig::default();
/// let transport = StreamingTransport::new(base_transport, config);
/// # });
/// ```
pub struct StreamingTransport<T> {
    inner: Arc<Mutex<T>>,
    parser: Arc<Mutex<StreamingParser>>,
    buffer_manager: Arc<BufferManager>,
}

impl<T> StreamingTransport<T>
where
    T: Transport + ZeroCopyTransport + Send + Sync,
{
    /// Create a new streaming transport wrapping an existing transport
    pub fn new(inner: T, config: StreamingConfig) -> Self {
        let buffer_config = BufferConfig::default();
        let buffer_manager = Arc::new(BufferManager::new(buffer_config));
        Self {
            inner: Arc::new(Mutex::new(inner)),
            parser: Arc::new(Mutex::new(StreamingParser::new(config))),
            buffer_manager,
        }
    }

    /// Create a streaming transport with default configuration
    pub fn new_default(inner: T) -> Self {
        Self::new(inner, StreamingConfig::default())
    }

    /// Parse a JSON message from bytes using the streaming parser
    pub async fn parse_message(&self, data: &[u8]) -> Result<ParsedMessage, StreamingError> {
        let mut parser = self.parser.lock().await;
        parser.parse_from_bytes(data).await
    }

    /// Parse multiple JSON messages from a single buffer
    pub async fn parse_multiple_messages(
        &self,
        data: &[u8],
    ) -> Result<Vec<ParsedMessage>, StreamingError> {
        let mut parser = self.parser.lock().await;
        parser.parse_multiple_from_bytes(data).await
    }

    /// Receive and parse a message using streaming
    pub async fn receive_parsed(&self) -> Result<ParsedMessage, TransportError> {
        // Receive raw message from underlying transport
        let mut transport = self.inner.lock().await;
        let message_bytes = transport
            .receive()
            .await
            .map_err(|e| TransportError::other(format!("Transport receive error: {e}")))?;

        // Parse using streaming parser
        let parsed = self
            .parse_message(&message_bytes)
            .await
            .map_err(|e| TransportError::other(format!("Streaming parse error: {e}")))?;

        Ok(parsed)
    }

    /// Send a parsed message efficiently
    pub async fn send_parsed(&self, message: &ParsedMessage) -> Result<(), TransportError> {
        // Convert message to bytes
        let bytes = message
            .to_bytes()
            .map_err(|e| TransportError::other(format!("Message serialization error: {e}")))?;

        let mut transport = self.inner.lock().await;

        // Try zero-copy send first
        if let Ok(()) = transport.send_bytes(&bytes).await {
            Ok(())
        } else {
            // Fallback to regular send with byte slice
            transport
                .send(&bytes)
                .await
                .map_err(|e| TransportError::other(format!("Transport send error: {e}")))
        }
    }

    /// Get buffer usage statistics
    pub async fn buffer_stats(&self) -> crate::base::jsonrpc::streaming::BufferStats {
        let parser = self.parser.lock().await;
        parser.buffer_stats()
    }

    /// Get buffer manager for advanced buffer operations
    pub fn buffer_manager(&self) -> &Arc<BufferManager> {
        &self.buffer_manager
    }
}

/// Statistics for streaming transport operations
#[derive(Debug, Clone)]
pub struct StreamingStats {
    /// Number of messages parsed successfully
    pub messages_parsed: u64,
    /// Number of parse errors encountered
    pub parse_errors: u64,
    /// Number of multi-message batches processed
    pub batch_operations: u64,
    /// Total bytes processed through streaming parser
    pub bytes_processed: u64,
    /// Average message size in bytes
    pub avg_message_size: f64,
}

impl StreamingStats {
    /// Create new streaming stats
    pub fn new() -> Self {
        Self {
            messages_parsed: 0,
            parse_errors: 0,
            batch_operations: 0,
            bytes_processed: 0,
            avg_message_size: 0.0,
        }
    }

    /// Calculate parsing success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        let total = self.messages_parsed + self.parse_errors;
        if total == 0 {
            1.0
        } else {
            self.messages_parsed as f64 / total as f64
        }
    }

    /// Update statistics with a successful parse operation
    pub fn record_success(&mut self, bytes_processed: usize) {
        self.messages_parsed += 1;
        self.bytes_processed += bytes_processed as u64;
        self.avg_message_size = self.bytes_processed as f64 / self.messages_parsed as f64;
    }

    /// Record a parse error
    pub fn record_error(&mut self) {
        self.parse_errors += 1;
    }

    /// Record a batch operation
    pub fn record_batch(&mut self, message_count: usize) {
        self.batch_operations += 1;
        self.messages_parsed += message_count as u64;
    }
}

impl Default for StreamingStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::StdioTransport;

    #[tokio::test]
    async fn test_streaming_transport_creation() {
        let base_transport = StdioTransport::new().await.unwrap();
        let config = StreamingConfig::default();
        let transport = StreamingTransport::new(base_transport, config);

        // Verify the transport is created successfully
        let metrics = transport.buffer_manager.metrics();
        // Just verify metrics exist (no need to compare with 0 for usize)
        let _hit_count = metrics
            .buffer_hits
            .load(std::sync::atomic::Ordering::Relaxed);
    }

    #[tokio::test]
    async fn test_parse_message() {
        let base_transport = StdioTransport::new().await.unwrap();
        let transport = StreamingTransport::new_default(base_transport);

        let json = br#"{"jsonrpc":"2.0","method":"ping","id":"test-123"}"#;
        let message = transport.parse_message(json).await.unwrap();

        match message {
            ParsedMessage::Request(request) => {
                assert_eq!(request.method, "ping");
            }
            _ => panic!("Expected request message"),
        }
    }

    #[tokio::test]
    async fn test_parse_multiple_messages() {
        let base_transport = StdioTransport::new().await.unwrap();
        let transport = StreamingTransport::new_default(base_transport);

        let json =
            br#"{"jsonrpc":"2.0","method":"ping","id":1}{"jsonrpc":"2.0","method":"pong","id":2}"#;
        let messages = transport.parse_multiple_messages(json).await.unwrap();

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].method(), Some("ping"));
        assert_eq!(messages[1].method(), Some("pong"));
    }

    #[tokio::test]
    async fn test_buffer_stats() {
        let base_transport = StdioTransport::new().await.unwrap();
        let transport = StreamingTransport::new_default(base_transport);

        let stats = transport.buffer_stats().await;
        // Verify stats are valid (no need to compare capacity with 0 for usize)
        assert!(stats.max_size > 0);
    }

    #[tokio::test]
    async fn test_streaming_stats() {
        let mut stats = StreamingStats::new();

        stats.record_success(100);
        stats.record_success(200);
        stats.record_error();

        assert_eq!(stats.messages_parsed, 2);
        assert_eq!(stats.parse_errors, 1);
        assert_eq!(stats.bytes_processed, 300);
        assert_eq!(stats.avg_message_size, 150.0);
        assert!(stats.success_rate() > 0.6 && stats.success_rate() < 0.7);
    }

    #[tokio::test]
    async fn test_streaming_stats_batch_operations() {
        let mut stats = StreamingStats::new();

        stats.record_batch(5);
        stats.record_batch(3);

        assert_eq!(stats.batch_operations, 2);
        assert_eq!(stats.messages_parsed, 8);
    }
}
