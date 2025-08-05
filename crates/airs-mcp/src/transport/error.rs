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

    /// Message formatting or framing error
    #[error("Message format error: {message}")]
    Format { message: String },

    /// Connection timeout
    #[error("Connection timeout after {duration_ms}ms")]
    Timeout { duration_ms: u64 },

    /// Buffer overflow or resource exhaustion
    #[error("Buffer overflow: {details}")]
    BufferOverflow { details: String },

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

    /// Create a transport-specific error with details
    pub fn other(details: impl Into<String>) -> Self {
        Self::Other {
            details: details.into(),
        }
    }

    /// Create a connection closed error
    pub fn connection_closed() -> Self {
        Self::Closed
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
        let debug_str = format!("{:?}", err);
        assert!(!debug_str.is_empty());
    }
}
