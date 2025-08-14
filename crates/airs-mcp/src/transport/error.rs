//! Transport Error Types
//!
//! This module defines common error types used across all transport implementations.

/// Common transport error types.
///
/// This enum provides a standardized set of error variants that can be used
/// by transport implementations, while still allowing for transport-specific
/// error types through the `Other` variant.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    /// I/O operation failed
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Transport connection is closed
    #[error("Transport connection is closed")]
    Closed,

    /// Connection limit exceeded
    #[error("Connection limit exceeded: {0}")]
    ConnectionLimit(String),

    /// Invalid connection reference
    #[error("Invalid connection: {0}")]
    InvalidConnection(String),

    /// Session management error
    #[error("Session error: {0}")]
    SessionError(String),

    /// Message formatting or framing error
    #[error("Message format error: {message}")]
    Format { message: String },

    /// Connection timeout
    #[error("Connection timeout after {duration_ms}ms")]
    Timeout { duration_ms: u64 },

    /// Buffer overflow or resource exhaustion
    #[error("Buffer overflow: {details}")]
    BufferOverflow { details: String },

    /// Message size exceeds maximum allowed
    #[error("Message too large: {size} bytes (max: {max_size} bytes)")]
    MessageTooLarge { size: usize, max_size: usize },

    /// Incomplete message received
    #[error("Incomplete message received")]
    IncompleteMessage,

    /// JSON parsing error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// JSON serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Transport-specific error
    #[error("Transport error: {details}")]
    Other { details: String },
}

impl TransportError {
    /// Create a format error with a message
    pub fn format(message: impl Into<String>) -> Self {
        Self::Format {
            message: message.into(),
        }
    }

    /// Create a timeout error with duration
    pub fn timeout(duration_ms: u64) -> Self {
        Self::Timeout { duration_ms }
    }

    /// Create a buffer overflow error with details
    pub fn buffer_overflow(details: impl Into<String>) -> Self {
        Self::BufferOverflow {
            details: details.into(),
        }
    }

    /// Create a message too large error
    pub fn message_too_large(size: usize, max_size: usize) -> Self {
        Self::MessageTooLarge { size, max_size }
    }

    /// Create an incomplete message error
    pub fn incomplete_message() -> Self {
        Self::IncompleteMessage
    }

    /// Create a parse error
    pub fn parse_error(error: impl Into<String>) -> Self {
        Self::ParseError(error.into())
    }

    /// Create a serialization error
    pub fn serialization_error(error: impl Into<String>) -> Self {
        Self::SerializationError(error.into())
    }

    /// Create a transport-specific error with details
    pub fn other(details: impl Into<String>) -> Self {
        Self::Other {
            details: details.into(),
        }
    }

    /// Create a connection closed error
    pub fn closed() -> Self {
        Self::Closed
    }

    /// Create a connection limit error
    pub fn connection_limit(message: impl Into<String>) -> Self {
        Self::ConnectionLimit(message.into())
    }

    /// Create an invalid connection error
    pub fn invalid_connection(message: impl Into<String>) -> Self {
        Self::InvalidConnection(message.into())
    }

    /// Create a session error
    pub fn session_error(message: impl Into<String>) -> Self {
        Self::SessionError(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_error_creation() {
        // Test format error
        let err = TransportError::format("invalid JSON");
        assert!(matches!(err, TransportError::Format { .. }));
        assert_eq!(err.to_string(), "Message format error: invalid JSON");

        // Test timeout error
        let err = TransportError::timeout(5000);
        assert!(matches!(err, TransportError::Timeout { duration_ms: 5000 }));
        assert_eq!(err.to_string(), "Connection timeout after 5000ms");

        // Test buffer overflow error
        let err = TransportError::buffer_overflow("message too large");
        assert!(matches!(err, TransportError::BufferOverflow { .. }));
        assert_eq!(err.to_string(), "Buffer overflow: message too large");

        // Test other error
        let err = TransportError::other("custom transport error");
        assert!(matches!(err, TransportError::Other { .. }));
        assert_eq!(err.to_string(), "Transport error: custom transport error");

        // Test closed error
        let err = TransportError::Closed;
        assert_eq!(err.to_string(), "Transport connection is closed");
    }

    #[test]
    fn test_transport_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broken");
        let transport_err = TransportError::from(io_err);

        assert!(matches!(transport_err, TransportError::Io(_)));
        assert!(transport_err.to_string().contains("pipe broken"));
    }

    #[test]
    fn test_transport_error_traits() {
        let err = TransportError::Closed;

        // Test Error trait
        assert!(std::error::Error::source(&err).is_none());

        // Test Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TransportError>();

        // Test Debug
        let debug_str = format!("{err:?}");
        assert!(!debug_str.is_empty());
    }
}
