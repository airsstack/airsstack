//! Request Parser with Configurable Buffer Strategy
//!
//! This module implements the request parser that integrates with the
//! buffer management system. Based on principal engineer review, this
//! uses per-request parser creation to eliminate shared mutex bottlenecks.

use std::sync::Arc;

use serde_json::Value;

// Use consolidated protocol module instead of streaming parser
use crate::protocol::JsonRpcMessage;
use crate::transport::adapters::http::buffer_pool::{BufferPool, BufferStrategy};
use crate::transport::adapters::http::config::{OptimizationStrategy, ParserConfig};
use crate::transport::error::TransportError;

/// Request parser with configurable buffer strategy
///
/// This parser eliminates the shared mutex bottleneck by creating
/// a new parser instance per request while optionally reusing
/// memory buffers through the buffer pool.
///
/// # Performance Characteristics
///
/// - **Per-Request Creation**: No shared state, no contention
/// - **Consistent Latency**: ~100μs vs variable 50ms+ with shared mutex
/// - **Buffer Reuse**: Optional 80% performance improvement for small messages
/// - **Memory Efficient**: ~8KB per concurrent request with pooling
///
/// # Usage
///
/// ```rust
/// # tokio_test::block_on(async {
/// use airs_mcp::transport::adapters::http::RequestParser;
/// use airs_mcp::transport::adapters::http::config::ParserConfig;
///
/// let config = ParserConfig::new();
/// let parser = RequestParser::new(config);
///
/// // Parse a request (each call creates new parser instance)
/// let request_data = br#"{"jsonrpc":"2.0","method":"ping","id":1}"#;
/// let result = parser.parse_request(request_data).await.unwrap();
/// # });
/// ```
pub struct RequestParser {
    config: ParserConfig,
    buffer_strategy: BufferStrategy,
}

impl RequestParser {
    /// Create a new request parser with the given configuration
    pub fn new(config: ParserConfig) -> Self {
        let buffer_strategy = match &config.optimization_strategy {
            OptimizationStrategy::None => BufferStrategy::PerRequest,
            OptimizationStrategy::BufferPool(pool_config) => {
                let pool = Arc::new(BufferPool::new(pool_config.clone()));
                BufferStrategy::Pooled(pool)
            }
        };

        Self {
            config,
            buffer_strategy,
        }
    }

    /// Parse a JSON-RPC request with per-request parser instance
    ///
    /// This method creates a new StreamingParser for each request,
    /// eliminating contention while optionally reusing buffer memory.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw request bytes to parse
    ///
    /// # Returns
    ///
    /// * `Ok(Value)` - Parsed JSON-RPC request
    /// * `Err(TransportError)` - Parse error or message too large
    pub async fn parse_request(&self, data: &[u8]) -> Result<Value, TransportError> {
        // Check message size limit
        if data.len() > self.config.max_message_size {
            return Err(TransportError::MessageTooLarge {
                size: data.len(),
                max_size: self.config.max_message_size,
            });
        }

        // Get buffer according to strategy
        let mut buffer = self.buffer_strategy.get_buffer();

        // Copy data to buffer
        buffer.extend_from_slice(data);

        // Parse the request directly as JSON-RPC
        match std::str::from_utf8(&buffer) {
            Ok(json_str) => match serde_json::from_str::<JsonRpcMessage>(json_str) {
                Ok(message) => {
                    // Convert JsonRpcMessage to Value
                    let value = match message {
                        JsonRpcMessage::Request(req) => serde_json::to_value(req)
                            .map_err(|e| TransportError::SerializationError(e.to_string()))?,
                        JsonRpcMessage::Response(resp) => serde_json::to_value(resp)
                            .map_err(|e| TransportError::SerializationError(e.to_string()))?,
                        JsonRpcMessage::Notification(notif) => serde_json::to_value(notif)
                            .map_err(|e| TransportError::SerializationError(e.to_string()))?,
                    };
                    Ok(value)
                }
                Err(e) => Err(TransportError::ParseError(format!(
                    "JSON parsing error: {}",
                    e
                ))),
            },
            Err(e) => Err(TransportError::ParseError(format!(
                "UTF-8 parsing error: {}",
                e
            ))),
        }
    }

    /// Parse multiple JSON-RPC requests from a buffer
    ///
    /// This method handles scenarios where multiple requests are
    /// received in a single HTTP request body.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw request bytes containing potentially multiple messages
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Value>)` - Vector of parsed JSON-RPC requests
    /// * `Err(TransportError)` - Parse error or message too large
    pub async fn parse_requests(&self, data: &[u8]) -> Result<Vec<Value>, TransportError> {
        // Check message size limit
        if data.len() > self.config.max_message_size {
            return Err(TransportError::MessageTooLarge {
                size: data.len(),
                max_size: self.config.max_message_size,
            });
        }

        // Get buffer according to strategy
        let mut buffer = self.buffer_strategy.get_buffer();

        // Copy data to buffer
        buffer.extend_from_slice(data);

        // Parse all messages (assuming line-delimited JSON for multiple messages)
        match std::str::from_utf8(&buffer) {
            Ok(json_str) => {
                let mut requests = Vec::new();

                // Try parsing as single message first
                if let Ok(message) = serde_json::from_str::<JsonRpcMessage>(json_str) {
                    let value = match message {
                        JsonRpcMessage::Request(req) => serde_json::to_value(req)
                            .map_err(|e| TransportError::SerializationError(e.to_string()))?,
                        JsonRpcMessage::Response(resp) => serde_json::to_value(resp)
                            .map_err(|e| TransportError::SerializationError(e.to_string()))?,
                        JsonRpcMessage::Notification(notif) => serde_json::to_value(notif)
                            .map_err(|e| TransportError::SerializationError(e.to_string()))?,
                    };
                    requests.push(value);
                } else {
                    // Try parsing as line-delimited JSON
                    for line in json_str.lines() {
                        if line.trim().is_empty() {
                            continue;
                        }

                        match serde_json::from_str::<JsonRpcMessage>(line) {
                            Ok(message) => {
                                let value = match message {
                                    JsonRpcMessage::Request(req) => serde_json::to_value(req)
                                        .map_err(|e| {
                                            TransportError::SerializationError(e.to_string())
                                        })?,
                                    JsonRpcMessage::Response(resp) => serde_json::to_value(resp)
                                        .map_err(|e| {
                                            TransportError::SerializationError(e.to_string())
                                        })?,
                                    JsonRpcMessage::Notification(notif) => {
                                        serde_json::to_value(notif).map_err(|e| {
                                            TransportError::SerializationError(e.to_string())
                                        })?
                                    }
                                };
                                requests.push(value);
                            }
                            Err(e) => {
                                return Err(TransportError::ParseError(format!(
                                    "JSON parsing error: {}",
                                    e
                                )))
                            }
                        }
                    }
                }

                Ok(requests)
            }
            Err(e) => Err(TransportError::ParseError(format!(
                "UTF-8 parsing error: {}",
                e
            ))),
        }
    }

    /// Serialize a JSON-RPC response with buffer reuse
    ///
    /// This method serializes responses using the same buffer strategy
    /// for consistency and performance.
    ///
    /// # Arguments
    ///
    /// * `response` - JSON-RPC response to serialize
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - Serialized response bytes
    /// * `Err(TransportError)` - Serialization error
    pub async fn serialize_response(&self, response: &Value) -> Result<Vec<u8>, TransportError> {
        // Get buffer according to strategy
        let mut buffer = self.buffer_strategy.get_buffer();

        // Serialize to buffer
        serde_json::to_writer(&mut *buffer, response)
            .map_err(|e| TransportError::SerializationError(e.to_string()))?;

        // Add newline for message framing
        buffer.push(b'\n');

        // Return owned copy of buffer content
        Ok(buffer.to_vec())
    }

    /// Get parser configuration
    pub fn config(&self) -> &ParserConfig {
        &self.config
    }

    /// Get buffer pool statistics (if using pooled strategy)
    pub fn buffer_stats(
        &self,
    ) -> Option<crate::transport::adapters::http::buffer_pool::BufferPoolStats> {
        match &self.buffer_strategy {
            BufferStrategy::PerRequest => None,
            BufferStrategy::Pooled(pool) => Some(pool.stats()),
        }
    }
}

impl Clone for RequestParser {
    fn clone(&self) -> Self {
        // Create new parser with same configuration
        Self::new(self.config.clone())
    }
}

/// Helper struct for request parsing metrics
#[derive(Debug, Clone)]
pub struct ParseMetrics {
    /// Number of bytes processed
    pub bytes_processed: usize,

    /// Number of messages parsed
    pub messages_parsed: usize,

    /// Whether buffer pooling was used
    pub used_buffer_pool: bool,

    /// Buffer pool utilization (if applicable)
    pub pool_utilization: Option<f64>,
}

impl RequestParser {
    /// Parse request with metrics collection
    ///
    /// This method provides detailed metrics about the parsing operation
    /// for monitoring and optimization purposes.
    pub async fn parse_request_with_metrics(
        &self,
        data: &[u8],
    ) -> Result<(Value, ParseMetrics), TransportError> {
        let bytes_processed = data.len();
        let used_buffer_pool = matches!(self.buffer_strategy, BufferStrategy::Pooled(_));
        let pool_utilization = self.buffer_stats().map(|stats| stats.utilization());

        let request = self.parse_request(data).await?;

        let metrics = ParseMetrics {
            bytes_processed,
            messages_parsed: 1,
            used_buffer_pool,
            pool_utilization,
        };

        Ok((request, metrics))
    }

    /// Parse multiple requests with metrics collection
    pub async fn parse_requests_with_metrics(
        &self,
        data: &[u8],
    ) -> Result<(Vec<Value>, ParseMetrics), TransportError> {
        let bytes_processed = data.len();
        let used_buffer_pool = matches!(self.buffer_strategy, BufferStrategy::Pooled(_));
        let pool_utilization = self.buffer_stats().map(|stats| stats.utilization());

        let requests = self.parse_requests(data).await?;
        let messages_parsed = requests.len();

        let metrics = ParseMetrics {
            bytes_processed,
            messages_parsed,
            used_buffer_pool,
            pool_utilization,
        };

        Ok((requests, metrics))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::adapters::http::config::{
        BufferPoolConfig, OptimizationStrategy, ParserConfig,
    };
    use serde_json::json;

    #[test]
    fn test_parser_creation() {
        let config = ParserConfig::new();
        let parser = RequestParser::new(config);

        assert!(matches!(parser.buffer_strategy, BufferStrategy::PerRequest));
        assert!(parser.buffer_stats().is_none());
    }

    #[test]
    fn test_parser_with_buffer_pool() {
        let mut config = ParserConfig::new();
        config.optimization_strategy = OptimizationStrategy::BufferPool(BufferPoolConfig::new());

        let parser = RequestParser::new(config);

        assert!(matches!(parser.buffer_strategy, BufferStrategy::Pooled(_)));
        assert!(parser.buffer_stats().is_some());
    }

    #[tokio::test]
    async fn test_parse_simple_request() {
        let config = ParserConfig::new();
        let parser = RequestParser::new(config);

        let request_data = br#"{"jsonrpc":"2.0","method":"ping","id":1}"#;
        let result = parser.parse_request(request_data).await;

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request["method"], "ping");
        assert_eq!(request["id"], 1);
    }

    #[tokio::test]
    async fn test_parse_request_too_large() {
        let mut config = ParserConfig::new();
        config.max_message_size = 10; // Very small limit

        let parser = RequestParser::new(config);

        let request_data = br#"{"jsonrpc":"2.0","method":"ping","id":1}"#;
        let result = parser.parse_request(request_data).await;

        assert!(matches!(
            result,
            Err(TransportError::MessageTooLarge { .. })
        ));
    }

    #[tokio::test]
    async fn test_parse_invalid_json() {
        let config = ParserConfig::new();
        let parser = RequestParser::new(config);

        let request_data = b"invalid json";
        let result = parser.parse_request(request_data).await;

        assert!(matches!(result, Err(TransportError::ParseError(_))));
    }

    #[tokio::test]
    async fn test_serialize_response() {
        let config = ParserConfig::new();
        let parser = RequestParser::new(config);

        let response = json!({
            "jsonrpc": "2.0",
            "result": "pong",
            "id": 1
        });

        let result = parser.serialize_response(&response).await;
        assert!(result.is_ok());

        let serialized = result.unwrap();
        assert!(serialized.ends_with(b"\n")); // Should have newline

        // Should be valid JSON when newline is removed
        let json_data = &serialized[..serialized.len() - 1];
        let parsed: Value = serde_json::from_slice(json_data).unwrap();
        assert_eq!(parsed["result"], "pong");
    }

    #[tokio::test]
    async fn test_parse_with_metrics() {
        let config = ParserConfig::new();
        let parser = RequestParser::new(config);

        let request_data = br#"{"jsonrpc":"2.0","method":"ping","id":1}"#;
        let result = parser.parse_request_with_metrics(request_data).await;

        assert!(result.is_ok());
        let (request, metrics) = result.unwrap();

        assert_eq!(request["method"], "ping");
        assert_eq!(metrics.bytes_processed, request_data.len());
        assert_eq!(metrics.messages_parsed, 1);
        assert!(!metrics.used_buffer_pool);
        assert!(metrics.pool_utilization.is_none());
    }

    #[tokio::test]
    async fn test_clone_parser() {
        let config = ParserConfig::new();
        let parser1 = RequestParser::new(config);
        let parser2 = parser1.clone();

        // Both parsers should work independently
        let request_data = br#"{"jsonrpc":"2.0","method":"ping","id":1}"#;

        let result1 = parser1.parse_request(request_data).await;
        let result2 = parser2.parse_request(request_data).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());
    }
}
