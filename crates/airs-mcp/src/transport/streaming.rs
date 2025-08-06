//! Enhanced transport integration with streaming JSON parser
//!
//! This module provides transport helpers that integrate with the
//! streaming JSON parser for efficient message processing.

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::base::jsonrpc::streaming::{
    ParsedMessage, StreamingConfig, StreamingError, StreamingParser,
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
    use crate::transport::stdio::StdioTransport;

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
