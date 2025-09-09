//! Streaming Protocol Optimizations
//!
//! This module provides streaming JSON parser optimizations migrated from:
//! - `src/base/jsonrpc/streaming.rs`
//!
//! Features:
//! - Incremental parsing from streaming input
//! - Zero-copy buffer management
//! - Memory-efficient processing of large JSON payloads
//! - Integration with the buffer pool system

// Layer 1: Standard library imports
// (Unused imports removed)

// Layer 2: Third-party crate imports
use bytes::BytesMut;
// (Unused imports removed)

// Layer 3: Internal module imports
use crate::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

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

/// Streaming JSON parser for efficient message processing
///
/// The StreamingParser can incrementally parse JSON messages from async readers,
/// buffers, or byte streams without loading the entire message into memory.
pub struct StreamingParser {
    #[allow(dead_code)] // Internal infrastructure - may be used later
    config: StreamingConfig,
    #[allow(dead_code)] // Internal infrastructure - may be used later
    buffer: BytesMut,
}

impl StreamingParser {
    /// Create a new streaming parser with the given configuration
    pub fn new(config: StreamingConfig) -> Self {
        let buffer_size = config.read_buffer_size;
        Self {
            config,
            buffer: BytesMut::with_capacity(buffer_size),
        }
    }

    /// Create a streaming parser with default configuration
    pub fn new_default() -> Self {
        Self::new(StreamingConfig::default())
    }
}

/// Statistics for buffer performance monitoring
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

// PHASE 1: Placeholder - actual implementation will be added in Phase 2
