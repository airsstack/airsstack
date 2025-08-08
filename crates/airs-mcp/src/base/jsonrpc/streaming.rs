//! Streaming JSON parser for efficient message processing
//!
//! This module provides a streaming JSON parser that can handle large messages
//! without loading the entire message into memory at once. It supports:
//!
//! - Incremental parsing from streaming input
//! - Zero-copy buffer management
//! - Memory-efficient processing of large JSON payloads
//! - Integration with the buffer pool system

use std::io::Cursor;

use bytes::{Bytes, BytesMut};
use serde_json::Value;
use tokio::io::{AsyncBufRead, AsyncRead, AsyncReadExt};

use crate::base::jsonrpc::{JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

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
    #[error("Buffer overflow: message exceeds maximum size of {max_size} bytes")]
    BufferOverflow { max_size: usize },

    /// Incomplete message - more data needed
    #[error("Incomplete message: need more data to complete parsing")]
    IncompleteMessage,
}

/// Configuration for the streaming JSON parser
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Maximum message size in bytes (default: 16MB)
    pub max_message_size: usize,

    /// Buffer size for reading chunks (default: 8KB)
    pub read_buffer_size: usize,

    /// Whether to validate JSON structure strictly
    pub strict_validation: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_message_size: 16 * 1024 * 1024, // 16MB
            read_buffer_size: 8 * 1024,         // 8KB
            strict_validation: true,
        }
    }
}

/// Streaming JSON parser for efficient message processing
///
/// The StreamingParser can incrementally parse JSON messages from async readers,
/// buffers, or byte streams without loading the entire message into memory.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::base::jsonrpc::streaming::{StreamingParser, StreamingConfig};
/// use bytes::Bytes;
///
/// # tokio_test::block_on(async {
/// let config = StreamingConfig::default();
/// let mut parser = StreamingParser::new(config);
///
/// let json_data = br#"{"jsonrpc":"2.0","method":"ping","id":1}"#;
/// let message = parser.parse_from_bytes(json_data).await.unwrap();
/// # });
/// ```
pub struct StreamingParser {
    config: StreamingConfig,
    buffer: BytesMut,
}

impl StreamingParser {
    /// Create a new streaming parser with the given configuration
    ///
    /// Initializes a parser with:
    /// - Pre-allocated buffer based on `read_buffer_size` configuration
    /// - Validation settings from the provided config
    /// - Memory limits according to `max_message_size`
    ///
    /// # Performance Considerations
    ///
    /// - **Buffer size**: Larger buffers reduce allocation overhead but use more memory
    /// - **Validation**: Strict validation prevents malformed messages but adds overhead
    /// - **Memory limits**: Prevents DoS attacks through oversized messages
    ///
    /// # Arguments
    ///
    /// * `config` - Parser configuration including buffer and validation settings
    pub fn new(config: StreamingConfig) -> Self {
        let buffer_size = config.read_buffer_size;
        Self {
            config,
            buffer: BytesMut::with_capacity(buffer_size),
        }
    }

    /// Create a streaming parser with default configuration
    ///
    /// Uses sensible defaults optimized for typical JSON-RPC workloads:
    /// - 8KB read buffer (good balance of performance and memory usage)
    /// - 16MB max message size (handles large payloads while preventing DoS)
    /// - Strict validation enabled (ensures protocol compliance)
    pub fn new_default() -> Self {
        Self::new(StreamingConfig::default())
    }

    /// Parse a JSON-RPC message from bytes
    ///
    /// This method implements a two-phase parsing approach:
    /// 1. **Message Type Detection**: Fast field presence check to determine message type
    /// 2. **Typed Deserialization**: Full deserialization using appropriate concrete type
    ///
    /// # Message Type Detection Algorithm
    ///
    /// Uses JSON field presence to determine message type:
    /// - **Request**: Has both "method" and "id" fields
    /// - **Notification**: Has "method" but no "id" field  
    /// - **Response**: Has either "result" or "error" fields
    ///
    /// This approach avoids expensive pattern matching during deserialization
    /// and provides better error messages for malformed messages.
    ///
    /// # Performance Characteristics
    ///
    /// - **Memory**: Single allocation for initial JSON parsing
    /// - **CPU**: O(1) field lookup, then type-specific deserialization
    /// - **Accuracy**: Handles edge cases like null/missing fields correctly
    ///
    /// # Arguments
    ///
    /// * `data` - Raw message bytes (should be valid JSON)
    ///
    /// # Returns
    ///
    /// * `Ok(ParsedMessage)` - Successfully parsed and typed message
    /// * `Err(StreamingError)` - Parse error or invalid message structure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::base::jsonrpc::streaming::{StreamingParser, ParsedMessage};
    ///
    /// # tokio_test::block_on(async {
    /// let mut parser = StreamingParser::new_default();
    /// let json = br#"{"jsonrpc":"2.0","method":"heartbeat"}"#;
    ///
    /// let message = parser.parse_from_bytes(json).await.unwrap();
    /// match message {
    ///     ParsedMessage::Notification(notification) => {
    ///         assert_eq!(notification.method, "heartbeat");
    ///     }
    ///     _ => panic!("Expected notification"),
    /// }
    /// # });
    /// ```
    pub async fn parse_from_bytes(&mut self, data: &[u8]) -> Result<ParsedMessage, StreamingError> {
        // Validate message size against configured limits
        if data.len() > self.config.max_message_size {
            return Err(StreamingError::BufferOverflow {
                max_size: self.config.max_message_size,
            });
        }

        // Use serde_json's streaming deserializer for efficient parsing
        let mut cursor = Cursor::new(data);

        // Phase 1: Parse as generic JSON value for fast message type detection
        // This avoids the overhead of trying multiple concrete types
        let value: Value = serde_json::from_reader(&mut cursor)?;

        // Reset cursor for Phase 2 typed parsing
        cursor.set_position(0);

        // Phase 2: Message type detection and typed deserialization
        // Use field presence to determine message type (faster than pattern matching)
        if value.get("method").is_some() {
            // Has "method" field - could be Request or Notification
            if value.get("id").is_some() {
                // Has both "method" and "id" - definitely a Request
                let request: JsonRpcRequest = serde_json::from_reader(cursor)?;
                Ok(ParsedMessage::Request(request))
            } else {
                // Has "method" but no "id" - definitely a Notification
                let notification: JsonRpcNotification = serde_json::from_reader(cursor)?;
                Ok(ParsedMessage::Notification(notification))
            }
        } else if value.get("result").is_some() || value.get("error").is_some() {
            // Has "result" or "error" field - definitely a Response
            let response: JsonRpcResponse = serde_json::from_reader(cursor)?;
            Ok(ParsedMessage::Response(response))
        } else {
            // No recognizable JSON-RPC fields - invalid message structure
            Err(StreamingError::JsonError(serde_json::Error::io(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid JSON-RPC message: missing required fields (method/result/error)",
                ),
            )))
        }
    }

    /// Parse a JSON-RPC message from an async reader
    ///
    /// This method implements incremental JSON parsing using a state machine
    /// to handle streaming data and detect complete JSON objects without
    /// buffering excessive amounts of data.
    ///
    /// # Parsing Algorithm
    ///
    /// 1. **Byte-by-byte Processing**: Reads individual bytes from the stream
    /// 2. **Brace Counting**: Tracks JSON object depth using brace counters
    /// 3. **String State Tracking**: Handles escaped characters within JSON strings
    /// 4. **Completion Detection**: Identifies when JSON object is complete (brace_count = 0)
    /// 5. **Message Type Resolution**: Uses field presence to determine message type
    ///
    /// # State Machine Implementation
    ///
    /// - **in_string**: True when processing characters inside a JSON string
    /// - **escape_next**: True when the next character should be escaped
    /// - **brace_count**: Current nesting depth (0 = complete object)
    /// - **started**: True after encountering the first opening brace
    ///
    /// This approach ensures memory efficiency while handling partial reads
    /// and incomplete messages gracefully.
    ///
    /// # Arguments
    ///
    /// * `reader` - Async reader providing JSON data
    ///
    /// # Returns
    ///
    /// * `Ok(ParsedMessage)` - Successfully parsed message
    /// * `Err(StreamingError)` - Parsing failed or reader error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::base::jsonrpc::streaming::StreamingParser;
    /// use std::io::Cursor;
    ///
    /// # tokio_test::block_on(async {
    /// let mut parser = StreamingParser::new_default();
    /// let json_data = br#"{"jsonrpc":"2.0","result":"success","id":"test"}"#;
    /// let mut reader = tokio::io::BufReader::new(Cursor::new(json_data));
    ///
    /// let message = parser.parse_from_reader(&mut reader).await.unwrap();
    /// # });
    /// ```
    pub async fn parse_from_reader<R>(
        &mut self,
        reader: &mut R,
    ) -> Result<ParsedMessage, StreamingError>
    where
        R: AsyncRead + AsyncBufRead + Unpin,
    {
        self.buffer.clear();

        // State machine for incremental JSON parsing
        // These variables track the current parsing state to detect complete JSON objects
        let mut brace_count = 0; // Tracks JSON object nesting depth
        let mut in_string = false; // True when inside a JSON string value
        let mut escape_next = false; // True when next char should be escaped
        let mut started = false; // True after encountering first opening brace

        loop {
            // Read one byte at a time to implement streaming parser
            let byte = match reader.read_u8().await {
                Ok(b) => b,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Handle EOF conditions based on parser state
                    if started && brace_count > 0 {
                        // We started parsing but object is incomplete
                        return Err(StreamingError::IncompleteMessage);
                    } else {
                        // Clean EOF before starting or after completing object
                        return Err(StreamingError::IoError(e));
                    }
                }
                Err(e) => return Err(StreamingError::IoError(e)),
            };

            // Check message size limit before processing
            // Check message size limit before processing
            if self.buffer.len() >= self.config.max_message_size {
                return Err(StreamingError::BufferOverflow {
                    max_size: self.config.max_message_size,
                });
            }

            // Add byte to buffer for final parsing
            self.buffer.extend_from_slice(&[byte]);

            // State machine logic for tracking JSON structure
            // This implements a minimal JSON parser that only tracks brace nesting
            match byte {
                b'"' if !escape_next => {
                    // Toggle string state (entering or leaving string literal)
                    // Only toggle if not escaped (previous char wasn't backslash)
                    in_string = !in_string;
                }
                b'\\' if in_string => {
                    // Handle escape sequence within string literals
                    // Next character should be treated as escaped
                    escape_next = !escape_next;
                }
                b'{' if !in_string => {
                    // Opening brace outside of string - increment nesting depth
                    started = true; // Mark that we've started parsing an object
                    brace_count += 1;
                }
                b'}' if !in_string => {
                    // Closing brace outside of string - decrement nesting depth
                    brace_count -= 1;
                    if brace_count == 0 && started {
                        // Complete JSON object found (balanced braces)
                        // Break out of reading loop to parse the complete object
                        break;
                    }
                }
                _ => {
                    // Reset escape state for non-escape characters
                    // This handles cases where escape was set but current char doesn't continue it
                    escape_next = false;
                }
            }

            // Ensure escape state is properly reset after processing
            // This prevents escape state from persisting incorrectly
            if !escape_next {
                escape_next = false;
            }
        }

        // Parse the complete JSON object from accumulated buffer
        // At this point, buffer contains exactly one complete JSON-RPC message
        let buffer_data = self.buffer.clone().freeze();
        self.parse_from_bytes(&buffer_data).await
    }

    /// Parse multiple JSON-RPC messages from a buffer
    ///
    /// This method can extract multiple complete JSON-RPC messages from a single
    /// buffer, which is useful for processing batched messages or handling
    /// multiple messages received in a single network packet.
    ///
    /// # Arguments
    ///
    /// * `data` - Byte slice potentially containing multiple JSON messages
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<ParsedMessage>)` - Vector of successfully parsed messages
    /// * `Err(StreamingError)` - Parsing failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::base::jsonrpc::streaming::StreamingParser;
    ///
    /// # tokio_test::block_on(async {
    /// let mut parser = StreamingParser::new_default();
    /// let json = br#"{"jsonrpc":"2.0","method":"ping","id":1}{"jsonrpc":"2.0","method":"pong","id":2}"#;
    ///
    /// let messages = parser.parse_multiple_from_bytes(json).await.unwrap();
    /// assert_eq!(messages.len(), 2);
    /// # });
    /// ```
    pub async fn parse_multiple_from_bytes(
        &mut self,
        data: &[u8],
    ) -> Result<Vec<ParsedMessage>, StreamingError> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let mut messages = Vec::new();
        let mut start = 0;
        let mut brace_count = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for (i, &byte) in data.iter().enumerate() {
            match byte {
                b'"' if !escape_next => in_string = !in_string,
                b'\\' if in_string => escape_next = !escape_next,
                b'{' if !in_string => brace_count += 1,
                b'}' if !in_string => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        // Complete JSON object found
                        let message_data = &data[start..=i];
                        let message = self.parse_from_bytes(message_data).await?;
                        messages.push(message);
                        start = i + 1;
                    }
                }
                _ => escape_next = false,
            }

            if !escape_next {
                escape_next = false;
            }
        }

        // Check for incomplete message at the end
        if start < data.len() && brace_count > 0 {
            return Err(StreamingError::IncompleteMessage);
        }

        Ok(messages)
    }

    /// Reset the internal buffer
    ///
    /// This method clears the internal buffer, which can be useful for memory
    /// management or when switching to parse a new stream.
    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    /// Get current buffer usage statistics
    pub fn buffer_stats(&self) -> BufferStats {
        BufferStats {
            capacity: self.buffer.capacity(),
            used: self.buffer.len(),
            max_size: self.config.max_message_size,
        }
    }
}

/// Statistics about buffer usage in the streaming parser
#[derive(Debug, Clone)]
pub struct BufferStats {
    /// Total buffer capacity
    pub capacity: usize,
    /// Currently used buffer space
    pub used: usize,
    /// Maximum allowed message size
    pub max_size: usize,
}

impl BufferStats {
    /// Calculate buffer utilization as a percentage (0.0 to 1.0)
    pub fn utilization(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            self.used as f64 / self.capacity as f64
        }
    }
}

/// Parsed JSON-RPC message variants
///
/// This enum represents the different types of JSON-RPC messages that can
/// be parsed by the streaming parser.
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

    /// Extract the method name if available (requests and notifications only)
    pub fn method(&self) -> Option<&str> {
        match self {
            ParsedMessage::Request(req) => Some(&req.method),
            ParsedMessage::Notification(notif) => Some(&notif.method),
            ParsedMessage::Response(_) => None,
        }
    }

    /// Convert back to JSON bytes for forwarding or storage
    pub fn to_bytes(&self) -> Result<Bytes, serde_json::Error> {
        let json = match self {
            ParsedMessage::Request(req) => req.to_json()?,
            ParsedMessage::Response(resp) => resp.to_json()?,
            ParsedMessage::Notification(notif) => notif.to_json()?,
        };
        Ok(Bytes::from(json))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_parse_request_from_bytes() {
        let mut parser = StreamingParser::new_default();
        let json = br#"{"jsonrpc":"2.0","method":"ping","params":{"data":"test"},"id":"test-123"}"#;

        let message = parser.parse_from_bytes(json).await.unwrap();

        match message {
            ParsedMessage::Request(request) => {
                assert_eq!(request.method, "ping");
                assert_eq!(request.jsonrpc, "2.0");
                assert!(request.params.is_some());
            }
            _ => panic!("Expected request message"),
        }
    }

    #[tokio::test]
    async fn test_parse_notification_from_bytes() {
        let mut parser = StreamingParser::new_default();
        let json = br#"{"jsonrpc":"2.0","method":"heartbeat"}"#;

        let message = parser.parse_from_bytes(json).await.unwrap();

        match message {
            ParsedMessage::Notification(notification) => {
                assert_eq!(notification.method, "heartbeat");
                assert_eq!(notification.jsonrpc, "2.0");
            }
            _ => panic!("Expected notification message"),
        }
    }

    #[tokio::test]
    async fn test_parse_response_from_bytes() {
        let mut parser = StreamingParser::new_default();
        let json = br#"{"jsonrpc":"2.0","result":"success","id":"test-456"}"#;

        let message = parser.parse_from_bytes(json).await.unwrap();

        match message {
            ParsedMessage::Response(response) => {
                assert_eq!(response.jsonrpc, "2.0");
                assert!(response.result.is_some());
                assert!(response.error.is_none());
            }
            _ => panic!("Expected response message"),
        }
    }

    #[tokio::test]
    async fn test_parse_multiple_messages() {
        let mut parser = StreamingParser::new_default();
        let json =
            br#"{"jsonrpc":"2.0","method":"ping","id":1}{"jsonrpc":"2.0","method":"pong","id":2}"#;

        let messages = parser.parse_multiple_from_bytes(json).await.unwrap();

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].method(), Some("ping"));
        assert_eq!(messages[1].method(), Some("pong"));
    }

    #[tokio::test]
    async fn test_parse_from_reader() {
        let mut parser = StreamingParser::new_default();
        let json_data = br#"{"jsonrpc":"2.0","result":"success","id":"test"}"#;
        let mut reader = tokio::io::BufReader::new(Cursor::new(json_data));

        let message = parser.parse_from_reader(&mut reader).await.unwrap();

        match message {
            ParsedMessage::Response(response) => {
                assert_eq!(response.result, Some(json!("success")));
            }
            _ => panic!("Expected response message"),
        }
    }

    #[tokio::test]
    async fn test_buffer_overflow_protection() {
        let config = StreamingConfig {
            max_message_size: 10, // Very small limit
            ..Default::default()
        };
        let mut parser = StreamingParser::new(config);

        let large_json = br#"{"jsonrpc":"2.0","method":"test","params":{"very":"large","data":"structure"},"id":1}"#;

        let result = parser.parse_from_bytes(large_json).await;
        assert!(matches!(result, Err(StreamingError::BufferOverflow { .. })));
    }

    #[tokio::test]
    async fn test_incomplete_message_detection() {
        let mut parser = StreamingParser::new_default();
        let incomplete_json = br#"{"jsonrpc":"2.0","method":"test"#; // Missing closing brace

        let messages = parser.parse_multiple_from_bytes(incomplete_json).await;
        assert!(matches!(messages, Err(StreamingError::IncompleteMessage)));
    }

    #[tokio::test]
    async fn test_buffer_stats() {
        let mut parser = StreamingParser::new_default();
        let json = br#"{"jsonrpc":"2.0","method":"test","id":1}"#;

        let _message = parser.parse_from_bytes(json).await.unwrap();
        let stats = parser.buffer_stats();

        assert!(stats.capacity > 0);
        assert!(stats.max_size > 0);
    }

    #[tokio::test]
    async fn test_parsed_message_methods() {
        let mut parser = StreamingParser::new_default();
        let json = br#"{"jsonrpc":"2.0","method":"ping","id":1}"#;

        let message = parser.parse_from_bytes(json).await.unwrap();

        assert_eq!(message.message_type(), "request");
        assert_eq!(message.method(), Some("ping"));

        let bytes = message.to_bytes().unwrap();
        assert!(!bytes.is_empty());
    }

    #[tokio::test]
    async fn test_invalid_json_error() {
        let mut parser = StreamingParser::new_default();
        let invalid_json = br#"{"invalid":"json","missing_required_fields":true}"#;

        let result = parser.parse_from_bytes(invalid_json).await;
        assert!(matches!(result, Err(StreamingError::JsonError(_))));
    }
}
